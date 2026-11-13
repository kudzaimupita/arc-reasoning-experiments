# Experiment 25: library learning (development result, 12 Nov 2026)

Status: **stopped before pre-registration.** The corpus has nothing to compress, so no run was made.

## Check
- **Corpus:** the correct programs for every train-pool task the reasoner has solved in Experiments 12 to 24, at
  most one per experiment: **73 tasks**.
- **Program lengths:** 0 steps: 4, 1 step: 52, 2 steps: 32, 3 steps: 9, 4 steps: 2, 5 steps: 2.
- **Recurring sequences:** step sequences of length 2 and 3, with colour arguments abstracted. The most common
  appear in **2** distinct tasks each:
  - `Recolour > AndLR`, `FillBg > CropContent`, `FlipV > OutlineObjects`, `OutlineObjects > Recolour`;
  - largest-then-smallest recolouring, which appears twice, in two DSL versions.
- **None reaches the pre-set threshold** of use in at least 3 distinct tasks.

## Why
DreamCoder-style abstraction compresses long programs that share structure across many tasks. Our solutions are the
tasks the DSL can already reach in 1 or 2 steps, so there is no shared deep structure to pull out. More instances of
the same families (RE-ARC variants of train tasks) would give more copies of the same short programs, not new
structure. Library learning needs a solve rate and solution depth this system does not yet have: a chicken-and-egg
problem that DreamCoder solved with an easy curriculum, which ARC lacks.

## Consequence
The "grow the library by itself" route is blocked at the current capability level. Other routes:
- translate Hodel's 400 hand solutions into the DSL as a corpus for abstraction (a counted human prior);
- learned perception and concepts;
- test-time-trained neural models (VARC, CompressARC), which are reported to reach 20 to 54% on ARC-1 eval with
  small networks and no LLM.

These are direction choices, not results.
