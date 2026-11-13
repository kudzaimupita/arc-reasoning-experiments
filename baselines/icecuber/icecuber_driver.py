"""icecuber_driver.py: runs icecuber (top-quarks/ARC-solution, patched; see patches/) on stripped tasks with a hard
per-task CPU budget. Experiment 11 baseline.

What it does: reads $ARC_JOB_IN/tasks.txt and the stripped task files (train pairs, test inputs only).
For each task it writes the task into its own work folder under $ARC_JOB_OUT/work as
dataset/test/<id>.json, then runs `icecuber <k> <depth> <mem_mb>` there once per test input k, one
after another. The task's CPU budget (default 60 CPU-s) is shared by its test inputs: each run gets
RLIMIT_CPU = the budget left (whole seconds); a run killed at the limit gives no answer. icecuber is
single-threaded, so one process = one thread per task. Up to --parallel tasks run at once. icecuber's
own memory guard (patch 0004) exits before its footprint passes --mem-mb.
Writes $ARC_JOB_OUT/out/<id>.json (up to 3 attempts per test input, in icecuber's rank order, each
{"grid", "cpu_ms"}; cpu_ms = the task's CPU-ms when that run ended) and one line per icecuber run to
$ARC_JOB_OUT/metrics.jsonl (status, CPU-s, peak RSS, peak footprint).

Run as: ICECUBER_BIN=/path/to/icecuber ARC_JOB_IN=<dir> ARC_JOB_OUT=<dir> \
          python3 icecuber_driver.py --depth 2 --parallel 8 [--cpu 60] [--mem-mb 1300]
"""
import json
import math
import os
import re
import resource
import signal
import subprocess
import sys
import time

BIN = os.environ.get("ICECUBER_BIN", "icecuber")


def arg(name, default, cast=int):
    a = sys.argv[1:]
    if name in a:
        return cast(a[a.index(name) + 1])
    return default


DEPTH = arg("--depth", 2)
PARALLEL = arg("--parallel", 8)
CPU = arg("--cpu", 60)
MEM_MB = arg("--mem-mb", 1300)
if DEPTH not in (2, 3, 4) or not 1 <= PARALLEL <= 12 or not 1 <= CPU <= 600 or not 100 <= MEM_MB <= 5800:
    raise SystemExit("bad arguments")

in_dir = os.environ["ARC_JOB_IN"]
home = os.path.abspath(os.environ.get("ARC_JOB_OUT", "."))
out_dir = os.path.join(home, "out")
work_root = os.path.join(home, "work")
os.makedirs(out_dir, exist_ok=True)
os.makedirs(work_root, exist_ok=True)
metrics = open(os.path.join(home, "metrics.jsonl"), "w")

with open(os.path.join(in_dir, "tasks.txt")) as f:
    ids = [line.strip() for line in f if line.strip()]
for task_id in ids:
    if not re.fullmatch(r"[0-9a-f]{8}", task_id):   # icecuber's reader needs 8 lowercase hex/alnum
        raise SystemExit("bad task id")


def parse_answers(path):
    """icecuber's answer file: a line '<id>_<k>', then '|row|row|... <score>' per candidate, best first."""
    try:
        with open(path) as f:
            lines = [l.strip() for l in f if l.strip()]
    except FileNotFoundError:
        return None
    grids = []
    for line in lines[1:]:
        img, score = line.split()
        rows = img.strip("|").split("|")
        grid = [[int(c) for c in r] for r in rows]
        if not 1 <= len(grid) <= 30 or not all(1 <= len(r) <= 30 and len(r) == len(grid[0]) for r in grid):
            raise ValueError("bad grid")
        # Upstream writes a 1x1 black dummy with score -1 when it has no candidate: not an answer.
        if float(score) == -1 and grid == [[0]] and len(lines) == 2:
            continue
        grids.append(grid)
    return grids[:3]


class Task:
    def __init__(self, task_id):
        self.id = task_id
        with open(os.path.join(in_dir, task_id + ".json")) as f:
            t = json.load(f)
        self.n = len(t["test"])
        self.wd = os.path.join(work_root, task_id)
        os.makedirs(os.path.join(self.wd, "dataset", "test"), exist_ok=True)
        os.makedirs(os.path.join(self.wd, "output"), exist_ok=True)
        clean = {"train": [{"input": p["input"], "output": p["output"]} for p in t["train"]],
                 "test": [{"input": p["input"]} for p in t["test"]]}
        with open(os.path.join(self.wd, "dataset", "test", task_id + ".json"), "w") as f:
            json.dump(clean, f)
        self.k = 0
        self.cpu = 0.0
        self.preds = []


