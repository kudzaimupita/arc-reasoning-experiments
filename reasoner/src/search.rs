//! Bottom-up search over programs, cheapest first, with observational equivalence:
//! two programs that give the same grids on every input of the task count as one.
use crate::dsl::{apply, ops_for, Op};
use crate::grid::Grid;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;
use std::time::Instant;

/// CPU time used by the calling thread, in seconds. Each task is solved on one thread, so this is the task's CPU
/// time, unaffected by other load on the machine. None if the clock can't be read (then wall-clock time is used).
#[repr(C)] struct Timespec { tv_sec: i64, tv_nsec: i64 }
extern "C" { fn clock_gettime(clk: i32, tp: *mut Timespec) -> i32; }
#[cfg(target_os = "macos")] const CLOCK_THREAD_CPUTIME_ID: i32 = 16;
#[cfg(not(target_os = "macos"))] const CLOCK_THREAD_CPUTIME_ID: i32 = 3;   // Linux
pub fn thread_cpu_secs() -> Option<f64> {
    let mut t = Timespec { tv_sec: 0, tv_nsec: 0 };
    if unsafe { clock_gettime(CLOCK_THREAD_CPUTIME_ID, &mut t) } != 0 { return None; }
    Some(t.tv_sec as f64 + t.tv_nsec as f64 * 1e-9)
}

pub struct Task { pub train: Vec<(Grid, Grid)>, pub test: Vec<Grid> }

pub struct Candidate { pub program: String, pub cost: u16, pub test: Vec<Grid>, /** nodes explored when this answer was first found */ pub found_at: usize }

pub struct Outcome { pub candidates: Vec<Candidate>, pub nodes: usize, pub ms: u128, pub cpu_ms: u128, pub clock: &'static str, pub status: &'static str, pub depth_reached: u8,
    /// leave-one-out check: "off", "n/a" (fewer than 2 train pairs or no candidate), "pass" or "fail"
    pub loo: &'static str, pub loo_pair: Option<usize>, pub loo_margin: usize, pub withheld: Vec<Candidate> }

struct Node { grids: Vec<Grid>, parent: u32, op: Option<Op>, cost: u16, depth: u8 }

const MAX_MAP_DEPTH: u8 = 2;

/// `seconds` is CPU seconds of the solving thread (E16; before, wall-clock). A wall-clock cap of 4x plus 30 s stops a
/// task that is starved of CPU.
pub struct Limits { pub seconds: f64, pub max_nodes: usize, pub max_depth: u8, pub loo: bool,
    /// E17: expand the cheapest program next across all depths (otherwise level by level).
    pub best_first: bool,
    /// E17: make kinds that appear in correct train-pool programs cheaper (src/prior.txt).
    pub prior: bool,
    /// E18: Probe-style per-task learning: a first pass on a third of the budget, then a restart where operation kinds
    /// from near misses (programs that get at least 90% of train cells right) are cheaper.
    pub probe: bool,
    /// E19: the hint network's predicted operation kinds make likely kinds cheaper and unlikely ones dearer.
    pub hints: bool,
    /// E20: below depth 1, try only operations whose kind is among the hint network's top `prune` kinds (0 = off).
    pub prune: usize,
    /// E21: pruning applies only below this depth (default 1: depth 1 keeps full breadth).
    pub prune_from: u8,
    /// E22: which hint weights to use (1 = src/hints.json, 2 = src/hints2.json).
    pub hints_version: u8 }

/// Operation cost with the train-pool prior: each kind seen n times in correct programs gets
/// round(1.5 * log2(1 + n)) bits cheaper, never below 2.
pub fn prior_counts() -> HashMap<String, u32> {
    include_str!("prior.txt").lines().filter(|l| !l.starts_with('#'))
        .filter_map(|l| { let mut it = l.rsplitn(2, ' '); let n = it.next()?.parse().ok()?; Some((it.next()?.to_string(), n)) }).collect()
}
fn op_cost(op: Op, prior: Option<&HashMap<String, u32>>) -> u16 {
    let base = op.cost();
    match prior.and_then(|p| p.get(&op.kind())) {
        Some(&n) => { let bonus = (1.5 * (1.0 + n as f64).log2()).round() as u16; base.saturating_sub(bonus).max(2) }
        None => base,
    }
}

fn combined_hash(gs: &[Grid]) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for g in gs { h ^= g.hash64().wrapping_add(0x9e3779b97f4a7c15); h = h.wrapping_mul(1099511628211); }
    h
}

