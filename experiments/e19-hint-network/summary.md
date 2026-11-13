# Experiment 19 results: the hint network (12 Nov 2026)

Pre-registration: `PREREG.md` with its amendment (H1 uses nodes explored until the correct answer was first found).
Ran on eight `s-8vcpu-16gb-amd` droplets (binary sha256 c42e04b3…, commit 5669771), graded on the Mac, with none
missing. The droplets were destroyed after the runs.

| Arm | Solved of 360 | Answered | Wrong | CPU-s |
|---|---|---|---|---|
| bf | 64 | 69 | 5 (7.2%) | 8,988 |
| bf+hints | 64 | 69 | 5 (7.2%) | 10,934 |
| bf+prior+hints | 63 | 69 | 6 (8.7%) | n/a |

- **H1 (at least 2x fewer nodes to solution): not met.** bf over bf+hints, geometric mean on the 64 tasks both solve:
  1.12x, 95% CI [0.94, 1.36]. Faster on 9 tasks, slower on 12.
- **H2 (more solved): not met.** +0.0 points.
- **H3 (precision): met.**

## Diagnosis: the network is informative, the search uses it weakly
On the 52 correct train-pool programs (control arm), using `reasoner hintrank`:
- a needed kind's median rank is **4 of 140**;
- all needed kinds are in the top 10 for 35 of the 52 programs;
- 66 of 77 needed kinds are in the top 30.

So the predictions transfer from dreams to real tasks. Two things kept that from paying off:
- **The solved tasks are easy.** Most are found at depth 1 or 2, where every order reaches them within a few
  thousand nodes. There was little to save.
- **Hints only shift costs by up to 6 bits.** The search still tries all of about 300 operations under every node,
  so breadth at depths 3 and 4, which is what stops 200 tasks, is unchanged.

These results are train-pool numbers, and dreams use train-pool grids. The select-pool calibration was not looked at.

## Decision
The pre-registered "both fail" branch said to check calibration before scaling. It is good enough to act on. Next
(Experiment 20): use hints to **prune**, trying only the top-k kinds (k pre-registered, for example 30) at each node,
so the same node budget reaches depth 4 and beyond. Keep full breadth at depth 1 as a safety net. The 3x
hint target is not met yet.
