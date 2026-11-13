# Experiment 14 results: ranking and colour-map fixes (12 Nov 2026)

Pre-registration: `PREREG.md` (committed with the code in 51db625, before the runs). Runs `rsn-e14-train` and
`rsn-e14-select`: exit 0. Peak memory 2.8 GB. Train used 5,200 CPU-s.

## Primary result

| System | Train (320) | Select (40) | Both (360) | Answered | Wrong | Wrong share |
|---|---|---|---|---|---|---|
| reasoner v1.1 (this) | 51 | 8 | 59 (16.4%) | 63 | 4 | 6.3% |
| reasoner v1 (Experiment 13) | 50 | 8 | 58 (16.1%) | 65 | 7 | 10.8% |
| reasoner v0 (Experiment 12) | 42 | 8 | 50 (13.9%) | 51 | 1 | 2.0% |
| icecuber depth 3 | 152 | 15 | 167 (46.4%) | | | |

- **H1 (wrong answers at most 4%): not met.** 4 of 63 (6.3%), down from 7 of 65.
- **H2 (no loss against v1, CI lower bound above -1.5 points): met.** v1.1 minus v1 = +0.3 points,
  95% CI [+0.0, +0.8]. Gained `1b2d62fb`, lost nothing.
- Against v0: +2.5 points, CI [+1.1, +4.2]. Against icecuber depth 3: -30.0 points, CI [-35.0, -25.0].

## What changed
- **Fixed by the ranking rule:** `1b2d62fb` (now right, with the program `Recolour(1, 9) -> AndLR(8)`).
- **Fixed by the evidence rule:** `b9b7f026` and `de1cd16c` now give no answer instead of a wrong one.
- **Still wrong:**
  - `2dc579da` (symmetry then crop) and `60b61512` (filled boxes): new operations fitting by chance.
  - `94f9d214`: also wrong in v0.
  - `b230c067`: "recolour the smallest object" fits the examples, but the real rule is "recolour the odd one out".
  None of these four uses a colour map in a way the evidence rule can catch.

## Caveat on the pools
All 7 of v1's wrong answers, and all 4 of v1.1's, are train-pool tasks. These are the tasks the rules were designed
from. Both versions answered 8 select tasks with 0 wrong. So the select pool can't separate the two versions on
precision: with 8 answers, a 95% upper bound on the wrong share is about 37%. The precision numbers above are
mostly train-pool numbers.

## Search status (360 tasks)

| | v1.1 | v1 |
|---|---|---|
| Answered, full search | 52 | 55 |
| Answered, search cut short | 11 | 10 |
| Exhausted | 148 | 152 |
| Node cap | 105 | 104 |
| Time limit | 44 | 39 |

The rules make the search slightly slower (5,200 against 4,756 CPU-s on train). Budget limits still stop 149 tasks.

## Decision (as pre-registered)
H2 holds, so the rules stay. H1 fails, so before adding more operations, the next experiment tests a stricter answer
check. Leave-one-out: a program counts only if, when one train pair is held back, the search from the remaining
pairs still finds a program that predicts the held-back pair. This targets chance fits by any operation, not just
colour maps. DSL v1 part 2 moves back one place.
