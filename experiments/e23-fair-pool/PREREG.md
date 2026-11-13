# Experiment 23: a larger fair pool, and the hint test on it (pre-registered 2026-11-12)

## The pool (`select-re-arc`)
400 new tasks: 10 from each of the 40 select-pool generators. Each task has 4 train pairs and 1 test pair.
- **Source:** RE-ARC (Hodel, MIT), commit e5b7f1d, code only. It ran in an isolated job
  (`rearc-gen-2`, exit 0) via `baselines/re-arc/rearc_gen.py`, with seed 2026
  and difficulty 0 to 1.
- **Verified:** every example passed RE-ARC's own verifier, has output different from input, and has a unique input.
- **One change to RE-ARC:** its `utils.py` loses two matplotlib import lines, used only for plotting.
- **Imported** with a strict JSON loader and full-task check: 400 imported, 0 rejected.
- **Unseen:** no part of this pool or its generators has been used in any training (dreams use train-pool inputs;
  hints v2's real examples are train-pool solutions).

**Statistics:** tasks from one generator are not independent, so the paired bootstrap resamples **generators**
(clusters of 10), 10,000 times, seed 12 (`tools/compare_clustered.py`).

## Arms
Binary from commit 53a28a0 (Linux sha256 fd7959a9…, the Experiment 22 build), on eight droplets as before, graded on
the Mac. Every arm uses `--cpu-budget 60 --node-budget 150000 --threads 8`:
- **control:** `--max-depth 4`;
- **p15d5-v1:** `--prune 15 --max-depth 5`, hints v1;
- **p15d5-v2:** `--prune 15 --max-depth 5 --hints-version 2`.

## Hypotheses
- **H1, hints help on unseen tasks.** p15d5-v2 minus control: clustered CI lower bound above 0.
- **H2, v2 better than v1.** p15d5-v2 minus p15d5-v1: clustered CI lower bound above 0.
- **H3, precision.** Each arm's wrong share is at most control's plus 2 points.

Secondary: nodes to solution (`found_at`), per-generator results, and how close this pool's control score is to
the original 40 select tasks (8 of 40).

## What would change the plan
- H1 holds: hints generalise; start the wake–sleep loop, judged on this pool.
- H1 fails: the train-pool gains were memorisation; dreams must change before the loop is worth running.
