# Experiment 24 results: DSL v2 batch 1 (12 Nov 2026)

Pre-registration: `PREREG.md`. Binary from commit 6d254ea (Linux sha256 b3691eed…), run on eight droplets with no
outbound traffic, graded on the Mac with none missing. The droplets were destroyed after the run. Controls are the
Experiment 22 and 23 runs (same earlier binary, same droplet type).

| Pool | Control | DSL v2 batch 1 | Difference | Wrong (batch 1) |
|---|---|---|---|---|
| **Fair pool, 400 unseen RE-ARC tasks** | 63 (7 of 40 generators) | **63 (7 of 40 generators)** | **+0.0** (the same tasks) | 0 |
| Train + select, 360 | 64 | 78 | +3.9 points, CI [+1.9, +6.1] | 10 of 88 answers |
| of which train (320) | 56 | 70 | +14 | 10 (12.5%) |
| of which select (40) | 8 | 8 | 0 | 0 |

- **H1 (gain on the fair pool): not met.** Exactly the same 63 tasks are solved.
- **H2 (more generators): not met.** 7 of 40, the same 7.
- **H3 (fair-pool wrong share at most 5%): met.** 0%.

## Reading
The new operations were chosen from Hodel's solutions of unsolved **train** tasks. They solve 14 more train tasks,
at the cost of 10 wrong train answers, and **nothing** on unseen tasks or unseen rule families. Each hand-written
operation covers the specific tasks it was taken from. ARC's rule families are diverse enough that 20 operations
from 26 train tasks don't reach any of the 33 unsolved select families. This is overfitting at the level of the DSL.
The train-pool gains of Experiments 13 to 16 should be read the same way, and the select pool hasn't moved from 8
of 40 since Experiment 12.

## What it means for the next steps
Adding hand-written operations is not the path to new rule families. Each costs search breadth and adds chance
fits, and none generalised here. The natural next step is library learning. New operations are discovered
as reusable compositions of existing ones, from the system's own solutions across many tasks, and kept only if
they help on tasks they weren't found on. They are judged on the fair pool. The next experiment should test that
directly at small scale: abstraction learning in the DreamCoder style, over the reasoner's own correct programs
plus dreams.

## Status of batch 1
Kept in the code (it adds train-pool coverage and costs no fair-pool tasks) but marked as hand-written priors. They
are counted as human-supplied prior knowledge.
