# Experiment 12: reasoner version 0 against icecuber (pre-registered 2026-11-12)

## Question
How does reasoner v0 (Rust, first-order DSL v0 with 44 operation kinds, bottom-up search with
observational equivalence) compare with icecuber at an equal per-task budget on this Mac, and which
abilities does v0 lack?

## Systems
- reasoner v0 at commit a8593c9, no dependencies.
  Settings fixed now: --cpu-budget 60 --node-budget 150000 --max-depth 4 --threads 8.
- icecuber, from Experiment 11, at the depth closest to equal CPU (decided from its PREREG; all depths reported).

## Pools
train (320) and select-arc1 (40). No held-out data is read.

## Measures
- Primary: tasks solved with 2 attempts, per pool. A task counts only if every test output is correct.
- Also: solved test outputs, CPU-ms per task, nodes, search status (solved, time, nodes, exhausted).
- "Knows it does not know": count of tasks with no candidate (honest unsolved) and tasks with a candidate
  that is wrong (confidently wrong). Report both rates.

## Analysis
- Paired comparison on the same 360 tasks: difference in solve rate (v0 minus icecuber) with a 95% bootstrap
  interval (10,000 resamples of tasks, seed 12).
- Lists: tasks icecuber solves and v0 does not, each tagged by hand with the missing ability
  (for example: colour mapping, object-level operation, counting, pattern completion, line drawing).
  Tasks v0 solves and icecuber does not.
- Nothing is tuned on the select pool for this experiment.

## Expectation (stated before running)
v0 is far below icecuber: icecuber has a much larger operation set and years of tuning. The value of this run
is the list of missing abilities that sets the order of DSL v1 work.
