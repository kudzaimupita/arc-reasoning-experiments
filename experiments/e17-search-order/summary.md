# Experiment 17 results: search order (12 Nov 2026)

Pre-registration: `PREREG.md`. One binary (commit b048595), four arms. Ran on eight DigitalOcean `s-8vcpu-16gb-amd`
droplets: each droplet ran all arms on one eighth of the tasks, with an unprivileged user and no outbound traffic.
Outputs were graded on the Mac; none missing or invalid. Total wall time: about 25
minutes for 1,440 task runs.

| Arm | Solved of 360 | Answered | Wrong | CPU-s, all tasks | Node cap | Time limit |
|---|---|---|---|---|---|---|
| control (level by level) | 64 | 69 | 5 (7.2%) | 8,430 | 132 | 69 |
| bf (best-first) | 64 | 69 | 5 (7.2%) | 8,813 | 132 | 70 |
| prior | 64 | 70 | 6 (8.6%) | 8,453 | 131 | 70 |
| bf+prior | 64 | 70 | 6 (8.6%) | 11,331 | 112 | 88 |

- **H1 (bf+prior beats control): not met.** The difference is 0.0 points: exactly the same 64 tasks are solved by
  every arm.
- **H2 (wrong share at most control's + 2 points): met.** 8.6% against a limit of 9.2%.

## Reading
- Search order alone changes nothing at this budget. Every task that some order can solve within the limits is found
  by all of them. The tasks still missing need programs the order never reaches within 150,000 stored nodes, or
  operations the DSL doesn't have.
- **A measurement gap found here:** "nodes" was the total explored. Best-first keeps searching after its first answer,
  to show nothing cheaper exists, so on tasks every arm solves it explored more (geometric mean 0.39x as efficient
  as control by this count). The reasoner now records `found_at`, the nodes explored when each answer was first
  found, and Experiment 19 measures that.
- The train-pool prior added one wrong answer and no right ones.

## Decision
H1 fails, so search order without learning is not enough. As pre-registered, go on to per-task learning (E18) and the
hint network (E19), which learn the order. Since the arms tie, the default search stays level by level.
