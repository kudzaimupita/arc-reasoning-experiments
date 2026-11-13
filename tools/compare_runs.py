"""Paired comparison of two systems' grade.json files on the same tasks.
usage: python3 tools/compare_runs.py <name A> <grade A>... -- <name B> <grade B>...
Each side may list several grade.json files (e.g. train and select). Prints solve counts, the paired
difference with a 95% bootstrap interval (10,000 resamples, seed 12), and the task lists."""
import json, random, sys

def load(files):
    out = {}
    for f in files:
        for t in json.load(open(f))["tasks"]:
            out[t["task"]] = bool(t["pass"])
    return out

args = sys.argv[1:]
cut = args.index("--")
name_a, files_a, name_b, files_b = args[0], args[1:cut], args[cut + 1], args[cut + 2:]
a, b = load(files_a), load(files_b)
tasks = sorted(set(a) & set(b))
sa, sb = sum(a[t] for t in tasks), sum(b[t] for t in tasks)
diffs = [int(a[t]) - int(b[t]) for t in tasks]
rng = random.Random(12)
boot = sorted(sum(rng.choice(diffs) for _ in tasks) / len(tasks) for _ in range(10000))
lo, hi = boot[250], boot[9750]
print(f"tasks compared: {len(tasks)}")
print(f"{name_a}: {sa} solved ({sa/len(tasks):.1%})   {name_b}: {sb} solved ({sb/len(tasks):.1%})")
print(f"difference {name_a} - {name_b}: {(sa-sb)/len(tasks):+.1%}  95% CI [{lo:+.1%}, {hi:+.1%}]")
only_a = [t for t in tasks if a[t] and not b[t]]
only_b = [t for t in tasks if b[t] and not a[t]]
print(f"only {name_a} ({len(only_a)}): {' '.join(only_a)}")
print(f"only {name_b} ({len(only_b)}): {' '.join(only_b)}")
