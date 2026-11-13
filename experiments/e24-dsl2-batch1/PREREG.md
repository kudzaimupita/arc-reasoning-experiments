# Experiment 24: DSL v2 batch 1, closing rule-family gaps (pre-registered 2026-11-12)

## Question
Do operations chosen from the train-pool gap analysis add **rule families** on the fair pool (400 unseen RE-ARC tasks,
Experiment 23)?

## Where the operations come from
Hodel's solutions of the 264 train-pool tasks that control does not solve (E22). The shortest of those show
recurring patterns the DSL can't express. **Only train-pool tasks were looked at.** No select task, select
generator or fair-pool task was used to choose or tune anything.

## New operations (batch 1)
- **Neighbours:** `HaloOrth(a, c)` and `HaloDiag(a, c)` paint the 4-neighbours or diagonal neighbours of colour-a
  cells with c.
- **Lines:** `LinesV(a, c)` and `LinesH(a, c)` draw full column or row lines through colour-a cells.
- **Connect:** `ConnectWith(a, c)` fills the gap between colour-a cells in a row or column with c.
- **Boxes:**
  - `BoxAround(a, c)` draws an outline one cell outside each colour-a object.
  - `FillInbox(c)` fills each object's bounding-box interior.
- **Shooting:** `ShootOwn(dir)`: every coloured cell extends its own colour over the background.
- **Size:**
  - `DownSample2` and `DownSample3` sample one cell per block;
  - `Period` crops to the smallest repeating tile;
  - `UpByColours` and `UpByCells` upscale by the number of non-background colours, or of cells.
- **Counting:** `CountRow`, `CountObjRow` and `ColourOfLargest`.
- **Tiling:** `Rot4Tile` puts the grid and its three rotations in a 2×2 tile.
- **Object parsing:** a third parsing for `Sel`: `Diag` (single colour, 8-connected).

Costs: 5 for the size, shooting, counting and tiling operations, 6 for `FillInbox`, 8 for colour-pair operations.

## Development note
Dev check on the 26 motivating train tasks, which were all unsolved by control, at 30 CPU-s: 10 right, 5 wrong.
Train-pool results are therefore optimistic. The fair pool is the test.

## Arms
- **Treatment:** one binary built from the commit that adds this file, `--max-depth 4`, CPU-second budget 60 s, node
  budget 150,000. It runs on the train pool (320), the select pool (40) and the fair pool (400), on eight droplets,
  graded on the Mac.
- **Control:** not rerun. The existing control runs come from the same Experiment 22 binary (fd7959a9) on the same
  droplet type: train and select from E22, fair from E23.

## Hypotheses (fair pool; cluster bootstrap by generator, 10,000, seed 12)
- **H1:** treatment minus control, CI lower bound above 0.
- **H2:** treatment solves at least one task from more generators than control (7 of 40).
- **H3, precision:** the wrong share on the fair pool is at most 5%. Control has 0%, and new operations fit by chance
  more often.

Secondary: train and select pools, paired bootstrap, and which new operations appear in correct programs.