fn program(nodes: &[Node], mut i: usize) -> String {
    let mut ops = Vec::new();
    while let Some(op) = nodes[i].op { ops.push(op.name()); i = nodes[i].parent as usize; }
    if ops.is_empty() { return "input".into(); }
    // innermost first: show as a pipeline input -> a -> b
    ops.reverse();
    format!("input -> {}", ops.join(" -> "))
}

/// One colour map that turns every grid in `outs` into its target, backed by evidence (E14): every changed colour is
/// seen in at least 2 pairs or on at least 3 cells. None if no such map, or if the map changes nothing.
fn fit_map(outs: &[&Grid], targets: &[&Grid]) -> Option<[u8; 10]> {
    let mut map = [255u8; 10];
    let mut cells = [0u32; 10];
    let mut pairs = [0u16; 10];
    for (a, b) in outs.iter().zip(targets) {
        if a.h != b.h || a.w != b.w { return None; }
        let mut in_pair = 0u16;
        for (x, y) in a.c.iter().zip(&b.c) {
            let m = &mut map[*x as usize];
            if *m == 255 { *m = *y; } else if *m != *y { return None; }
            cells[*x as usize] += 1; in_pair |= 1 << *x;
        }
        for c in 0..10 { if in_pair & (1 << c) != 0 { pairs[c] += 1; } }
    }
    if (0..10).all(|c| map[c] == 255 || map[c] as usize == c) { return None; }
    for c in 0..10 { if map[c] != 255 && map[c] as usize != c && pairs[c] < 2 && cells[c] < 3 { return None; } }
    Some(map)
}

/// Apply a colour map; None if the grid has a colour the map has never seen (it would not know what it becomes).
fn apply_map(g: &Grid, map: &[u8; 10]) -> Option<Grid> {
    if g.c.iter().any(|v| map[*v as usize] == 255) { return None; }
    let mut t = g.clone(); for v in t.c.iter_mut() { *v = map[*v as usize]; } Some(t)
}

/// Do these grids solve the task? Exact match, or a match up to one consistent, evidenced colour map (`fit_map`).
/// Returns test predictions and the map if used.
fn matches(task: &Task, out: &[Grid], allow_map: bool) -> Option<(Vec<Grid>, Option<[u8; 10]>)> {
    let n = task.train.len();
    if (0..n).all(|i| out[i] == task.train[i].1) { return Some((out[n..].to_vec(), None)); }
    if !allow_map { return None; }
    let outs: Vec<&Grid> = out[..n].iter().collect();
    let targets: Vec<&Grid> = task.train.iter().map(|p| &p.1).collect();
    let map = fit_map(&outs, &targets)?;
    let test: Option<Vec<Grid>> = out[n..].iter().map(|g| apply_map(g, &map)).collect();
    Some((test?, Some(map)))
}

/// Leave-one-out bookkeeping (E15). For each train pair i, the two cheapest distinct WRONG predictions for pair i made
/// by programs that fit every other train pair. If a search shown only the other pairs would have ranked two wrong
/// answers for pair i above the best answer found, the examples do not pin the rule down, and the answer is withheld.
struct Loo { wrong: Vec<Vec<(u64, u16)>> }

