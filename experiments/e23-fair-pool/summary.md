# Experiment 23 results: the fair pool and the hint test on it (12 Nov 2026)

Pre-registration: `PREREG.md`. 400 RE-ARC tasks (10 from each of the 40 select generators), never used in training.
Binary fd7959a9… (commit 53a28a0) ran on eight droplets with no outbound traffic from creation. Graded on the Mac,
none missing or invalid. The droplets were destroyed after the run.

| Arm | Solved of 400 | Generators with any solve | Answered | Wrong | CPU-s |
|---|---|---|---|---|---|
| control | **63** (15.8%) | 7 of 40 | 63 | 0 | 10,986 |
| p15d5-v1 | 57 | 7 | 57 | 0 | 8,298 |
| p15d5-v2 | 57 | 7 | 57 | 0 | 6,861 |

Cluster bootstrap by generator, 10,000 resamples, seed 12:
- **H1 (v2 hints beat control on unseen tasks): not met.** -1.5 points, CI [-4.5, +0.0]. Hints are slightly worse.
- **H2 (v2 beats v1): not met.** Identical solved sets.
- **H3 (precision): met.** No wrong answers in any arm.

## What this shows
- **Every loss is one generator.** Both pruned arms lose 6 of `2013d3e2`'s 10 tasks: a kind it needs falls outside the
  top 15. Nothing is gained anywhere.
- **The reasoner knows rule families, not tasks.** It solves something from 7 of 40 generators and nothing from the
  other 33, close to the 8 of 40 original select tasks. Faster search (1.18x fewer nodes to solution here) only
  speeds up families it can already express. It doesn't add any.
- **Hints v2's train-pool gain (E22) was memorisation of those tasks' solutions.** On unseen tasks, v1 and v2 behave
  the same.
- **This pool works as intended:** 400 tasks and 40 clusters, 0% wrong answers on control, and a clear, stable signal.
  It is the yardstick for every later experiment.

## Decision
The bottleneck is DSL coverage of rule families, not search order. Next:
1. Measure which of the 33 unsolved generators are near misses: which primitives and compositions each needs (Hodel's
   solvers as a proxy, as in Experiment 12).
2. Add per-family capabilities (painting patterns, counting, object correspondence) in small batches, each judged on
   this pool by the number of generators solved.
3. Pruning stays off by default. Hints become worth revisiting once the DSL is larger and search budget binds again.
