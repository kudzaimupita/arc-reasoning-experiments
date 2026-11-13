# Experiment 18: learning within a task (Probe-style) (pre-registered 2026-11-12)

## Question
Does learning which operations look promising while solving a task, then restarting, solve more tasks at the same
budget?

## Method (`--probe`)
1. **First pass:** a third of the 60 CPU-s. Every program whose train outputs have the right sizes and at least 90%
   of cells right (a near miss) credits each of its operation kinds with its closeness.
2. **Stop early** if the first pass already solved the task or tried everything.
3. **Second pass:** restart with the rest of the budget. Each credited kind is round(2 · log2(1 + credit)) bits
   cheaper, never below 2.
4. **Candidates:** the answers from both passes are merged, with the cheapest program kept per answer.

No train-pool or select-pool information is used: the learning happens inside one task from its own train pairs.
Based on Probe (Barke et al. 2020), which did this for string programs, not ARC.

## Arms
One binary, built from the commit that adds this file. Every arm uses DSL v1 part 2 and the CPU-second budget,
`--cpu-budget 60 --node-budget 150000 --max-depth 4 --threads 8`:
- **control:** no flags;
- **probe:** `--probe`;
- **probe+bf+prior:** `--probe --best-first --prior` (combined with Experiment 17's changes, whatever E17 shows).

## Where it runs
The same eight droplets as Experiment 17, with the same setup, sharding and grading on the Mac.

## Development note
Smoke test on 3 train tasks at 15 CPU-s only checked that the code runs. A bug found there (stopping after the first
pass when there were no near misses) was fixed before this pre-registration.

## Hypotheses
Paired bootstrap, 10,000, seed 12, on 360 tasks:
- **H1:** probe minus control, CI lower bound above 0.
- **H2:** probe's wrong-answer share is at most control's plus 2 points.

Secondary: probe+bf+prior against control and against probe; status counts; CPU.

## What would change the plan
- H1 holds: keep `--probe` in the default search.
- H1 fails: per-task learning from near misses does not transfer to ARC's discrete grids. The hint network (E19),
  which learns across tasks, is the remaining route to search order.