impl Loo {
    fn new(n: usize) -> Self { Loo { wrong: vec![Vec::new(); n] } }
    fn note_wrong(&mut self, i: usize, h: u64, cost: u16) {
        let w = &mut self.wrong[i];
        if let Some(e) = w.iter_mut().find(|e| e.0 == h) { if cost < e.1 { e.1 = cost; } }
        else { w.push((h, cost)); }
        w.sort_by_key(|e| e.1);
        w.truncate(2);
    }
    /// Record what a program (its train outputs `out`, total cost `cost`, length `depth`) says about each held-out pair.
    fn note(&mut self, task: &Task, out: &[Grid], cost: u16, depth: u8) {
        let n = task.train.len();
        if n < 2 { return; }
        let miss: Vec<usize> = (0..n).filter(|&i| out[i] != task.train[i].1).collect();
        if miss.len() == 1 { let i = miss[0]; self.note_wrong(i, out[i].hash64(), cost); }
        if depth > MAX_MAP_DEPTH || miss.is_empty() { return; }
        for i in 0..n {
            let outs: Vec<&Grid> = (0..n).filter(|&j| j != i).map(|j| &out[j]).collect();
            let targets: Vec<&Grid> = (0..n).filter(|&j| j != i).map(|j| &task.train[j].1).collect();
            if let Some(map) = fit_map(&outs, &targets) {
                if let Some(pred) = apply_map(&out[i], &map) {
                    if pred != task.train[i].1 { self.note_wrong(i, pred.hash64(), cost + 5); }
                }
            }
        }
    }
    /// The most wrong answers (0, 1 or 2) ranked above the best answer on any one held-out pair.
    fn margin(&self, best: u16) -> usize {
        self.wrong.iter().map(|w| w.iter().filter(|e| e.1 < best).count()).max().unwrap_or(0)
    }
    /// The first held-out pair on which the best answer (cost `best`) would have lost, if any.
    fn fails(&self, best: u16) -> Option<usize> {
        (0..self.wrong.len()).find(|&i| self.wrong[i].len() == 2 && self.wrong[i][1].1 < best)
    }
}

/// Add a candidate answer. One entry per distinct test answer, holding the cheapest program found for it (E14:
/// v1 kept the first one found, so a correct answer could carry a colour map's cost and drop out of the top two).
fn add_candidate(cands: &mut Vec<Candidate>, program: String, cost: u16, test: Vec<Grid>, found_at: usize) {
    if let Some(c) = cands.iter_mut().find(|c| c.test == test) {
        if cost < c.cost { c.cost = cost; c.program = program; }
        c.found_at = c.found_at.min(found_at);
    } else {
        cands.push(Candidate { program, cost, test, found_at });
    }
}

fn map_name(m: &[u8; 10]) -> String {
    let pairs: Vec<String> = (0..10).filter(|&c| m[c] != 255 && m[c] as usize != c).map(|c| format!("{}>{}", c, m[c])).collect();
    format!("MapColours({})", pairs.join(","))
}

fn hint_model(version: u8) -> Option<&'static crate::hints::Hints> {
    static M1: std::sync::OnceLock<Option<crate::hints::Hints>> = std::sync::OnceLock::new();
    static M2: std::sync::OnceLock<Option<crate::hints::Hints>> = std::sync::OnceLock::new();
    if version == 2 { M2.get_or_init(|| crate::hints::Hints::from_text(include_str!("hints2.json"))).as_ref() }
    else { M1.get_or_init(|| crate::hints::Hints::from_text(include_str!("hints.json"))).as_ref() }
}

/// Per-pass settings for `solve_inner`.
struct Pass<'a> { seconds: f64, bonus: Option<&'a HashMap<String, u16>>, collect: bool }

/// How close a program's train outputs are to the targets: mean share of equal cells (0 for a pair with the wrong size).
fn closeness(task: &Task, out: &[Grid]) -> f64 {
    let n = task.train.len();
    let mut sum = 0.0;
    for i in 0..n {
        let (a, b) = (&out[i], &task.train[i].1);
        if a.h != b.h || a.w != b.w { return 0.0; }
        sum += a.c.iter().zip(&b.c).filter(|(x, y)| x == y).count() as f64 / a.c.len() as f64;
    }
    sum / n as f64
}
fn chain_kinds(nodes: &[Node], mut i: usize) -> Vec<String> {
    let mut v = Vec::new();
    while let Some(op) = nodes[i].op { v.push(op.kind()); i = nodes[i].parent as usize; }
    v
}

