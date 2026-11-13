# Experiment 12 results: reasoner v0 against icecuber (12 Nov 2026)

Pre-registration: `PREREG.md` (written before the runs). Runs `rsn-v0-train`, `rsn-v0-select`: exit 0.
Peak memory 2.5 GB (one process, 8 threads), 2,539 CPU-s on train.

| System | Train (320) | Select (40) | Both (360) |
|---|---|---|---|
| reasoner v0 (60 CPU-s, depth 4) | 42 (13.1%) | 8 (20.0%) | 50 (13.9%) |
| icecuber depth 3 (Experiment 11) | 152 (47.5%) | 15 (37.5%) | 167 (46.4%) |

Paired difference v0 minus icecuber: **-32.5 points, 95% bootstrap CI [-37.2, -27.8]** (10,000 resamples, seed 12).
v0 solves no task that icecuber misses; icecuber solves 117 that v0 misses.

**Knows what it does not know.** v0 gave an answer on 51 tasks: 50 right, 1 wrong (`94f9d214`), 2% of its answers.
It gave no answer on 309 tasks.

**Why it fails.** Search status: 257 tasks "exhausted" (every program up to the limits was tried and none fit),
38 hit the node cap, 14 hit the time limit. So the gap is mostly missing operations, not search speed.

**Missing abilities** (proxy: what Hodel's hand-written solutions of the 117 icecuber-only tasks use; one way to
solve each task, not the only way). Median solution length 9 steps; only 17 of 117 need 4 steps or fewer.

| Ability | Share of the 117 |
|---|---|
| painting cells and patches | 79% |
| objects: find, filter, pick by size or colour | 65% |
| higher-order composition | 58% |
| counting and statistics | 52% |
| drawing lines and shapes | 40% |
| cropping and splitting | 32% |
| moving and placing objects | 28% |
| scaling and joining grids | 26% |
| logic and comparison | 25% |
| geometry | 13% |

Median nodes explored per train task: 265,412.
