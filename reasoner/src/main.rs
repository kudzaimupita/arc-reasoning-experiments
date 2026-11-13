//! reasoner: DSL program search for ARC tasks (Experiments 12 to 24). No dependencies.
//!   --cpu-budget is CPU seconds of the thread solving the task (wall-clock before Experiment 16).
//!   reasoner solve <task.json> [--cpu-budget S] [--node-budget N] [--max-depth D] [--loo]   one task, JSON to stdout
//!   --best-first expands the cheapest program next; --prior makes operation kinds seen in correct train-pool
//!   programs cheaper (Experiment 17).
//!   --probe: a first pass on a third of the budget, then a restart that favours operations from near misses (E18).
//!   --prune-from D: prune only below depth D (E21). --hints-version 2: weights from src/hints2.json (E22).
//!   --prune K: below depth 1, only operations whose kind is in the hint network's top K (E20).
//!   --hints: operation costs from the hint network in src/hints.json (E19).
//!   --loo withholds an answer when the examples do not pin the rule down (leave-one-out check, Experiment 15).
//!   reasoner job [--cpu-budget S] [--node-budget N] [--max-depth D] [--threads T]     a whole batch of tasks:
//!       reads $ARC_JOB_IN/tasks.txt and $ARC_JOB_IN/<id>.json (test outputs may be stripped), writes
//!       $ARC_JOB_OUT/out/<id>.json (predictions, attempt_1/attempt_2 per test input) and
//!       $ARC_JOB_OUT/results.jsonl (status, nodes, ms, programs). ARC_JOB_OUT defaults to the current folder.
//!       Unsolved tasks get no attempt: an honest "don't know".
//!   reasoner dream --count N --seed S [--threads T] [--weighted] [--long]: random programs on the input grids of the
//!       tasks in $ARC_JOB_IN; writes $ARC_JOB_OUT/dreams-<thread>.jsonl ({"f": features, "k": kind indices}) and
//!       $ARC_JOB_OUT/kinds.json (E19). --weighted --long: kind-weighted operations, lengths 1-5 (E22).
//!   reasoner features <task.json>: the 64 hint features of a task's train pairs (E22).
//!   reasoner hintrank <task.json>: the hint network's probability for each operation kind (E19).
mod dsl; mod grid; mod hints; mod json; mod search;
use grid::Grid;
use json::{esc, Json};
use search::{solve, Limits, Outcome, Task};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

