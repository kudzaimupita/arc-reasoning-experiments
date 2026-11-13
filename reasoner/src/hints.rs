//! Hint network support (Experiment 19): task features, "dreams" (random programs run on train-pool inputs, giving
//! synthetic tasks with known operation kinds), and a small MLP that predicts which kinds a task needs.
//! The MLP is trained outside (tools/train_hints.py) and compiled in from src/hints.json.
use crate::dsl::{apply, ops_for, Op};
use crate::grid::Grid;
use crate::search::Task;
use std::collections::HashMap;

/// Every operation kind the hint network scores (the kinds `ops_for` can produce), in a fixed order.
pub fn kind_vocab() -> Vec<String> {
    let mut v: Vec<String> = ops_for(&(0..10).collect::<Vec<u8>>()).iter().map(|o| o.kind()).collect();
    v.sort(); v.dedup(); v
}

fn flip_h(g: &Grid) -> Grid { let mut o = g.clone(); for r in 0..g.h { for k in 0..g.w { o.set(r, k, g.at(r, g.w - 1 - k)); } } o }
fn flip_v(g: &Grid) -> Grid { let mut o = g.clone(); for r in 0..g.h { for k in 0..g.w { o.set(r, k, g.at(g.h - 1 - r, k)); } } o }
fn transpose(g: &Grid) -> Grid { let mut o = Grid::new(g.w, g.h, 0); for r in 0..g.h { for k in 0..g.w { o.set(k, r, g.at(r, k)); } } o }
fn objects_count(g: &Grid) -> usize {
    let bg = g.bg(); let mut seen = vec![false; g.h * g.w]; let mut n = 0;
    for s in 0..g.h * g.w {
        if seen[s] || g.c[s] == bg { continue; }
        n += 1; seen[s] = true; let mut st = vec![s];
        while let Some(p) = st.pop() {
            let (r, k) = (p / g.w, p % g.w);
            let mut nb = Vec::with_capacity(4);
            if r > 0 { nb.push(p - g.w); } if r + 1 < g.h { nb.push(p + g.w); } if k > 0 { nb.push(p - 1); } if k + 1 < g.w { nb.push(p + 1); }
            for q in nb { if !seen[q] && g.c[q] != bg { seen[q] = true; st.push(q); } }
        }
    }
    n
}
fn is_subgrid(small: &Grid, big: &Grid) -> bool {
    if small.h > big.h || small.w > big.w || (small.h == big.h && small.w == big.w) { return false; }
    for r0 in 0..=big.h - small.h { for k0 in 0..=big.w - small.w {
        if (0..small.h).all(|r| (0..small.w).all(|k| small.at(r, k) == big.at(r0 + r, k0 + k))) { return true; }
    } }
    false
}
fn sym_h(g: &Grid) -> bool { (0..g.h).all(|r| (0..g.w).all(|k| g.at(r, k) == g.at(r, g.w - 1 - k))) }
fn sym_v(g: &Grid) -> bool { (0..g.h).all(|r| (0..g.w).all(|k| g.at(r, k) == g.at(g.h - 1 - r, k))) }
fn hist(g: &Grid) -> [f32; 10] { let mut h = [0f32; 10]; for &v in &g.c { h[v as usize] += 1.0; } let n = g.c.len() as f32; for x in h.iter_mut() { *x /= n; } h }
fn ncol(g: &Grid) -> f32 { g.colours().count_ones() as f32 }

pub const NFEAT: usize = 64;

