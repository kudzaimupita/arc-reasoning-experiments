# Experiment 20 results: hint-pruned search (12 Nov 2026)

Pre-registration: `PREREG.md`. One binary (commit ddba20b, Linux build sha256 386a53b5…), run on eight
`s-8vcpu-16gb-amd` droplets (unprivileged user, no outbound traffic), graded on the Mac with none missing. The droplets were
destroyed after the runs.

| Arm | Solved of 360 | Answered | Wrong | CPU-s | Nodes to solution, control ÷ arm |
|---|---|---|---|---|---|
| control (depth 4) | 64 | 69 | 5 (7.2%) | 8,855 | n/a |
| p30 (top 30 kinds) | 63 | 67 | 4 (6.0%) | 5,639 | 1.27x [1.14, 1.44] |
| **p15 (primary)** | 61 | 64 | 3 (4.7%) | 3,290 | **1.40x [1.20, 1.66]** |
| p15d5 (depth 5) | 63 | 66 | 3 (4.5%) | 3,441 | 1.40x [1.20, 1.66] |
| p8d5 | 60 | 63 | 3 (4.8%) | 2,077 | 1.42x [1.21, 1.69] |

Node ratios are geometric means of `found_at` for the correct answer, on tasks both arms solve. The arm was faster on
17 tasks and slower on none.

- **H1 (at least 2x fewer nodes to solution): not met.** 1.40x, but the CI excludes 1: hints give a real, smaller
  speed-up.
- **H2 (p15 solves more): not met.** -0.8 points, CI [-2.2, +0.3].
- **H3 (precision): met.** Wrong answers fell from 7.2% to 4.7%.

## What changed
- **Cost:** p15 uses 37% of control's CPU. 252 of its searches finish the whole pruned space (control: 82), so it is
  bounded by depth and by the network's choices, not by budget.
- **Lost by pruning** (a needed kind was outside the top 15): `0b148d64`, `3618c87e`, `8f2ea7aa`, `ce602527`.
- **Gained:**
  - `94f9d214`: wrong in every earlier version, now right.
  - `72ca375d`: p30 only.
  - **`44f52bb0` and `a79310a0`:** p15d5's **first depth-5 solutions** of the project.
- **Union:** control and p15d5 together solve 67 distinct tasks.

## Reading
The network's top 15 kinds contain what most solutions need. Pruning turns the budget surplus into depth, and depth 5
starts paying. Two failures remain:
- A fixed top-k drops the needed kind on about 6% of solved tasks.
- The dreams are random 1 to 3 step programs, so the network has seen little of the long programs depth 5 needs.

## Decision
Hints become part of the search, but not as hard pruning alone. Next steps:
- **E21:** a hybrid that searches full breadth to depth 2, then prunes to the top 15 down to depth 5, so no
  shallow solution is lost.
- **E22:** retrain the network on dreams that look more like real programs:
  - length 1 to 5;
  - sampled from the operation-kind frequencies of the reasoner's own solutions on the train pool;
  - plus hindsight relabelling of programs it ran.

  That is the start of a wake–sleep loop.
