# Experiment 13: DSL v1 (pre-registered 2026-11-12)

## Question
Does DSL v1 close part of the gap to icecuber found in Experiment 12, without raising the wrong-answer rate much?

## System
reasoner at commit 6cdce28. Same settings as Experiment 12:
--cpu-budget 60 --node-budget 150000 --max-depth 4 --threads 8.
DSL v1 adds: colour-map matching as a final step (only after programs of at most 2 operations, to stop accidental
fits seen in a local check), in-place object operations (keep, remove, recolour largest / smallest / single cells),
boxes and filled bounding boxes around objects, lines shot from coloured cells and connections between
same-coloured cells, symmetry completion (horizontal, vertical, both, diagonal), crop inside a frame, and
self-similar scaling. Chosen from Experiment 12's missing-ability list (Hodel-solution proxy).

## Development note (stated honestly)
Operations were chosen from the train-pool gap analysis, and checked locally on the first 80 train tasks at
10 CPU-s (12 right, 3 wrong before the colour-map depth guard). No select-pool runs before this experiment.

## Pools and measures
Train (320) and select-arc1 (40). Primary: solved, top-2, per task. Also: wrong-answer rate among answered tasks,
honest no-answer count, search status counts, CPU and memory.

## Analysis
Paired bootstrap (10,000, seed 12) of v1 minus v0 and of v1 minus icecuber depth 3, on the same 360 tasks.
Tasks gained and lost against v0.
