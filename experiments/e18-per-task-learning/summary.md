# Experiment 18 results: learning within a task (12 Nov 2026)

Ran with Experiment 19 on eight droplets: one binary (commit 5669771), graded on the Mac. Results are in
`experiments/e19-hint-network/results-*.jsonl`.

| Arm | Solved of 360 | Answered | Wrong | CPU-s |
|---|---|---|---|---|
| control | 64 | 69 | 5 | 8,565 |
| probe | 64 | 70 | 6 | 10,778 |
| probe+bf+prior | 64 | 70 | 6 | 14,034 |

- **H1 (probe beats control): not met.** The same 64 tasks are solved; probe found its first correct answer at the
  same node count on every one of them.
- **H2 (precision): met.** 8.6% wrong against a limit of 9.2%.

Near misses (at least 90% of train cells right) did not point the restart at operations that finish the job within
budget, and the restart costs 26% more CPU.

## Decision
`--probe` stays off. As pre-registered, the hint network (E19) is the route to search order.
