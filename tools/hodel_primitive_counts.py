"""Count how often each Hodel DSL primitive is used by the solvers of the 320 train-pool tasks.
Static text analysis of arc-dsl (github.com/michaelhodel/arc-dsl; nothing is run). A primitive counts when its name
appears as a whole word in a solver body: called, or passed to another primitive (e.g. apply(first, x)).
Writes reports/hodel-primitive-counts.csv: primitive, uses, solvers_using.
Usage: python3 tools/hodel_primitive_counts.py <arc-dsl folder> <file of train-pool task ids> [repo root]"""
import csv, re, sys
from collections import Counter

DSL_DIR, IDS = sys.argv[1], sys.argv[2]
ROOT = sys.argv[3] if len(sys.argv) > 3 else "."
dsl = open(f"{DSL_DIR}/dsl.py").read()
prims = re.findall(r"^def (\w+)\(", dsl, re.M)
train = {l.strip() for l in open(IDS) if l.strip()}
src = open(f"{DSL_DIR}/solvers.py").read()
blocks = re.split(r"^def solve_", src, flags=re.M)[1:]
uses, solvers = Counter(), Counter()
lengths = []
for b in blocks:
    tid, body = b[:8], b[b.index("\n"):]
    if tid not in train:
        continue
    words = re.findall(r"\b[a-z_][a-z0-9_]*\b", body)
    seen = set()
    for w in words:
        if w in prims:
            uses[w] += 1
            seen.add(w)
    for w in seen:
        solvers[w] += 1
    lengths.append(len(re.findall(r"^\s+(x\d+|O) = ", body, re.M)))
with open(f"{ROOT}/reports/hodel-primitive-counts.csv", "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["primitive", "uses", "solvers_using"])
    for p in sorted(prims, key=lambda p: (-uses[p], p)):
        w.writerow([p, uses[p], solvers[p]])
used = sum(1 for p in prims if uses[p])
lengths.sort()
print(f"{len(lengths)} train solvers; {len(prims)} primitives, {used} used, {len(prims) - used} unused")
print(f"steps per solver: median {lengths[len(lengths)//2]}, max {lengths[-1]}, "
      f"<=3 steps {sum(l <= 3 for l in lengths)}, <=5 {sum(l <= 5 for l in lengths)}, <=10 {sum(l <= 10 for l in lengths)}")
print("top 12:", ", ".join(f"{p} {uses[p]}" for p in sorted(prims, key=lambda p: -uses[p])[:12]))
