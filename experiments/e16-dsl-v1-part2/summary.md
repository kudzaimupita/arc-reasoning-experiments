# Experiment 16 results: DSL v1 part 2 (12 Nov 2026)

Pre-registration: `PREREG.md` (with its amendment) and `PREREG-control.md`. Both arms ran on one rented droplet
(20 vCPUs, x86-64) at the same time, 8 threads each. Outputs were graded on the Mac. Every task used the thread CPU clock (360 of 360 in both arms). Binary hashes are in
`remote-binaries.sha256`. Wall time: control 522 s train + 70 s select, part 2 747 s + 90 s.

## Primary result

| Arm | Train (320) | Select (40) | Both (360) | Answered | Wrong | Wrong share |
|---|---|---|---|---|---|---|
| v1.2, DSL v1 part 2 | 56 | 8 | 64 (17.8%) | 69 | 5 | 7.2% |
| v1.1, control (CPU-second budget) | 51 | 8 | 59 (16.4%) | 63 | 4 | 6.3% |

Paired bootstrap (10,000, seed 12): **part 2 minus control = +1.4 points, 95% CI [+0.3, +2.8]**. Gained 5 tasks,
lost 0.
- **H1 (gain, CI above 0): met.**
- **H2 (no harm, CI above -1.5): met.**
- **H3 (wrong share at most control's + 2 points, so at most 8.3%): met** at 7.2%.

The control matches Experiment 14 exactly (59 of 360), so moving to a CPU-second budget and another machine changed
nothing at the level of solved tasks.

## What the new operations did
Gained tasks, each solved by one new template:
- `1c786137`: `Sel(Crop, Mono, Largest) -> CropContent`
- `810b9b61`: `Sel(Recolour(3), Multi, Holed)`
- `b2862040`: `Sel(Recolour(8), Multi, Holed)`
- `c909285e`: `Sel(Crop, Mono, OddColour)`
- `ce602527`: `Sel(Remove, Multi, Border) -> Dedup -> CropLargest`

Templates appear in 11 of the 64 correct programs. Holes, the one-colour parsing and odd-one-out each earn at least
one task. `Move` and `EachObject` won nothing.

Wrong answers:
- `94f9d214` now gets no answer.
- Two tasks are newly wrong:
  - `b9b7f026`: the evidence rule had blocked v1's colour-map fit here; now a template fits by chance.
  - `fcb5c309`.
- Still wrong: `2dc579da`, `60b61512`, `b230c067`. For `b230c067`, "recolour the smallest plus a colour map" is still
  cheaper than "recolour the odd one out".

## Search cost

| Search status (360) | Part 2 | Control |
|---|---|---|
| Answered, full search | 50 | 55 |
| Answered, cut short | 19 | 8 |
| Exhausted | 113 | 180 |
| Node cap | 132 | 105 |
| Time limit | 46 | 12 |
| CPU-seconds, all tasks | 6,379 | 4,334 |

The larger DSL costs about 47% more CPU. It also moves 67 tasks from "exhausted" to a budget limit: 178 tasks now
stop on the node cap or the time limit, against 117. Every operation added now buys a little coverage and makes
search order matter more.

## Decision (as pre-registered)
H1 holds, so part 2 stays and the work goes on to search order (E17). With 178 of 360 tasks stopped by budget, the
case for learned search guidance is now stronger than the case for more operations.
