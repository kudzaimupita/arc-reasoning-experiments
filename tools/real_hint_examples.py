#!/usr/bin/env python3
"""Real training examples for the hint network (Experiment 22): for every TRAIN-pool task the reasoner has solved
(Experiments 12 to 20), the task's features and the kinds in its correct program. Select-pool results are never read.
Usage: ARC_TRAIN_DIR=<ARC-AGI-1 data/training folder> python3 tools/real_hint_examples.py <kinds.json> <out.jsonl>"""
import json, os, re, subprocess, sys
TRAIN_DIR = os.environ.get("ARC_TRAIN_DIR", "data/arc1/training")
RUNS = [("experiments/e12-reasoner-v0/results-train.jsonl", "experiments/e12-reasoner-v0/grades/train.json"),
        ("experiments/e13-dsl-v1/results-train.jsonl", "experiments/e13-dsl-v1/grades/train.json"),
        ("experiments/e14-ranking-colour-maps/results-train.jsonl", "experiments/e14-ranking-colour-maps/grades/train.json"),
        ("experiments/e16-dsl-v1-part2/results-remote-part2-train.jsonl", "experiments/e16-dsl-v1-part2/grades/part2-train.json"),
        ("experiments/e20-hint-pruning/results-p30-train.jsonl", "experiments/e20-hint-pruning/grades/p30-train.json"),
        ("experiments/e20-hint-pruning/results-p15d5-train.jsonl", "experiments/e20-hint-pruning/grades/p15d5-train.json")]
def kind(step):
    m = re.match(r"Sel\((\w+)(?:\(\d+\))?, (\w+), (\w+)\)$", step)
    return f"Sel({m[1]},{m[2]},{m[3]})" if m else step.split("(")[0]
vocab = {k: i for i, k in enumerate(json.load(open(sys.argv[1])))}
progs = {}
for res, gr in RUNS:
    passed = {t["task"] for t in json.load(open(gr))["tasks"] if t["pass"]}
    for line in open(res):
        d = json.loads(line)
        if d["task"] in passed and d.get("programs"): progs.setdefault(d["task"], set()).add(d["programs"][0])
n = 0
with open(sys.argv[2], "w") as out:
    for task, ps in sorted(progs.items()):
        f = json.loads(subprocess.run(["./reasoner/target/release/reasoner", "features", f"{TRAIN_DIR}/{task}.json"], capture_output=True, text=True, check=True).stdout)
        for p in ps:
            ks = sorted({vocab[kind(s)] for s in p.split(" -> ")[1:] if not s.startswith("MapColours") and kind(s) in vocab})
            if ks: out.write(json.dumps({"f": f, "k": ks}) + "\n"); n += 1
print(f"{len(progs)} solved train tasks, {n} real examples")
