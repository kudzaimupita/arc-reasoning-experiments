# Experiment 14: ranking and colour-map fixes (pre-registered 2026-11-12)

## Question
Do two small fixes to answer selection bring the wrong-answer rate back near v0's, without losing solved tasks?

## System
Reasoner v1.1: DSL v1 unchanged (commit 6cdce28) plus two changes in `reasoner/src/search.rs`, made in response to
Experiment 13:
1. **Cheapest program per answer.** When several programs give the same test answer, the answer keeps the cheapest
   program's cost. v1 kept the first one found, so the correct answer to `1b2d62fb` carried a colour map's +5 cost
   and dropped out of the top two.
2. **Colour maps need evidence.** Every colour a map changes must appear in at least 2 train pairs, or on at least 3
   cells. Every colour in a test prediction must have been seen in train, so the map says what it becomes (v1 passed
   unseen colours through unchanged).

Same settings as Experiments 12 and 13: `--cpu-budget 60 --node-budget 150000 --max-depth 4 --threads 8`.
Runs: `rsn-e14-train` and `rsn-e14-select`.

## Development note (stated honestly)
Both rules were designed from Experiment 13's 7 wrong answers, all of which are train-pool tasks. A local check ran
on 24 train tasks only (the 7 wrong ones and the 17 train tasks that v1 solved with a colour map), at the full
budget: 4 wrong (2 became "no answer", 1 became right), 17 of 17 map tasks still right. No select-pool task was
looked at or run. So the train result is optimistic for these 24 tasks; the select pool is the fairer check.

## Hypotheses and measures
Primary (both pools, 360 tasks):
- **H1, precision.** Wrong answers among answered tasks: at most 4% (v1: 10.8%, 7 of 65; v0: 2.0%).
- **H2, no loss.** Solved, top-2, per task: v1.1 minus v1 paired difference with a 95% CI lower bound above
  -1.5 points (non-inferiority).
Secondary: solved against v0 and icecuber depth 3 (paired bootstrap, 10,000, seed 12); tasks gained and lost
against v1; search status counts; select pool on its own.

## What would change the plan
- H1 and H2 hold: keep the rules, go on to DSL v1 part 2 (Experiment 15).
- H1 fails: look for a stricter answer test before adding operations, for example holding out one train pair and
  requiring the program to predict it.
- H2 fails: the evidence rule is too strict; loosen it (cell threshold) and report both.
