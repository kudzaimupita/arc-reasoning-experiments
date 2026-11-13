# Experiment 16: DSL v1 part 2 (pre-registered 2026-11-12)

## Question
Do fixed higher-order templates over objects add solved tasks, against the CPU-budget control arm (Experiment 16a,
`PREREG-control.md`), without raising the wrong-answer rate?

## System
Reasoner v1.2: the control arm's code (commit f4fda80) plus DSL v1 part 2, from Experiment 12's gap list (objects:
find, filter, pick; composition; moving objects):
- **Two object parsings:** `Multi` (4-connected non-background cells of any colours, as before) and `Mono` (one
  colour).
- **Selection template** `Sel(act, parse, pred)`:
  - Actions: keep, remove, crop (exactly one object) or recolour to c.
  - Predicates: largest, smallest (a unique extreme), single cell, touches border, inner, odd one out by shape,
    colour or size (exactly one object differs, at least 3 objects), has a hole, has no hole.
  - Replaces v1's KeepLargest, RemoveLargest, KeepSmallest, RemoveSmallest, RecolourLargest, RecolourSmallest and
    RecolourSingletons, which are `Sel` with `Multi` and the same predicate. CropLargest and CropSmallest stay
    (their tie handling differs).
- **Per-object transforms** `EachObject(FlipH | FlipV | Rot180)`, each object within its own bounding box.
- **Rigid moves** `Move(Up | Down | Left | Right)`: objects slide until they meet the border or another object.

Costs: `Sel` 6 (recolour 7), `EachObject` and `Move` 5. Everything else is unchanged:
`--cpu-budget 60 --node-budget 150000 --max-depth 4 --threads 8`, CPU-second budget.
Runs: `rsn-e16-train` and `rsn-e16-select`. They start only after the control arm has
finished, with no other heavy job on the Mac.

## Development note (stated honestly)
Operations chosen from the train-pool gap list. Local check on the first 80 train tasks (sorted IDs) at the full
budget: against Experiment 14 on the same tasks, 13 right (+1, `1c786137`), 0 lost, 1 wrong (unchanged).
That check ran at the same time as the control arm, so 12 of the 80 tasks hit the wall-clock cap; its numbers
understate both arms. `b230c067` (recolour the odd one out) is still answered wrongly: the cheaper "recolour the
smallest plus a colour map" wins. No select-pool task was looked at or run.

## Hypotheses
Paired on the same 360 tasks (train + select), bootstrap 10,000, seed 12:
- **H1, gain.** v1.2 minus control: 95% CI lower bound above 0.
- **H2, no harm.** v1.2 minus control: CI lower bound above -1.5 points.
- **H3, precision.** v1.2's wrong-answer share at most the control's plus 2 points.

Secondary: tasks gained and lost; which new operations appear in winning programs; search status counts (a larger
DSL is expected to hit the node cap more often); select pool alone; difference from icecuber depth 3.

## What would change the plan
- H1 holds: keep part 2, go on to search order (E17).
- Only H2 holds: keep part 2 and treat search order as the bottleneck. Report how many more tasks stop on the node
  cap.
- H2 fails: the templates cost more search than they add; prune the predicate list by use counts and rerun once.

## Amendment (2026-11-12, before any result was seen)
The Mac control run (`rsn-e16c-train`) stopped on `watchdog-late` because the Mac was overloaded by other work, and
16 tasks hit the wall-clock cap. Both arms therefore ran on a rented machine instead:
DigitalOcean `gpu-h100x1-80gb` (20 vCPUs, x86-64, Ubuntu), one droplet in nyc2, destroyed after 3 hours.
- **Same code:** commit f4fda80 (control) and 0354f86 (part 2), built there with Rust 1.99 (no dependencies). Same
  settings and CPU-second budget. The two arms ran at the same time, 8 threads each on 20 vCPUs, so they did not
  compete.
- **Simpler isolation than on the Mac:**
  - an unprivileged user;
  - a provider firewall with SSH in from one address and no outbound traffic at all;
  - a 16 GB address-space limit and a 90-minute job timeout.

  No sandbox profile and no watchdog there. The machine held only stripped train and select inputs, and no test
  outputs, tokens or keys.
- **Grading:** outputs were pulled back and graded on the Mac by the strict grader.
- **Not comparable across machines:** a CPU-second on the droplet is not a CPU-second on the M1 Max. Results compare
  the two arms with each other only, not with Experiments 12 to 14 or icecuber's Mac runs.