def start(task):
    """Start icecuber on the task's next test input; returns (pid, info) or None if no budget is left."""
    soft = math.floor(CPU - task.cpu)
    if soft < 1:
        return None
    def limits():
        resource.setrlimit(resource.RLIMIT_CPU, (soft, soft + 1))   # SIGXCPU at soft, SIGKILL at hard
    out = open(os.path.join(task.wd, f"stdout_{task.k}.txt"), "w")
    err = open(os.path.join(task.wd, f"stderr_{task.k}.txt"), "w")
    p = subprocess.Popen([BIN, str(task.k), str(DEPTH), str(MEM_MB)], cwd=task.wd, stdin=subprocess.DEVNULL,
                         stdout=out, stderr=err, preexec_fn=limits, close_fds=False)
    # close_fds=False: CPython would list /dev/fd in the child (denied in the original run environment). Python opens every
    # file non-inheritable (PEP 446), so the child still gets only stdin, stdout and stderr.
    out.close()
    err.close()
    return p, {"task": task, "k": task.k, "limit": soft, "t0": time.monotonic(), "wall_killed": False}


def finish(info, status, ru):
    task, k = info["task"], info["k"]
    cpu = ru.ru_utime + ru.ru_stime
    task.cpu += cpu
    footprint = None
    try:
        with open(os.path.join(task.wd, f"stderr_{k}.txt")) as f:
            m = re.search(r"mem-guard: (\w+) peak_footprint_mb=(\d+)", f.read()[-4096:])
            if m:
                footprint = int(m.group(2))
    except OSError:
        pass
    if info["wall_killed"]:
        why = "wall"
    elif os.WIFEXITED(status):
        why = {0: "ok", 75: "memory", 76: "cpu"}.get(os.WEXITSTATUS(status), f"exit{os.WEXITSTATUS(status)}")
    else:
        sig = os.WTERMSIG(status)
        why = "cpu" if sig in (signal.SIGXCPU, signal.SIGKILL) else f"signal{sig}"
    grids = None
    if why == "ok":
        try:
            grids = parse_answers(os.path.join(task.wd, "output", f"answer_{k}_{DEPTH}.csv"))
        except (ValueError, OSError):
            why = "bad-output"
    cpu_ms = int(round(task.cpu * 1000))
    task.preds.append({f"attempt_{i + 1}": {"grid": g, "cpu_ms": cpu_ms} for i, g in enumerate(grids or [])})
    metrics.write(json.dumps({"task": task.id, "test": k, "depth": DEPTH, "status": why, "cpu_s": round(cpu, 3),
                              "limit_s": info["limit"], "maxrss_mb": round(ru.ru_maxrss / 1048576, 1),
                              "footprint_mb": footprint, "candidates": len(grids or [])}) + "\n")
    metrics.flush()
    task.k += 1


def write_pred(task):
    while len(task.preds) < task.n:   # test inputs never started (budget used up): no answer
        metrics.write(json.dumps({"task": task.id, "test": len(task.preds), "depth": DEPTH, "status": "budget",
                                  "cpu_s": 0, "limit_s": 0, "maxrss_mb": None, "footprint_mb": None, "candidates": 0}) + "\n")
        task.preds.append({})
    with open(os.path.join(out_dir, task.id + ".json"), "w") as f:
        json.dump(task.preds, f)
    print(f"{task.id} cpu={task.cpu:.1f}s answers={[len(p) for p in task.preds]}", flush=True)


queue = list(ids)
running = {}   # pid -> (Popen, info)
t_start = time.monotonic()


def launch_next(task):
    """Start the task's next run, or finish the task. Returns True if a process was started."""
    while task.k < task.n:
        r = start(task)
        if r is None:
            break
        p, info = r
        running[p.pid] = (p, info)
        return True
    write_pred(task)
    return False


while queue or running:
    while queue and len(running) < PARALLEL:
        launch_next(Task(queue.pop(0)))
    if not running:
        continue
    pid, status, ru = os.wait4(-1, os.WNOHANG)
    if pid == 0:
        now = time.monotonic()
        for p, info in running.values():
            if not info["wall_killed"] and now - info["t0"] > info["limit"] * 1.5 + 15:
                info["wall_killed"] = True
                p.kill()
        time.sleep(0.05)
        continue
    p, info = running.pop(pid)
    p.returncode = status   # reaped here with wait4 (for its rusage); Popen must not wait again
    finish(info, status, ru)
    launch_next(info["task"])

metrics.close()
print(f"icecuber_driver: depth {DEPTH}, {len(ids)} tasks, parallel {PARALLEL}, cpu {CPU}s, mem {MEM_MB} MB, "
      f"wall {time.monotonic() - t_start:.0f}s", flush=True)
