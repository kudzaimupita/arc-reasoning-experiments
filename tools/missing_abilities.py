"""Tag tasks by the abilities Hodel's hand-written solution uses (a proxy for 'what v0 is missing').
usage: ARC_DSL_DIR=<arc-dsl folder> python3 tools/missing_abilities.py <task ids...>  -> counts per ability across the tasks.
A proxy: Hodel's solution is one way to solve a task, not the only one."""
import os, re, sys
from collections import Counter

CATS = {
    "objects (find, filter, pick by size or colour)": "objects partition fgpartition colorfilter sizefilter argmax argmin mostcommon leastcommon extract sfilter mfilter totuple order",
    "drawing lines and shapes": "connect shoot box outbox inbox corners backdrop delta neighbors dneighbors ineighbors hfrontier vfrontier",
    "painting cells and patches": "fill underfill paint underpaint recolor cellwise switch replace",
    "moving and placing objects": "shift move gravitate position center centerofmass ulcorner urcorner llcorner lrcorner normalize",
    "counting and statistics": "size colorcount numcolors mostcolor leastcolor palette width height shape height valmax valmin maximum minimum",
    "cropping and splitting": "crop subgrid trim hsplit vsplit tophalf bottomhalf lefthalf righthalf",
    "scaling and joining grids": "upscale hupscale vupscale downscale hconcat vconcat canvas",
    "geometry (flip, rotate)": "hmirror vmirror dmirror cmirror rot90 rot180 rot270",
    "higher-order composition": "compose fork lbind rbind apply mapply rapply papply mpapply prapply chain matcher power",
    "logic and comparison": "equality contained branch flip both either positive even greater identity",
}
cat_of = {p: c for c, ps in CATS.items() for p in ps.split()}
src = open(os.path.join(os.environ.get("ARC_DSL_DIR", "arc-dsl"), "solvers.py")).read()
bodies = {b[:8]: b[b.index("\n"):] for b in re.split(r"^def solve_", src, flags=re.M)[1:]}
ids = sys.argv[1:]
counts, steps = Counter(), []
for t in ids:
    body = bodies.get(t, "")
    words = set(re.findall(r"\b[a-z_][a-z0-9_]*\b", body))
    for c in {cat_of[w] for w in words if w in cat_of}:
        counts[c] += 1
    steps.append(len(re.findall(r"^\s+(x\d+|O) = ", body, re.M)))
steps.sort()
print(f"{len(ids)} tasks; Hodel solution steps: median {steps[len(steps)//2]}, <=4 steps {sum(s <= 4 for s in steps)}")
for c, n in counts.most_common():
    print(f"{n:4d}  {n/len(ids):4.0%}  {c}")
