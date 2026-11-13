# Experiments 21 and 22: hybrid pruning and retrained hints (pre-registered 2026-11-12)

## Questions
- **E21:** does keeping full breadth to depth 2 and pruning only below it (top-15 kinds, to depth 5) keep control's
  shallow solutions and add Experiment 20's deep ones?
- **E22:** do hints retrained on realistic dreams and the reasoner's own solutions prune better than E19's hints?

## Hint network v2 (`reasoner/src/hints2.json`)
The architecture is the same as v1 (MLP 64→512→512→140, 0.37M parameters, 8 epochs on the Mac's GPU, about
80 s). The training data:
- **3,000,000 dreams** (`reasoner dream --seed 22 --weighted --long`):
  - programs of 1 to 5 operations, with lengths 25/30/25/12/8%;
  - operations drawn in proportion to 1 + 4 × (how often their kind appears in correct train-pool programs,
    `reasoner/src/prior.txt`);
  - inputs from train-pool tasks only.
- **78 real examples:** correct programs for 60 solved train-pool tasks from Experiments 12 to 20, with features from
  `reasoner features` (`tools/real_hint_examples.py`), each repeated 300 times.
- **No select-pool data.**

Recall@10 on held-out data is 0.917. That held-out set contains copies of the repeated real examples, so the figure
is optimistic. **Train-pool results are optimistic for v2:** it has seen those tasks' solutions. The select pool is
the fair test, and is reported separately.

## Arms
One binary, built from the commit that adds this file, run on eight droplets as in Experiment 20 and graded on the
Mac. Every arm uses DSL v1 part 2, level-by-level search and the CPU-second budget,
`--cpu-budget 60 --node-budget 150000 --threads 8`:
- **control:** `--max-depth 4`;
- **hyb (E21 primary):** `--prune 15 --prune-from 2 --max-depth 5`;
- **hyb-v2 (E22 primary):** `--prune 15 --prune-from 2 --max-depth 5 --hints-version 2`;
- **p15d5-v2:** `--prune 15 --max-depth 5 --hints-version 2`, the E20 setting with v2 hints.

## Hypotheses
Paired bootstrap, 10,000, seed 12:
- **E21-H1:** hyb minus control on 360 tasks, CI lower bound above 0.
- **E21-H2:** hyb loses at most 1 task that control solves.
- **E22-H1:** hyb-v2 minus hyb on 360 tasks, CI lower bound above 0.
- **E22-H2, the fair pool:** hyb-v2 is at least hyb on the 40 select tasks. With so few tasks, this is reported, not
  tested.
- **E22-H3:** nodes to solution (`found_at`), control over hyb-v2, at least 2x on tasks both solve.
- **All arms, precision:** wrong share at most control's plus 2 points.

## What would change the plan
- Gains hold: make hyb-v2 the default and run the wake–sleep loop. Each cycle retrains hints on the newly solved
  tasks, then reruns; a cycle counts as a gain only on the select pool.
- No gain from v2: the own-solution signal is too small (60 tasks); dreams need to come from the search's own
  frontier instead.