/// A fixed-length description of a task's train pairs (averaged over pairs). Uses train pairs only.
pub fn features(pairs: &[(Grid, Grid)]) -> Vec<f32> {
    let mut f = vec![0f32; NFEAT];
    let n = pairs.len().max(1) as f32;
    for (a, b) in pairs {
        let same = a.h == b.h && a.w == b.w;
        let mut v: Vec<f32> = Vec::with_capacity(NFEAT);
        v.push(same as u8 as f32);
        v.push((b.h as f32 / a.h as f32).min(4.0) / 4.0);
        v.push((b.w as f32 / a.w as f32).min(4.0) / 4.0);
        v.push(((b.h * b.w) < (a.h * a.w)) as u8 as f32);
        v.push(((b.h * b.w) > (a.h * a.w)) as u8 as f32);
        v.push((b.h == 1 && b.w == 1) as u8 as f32);
        v.push((b.h % a.h == 0 && b.w % a.w == 0 && !same) as u8 as f32);
        v.push((a.h % b.h == 0 && a.w % b.w == 0 && !same) as u8 as f32);
        v.push(ncol(a) / 10.0);
        v.push(ncol(b) / 10.0);
        v.push((b.colours() & !a.colours()).count_ones() as f32 / 10.0);
        v.push((a.colours() & !b.colours()).count_ones() as f32 / 10.0);
        v.push((a.bg() == b.bg()) as u8 as f32);
        let (chg, from_bg) = if same {
            let bg = a.bg(); let mut c = 0; let mut fb = 0;
            for (x, y) in a.c.iter().zip(&b.c) { if x != y { c += 1; if *x == bg { fb += 1; } } }
            (c as f32 / a.c.len() as f32, if c > 0 { fb as f32 / c as f32 } else { 0.0 })
        } else { (1.0, 0.0) };
        v.push(chg); v.push(from_bg);
        v.push((same && *b == flip_h(a)) as u8 as f32);
        v.push((same && *b == flip_v(a)) as u8 as f32);
        let t = transpose(a);
        v.push((t.h == b.h && t.w == b.w && t == *b) as u8 as f32);
        v.push(is_subgrid(b, a) as u8 as f32);
        v.push(is_subgrid(a, b) as u8 as f32);
        v.push(sym_h(a) as u8 as f32); v.push(sym_v(a) as u8 as f32);
        v.push(sym_h(b) as u8 as f32); v.push(sym_v(b) as u8 as f32);
        let (oa, ob) = (objects_count(a) as f32, objects_count(b) as f32);
        v.push((1.0 + oa).ln() / 4.0); v.push((1.0 + ob).ln() / 4.0);
        v.push((ob < oa) as u8 as f32); v.push((ob > oa) as u8 as f32); v.push((ob == oa) as u8 as f32);
        v.push((a.h as f32) / 30.0); v.push((a.w as f32) / 30.0); v.push((b.h as f32) / 30.0); v.push((b.w as f32) / 30.0);
        let (ha, hb) = (hist(a), hist(b));
        v.extend_from_slice(&ha); v.extend_from_slice(&hb);
        // pad / trim to NFEAT
        v.resize(NFEAT, 0.0);
        for i in 0..NFEAT { f[i] += v[i] / n; }
    }
    // constant output size across pairs
    if pairs.len() >= 2 { f[NFEAT - 1] = pairs.windows(2).all(|w| w[0].1.h == w[1].1.h && w[0].1.w == w[1].1.w) as u8 as f32; }
    f
}

/// Tiny xorshift generator (no dependencies).
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self { Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1) }
    pub fn next(&mut self) -> u64 { let mut x = self.0; x ^= x << 13; x ^= x >> 7; x ^= x << 17; self.0 = x; x }
    pub fn below(&mut self, n: usize) -> usize { (self.next() % n as u64) as usize }
}