pub fn solve(task: &Task, lim: &Limits) -> Outcome {
    if !lim.probe { return solve_inner(task, lim, &Pass { seconds: lim.seconds, bonus: None, collect: false }).0; }
    let (first, credit) = solve_inner(task, lim, &Pass { seconds: lim.seconds / 3.0, bonus: None, collect: true });
    if first.status == "solved" || first.status == "exhausted" { return first; }   // nothing left to find
    let bonus: HashMap<String, u16> = credit.iter().map(|(k, &c)| (k.clone(), (2.0 * (1.0 + c).log2()).round() as u16)).filter(|(_, b)| *b > 0).collect();
    let left = (lim.seconds - first.cpu_ms as f64 / 1000.0).max(1.0);
    let (mut second, _) = solve_inner(task, lim, &Pass { seconds: left, bonus: Some(&bonus), collect: false });
    let mut cands = first.candidates;
    for c in second.candidates.drain(..) { add_candidate(&mut cands, c.program, c.cost, c.test, first.nodes + c.found_at); }
    cands.sort_by_key(|c| c.cost); cands.truncate(2);
    second.status = if cands.is_empty() { second.status } else if second.status == "solved" || second.status == "exhausted" { "solved" } else { "solved-partial-search" };
    second.candidates = cands;
    second.nodes += first.nodes; second.cpu_ms += first.cpu_ms; second.ms += first.ms;
    second
}

