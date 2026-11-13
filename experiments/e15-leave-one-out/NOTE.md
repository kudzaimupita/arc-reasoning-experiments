# Experiment 15: leave-one-out answer check (development result, 12 Nov 2026)

Status: **stopped at the development check, before pre-registration.** The check was tried on train-pool tasks only.
It does not separate right answers from wrong ones, so it was not run on the select pool. The code stays in the
reasoner behind a flag (`--loo`, off by default) for later ablations.

## The check
In the same search, record every program that fits all train pairs but one, and what it predicts for the pair it
misses. For each train pair, keep the two cheapest distinct wrong predictions. If, on some pair, two wrong
predictions are cheaper than the best answer, a search shown only the other pairs would have ranked two wrong
answers first. The examples then don't pin the rule down, so the answer is withheld. This costs no extra searches.
`loo_margin` reports the most cheaper wrong predictions on any one pair: 0, 1 or 2.

## Development data
55 train tasks: every train task reasoner v1.1 answered in Experiment 14 (51 right, 4 wrong). Full budget.

| Margin | Right answers | Wrong answers |
|---|---|---|
| 0 | 44 | 2 (`60b61512`, `b230c067`) |
| 1 | 5 | 1 (`2dc579da`) |
| 2 | 2 | 1 (`94f9d214`) |

- **Withhold at margin 2** (the planned rule): 1 wrong answer withheld, 2 right answers lost (`0520fde7`,
  `23b5c85d`). Wrong share 3 of 52 (5.8%), against 4 of 55 (7.3%) on these tasks, for 2 fewer solved.
- **Withhold at margin 1 or more:** 2 wrong withheld, 7 right lost.
- Two of the four wrong answers have margin 0. Their wrong program is the simplest explanation of every subset of
  the examples, so no leave-one-out rule over the same search can catch them.

Without `--loo`, 23 of the 24 Experiment 14 development tasks give identical answers. The 24th (`a740d043`) has
the same first answer but lost its second, because it hit the time limit this time (60 s, against 16.6 s before).
The machine was busier. This exposed a gap: `--cpu-budget` is measured in wall-clock seconds (`Instant`), not CPU
seconds as the pre-registrations say. So results depend on machine load, which matters for every
comparison with icecuber. To fix before the next counted run.

## Conclusion
The remaining wrong answers aren't caused by too few examples. The DSL has an operation that fits the examples
for the wrong reason, for example "recolour the smallest" where the rule is "recolour the odd one out". Checking
against the same search can't fix that. Better operations and a better prior over programs can. So the plan goes
back to DSL v1 part 2, then search order and the hint network. Precision is tracked in every experiment.
