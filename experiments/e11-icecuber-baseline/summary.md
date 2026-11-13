# Experiment 11 results: icecuber at 60 CPU-s per task (2026-11-12)

PREREG.md was written before the full runs (sha256 in manifest.json) and is unchanged. Per-task rows:
results.jsonl (built from the grade files and the driver's metrics). No held-out data was read.

| Pool | Depth | Tasks | Solved top-2 | 95% CI | Solved top-3 | Outputs top-2 | Outputs top-3 | CPU-s mean / median | Killed at 60 s | Guard stops | Peak RSS max / p95 (MB) | Peak footprint max (MB) | Wall (min) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| arc1-train | 2 | 320 | 135 (42.2%) | 36.9-47.7% | 137 (42.8%) | 146/334 | 148/334 | 0.2 / 0.1 | 0 | 0 | 70 / 43 | 69 | 0.2 |
| arc1-train | 3 | 320 | 152 (47.5%) | 42.1-53.0% | 156 (48.8%) | 163/334 | 169/334 | 5.3 / 3.4 | 0 | 0 | 1279 / 444 | 1329 | 9.8 |
| arc1-train | 4 | 320 | 107 (33.4%) | 28.5-38.8% | 111 (34.7%) | 119/334 | 124/334 | 38.3 / 45.6 | 99 | 32 | 3380 / 3142 | 3381 | 71.5 |
| select-arc1 | 2 | 40 | 15 (37.5%) | 24.2-53.0% | 16 (40.0%) | 15/40 | 16/40 | 0.3 / 0.1 | 0 | 0 | 105 / 44 | 103 | 0.1 |
| select-arc1 | 3 | 40 | 15 (37.5%) | 24.2-53.0% | 17 (42.5%) | 15/40 | 17/40 | 7.1 / 3.2 | 1 | 0 | 1175 / 633 | 1166 | 1.8 |
| select-arc1 | 4 | 40 | 11 (27.5%) | 16.1-42.8% | 12 (30.0%) | 11/40 | 12/40 | 39.7 / 40.5 | 14 | 2 | 2734 / 2356 | 3345 | 9.6 |
| select-arc2-only | 2 | 116 | 3 (2.6%) | 0.9-7.3% | 4 (3.4%) | 3/137 | 4/137 | 0.3 / 0.2 | 0 | 0 | 81 / 41 | 80 | 0.1 |

Columns: solved = every test output right; 95% CI = Wilson interval on top-2; killed = tasks with a run
stopped at the 60 CPU-s limit; guard stops = tasks with a run stopped by the memory guard (depth 3/4:
3300 MB, all at 44-60 CPU-s, so each lost at most 16 s); peak RSS and footprint are per process. Wall
is the sum of the job wall clocks (8 tasks at once for depth 2, 3 for depth 3 and 4). Depth-4 train
is two jobs of 160 tasks.

## Key numbers
- Best depth at 60 CPU-s on train + select (top-2): depth 3, 167/360 (46.4%). Depth 2: 150/360 (41.7%).
  Depth 4: 118/360 (32.8%): 31% of its tasks hit the 60 s kill and 9% the memory guard.
- Depth 3 solves 51 train tasks depth 4 misses; depth 4 solves 6 that depth 3 misses. Depth 2 solves
  4 train tasks depth 3 misses.
- Union of depths 2-4 (an oracle, not a system): train 162/320 top-2, 168/320 top-3 (52.5%);
  select 18/40 top-2, 19/40 top-3.
- ARC-2-only select, depth 2: 3/116 top-2 (2.6%, Wilson 0.9-7.3%), 4/116 top-3. Under 5%.
- Build check: claimed 209/400 (52.3%, full solver, top-3) on ARC-1 training. Ours: depth 3 top-3
  156/320 (48.8%); union of depths top-3 168/320 (52.5%). Consistent with the claim (rates on a 320-task subset, single depths, 60 s).
- Memory: depth 2 under 105 MB, depth 3 under 1.4 GB, depth 4 up to 3.4 GB per process by the kill.
  Largest job totals: 7.8 GB (depth 4, 3 at once).
