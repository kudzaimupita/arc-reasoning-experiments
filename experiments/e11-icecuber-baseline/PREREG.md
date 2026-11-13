# Experiment 11: icecuber baseline on train and select (pre-registered 2026-11-12)

Written after the build and a 10-task memory probe, before any full run. No held-out data is read.

## Question
What does icecuber solve on this Mac (Apple M1 Max) with a hard 60 CPU-second budget per task, at
search depth 2, 3 and 4, on the train and select pools? This is the reference number for the reasoner experiments.

## System
- icecuber: github.com/top-quarks/ARC-solution, commit 9407072659de1270358c2ba34c527785214dd68b
  (MIT). Cloned with --depth 1; its bundled dataset/ folder (training, evaluation, test) and its .git store
  were deleted unread before anything else.
- Patches (baselines/icecuber/patches/): 0001 standard includes for libc++; 0002 read tasks only from
  ./dataset/ (no /kaggle probe), sorted; 0003 eval = 1 (the solver never sees test outputs); 0004 a
  memory and CPU guard (src/mem_guard.cpp): exits before its own footprint passes a given limit, and
  reports its peak footprint. No change to the search, the transforms or the ranking.
- Built with Apple clang 21 (-std=c++17 -O2) using baselines/icecuber/Makefile.icecuber. evals.cpp and
  tasks.cpp (hand solutions run() never calls) are not built. Binary sha256
  a3338de9e29df9ce2aa2dfa48c844ebb61ab124b1472bd890cc1ff63036a6eca.
- Driver: baselines/icecuber/icecuber_driver.py, run by Python 3.12.13 on stripped tasks (train pairs and
  test inputs only); a separate grader holds the test outputs.

## Configurations (fixed now)
- icecuber depth argument 2, 3 and 4 (upstream MAXDEPTH 20, 30, 40), colour normalisation on, no
  flipped-input runs. This is NOT upstream's full submission pipeline (depth 3, two flipped depth-3
  runs and depth 4, candidates merged by score, 9 hours on 100 tasks); that pipeline is not run here.
- Budget: 60 CPU-seconds per task (user + system), enforced with RLIMIT_CPU on each icecuber process.
  A task's test inputs run one after another (icecuber solves one test input per process); each gets
  the whole seconds left of the task's 60. A process killed at the limit gives no answer for that test
  input. CPU-s per task are measured with wait4 rusage.
- One thread per task (icecuber is single-threaded; OpenMP is off upstream too).
- Tasks at once and memory guard per process, chosen from the probe (peak footprint with 1 process on
  the first 10 train tasks, jobs ice-mem-d2/d3/d4: depth 2 max 43 MB and 0.5 CPU-s; depth 3 max 427 MB
  and 15 CPU-s; depth 4 max 3447 MB, 5 of 10 killed at 60 s, footprint grows to 1.7-3.4 GB by the kill)
  so that the total stays under a 12 GB job limit and the machine keeps its 20% free memory:
  - depth 2: 8 tasks at once, guard 1300 MB per process;
  - depth 3 and 4: 3 tasks at once, guard 3300 MB per process (at most 9.9 GB in all). The guard only
    binds on searches that are near the 60 s kill anyway.
  A process stopped by the guard gives no answer for that test input. Deviation from the original
  "8 tasks at once": depth 3 and 4 use 3, because 8 searches of up to 3.4 GB do not fit. With 10 cores
  and at most 8 processes, CPU-s per task do not depend on this choice (no oversubscription); wall
  time does.
- Attempts: icecuber's own ranking (up to 3 distinct candidates, best first). Upstream's 1x1 black
  placeholder (score -1, written when it has no candidate) is dropped: it is not a candidate.

## Pools and runs
- arc1-train (320 tasks): depths 2, 3, 4.
- select-arc1 (40 tasks): depths 2, 3, 4.
- select-arc2-only (116 tasks): depth 2 only.
- One launcher job per depth and pool (depth 4 on train is split into two jobs, the first and last 160
  tasks of the train pool, to stay inside the 90-minute job wall clock). Every job is graded with top-2 and
  again with --attempts 3.

## Measures
- Primary: tasks solved, top-2 (a task counts only if every test output is correct), per pool and depth.
- Secondary: top-3 (only for comparison with published icecuber numbers); test outputs correct (top-2
  and top-3); CPU-s per task (mean, median, share killed at 60 s); peak footprint and peak RSS per
  process, per depth (max and 95th percentile); guard stops; wall time per job.
- Per-task rows in results.jsonl: task, pool, depth, solved top-2, solved top-3, CPU-s, peak RSS MB,
  peak footprint MB, status per test input.

## Comparisons
- Build check: the claimed 209/400 on ARC-1 training (52%, full solver; Bober-Irizar & Banerjee,
  claimed) against our best depth's train-pool rate, both top-3 and top-2. Rates, not counts: our 320
  train tasks are a subset of the 400. Expected: lower than 52%, because we run single depths with a
  60 s budget, not the full 9-hour pipeline.
- ARC-2-only select: icecuber depth-2 solve rate. If it is under about 5%, this pool
  adds little statistical power, and later PREREGs lean on RE-ARC instances clustered by generator.
- The icecuber depth that does best at 60 CPU-s on train + select is recorded here and used as the
  reference in later experiments.

## Expectations (stated before the full runs)
- Depth 2 solves well under depth 3; depth 4 at 60 s is not better than depth 3 by much, or is worse,
  because many depth-4 searches hit the 60 s kill and give nothing.
- ARC-2-only depth 2: under 5%.
