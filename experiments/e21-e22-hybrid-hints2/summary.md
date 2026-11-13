# Experiments 21 and 22 results: hybrid pruning and retrained hints (12 Nov 2026)

Pre-registration: `PREREG.md`. One binary (commit 53a28a0, Linux sha256 fd7959a9…), run on eight droplets, graded on
the Mac with none missing. The droplets were destroyed after the runs.

| Arm | Solved of 360 | Select (40) | Answered | Wrong | CPU-s | Nodes to solution, control ÷ arm |
|---|---|---|---|---|---|---|
| control | 64 | 8 | 69 | 5 (7.2%) | 8,724 | n/a |
| hyb (full breadth to depth 2, top-15 below, depth 5) | 65 | 8 | 71 | 6 (8.5%) | 2,559 | 1.07x [1.01, 1.18] |
| hyb-v2 | 66 | 8 | 74 | 8 (10.8%) | 2,356 | 1.12x [1.03, 1.24] |
| **p15d5-v2** (top-15 below depth 1, depth 5) | **66** | 8 | 69 | **3 (4.3%)** | 2,892 | **1.56x [1.29, 1.95]** |

## Pre-registered tests
- **E21-H1 (hyb beats control): not met.** +0.3 points, CI [-0.8, +1.7].
- **E21-H2 (hyb loses at most 1 task): not met.** It loses 2 (`3618c87e`, `8f2ea7aa`). Full breadth at depth 2 fills
  the node cap there on 204 tasks, so deeper pruned levels are starved.
- **E22-H1 (hyb-v2 beats hyb): not met.** +0.3, CI [-0.6, +1.4].
- **E22-H2 (select pool, reported):** every arm solves the same 8 of 40.
- **E22-H3 (at least 2x nodes for hyb-v2): not met.** 1.12x.
- **Precision:** met by every arm except **hyb-v2**, at 10.8% against a limit of 9.2%.

## Secondary: the best setting so far
**p15d5-v2:**
- solves 66 and loses none of control's 64 (+0.6 points, CI [+0.0, +1.4]);
- has the fewest wrong answers of any version (4.3%);
- uses a third of control's CPU;
- finds answers 1.56x faster, on 21 tasks faster and none slower.

Against E20's p15d5 (hints v1, same setting, different run), v2 hints recover all 4 tasks that v1's pruning lost.

**Caveat:** both of p15d5-v2's gains, `94f9d214` and `a79310a0`, were among the 60 solved train-pool tasks whose
solutions went into hints v2's training data. So the gain over control is not evidence of generalisation. On the
select pool, which no part of training saw, nothing changed. What v2 did show on unseen data is weaker but real:
it pruned without losing any task, where v1 lost 4.

## Reading
- Pruning plus depth 5 with better hints gives the cheapest, most precise search so far. It does not yet solve new
  unseen tasks.
- The hybrid wastes its node budget at depth 2.
- The hint test that matters is now the select pool, and it needs a larger fair pool than 40 tasks.
  ARC-1's 40 select tasks can't show a 1 to 2 task gain.

## Decision
Make p15d5-v2 the default search (cheaper and more precise, no loss). Before claiming learning gains, enlarge the fair
test: use RE-ARC-style generated variants of select tasks, or the ARC-2-only select pool (116 tasks). The wake–sleep
loop (solve → retrain hints → solve) must be judged only on tasks whose solutions were never in
training.
