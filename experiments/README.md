# Experiment records

One folder per experiment, in the order they were run (all on 12 November 2026).

- `PREREG*.md`: the pre-registration, written before the counted runs. Hypotheses, arms, budgets and the decision
  rule are fixed here. Amendments are marked as such and were made before any result was seen.
- `summary*.md` or `NOTE.md`: the result. A `NOTE.md` means the idea was stopped at a development check on the
  train pool and never pre-registered or run on the select pool.
- `results-*.jsonl`: one line per task from the solver itself: search status, nodes explored, CPU time, the clock
  used, the best programs, and `found_at` (nodes explored when each answer was first found; from E18 on).
  `"solved": true` means the solver gave an answer, not that the answer was right.
- `grades/*.json`: one line per task from the grader: `pass` is true only if every test output was right within
  two attempts (the ARC rule). `tools/compare_runs.py` and `tools/compare_clustered.py` read these.

Notes:

- These records were copy-edited for publication: internal paths and references to the lab's job infrastructure
  were removed or replaced. Numbers, hypotheses and verdicts are unchanged.
- Commit hashes in the records (for example `6cdce28`) refer to the history of the private lab repository the work
  was done in, not to this repository. Binary hashes are kept for the same reason.
- E18's results are in `e19-hint-network/` (the two experiments ran together). E21 and E22 share a folder.
- Pools: `train` is 320 ARC-AGI-1 training tasks, `select` is 40 other ARC-AGI-1 training tasks, and the fair pool
  (E23 onward) is 400 RE-ARC tasks, 10 from each select-task generator. Fair-pool task ids are `<generator>_<n>`.
