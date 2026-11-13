# Experiment 17: search order (pre-registered 2026-11-12)

## Question
Does trying likely programs first solve more tasks at the same budget, with no new operations?

## Arms
One binary, built from the commit that adds this file. Every arm uses DSL v1 part 2 and the CPU-second budget:
`--cpu-budget 60 --node-budget 150000 --max-depth 4 --threads 8`.
- **control:** no flags. Same search as Experiment 16's part 2 arm: level by level, cheapest parents first.
- **bf:** `--best-first`. Uniform-cost search: always expand the cheapest stored program, across depths. Stop once
  two answers exist and nothing cheaper than the second can still appear.
- **prior:** `--prior`. Level-by-level search, but operation kinds seen in correct train-pool programs get cheaper:
  round(1.5 · log2(1 + n)) bits off, never below 2.
  - Counts come from `reasoner/src/prior.txt`, written by `tools/make_prior.py` from Experiments 12 to 16.
  - Only train-pool results are used: 56 correct programs, 41 kinds.
  - No select-pool result is read.
- **bf+prior:** both flags.

## Where it runs
Eight DigitalOcean droplets (`s-8vcpu-16gb-amd`, 8 vCPUs each, nyc3), each with:
- an unprivileged user;
- a provider firewall with SSH in from one address and no outbound traffic after setup;
- a 16 GB address-space limit and a 90-minute job timeout.

Each droplet runs all four arms, one after another, on one eighth of the tasks (train and select IDs dealt round
robin). Inputs are stripped of test outputs before upload. Outputs are pulled back and graded on the Mac. Results compare arms with each other only.

## Development note (stated honestly)
The prior is fitted on train-pool solutions, so train-pool gains are optimistic for `prior` and `bf+prior`. The
select pool is the fairer test of the prior. A smoke test on 4 train tasks only checked that the code runs: bf used
more nodes before stopping, since it must show nothing cheaper exists.

## Hypotheses
Paired bootstrap, 10,000, seed 12, on 360 tasks:
- **H1:** bf+prior minus control, CI lower bound above 0.
- **H2:** every arm's wrong-answer share is at most control's plus 2 points.

Secondary:
- each single change against control;
- the select pool on its own (prior arms);
- search status counts;
- CPU-seconds used.

## What would change the plan
- H1 holds: keep the winner as the default search, and go on to per-task learning (E18).
- H1 fails: search order without learning is not enough; go on to E18 and E19 anyway, since those learn the order.
