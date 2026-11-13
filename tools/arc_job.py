#!/usr/bin/env python3
"""Prepare a batch of ARC tasks for the solvers and grade their predictions.

  python3 tools/arc_job.py prepare <task folder> <ids file> <job dir>
      Copies <task folder>/<id>.json for every id into <job dir>, with the test outputs removed, and writes
      <job dir>/tasks.txt. This is the ARC_JOB_IN folder that `reasoner job`, ttt/ttt_solver.py, ttt/varc.py and
      baselines/icecuber/icecuber_driver.py read.

  python3 tools/arc_job.py grade <task folder> <ids file> <predictions dir> <grade.json> [--attempts 2]
      Scores <predictions dir>/<id>.json against the full tasks. A task passes only if every test output is
      matched by one of its first --attempts attempts (the ARC rule, top-2 by default). Writes a grade file in the
      format used under experiments/*/grades/, which tools/compare_runs.py and tools/compare_clustered.py read.

  python3 tools/arc_job.py ids <grade.json>...
      Prints the task ids in grade files (for example the train or select pool of an experiment).
"""
import json, os, sys


def ids_from(path):
    return [l.strip() for l in open(path) if l.strip() and not l.startswith("#")]


def prepare(src, ids_file, job):
    os.makedirs(job, exist_ok=True)
    ids = ids_from(ids_file)
    for tid in ids:
        t = json.load(open(os.path.join(src, f"{tid}.json")))
        stripped = {"train": t["train"], "test": [{"input": p["input"]} for p in t["test"]]}
        with open(os.path.join(job, f"{tid}.json"), "w") as f:
            json.dump(stripped, f)
    with open(os.path.join(job, "tasks.txt"), "w") as f:
        f.write("\n".join(ids) + "\n")
    print(f"{len(ids)} tasks written to {job}")


def grid_of(a):
    return a["grid"] if isinstance(a, dict) and "grid" in a else a


def grade(src, ids_file, pred_dir, out, attempts):
    rows, outs_ok, outs = [], 0, 0
    for tid in ids_from(ids_file):
        truth = [p["output"] for p in json.load(open(os.path.join(src, f"{tid}.json")))["test"]]
        try:
            pred = json.load(open(os.path.join(pred_dir, f"{tid}.json")))
        except (OSError, ValueError):
            pred = []
        correct = 0
        for k, want in enumerate(truth):
            cand = pred[k] if k < len(pred) and isinstance(pred[k], dict) else {}
            tries = [grid_of(cand.get(f"attempt_{i + 1}")) for i in range(attempts)]
            correct += any(t == want for t in tries if t is not None)
        rows.append({"task": tid, "pass": correct == len(truth), "correct": correct, "total": len(truth)})
        outs_ok += correct; outs += len(truth)
    passed = sum(r["pass"] for r in rows)
    g = {"scoring": f"top-{attempts} (ARC rule)",
         "totals": {"tasks": len(rows), "passed": passed, "outputsCorrect": outs_ok, "outputsTotal": outs}, "tasks": rows}
    with open(out, "w") as f:
        json.dump(g, f, indent=1)
    print(f"{passed} of {len(rows)} tasks solved ({outs_ok} of {outs} test outputs)")


if __name__ == "__main__":
    a = sys.argv[1:]
    if a[:1] == ["prepare"] and len(a) == 4:
        prepare(*a[1:])
    elif a[:1] == ["grade"] and len(a) >= 5:
        n = int(a[a.index("--attempts") + 1]) if "--attempts" in a else 2
        grade(*a[1:5], n)
    elif a[:1] == ["ids"] and len(a) >= 2:
        for p in a[1:]:
            for t in json.load(open(p))["tasks"]:
                print(t["task"])
    else:
        print(__doc__)
        sys.exit(64)