/// One dream: a random program of 1 to 3 operations applied to some train-pool inputs. Returns (features, kind
/// indices), or None if the program fails, changes nothing, or gives the same output for every input.
pub fn dream(inputs: &[Grid], rng: &mut Rng, vocab: &HashMap<String, usize>, weights: Option<&HashMap<String, u32>>, long: bool) -> Option<(Vec<f32>, Vec<usize>)> {
    let mut mask = 0u16; for g in inputs { mask |= g.colours(); }
    let palette: Vec<u8> = (0..10u8).filter(|c| mask & (1 << c) != 0).collect();
    let ops = ops_for(&palette);
    // E22: lengths 1-5 shaped like real programs (25/30/25/12/8 %); otherwise 1-3 uniform (E19).
    let len = if long { match rng.below(100) { 0..=24 => 1, 25..=54 => 2, 55..=79 => 3, 80..=91 => 4, _ => 5 } } else { 1 + rng.below(3) };
    // E22: operations drawn in proportion to 1 + 4 x (times their kind appears in correct train-pool programs).
    let cum: Option<Vec<u64>> = weights.map(|w| {
        let mut acc = 0u64; ops.iter().map(|o| { acc += 1 + 4 * *w.get(&o.kind()).unwrap_or(&0) as u64; acc }).collect()
    });
    let pick = |rng: &mut Rng| -> Op {
        match &cum { Some(c) => { let t = rng.next() % c[c.len() - 1]; ops[c.partition_point(|&x| x <= t)] } None => ops[rng.below(ops.len())] }
    };
    let prog: Vec<Op> = (0..len).map(|_| pick(rng)).collect();
    let mut outs = Vec::with_capacity(inputs.len());
    for g in inputs {
        let mut x = g.clone();
        for &op in &prog { x = apply(op, &x)?; }
        outs.push(x);
    }
    if inputs.iter().zip(&outs).all(|(a, b)| a == b) { return None; }
    if outs.windows(2).all(|w| w[0] == w[1]) { return None; }
    let pairs: Vec<(Grid, Grid)> = inputs.iter().cloned().zip(outs).collect();
    let mut kinds: Vec<usize> = prog.iter().filter_map(|o| vocab.get(&o.kind()).copied()).collect();
    kinds.sort(); kinds.dedup();
    Some((features(&pairs), kinds))
}

/// The trained MLP: features -> one logit per kind. Weights from src/hints.json (empty file = no hints).
pub struct Hints { kinds: Vec<String>, layers: Vec<(Vec<f32>, Vec<f32>, usize, usize)> }
impl Hints {
    pub fn load() -> Option<Hints> { Self::from_text(include_str!("hints.json")) }
    pub fn from_text(text: &str) -> Option<Hints> {
        if text.trim().is_empty() { return None; }
        let j = crate::json::parse(text).ok()?;
        let kinds: Vec<String> = j.get("kinds")?.arr()?.iter().filter_map(|k| k.str().map(|s| s.to_string())).collect();
        let mut layers = Vec::new();
        for l in j.get("layers")?.arr()? {
            let rows = l.get("rows")?.num()? as usize; let cols = l.get("cols")?.num()? as usize;
            let w: Vec<f32> = l.get("w")?.arr()?.iter().filter_map(|x| x.num().map(|v| v as f32)).collect();
            let b: Vec<f32> = l.get("b")?.arr()?.iter().filter_map(|x| x.num().map(|v| v as f32)).collect();
            if w.len() != rows * cols || b.len() != rows { return None; }
            layers.push((w, b, rows, cols));
        }
        Some(Hints { kinds, layers })
    }
    /// Probability per kind for a task (sigmoid of the last layer; ReLU between layers).
    pub fn predict(&self, task: &Task) -> HashMap<String, f32> {
        let mut x = features(&task.train);
        for (li, (w, b, rows, cols)) in self.layers.iter().enumerate() {
            let mut y = b.clone();
            for r in 0..*rows { let mut s = 0f32; for c in 0..*cols { s += w[r * cols + c] * x[c]; } y[r] += s; }
            if li + 1 < self.layers.len() { for v in y.iter_mut() { if *v < 0.0 { *v = 0.0; } } }
            x = y;
        }
        self.kinds.iter().cloned().zip(x.iter().map(|z| 1.0 / (1.0 + (-z).exp()))).collect()
    }
}