fn solve_inner(task: &Task, lim: &Limits, pass: &Pass) -> (Outcome, HashMap<String, f64>) {
    let start = Instant::now();
    let mut credit: HashMap<String, f64> = HashMap::new();
    let cpu0 = thread_cpu_secs();
    let clock = if cpu0.is_some() { "thread-cpu" } else { "wall" };
    let used = || match cpu0 { Some(c0) => thread_cpu_secs().map(|c| c - c0).unwrap_or(f64::MAX), None => start.elapsed().as_secs_f64() };
    let mut since_check: u32 = 0;
    let mut palette_mask: u16 = 0;
    for (i, o) in &task.train { palette_mask |= i.colours() | o.colours(); }
    for t in &task.test { palette_mask |= t.colours(); }
    let palette: Vec<u8> = (0..10u8).filter(|c| palette_mask & (1 << c) != 0).collect();
    let ops = ops_for(&palette);
    let prior = if lim.prior { Some(prior_counts()) } else { None };
    let hint_p = if lim.hints || lim.prune > 0 { hint_model(lim.hints_version).map(|h| h.predict(task)) } else { None };
    let allowed: Vec<bool> = match (&hint_p, lim.prune) {
        (Some(p), k) if k > 0 => {
            let mut v: Vec<(&String, &f32)> = p.iter().collect();
            v.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
            let top: HashSet<String> = v.iter().take(k).map(|(name, _)| (*name).clone()).collect();
            ops.iter().map(|o| top.contains(&o.kind())).collect()
        }
        _ => vec![true; ops.len()],
    };
    let costs: Vec<u16> = ops.iter().map(|&o| {
        let mut c = op_cost(o, prior.as_ref());
        if let Some(p) = hint_p.as_ref().filter(|_| lim.hints).and_then(|m| m.get(&o.kind())) {
            let bonus = (2.0 * ((*p).max(1e-4) / 0.05).log2()).round().clamp(-3.0, 6.0) as i32;
            c = (c as i32 - bonus).clamp(2, 20) as u16;
        }
        match pass.bonus.and_then(|b| b.get(&o.kind())) { Some(&b) => c.saturating_sub(b).max(2), None => c }
    }).collect();

    let root: Vec<Grid> = task.train.iter().map(|p| p.0.clone()).chain(task.test.iter().cloned()).collect();
    let mut nodes = vec![Node { grids: root, parent: 0, op: None, cost: 0, depth: 0 }];
    let mut seen: HashSet<u64> = HashSet::new();
    seen.insert(combined_hash(&nodes[0].grids));
    let mut cands: Vec<Candidate> = Vec::new();
    let mut status = "exhausted";
    let mut loo = Loo::new(task.train.len());
    if lim.loo { loo.note(task, &nodes[0].grids, 0, 0); }
    let check = |nodes: &Vec<Node>, idx: usize, cands: &mut Vec<Candidate>, found_at: usize| {
        // A colour map only after a short program: long programs plus a map fit by accident (seen in tests).
        if let Some((test, map)) = matches(task, &nodes[idx].grids, nodes[idx].depth <= MAX_MAP_DEPTH) {
            let (mut prog, mut cost) = (program(nodes, idx), nodes[idx].cost);
            if let Some(m) = map { prog = format!("{} -> {}", prog, map_name(&m)); cost += 5; }
            add_candidate(cands, prog, cost, test, found_at);
        }
    };
    check(&nodes, 0, &mut cands, 1);

    let mut frontier: Vec<usize> = vec![0];
    let mut depth_reached = 0u8;
    let mut evaluated: usize = 0;
    if lim.best_first {
        // Uniform-cost search: pop the cheapest stored program, try every operation on it. Stop once two answers
        // are known and nothing cheaper than the second can still appear (children never cost less than parents).
        let mut heap: BinaryHeap<Reverse<(u16, usize)>> = BinaryHeap::new();
        heap.push(Reverse((0, 0)));
        'best: while let Some(Reverse((c, pi))) = heap.pop() {
            if cands.len() >= 2 {
                let mut cs: Vec<u16> = cands.iter().map(|x| x.cost).collect(); cs.sort();
                if c >= cs[1] { status = "solved"; break; }
            }
            let depth = nodes[pi].depth + 1;
            if depth > lim.max_depth { continue; }
            for (oi, &op) in ops.iter().enumerate() {
                if depth > lim.prune_from && !allowed[oi] { continue; }
                since_check += 1;
                if since_check >= 32 {
                    since_check = 0;
                    if used() > pass.seconds { status = "time"; break 'best; }
                    if start.elapsed().as_secs_f64() > pass.seconds * 4.0 + 30.0 { status = "wall"; break 'best; }
                }
                if nodes.len() >= lim.max_nodes || seen.len() >= 40_000_000 { status = "nodes"; break 'best; }
                let mut out = Vec::with_capacity(nodes[pi].grids.len());
                let mut ok = true;
                for g in &nodes[pi].grids { match apply(op, g) { Some(x) => out.push(x), None => { ok = false; break; } } }
                if !ok { continue; }
                let h = combined_hash(&out);
                if !seen.insert(h) { continue; }
                let cost = c + costs[oi];
                evaluated += 1;
                if depth > depth_reached { depth_reached = depth; }
                if lim.loo { loo.note(task, &out, cost, depth); }
                if pass.collect {
                    let sc = closeness(task, &out);
                    if sc >= 0.9 && sc < 1.0 { let mut ks = chain_kinds(&nodes, pi); ks.push(op.kind()); for k in ks { *credit.entry(k).or_insert(0.0) += sc; } }
                }
                if depth == lim.max_depth {
                    if let Some((test, map)) = matches(task, &out, depth <= MAX_MAP_DEPTH) {
                        let mut prog = format!("{} -> {}", program(&nodes, pi), op.name()).replace("input -> input", "input");
                        let mut c2 = cost;
                        if let Some(m) = map { prog = format!("{} -> {}", prog, map_name(&m)); c2 += 5; }
                        add_candidate(&mut cands, prog, c2, test, seen.len());
                    }
                    continue;
                }
                nodes.push(Node { grids: out, parent: pi as u32, op: Some(op), cost, depth });
                let idx = nodes.len() - 1;
                check(&nodes, idx, &mut cands, seen.len());
                heap.push(Reverse((cost, idx)));
            }
        }
        frontier.clear();
    }
    'outer: for depth in 1..=lim.max_depth {
        if lim.best_first { break; }
        // expand cheapest parents first
        frontier.sort_by_key(|&i| nodes[i].cost);
        let mut next = Vec::new();
        for &pi in &frontier {
            for (oi, &op) in ops.iter().enumerate() {
                if depth > lim.prune_from && !allowed[oi] { continue; }
                since_check += 1;
                if since_check >= 32 {
                    since_check = 0;
                    if used() > pass.seconds { status = "time"; break 'outer; }
                    if start.elapsed().as_secs_f64() > pass.seconds * 4.0 + 30.0 { status = "wall"; break 'outer; }
                }
                if nodes.len() >= lim.max_nodes && depth < lim.max_depth { status = "nodes"; break 'outer; }
                if seen.len() >= 40_000_000 { status = "nodes"; break 'outer; }
                let mut out = Vec::with_capacity(nodes[pi].grids.len());
                let mut ok = true;
                for g in &nodes[pi].grids { match apply(op, g) { Some(x) => out.push(x), None => { ok = false; break; } } }
                if !ok { continue; }
                let h = combined_hash(&out);
                if !seen.insert(h) { continue; }
                let cost = nodes[pi].cost + costs[oi];
                evaluated += 1;
                if lim.loo { loo.note(task, &out, cost, depth); }
                if pass.collect {
                    let sc = closeness(task, &out);
                    if sc >= 0.9 && sc < 1.0 { let mut ks = chain_kinds(&nodes, pi); ks.push(op.kind()); for k in ks { *credit.entry(k).or_insert(0.0) += sc; } }
                }
                if depth == lim.max_depth {
                    // last level: check only, do not keep (saves most of the memory)
                    if let Some((test, map)) = matches(task, &out, depth <= MAX_MAP_DEPTH) {
                        let mut prog = format!("{} -> {}", program(&nodes, pi), op.name()).replace("input -> input", "input");
                        let mut c2 = cost;
                        if let Some(m) = map { prog = format!("{} -> {}", prog, map_name(&m)); c2 += 5; }
                        add_candidate(&mut cands, prog, c2, test, seen.len());
                    }
                    continue;
                }
                nodes.push(Node { grids: out, parent: pi as u32, op: Some(op), cost, depth });
                let idx = nodes.len() - 1;
                check(&nodes, idx, &mut cands, seen.len());
                next.push(idx);
            }
        }
        depth_reached = depth;
        if cands.len() >= 2 { status = "solved"; break; }
        frontier = next;
        if frontier.is_empty() { break; }
    }
    if !cands.is_empty() && status != "solved" { status = if status == "exhausted" { "solved" } else { "solved-partial-search" }; }
    cands.sort_by_key(|c| c.cost);
    cands.truncate(2);
    let _ = evaluated;
    let (mut loo_state, mut loo_pair, mut withheld) = ("off", None, Vec::new());
    let loo_margin = if lim.loo && !cands.is_empty() { loo.margin(cands[0].cost) } else { 0 };
    if lim.loo {
        if task.train.len() < 2 || cands.is_empty() { loo_state = "n/a"; }
        else if let Some(i) = loo.fails(cands[0].cost) { loo_state = "fail"; loo_pair = Some(i); withheld = std::mem::take(&mut cands); }
        else { loo_state = "pass"; }
    }
    (Outcome { candidates: cands, nodes: seen.len(), ms: start.elapsed().as_millis(), cpu_ms: (used() * 1000.0) as u128, clock, status, depth_reached, loo: loo_state, loo_pair, loo_margin, withheld }, credit)
}
