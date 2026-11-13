#!/usr/bin/env python3
"""Count operation kinds in correct TRAIN-pool programs (Experiments 12 to 16) and write reasoner/src/prior.txt.

One program per solved task: the latest experiment that solved it. Select-pool results are never read, so the
prior holds no select information. Kind: the operation name without colour arguments; Sel keeps its action,
parsing and predicate, e.g. Sel(Recolour,Multi,Holed).
"""
import json, re, collections
RUNS = [  # (results file, grade file), oldest first
    ("experiments/e12-reasoner-v0/results-train.jsonl", "experiments/e12-reasoner-v0/grades/train.json"),
    ("experiments/e13-dsl-v1/results-train.jsonl", "experiments/e13-dsl-v1/grades/train.json"),
    ("experiments/e14-ranking-colour-maps/results-train.jsonl", "experiments/e14-ranking-colour-maps/grades/train.json"),
    ("experiments/e16-dsl-v1-part2/results-remote-part2-train.jsonl", "experiments/e16-dsl-v1-part2/grades/part2-train.json"),
]
def kind(step):
    m = re.match(r"Sel\((\w+)(?:\(\d+\))?, (\w+), (\w+)\)$", step)
    if m: return f"Sel({m[1]},{m[2]},{m[3]})"
    return step.split("(")[0]
best = {}
for res, gr in RUNS:
    passed = {t["task"] for t in json.load(open(gr))["tasks"] if t["pass"]}
    for line in open(res):
        if not line.strip(): continue
        d = json.loads(line)
        if d["task"] in passed and d.get("programs"): best[d["task"]] = d["programs"][0]
counts = collections.Counter()
for prog in best.values():
    for step in prog.split(" -> ")[1:]:
        if step.startswith("MapColours"): continue
        counts[kind(step)] += 1
with open("reasoner/src/prior.txt", "w") as f:
    f.write(f"# operation-kind counts in {len(best)} correct train-pool programs (tools/make_prior.py)\n")
    for k, n in counts.most_common(): f.write(f"{k} {n}\n")
print(len(best), "programs;", len(counts), "kinds;", counts.most_common(8))
