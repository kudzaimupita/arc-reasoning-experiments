# Experiment 13 results: DSL v1 (12 Nov 2026)

Pre-registration: `PREREG.md` (written before the runs). Reasoner at commit 6cdce28, settings unchanged from
Experiment 12 (60 s per task, node budget 150,000, depth 4, 8 threads). Runs `rsn-v1c-train` and `rsn-v1c-select` as
exit 0. Peak memory 3.1 GB
(one process). Train used 4,756 CPU-s, against 2,539 for v0.

Two earlier attempts (`rsn-v1-train`, `rsn-v1b-train`) were stopped by the job watchdog when `ps` timed out under load.
The second was caused by the Mac itself (swap 17.3 of 18.4 GB, load about 109). Both are logged,
and neither was graded or used. During the counted runs, a virtual machine on the same Mac was using
about 6 cores. The time budget is wall-clock, so v1 may have searched less per task than v0 did.

## Primary result

| System | Train (320) | Select (40) | Both (360) |
|---|---|---|---|
| reasoner v1 | 50 (15.6%) | 8 (20.0%) | 58 (16.1%) |
| reasoner v0 (Experiment 12) | 42 (13.1%) | 8 (20.0%) | 50 (13.9%) |
| icecuber depth 3 (Experiment 11) | 152 (47.5%) | 15 (37.5%) | 167 (46.4%) |

Paired bootstrap (10,000 resamples, seed 12, same 360 tasks):
- **v1 minus v0: +2.2 points, 95% CI [+0.6, +3.9].** Gained 9 tasks, lost 1 (`1b2d62fb`).
- **v1 minus icecuber depth 3: -30.3 points, 95% CI [-35.3, -25.3].** v1 solves 4 tasks icecuber misses; icecuber
  solves 113 that v1 misses.

DSL v1 closes a small, real part of the gap: about 2 of the 32.5 points.

## Wrong answers got worse

| | Answered | Right | Wrong | Wrong share | No answer |
|---|---|---|---|---|---|
| v1 | 65 | 58 | 7 | 10.8% | 295 |
| v0 | 51 | 50 | 1 | 2.0% | 309 |

The 7 wrong answers:
- **Accidental colour maps (3):** `b9b7f026`, `b230c067`, `de1cd16c`. A short program plus a colour map fits the
  examples by chance. The depth guard from development (maps only after at most 2 operations) was not enough.
- **New operations fitting by chance (2):** `2dc579da` (symmetry then cropping), `60b61512` (filled boxes).
- **Same as v0 (1):** `94f9d214`.
- **Ranking bug (1):** `1b2d62fb`, the lost task. v0's correct program `Recolour(1, 9) -> AndLR(8)` gives the
  same test answer as `Recolour(1, 9) -> AndLR(1) -> MapColours(1>8)`, which v1 finds first. The search keeps only
  the first program per distinct answer, so this answer carries the map's +5 cost. Two cheaper wrong programs from
  depth 3 then push it out of the top two. Fix for the next version: keep the cheapest program per answer.

## Where the search now stops

| Search status (360 tasks) | v1 | v0 |
|---|---|---|
| Answered, full search | 55 | 50 |
| Answered, search cut short | 10 | 1 |
| Exhausted (everything within limits tried, nothing fits) | 152 | 257 |
| Hit the node cap | 104 | 38 |
| Hit the time limit | 39 | 14 |

v0 mostly ran out of operations. v1 mostly runs out of budget: 143 tasks now stop on the node cap or time limit,
up from 52. A larger DSL makes blind search slower. From here on, search order matters as much as new operations.
This is the reason for the hint network and for Probe-style reweighting.

Median nodes explored per task: 373,909 (v0: 275,406). Median time to a correct answer: 2.2 s (v0: 1.1 s).
Colour maps appear in 18 of the 58 correct programs.

## Conclusions
1. DSL v1 helps a little (+2.2 points, CI above zero), but far less than its new operations' coverage suggested.
2. Precision fell from 98% to 89%. Before more operations are added, the next version needs:
   - the cheapest program kept for each answer
   - a stricter test for colour maps (for example, the map must be needed on at least two training pairs, or
     must be a bijection)
   - a preference for programs whose length matches what the data supports
3. The limit is now search budget, not just the operation set. Next steps, in order:
   1. the ranking and colour-map fixes, as a small experiment;
   2. DSL v1 part 2 (per-object templates, multi-colour objects, background handling);
   3. search order (Probe, then the hint network).
