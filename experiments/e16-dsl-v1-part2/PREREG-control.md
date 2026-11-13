# Experiment 16a: control arm, reasoner v1.1 with a CPU-second budget (pre-registered 2026-11-12)

## Why
Until commit f4fda80, `--cpu-budget` measured wall-clock seconds. Results in Experiments 12 to 14 therefore depended
on machine load: another virtual machine was using about 6 cores during Experiment 13, for example. The budget now reads the solving
thread's CPU clock. Experiment 16 compares DSL v1 part 2 against this control arm, run with the same clock.

## System
Reasoner at commit f4fda80: DSL v1, Experiment 14's answer rules, `--loo` off, CPU-second budget. Settings:
`--cpu-budget 60 --node-budget 150000 --max-depth 4 --threads 8`. Runs: `rsn-e16c-train` and `rsn-e16c-select`.

## Measures
Solved (top-2, per task), wrong answers among answered tasks, and search status counts on train and select.
Also `clock` per task: it must be `thread-cpu` for every task. If any task falls back to `wall`, the thread clock is
unavailable; this is then reported and the run is not used as a control.

## Expectation (not a hypothesis test)
Equal to or slightly above Experiment 14 (59 of 360). Wall-clock time under load gave less search than 60 CPU-s.
Any difference from Experiment 14 is the size of the clock effect, reported with a paired bootstrap.