fn arg<T: std::str::FromStr>(args: &[String], name: &str, default: T) -> T {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn load_task(text: &str) -> Result<Task, String> {
    let j = json::parse(text)?;
    let train = j.get("train").and_then(|t| t.arr()).ok_or("no train")?;
    let test = j.get("test").and_then(|t| t.arr()).ok_or("no test")?;
    let mut tr = Vec::new();
    for p in train {
        let i = p.get("input").and_then(Grid::from_json).ok_or("bad train input")?;
        let o = p.get("output").and_then(Grid::from_json).ok_or("bad train output")?;
        tr.push((i, o));
    }
    let mut te = Vec::new();
    for p in test { te.push(p.get("input").and_then(Grid::from_json).ok_or("bad test input")?); }
    if tr.is_empty() || te.is_empty() { return Err("empty task".into()); }
    Ok(Task { train: tr, test: te })
}

fn predictions_json(out: &Outcome, n_test: usize) -> String {
    let mut s = String::from("[");
    for t in 0..n_test {
        if t > 0 { s.push(','); }
        s.push('{');
        for (a, c) in out.candidates.iter().enumerate() { if a > 0 { s.push(','); } s.push_str(&format!("\"attempt_{}\":{}", a + 1, c.test[t].to_json())); }
        s.push('}');
    }
    s.push(']'); s
}

fn summary_json(id: &str, out: &Outcome) -> String {
    let progs: Vec<String> = out.candidates.iter().map(|c| esc(&c.program)).collect();
    let held: Vec<String> = out.withheld.iter().map(|c| esc(&c.program)).collect();
    let pair = out.loo_pair.map(|i| i.to_string()).unwrap_or_else(|| "null".into());
    format!("{{\"task\":{},\"status\":{},\"solved\":{},\"nodes\":{},\"ms\":{},\"cpu_ms\":{},\"clock\":{},\"depth\":{},\"programs\":[{}],\"found_at\":[{}],\"loo\":{},\"loo_pair\":{},\"loo_margin\":{},\"withheld\":[{}]}}",
        esc(id), esc(out.status), !out.candidates.is_empty(), out.nodes, out.ms, out.cpu_ms, esc(out.clock), out.depth_reached, progs.join(","), out.candidates.iter().map(|c| c.found_at.to_string()).collect::<Vec<_>>().join(","), esc(out.loo), pair, out.loo_margin, held.join(","))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let lim = Limits { seconds: arg(&args, "--cpu-budget", 60.0), max_nodes: arg(&args, "--node-budget", 150_000usize), max_depth: arg(&args, "--max-depth", 4u8), loo: args.iter().any(|a| a == "--loo"), best_first: args.iter().any(|a| a == "--best-first"), prior: args.iter().any(|a| a == "--prior"), probe: args.iter().any(|a| a == "--probe"), hints: args.iter().any(|a| a == "--hints"), prune: arg(&args, "--prune", 0usize), prune_from: arg(&args, "--prune-from", 1u8), hints_version: arg(&args, "--hints-version", 1u8) };
    match args.get(1).map(|s| s.as_str()) {
        Some("solve") => {
            let path = args.get(2).expect("usage: reasoner solve <task.json>");
            let text = std::fs::read_to_string(path).expect("cannot read task");
            let task = load_task(&text).unwrap_or_else(|e| { eprintln!("bad task: {e}"); std::process::exit(2) });
            let out = solve(&task, &lim);
            println!("{}", summary_json(path, &out));
            println!("{}", predictions_json(&out, task.test.len()));
        }
        Some("job") => {
            let in_dir = std::env::var("ARC_JOB_IN").unwrap_or_else(|_| { eprintln!("set ARC_JOB_IN to a folder with tasks.txt and <id>.json"); std::process::exit(2) });
            let home = std::env::var("ARC_JOB_OUT").unwrap_or_else(|_| ".".into());
            let out_dir = format!("{home}/out");
            std::fs::create_dir_all(&out_dir).expect("cannot create out");
            let ids: Vec<String> = std::fs::read_to_string(format!("{in_dir}/tasks.txt")).expect("tasks.txt").lines()
                .map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
            for id in &ids { if id.len() > 64 || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') { eprintln!("bad task id"); std::process::exit(2); } }
            let threads: usize = arg(&args, "--threads", 8usize).clamp(1, 12);
            let next = Arc::new(AtomicUsize::new(0));
            let ids = Arc::new(ids);
            let results = Arc::new(Mutex::new(Vec::<String>::new()));
            let mut hs = Vec::new();
            for _ in 0..threads {
                let (next, ids, results, in_dir, out_dir) = (next.clone(), ids.clone(), results.clone(), in_dir.clone(), out_dir.clone());
                let lim = Limits { seconds: lim.seconds, max_nodes: lim.max_nodes, max_depth: lim.max_depth, loo: lim.loo, best_first: lim.best_first, prior: lim.prior, probe: lim.probe, hints: lim.hints, prune: lim.prune, prune_from: lim.prune_from, hints_version: lim.hints_version };
                hs.push(std::thread::spawn(move || loop {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= ids.len() { break; }
                    let id = &ids[i];
                    let line = match std::fs::read_to_string(format!("{in_dir}/{id}.json")).map_err(|e| e.to_string()).and_then(|t| load_task(&t)) {
                        Ok(task) => {
                            let out = solve(&task, &lim);
                            std::fs::write(format!("{out_dir}/{id}.json"), predictions_json(&out, task.test.len())).expect("write prediction");
                            summary_json(id, &out)
                        }
                        Err(e) => format!("{{\"task\":{},\"status\":\"bad-task\",\"error\":{}}}", esc(id), esc(&e)),
                    };
                    results.lock().unwrap().push(line);
                }));
            }
            for h in hs { h.join().expect("worker"); }
            let lines = results.lock().unwrap();
            std::fs::write(format!("{home}/results.jsonl"), lines.join("\n") + "\n").expect("write results");
            let solved = lines.iter().filter(|l| l.contains("\"solved\":true")).count();
            println!("reasoner: {} tasks, {} with a candidate", lines.len(), solved);
        }
        Some("features") => {
            // E22: print the hint features of one task (train pairs only), for training on real solutions.
            let path = args.get(2).expect("usage: reasoner features <task.json>");
            let task = load_task(&std::fs::read_to_string(path).expect("read")).expect("task");
            println!("[{}]", hints::features(&task.train).iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>().join(","));
        }
        Some("hintrank") => {
            // Diagnostic (E19): the hint network's probability for each kind on one task, highest first.
            let path = args.get(2).expect("usage: reasoner hintrank <task.json>");
            let task = load_task(&std::fs::read_to_string(path).expect("read")).expect("task");
            let h = hints::Hints::load().expect("no hints compiled in");
            let mut v: Vec<(String, f32)> = h.predict(&task).into_iter().collect();
            v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            println!("[{}]", v.iter().map(|(k, p)| format!("[{},{:.5}]", esc(k), p)).collect::<Vec<_>>().join(","));
        }
        Some("dream") => {
            let in_dir = std::env::var("ARC_JOB_IN").expect("ARC_JOB_IN");
            let home = std::env::var("ARC_JOB_OUT").unwrap_or_else(|_| ".".into());
            std::fs::create_dir_all(&home).expect("cannot create ARC_JOB_OUT");
            let ids: Vec<String> = std::fs::read_to_string(format!("{in_dir}/tasks.txt")).expect("tasks.txt").lines()
                .map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
            let mut pools: Vec<Vec<Grid>> = Vec::new();
            for id in &ids {
                if id.len() > 64 || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') { continue; }
                if let Ok(t) = std::fs::read_to_string(format!("{in_dir}/{id}.json")).map_err(|e| e.to_string()).and_then(|t| load_task(&t)) {
                    pools.push(t.train.iter().map(|p| p.0.clone()).chain(t.test.iter().cloned()).collect());
                }
            }
            let vocab_list = hints::kind_vocab();
            let vocab: std::collections::HashMap<String, usize> = vocab_list.iter().cloned().enumerate().map(|(i, k)| (k, i)).collect();
            std::fs::write(format!("{home}/kinds.json"), format!("[{}]", vocab_list.iter().map(|k| esc(k)).collect::<Vec<_>>().join(","))).expect("kinds");
            let count: usize = arg(&args, "--count", 100_000usize);
            let seed: u64 = arg(&args, "--seed", 1u64);
            let threads: usize = arg(&args, "--threads", 8usize).clamp(1, 64);
            let weighted = args.iter().any(|a| a == "--weighted");
            let long = args.iter().any(|a| a == "--long");
            let weights = Arc::new(if weighted { Some(search::prior_counts()) } else { None });
            let pools = Arc::new(pools); let vocab = Arc::new(vocab);
            let mut hs = Vec::new();
            for t in 0..threads {
                let (pools, vocab, home, weights) = (pools.clone(), vocab.clone(), home.clone(), weights.clone());
                hs.push(std::thread::spawn(move || {
                    let mut rng = hints::Rng::new(seed * 1000 + t as u64);
                    let mut out = String::new(); let mut made = 0usize; let want = count / threads;
                    while made < want {
                        let p = &pools[rng.below(pools.len())];
                        if p.len() < 2 { continue; }
                        let k = (2 + rng.below(3)).min(p.len());
                        let start = rng.below(p.len() - k + 1);
                        if let Some((f, kinds)) = hints::dream(&p[start..start + k], &mut rng, &vocab, weights.as_ref().as_ref(), long) {
                            let fs: Vec<String> = f.iter().map(|x| format!("{:.4}", x)).collect();
                            let ks: Vec<String> = kinds.iter().map(|x| x.to_string()).collect();
                            out.push_str(&format!("{{\"f\":[{}],\"k\":[{}]}}\n", fs.join(","), ks.join(",")));
                            made += 1;
                        }
                    }
                    std::fs::write(format!("{home}/dreams-{t}.jsonl"), out).expect("write dreams");
                }));
            }
            for h in hs { h.join().expect("worker"); }
            println!("reasoner: {} dreams", count / threads * threads);
        }
        _ => { eprintln!("usage: reasoner solve <task.json> | job | dream | features <task.json> | hintrank <task.json> (see src/main.rs)"); std::process::exit(64); }
    }
}
