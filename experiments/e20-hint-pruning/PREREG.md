# Experiment 20: hint-pruned search (pre-registered 2026-11-12)

## Question
If the hint network decides which operation kinds are tried below depth 1, does the same budget reach deeper
programs and solve more tasks? This is the hint test again: Experiment 19 found good predictions (needed
kinds' median rank 4 of 140 on train-pool solutions) but used them weakly.

## Method (`--prune K`)
Depth 1 tries every operation, as a safety net. Below depth 1, only operations whose kind is among the network's
top K kinds for this task are tried. The network and weights are the same as in Experiment 19 (`reasoner/src/hints.json`,
trained only on dreams from train-pool inputs). Costs are unchanged.

## Arms
One binary, built from the commit that adds this file. Every arm uses DSL v1 part 2, level-by-level search and the
CPU-second budget, `--cpu-budget 60 --node-budget 150000 --threads 8`:
- **control:** `--max-depth 4`;
- **p30:** `--prune 30 --max-depth 4`;
- **p15:** `--prune 15 --max-depth 4` (**primary**);
- **p15d5:** `--prune 15 --max-depth 5`;
- **p8d5:** `--prune 8 --max-depth 5`.

## Where it runs
Eight `s-8vcpu-16gb-amd` droplets, set up as in Experiments 17 to 19:
- an unprivileged user;
- no outbound traffic after setup;
- the binary built once and copied to the others.

Graded on the Mac.

## Development note
Smoke test on 5 train tasks at 15 CPU-s:
- `0520fde7`: nodes to the correct answer 2,182 → 1,051 with p15.
- `1c786137`: 3,176 → 316 with p15.
- Three unsolved tasks: still unsolved. Their search reached depth 4 instead of stopping at depth 2.

Pruning can remove a needed kind: 11 of 77 needed kinds in train-pool solutions ranked below 30 (Experiment 19), so
some tasks will be lost. That cost is measured, not assumed away. Train-pool numbers are optimistic (dreams use
train-pool grids); select is the fairer pool.

## Hypotheses
On 360 tasks, paired bootstrap 10,000, seed 12:
- **H1, nodes to solution.** On tasks both solve, the geometric mean of `found_at` (correct answer), control over
  p15, is at least 2x, with the CI lower bound above 1.
- **H2, solved.** p15 minus control: CI lower bound above 0.
- **H3, precision.** p15's wrong share is at most control's plus 2 points.

Secondary: every other arm against control; tasks lost through pruning; depth reached; select pool alone.

## What would change the plan
- H1 and H2 hold: hints pass their first real test. Make pruning the default, then scale the network (more and
  better dreams, a grid encoder, training on the reasoner's own solutions).
- H1 holds, H2 fails: the saved search isn't yet reaching tasks the DSL can express. Pair pruning with deeper search
  and more operations.
- H1 fails: the network's top kinds miss too often on real tasks. Retrain with a dream distribution closer to real
  programs.
