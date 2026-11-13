# Experiment 19: the hint network (pre-registered 2026-11-12)

## Question
Does a small network that predicts which operation kinds a task needs make the search cheaper and solve more tasks?
This is the hint test, in a first, small form.

## The network
- **Input:** 64 features of a task's train pairs (`reasoner/src/hints.rs`): size ratios, colours added and removed,
  share of cells changed, flip and transpose matches, crop and tile relations, symmetries, object counts, colour
  histograms.
- **Output:** a probability for each of 140 operation kinds.
- **Shape:** MLP 64→512→512→140, 0.37M parameters, trained with PyTorch on the Mac's GPU in about 1 minute
  (`tools/train_hints.py`, 6 epochs).
- **Training data:** 2,000,000 "dreams" (`reasoner dream --seed 19`). Each is a random program of 1 to 3 operations
  run on 2 to 4 input grids from one train-pool task; the label is the program's kinds.
  - Only train-pool inputs are used.
  - No real task's solution is used.
  - No select-pool data is used.
- **Held-out dreams:** recall@10 of 0.872.
- **In the search** (`--hints`): each kind's cost changes by round(2 · log2(p / 0.05)) bits, clamped to [-3, +6]
  bits off and to costs of 2 to 20. Likely kinds get cheaper, unlikely ones dearer.

## Arms
One binary, built from the commit that adds this file. Every arm uses DSL v1 part 2 and the CPU-second budget:
`--cpu-budget 60 --node-budget 150000 --max-depth 4 --threads 8`. The arms run together with Experiment 18's on the
eight droplets, with the same setup and grading on the Mac.
- **bf:** `--best-first`, the comparison arm. Hints only change costs, and best-first search follows costs.
- **bf+hints:** `--best-first --hints`.
- **bf+prior+hints:** `--best-first --prior --hints`.

## Hypotheses
On 360 tasks:
- **H1, node reduction.** On tasks solved by both bf and bf+hints, the ratio of geometric-mean nodes explored, bf
  over bf+hints, is at least 2. The long-term target is 3x with a lower bound of 2x on censored nodes across seeds;
  this first version uses one seed and uncensored nodes, and is reported as such.
- **H2, solved.** bf+hints minus bf: paired bootstrap 95% CI lower bound above 0.
- **H3, precision.** bf+hints' wrong-answer share is at most bf's plus 2 points.

## Development note
Smoke test on 4 train tasks at 15 CPU-s, without best-first: no change in solved status. One task picked a different,
longer program first. Dreams draw on train-pool grids, so results on the train pool may be slightly optimistic;
the select pool is the fairer test.

## What would change the plan
- H1 and H2 hold: hints work. Scale them next: more dreams, a convolutional encoder on the grids, and training on
  the reasoner's own solutions (wake–sleep).
- H1 holds but H2 fails: hints save search but don't yet reach new tasks; push budget into depth.
- Both fail: features or dream distribution don't transfer to real tasks. Look at the per-kind calibration on train-pool
  solutions before scaling.

## Amendment (2026-11-12, before any Experiment 18 or 19 result was seen)
Experiment 17 showed that "nodes" counts every node explored, and best-first search keeps going after it finds an
answer, to show nothing cheaper exists. The reasoner now records `found_at`, the number of nodes explored when each
answer was first found. **H1 uses `found_at` of the correct answer** ("nodes to solution"), not total nodes. The first cloud start of Experiments 18 and 19 was stopped after about 3 minutes,
before any output was read; both now run with the binary from the commit that adds this amendment.
