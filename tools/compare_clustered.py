#!/usr/bin/env python3
"""Paired comparison of two runs, bootstrapping clusters (task IDs <cluster>_<n>, e.g. RE-ARC generators).
Usage: python3 tools/compare_clustered.py <name A> <grade A> -- <name B> <grade B>   (10,000 resamples, seed 12)"""
import json, random, sys, collections
a = sys.argv[1:]; i = a.index("--")
na, ga, nb, gb = a[0], a[1], a[i + 1], a[i + 2]
A = {t["task"]: t["pass"] for t in json.load(open(ga))["tasks"]}
B = {t["task"]: t["pass"] for t in json.load(open(gb))["tasks"]}
cl = collections.defaultdict(list)
for t in A:
    if t in B: cl[t.split("_")[0]].append(int(A[t]) - int(B[t]))
keys = sorted(cl); n = sum(len(v) for v in cl.values())
diff = sum(sum(v) for v in cl.values()) / n
rnd = random.Random(12); bs = []
for _ in range(10000):
    s = [cl[keys[rnd.randrange(len(keys))]] for _ in keys]
    bs.append(sum(map(sum, s)) / sum(map(len, s)))
bs.sort()
print(f"tasks {n} in {len(keys)} clusters; {na} {sum(A[t] for t in A if t in B)} solved, {nb} {sum(B[t] for t in B if t in A)} solved")
print(f"difference {na} - {nb}: {100*diff:+.1f}%  95% CI [{100*bs[250]:+.1f}%, {100*bs[9750]:+.1f}%] (cluster bootstrap)")
