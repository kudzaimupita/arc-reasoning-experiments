"""RE-ARC task generator driver. Builds fresh tasks from RE-ARC's generators (github.com/michaelhodel/re-arc, MIT):
    REARC_DIR=/path/to/re-arc python rearc_gen.py --ids a,b,... --per N --pairs P --seed S [--out DIR]
For each task id: N tasks of P train pairs + 1 test pair, every example verified by RE-ARC's own verifier and
non-degenerate (output differs from input). Writes <DIR>/out/<id>_<k>.json (full tasks) and <DIR>/stats.json
(DIR defaults to the current folder). E26: --raw N writes N examples per generator to <DIR>/out/<id>.json (pretraining data);
--workers W runs generators in parallel (fork); --cap S limits seconds per generator; --ids-file F reads IDs. Reads nothing but RE-ARC's code; no network, no other files.
"""
import json, os, sys, time, random
sys.path.insert(0, os.environ.get("REARC_DIR", "re-arc"))
import generators, verifiers                     # noqa: E402  (RE-ARC, MIT, commit e5b7f1d)
from utils import is_grid                        # noqa: E402

def arg(name, default):
    a = sys.argv
    return a[a.index(name) + 1] if name in a else default

ids = [x for x in arg("--ids", "").split(",") if x]
if arg("--ids-file", ""): ids = [l.strip() for l in open(arg("--ids-file", "")) if l.strip()]
per, pairs, seed = int(arg("--per", 10)), int(arg("--pairs", 4)), int(arg("--seed", 1))
raw = int(arg("--raw", 0))            # E26: N raw verified examples per generator (pretraining data), not tasks
workers, cap = int(arg("--workers", 1)), float(arg("--cap", 120))
home = arg("--out", "."); os.makedirs(f"{home}/out", exist_ok=True)
to_list = lambda g: [list(r) for r in g]

def one(tid):
    random.seed(f"{seed}-{tid}")
    gen, ver = getattr(generators, f"generate_{tid}", None), getattr(verifiers, f"verify_{tid}", None)
    if gen is None or ver is None: return tid, {"error": "no generator"}
    need = raw if raw else per * (pairs + 1)
    got, seen, tries, t0 = [], set(), 0, time.time()
    while len(got) < need and tries < need * 40 and time.time() - t0 < cap:
        tries += 1
        try:
            ex = gen(0, 1)
            i, o = ex["input"], ex["output"]
            if not (is_grid(i) and is_grid(o)) or i == o or ver(i) != o or hash(i) in seen: continue
            if not (1 <= len(i) <= 30 and 1 <= len(i[0]) <= 30 and 1 <= len(o) <= 30 and 1 <= len(o[0]) <= 30): continue
            seen.add(hash(i)); got.append({"input": to_list(i), "output": to_list(o)})
        except Exception:
            continue
    if raw:
        with open(f"{home}/out/{tid}.json", "w") as f: json.dump(got, f)
        return tid, {"examples": len(got), "tries": tries, "seconds": round(time.time() - t0, 1)}
    made = len(got) // (pairs + 1)
    for k in range(made):
        chunk = got[k * (pairs + 1):(k + 1) * (pairs + 1)]
        with open(f"{home}/out/{tid}_{k}.json", "w") as f: json.dump({"train": chunk[:pairs], "test": chunk[pairs:]}, f)
    return tid, {"tasks": made, "tries": tries, "seconds": round(time.time() - t0, 1)}

if workers > 1:
    # plain fork, no shared locks (the original run environment denied POSIX semaphores); each worker writes its own stats file
    kids = []
    for w in range(workers):
        pid = os.fork()
        if pid == 0:
            st = dict(one(t) for t in ids[w::workers])
            with open(f"{home}/stats-{w}.json", "w") as f: json.dump(st, f)
            os._exit(0)
        kids.append(pid)
    for pid in kids: os.waitpid(pid, 0)
    stats = {}
    for w in range(workers):
        try: stats.update(json.load(open(f"{home}/stats-{w}.json")))
        except Exception: pass
else:
    stats = dict(one(t) for t in ids)
with open(f"{home}/stats.json", "w") as f: json.dump(stats, f)
print(json.dumps({"generators": len(ids), "tasks": sum(s.get("tasks", 0) for s in stats.values()), "examples": sum(s.get("examples", 0) for s in stats.values())}))
