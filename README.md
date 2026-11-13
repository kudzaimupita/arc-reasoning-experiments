# ARC reasoning experiments

A research log of 16 small, pre-registered experiments (E11 to E26) on the
[Abstraction and Reasoning Corpus](https://github.com/fchollet/ARC-AGI) (ARC-AGI). Most of them test a
dependency-free Rust program-search solver ("the reasoner") and ways to guide its search. The last two test
small neural networks trained at test time.

Most of the results are negative, and they are reported as negative. The useful finding is *where* things
failed: once the solver was measured on a pool of tasks it had never been tuned on, neither faster search nor
hand-written operations added a single new rule family.

## Contents

- [The question](#the-question)
- [Setup](#setup)
- [Summary table](#summary-table)
- [The experiments](#the-experiments)
- [What worked, what didn't](#what-worked-what-didnt)
- [Lessons](#lessons)
- [Next steps](#next-steps)
- [Repository layout](#repository-layout)
- [Getting the data](#getting-the-data)
- [Building and running](#building-and-running)
- [References](#references)

## The question

How far can a small, transparent program-search system get on ARC with a fixed budget of 60 CPU-seconds per task,
on a laptop, and what actually limits it: the operations it can express, the order it searches in, or something
else? A secondary question ran through every experiment: does the system know when it doesn't know? It should
give no answer rather than a wrong one.

## Setup

**Tasks.** All tasks come from the public ARC-AGI-1 *training* set (400 tasks). They were split once, before any
experiment, into:

- **train** (320 tasks): used freely for development, gap analysis and anything learned;
- **select** (40 tasks): never used to choose or tune anything; the check on train-pool optimism;
- 40 tasks held back for a later test. They were not read by any experiment here, and nothing about them is
  included.

E11 also used **ARC-2-only select** (116 ARC-AGI-2 training tasks that don't overlap ARC-1).

**The fair pool (from E23).** Forty select tasks are too few to detect a gain of one or two tasks. So E23 built a
larger pool from [RE-ARC](https://github.com/michaelhodel/re-arc): 10 fresh tasks from each of the 40 select
tasks' generators, 400 tasks in all, each with 4 train pairs and 1 test pair, every example checked by RE-ARC's
own verifier. No generator in this pool was used in any training. Tasks from one generator are not independent,
so comparisons on it use a **cluster bootstrap by generator**. The number that matters on this pool is how many
of the 40 *generators* (rule families) the solver solves anything from.

**Scoring.** A task counts as solved only if every test output is exactly right within 2 attempts (the ARC rule,
"top-2"). Wrong answers are counted separately from no answer. "Wrong share" is wrong answers divided by tasks
answered.

**Pre-registration.** Before each counted run, a `PREREG.md` fixed the question, the arms, the budget, the
hypotheses with their test (paired bootstrap, 10,000 resamples, seed 12) and what each outcome would change.
Development checks ran only on train-pool tasks and are reported in the pre-registration. Ideas that failed their
development check were stopped and written up as a `NOTE.md` instead (E15, E25, E26 first part).

**Budget and machines.** 60 CPU-seconds per task, node budget 150,000, 8 tasks in parallel. E11 to E15 ran on an
Apple M1 Max laptop. From E16 the budget is measured on the solving thread's CPU clock rather than the wall clock
(E15 found the bug), and the runs moved to rented cloud CPU machines (eight 8-vCPU droplets with no outbound
network access), so results from E16 on are compared only between arms of the same experiment. E26b ran on one
rented AMD MI350X GPU.

**The reasoner.** Bottom-up enumeration of programs in a hand-written DSL over grids, cheapest programs first,
with observational equivalence: two programs that give the same grids on every input of the task count as one.
A program is a candidate when it reproduces every train pair; the two cheapest distinct test answers are
submitted. If nothing fits, it says nothing. v0 had 44 operation kinds; by E24 the hint network's vocabulary
covered 140 kinds.

## Summary table

Solved counts are top-2. "360" is train + select.

| # | Hypothesis (short) | Main result | Verdict |
|---|---|---|---|
| E11 | icecuber baseline at 60 CPU-s | depth 3 best: **167/360 (46.4%)**; depth 2 150, depth 4 118; ARC-2-only 3/116 (2.6%) | Reference set |
| E12 | Reasoner v0 vs icecuber | **50/360 (13.9%)**, -32.5 points [-37.2, -27.8]; 1 wrong of 51 answered | Far below, as expected; gap is missing operations |
| E13 | DSL v1 closes part of the gap | 58/360, +2.2 points [+0.6, +3.9] vs v0; wrong 7/65 (10.8%) | Small real gain; precision fell |
| E14 | Ranking + colour-map evidence fixes | 59/360; wrong 4/63 (6.3%), target was ≤4% | H2 (no loss) met; H1 (precision) not met |
| E15 | Leave-one-out answer check | withholding at margin 2 removes 1 wrong answer and 2 right ones | Stopped at development check |
| E16 | DSL v1 part 2 (object templates) | 64 vs 59 control, +1.4 points [+0.3, +2.8]; wrong 7.2%; +47% CPU | All hypotheses met |
| E17 | Search order (best-first, train prior) | all four arms solve the same 64 | Not met |
| E18 | Per-task learning (Probe-style) | 64 vs 64; +26% CPU | Not met |
| E19 | Hint network changes costs | 64 vs 64; nodes to solution 1.12x [0.94, 1.36] | Not met (needed kinds rank well, median 4 of 140) |
| E20 | Hint network prunes the search | p15: 61 vs 64, -0.8 [-2.2, +0.3]; 1.40x fewer nodes [1.20, 1.66]; 37% of the CPU; wrong 4.7% | Speed-up real but < 2x; no gain; first depth-5 solutions |
| E21 | Hybrid pruning (full breadth to depth 2) | 65 vs 64, +0.3 [-0.8, +1.7]; loses 2 | Not met |
| E22 | Hints retrained on realistic dreams + own solutions | hyb-v2 66, +0.3 vs hyb [-0.6, +1.4]; p15d5-v2 66, 4.3% wrong, 1.56x; select unchanged at 8/40 | Not met; train gains were on tasks it had trained on |
| E23 | Hints help on 400 unseen tasks | control **63/400 (15.8%), 7 of 40 generators, 0 wrong**; hints 57, -1.5 [-4.5, +0.0] | Not met; bottleneck is rule-family coverage |
| E24 | 20 hand-written operations add rule families | fair pool 63 → 63 (same tasks, same 7 generators); train 56 → 70 with 10 wrong | Not met; overfits the DSL to train tasks |
| E25 | Library learning over own solutions | 73 programs; no step sequence appears in 3+ tasks | Stopped: nothing to compress |
| E26 | Test-time training, from scratch and VARC-style | from scratch: 1 answer on 48 train tasks, wrong; VARC-style 4M: **0/400** fair, 0/320 dev | Not met; cheap version ruled out, not the method |

## The experiments

Full records, including per-task results, are in [`experiments/`](experiments/).

**E11, icecuber baseline.** *Question:* what does icecuber, a strong hand-built C++ search solver, solve on this
laptop at 60 CPU-seconds per task? *Done:* built from upstream with four small patches (libc++ includes, read only
the local task, never see test outputs, a memory and CPU guard), run at search depths 2, 3 and 4. *Result:*
depth 3 is best, 152/320 train and 15/40 select, **167/360 (46.4%)** top-2. Depth 2 gets 150/360; depth 4 gets
118/360 because 31% of its tasks hit the time limit and 9% the memory guard. Top-3 at depth 3 is 156/320 (48.8%)
on train and the union of depths reaches 168/320 (52.5%), consistent with the claimed 209/400 (52%) for the full
solver. On ARC-2-only tasks depth 2 solves 3/116 (2.6%). *Verdict:* reference number fixed at icecuber depth 3.

**E12, reasoner v0.** *Hypothesis:* v0 is far below icecuber, and the useful output is a list of missing
abilities. *Result:* **50/360 (13.9%)**, a paired difference of **-32.5 points [-37.2, -27.8]**. v0 solves nothing
icecuber misses. It answered 51 tasks, 1 wrongly (2%), and gave no answer on 309. 257 searches *exhausted* the
whole program space within limits, so the gap is missing operations, not speed. Using Hodel's hand-written
solutions of the 117 icecuber-only tasks as a proxy, the most-needed abilities were painting cells and patches
(79%), object selection (65%), higher-order composition (58%) and counting (52%); those solutions have a median of
9 steps. *Verdict:* as expected; sets the order of DSL work.

**E13, DSL v1.** *Hypothesis:* new operations (colour-map matching, object operations, boxes, lines, symmetry
completion, crops, self-similar scaling) close part of the gap without much loss of precision. *Result:* 58/360,
**+2.2 points [+0.6, +3.9]** over v0, about 2 of the 32.5 missing points. But wrong answers rose from 1/51 to
**7/65 (10.8%)**: three accidental colour maps, two chance fits of new operations, one old error and one ranking
bug. The larger DSL also made search the limit: tasks stopped by the node cap or time limit went from 52 to 143.
*Verdict:* small, real gain; precision got worse.

**E14, ranking and colour-map fixes.** *Hypothesis:* keeping the cheapest program per answer, and requiring
evidence for colour maps, brings the wrong share to at most 4% (H1) with no loss (H2). *Result:* 59/360, +0.3
points [+0.0, +0.8] over v1 (H2 met). Wrong answers 4/63 (**6.3%**, H1 not met). All wrong answers so far were
on train tasks, the ones the rules were designed from. *Verdict:* rules kept; precision target missed.

**E15, leave-one-out check (development only).** *Idea:* withhold an answer when, on some held-back train pair,
two wrong predictions would have ranked above it. *Result on 55 answered train tasks:* withholding at margin 2
removes 1 wrong answer and loses 2 right ones; two of the four wrong answers have margin 0 (the wrong rule is the
simplest explanation of every subset of the examples). The check also exposed that `--cpu-budget` was measuring
wall-clock time. *Verdict:* stopped; wrong answers come from operations that fit for the wrong reason, which no
check over the same search can catch.

**E16, DSL v1 part 2.** *Hypothesis:* object-selection templates (`Sel(action, parsing, predicate)` with
predicates such as largest, has a hole, odd one out), per-object transforms and rigid moves add tasks against a
control run with the corrected CPU clock. *Result:* **64 vs 59**, **+1.4 points [+0.3, +2.8]**, 5 gained, 0 lost;
wrong share 7.2% (H3 limit 8.3%). The control reproduced E14 exactly (59), so the clock fix and the machine move
changed nothing at the level of solved tasks. Cost: 47% more CPU, and 178 of 360 tasks now stopped on a budget
limit (control 117). `Move` and per-object transforms won nothing. *Verdict:* all three hypotheses met; the case
for better search order grew.

**E17, search order.** *Hypothesis:* best-first search plus a prior from correct train-pool programs solves more at
the same budget. *Result:* control, best-first, prior and best-first+prior **all solve exactly the same 64
tasks**. The prior added one wrong answer. E17 also showed that "total nodes" is the wrong cost measure for
best-first search; from here on `found_at` (nodes explored when the correct answer was first found) is recorded.
*Verdict:* not met; search order alone changes nothing.

**E18, per-task learning (Probe-style).** *Hypothesis:* a first pass that credits operations appearing in near
misses (≥90% of train cells right), followed by a re-weighted restart, solves more (after Barke et al. 2020).
*Result:* the same 64 tasks, the correct answer found at the same node count on every one, and 26% more CPU.
*Verdict:* not met.

**E19, the hint network.** *Hypothesis:* an MLP (64 task features → 140 operation kinds, 0.37M parameters) trained
on 2,000,000 "dreams" (random 1 to 3 step programs run on train-pool inputs) makes search at least 2x cheaper and
solves more. *Result:* the same 64 solved; nodes to solution **1.12x [0.94, 1.36]**, faster on 9 tasks and slower
on 12. Yet the predictions were good: on 52 correct train programs a needed kind's median rank was **4 of 140**,
and 66 of 77 needed kinds were in the top 30. The solved tasks were mostly found at depth 1 or 2, where there is
little to save, and a cost shift of at most 6 bits leaves the breadth at depths 3 and 4 unchanged. *Verdict:* not
met; the network is informative but used weakly.

**E20, hint-pruned search.** *Hypothesis:* trying only the network's top-K kinds below depth 1 gives ≥2x fewer
nodes to solution (H1) and more solved (H2). *Result:* primary arm p15: **61 vs 64**, -0.8 points [-2.2, +0.3];
nodes to solution **1.40x [1.20, 1.66]** (faster on 17 tasks, slower on none); 37% of control's CPU; wrong share
down from 7.2% to 4.7%. Four tasks were lost because a needed kind was outside the top 15. With depth 5 (p15d5,
63 solved) the project got its **first depth-5 solutions**. *Verdict:* H1 and H2 not met; a real but smaller
speed-up; hints make the search cheaper and more precise.

**E21 and E22, hybrid pruning and retrained hints.** *Hypotheses:* (E21) full breadth to depth 2, then pruning to
depth 5, keeps shallow solutions and adds deep ones; (E22) hints retrained on 3,000,000 more realistic dreams
(1 to 5 steps, weighted by kinds seen in real solutions) plus 78 correct programs the reasoner found for 60 train tasks prune
better. *Result:* hybrid 65 vs 64, +0.3 [-0.8, +1.7], and it loses 2 tasks because depth 2 at full breadth fills
the node cap. hyb-v2 66, +0.3 over hyb [-0.6, +1.4], with a wrong share of 10.8%, over the precision limit.
Every arm solves the same **8 of 40 select tasks**. The best setting, p15d5-v2, solves 66 with no loss, 4.3%
wrong, a third of the CPU and 1.56x fewer nodes, but both its gains were tasks whose solutions were in the hint
training data. *Verdict:* all pre-registered tests not met; p15d5-v2 is the cheapest, most precise search, with
no evidence of generalisation.

**E23, the fair pool.** *Hypothesis:* hints help on 400 unseen RE-ARC tasks (H1), and v2 beats v1 (H2).
*Result:* control **63/400 (15.8%)** with **0 wrong answers**, solving something from **7 of 40 generators**.
Both pruned arms solve 57, **-1.5 points [-4.5, +0.0]**, all of the loss from one generator whose needed kind is
pruned; v1 and v2 solve identical sets. *Verdict:* not met. The solver knows rule families, not tasks: faster
search only speeds up families it can already express. E22's train-pool gain was memorisation.

**E24, DSL v2 batch 1.** *Hypothesis:* 20 operations chosen from Hodel's solutions of unsolved train tasks
(neighbour halos, lines, connections, boxes, downsampling, periodic crops, counting, rotation tiling, a diagonal
object parsing) add rule families on the fair pool. *Result:* **exactly the same 63 fair-pool tasks, the same
7 generators**, 0 wrong. On train the same operations add 14 tasks (56 → 70, +3.9 points on 360 [+1.9, +6.1])
with 10 wrong answers (12.5%), and select stays at 8 of 40. *Verdict:* not met. This is overfitting at the level of
the DSL: each hand-written operation covers the tasks it was taken from. Select has not moved from 8 of 40 since
E12.

**E25, library learning (development only).** *Idea:* compress the reasoner's own correct programs into new
reusable operations, DreamCoder-style. *Check:* 73 correct train programs, nearly all 1 or 2 steps long (52 of
1 step, 32 of 2). The most common step sequences appear in only 2 tasks each; none reaches the pre-set threshold
of 3. *Verdict:* stopped. Abstraction learning needs long programs with shared structure, and a solver this weak
doesn't produce them.

**E26, test-time training.** *First part (development only):* a 0.3M-parameter convolutional net trained from
scratch per task on its own train pairs (with the 8 flips and rotations), answering only if it predicts a
held-back train pair exactly and 5 of 8 views agree. On the first 48 same-size train tasks it got the held-back
pair wrong on 47 and its one answer was wrong. It memorises pixels, not rules. *E26b (pre-registered):* a
VARC-style setup: a 4.02M-parameter conditioned convolutional net pretrained for 20,000 steps (609 s on one GPU)
on 86,400 RE-ARC examples from 288 train-pool generators, then fine-tuned per task. It answered **0 of 400**
fair-pool tasks and 0 of 320 dev tasks; even on its pretraining generators it predicted at most 5% of whole grids
exactly. A trend check at 7.28M parameters and 22,000 steps lowered the loss (0.16 → 0.149) but exact accuracy on
unseen dev generators stayed at **0.8% (2 of 256)**, the same as the 4M model. *Verdict:* not met. This rules out
the cheap version (a 4M conv net and about 10 GPU-minutes, against an 18M vision transformer and about
38 H100-hours in the paper), not the method.

## What worked, what didn't

**Worked**

- **Honest abstention.** The reasoner gives no answer when nothing fits. On the fair pool its control answered 63
  tasks and got all 63 right. Precision was tracked in every experiment and a regression (E13) was caught and
  partly fixed (E14).
- **Pre-registration with a decision rule.** It forced clear verdicts, including on the arms that tied exactly
  (E17, E18) and on gains that came from training on the tasks being scored (E22).
- **The fair pool with cluster statistics.** It turned "+1 or 2 tasks on train" into a question with a clear
  answer, and showed the real unit of progress: rule families (generators), not tasks.
- **Learned pruning as an efficiency tool.** Hint pruning cut CPU to about a third, cut wrong answers, and reached
  depth 5, with 1.4 to 1.56x fewer nodes to solution on train + select.
- **Small DSL extensions, on train + select.** DSL v1 (+2.2 points) and part 2 (+1.4 points) were real
  improvements on the 360-task pools.

**Didn't work**

- Search order without learning (E17), per-task near-miss learning (E18), and hints as cost shifts (E19).
- Hints as a route to new tasks: on unseen tasks they were slightly worse (E23).
- Hand-written operations as a route to new rule families (E24): +14 train tasks, 0 fair-pool tasks.
- A leave-one-out check against chance fits (E15), and library learning from a corpus of 1 to 2 step programs (E25).
- Cheap test-time-trained networks (E26): from scratch and VARC-style, both 0 useful answers.

## Lessons

1. **Measure on tasks you have never touched, with the right unit.** Every train-pool gain from E13 to E24 needs
   to be read as an upper bound. The select pool sat at 8 of 40 from E12 to E24, and the fair pool did not move
   from 7 of 40 generators.
2. **Coverage of rule families, not search order, is the limit for a DSL solver at this level.** Faster search
   only speeds up what the DSL can already express.
3. **A bigger DSL is not free.** Each operation buys a little coverage, costs search breadth and adds chance fits
   (E13: precision from 98% to 89%).
4. **Good predictions are not enough; how they are used matters.** The hint network ranked needed operations
   well from the start (median rank 4 of 140), but cost shifts did nothing and only pruning paid.
5. **Check the measurement itself.** Two bugs in the yardstick were found by experiments, not by review: the
   wall-clock "CPU" budget (E15) and the total-nodes cost measure (E17).
6. **Negative results are informative when they are clean.** Four arms solving exactly the same 64 tasks, or two
   DSL versions solving exactly the same 63 fair-pool tasks, say more than a noisy +1.

## Next steps

- A faithful VARC reproduction: an 18M vision transformer with scale and translation augmentation, per-task
  embeddings and tens of GPU-hours of pretraining, judged only by its fair-pool score and whether it solves
  generators the reasoner can't.
- A CompressARC-style per-task model (no pretraining, an equivariant architecture and a description-length
  objective), about 20 GPU-minutes per task on the fair pool.
- Translating Hodel's 400 hand-written solutions into the reasoner's DSL as a corpus for abstraction learning,
  counted as human-supplied prior knowledge.
- Combining the two routes: let a network propose rule families and let the reasoner verify programs, keeping its
  0% wrong-answer behaviour.

## Repository layout

```
reasoner/         Rust program-search solver (no dependencies); hint-network weights compiled in
  src/dsl.rs        operations and costs
  src/search.rs     bottom-up search, ranking, best-first, prior, probe, hints and pruning
  src/hints.rs      task features, hint network inference, dream generation
  src/hints*.json   trained hint weights (v1: E19-E20, v2: E22)
  src/prior.txt     operation-kind counts from correct train programs (E17), built by tools/make_prior.py
ttt/              ttt_solver.py (E26, per-task from scratch), varc.py (E26b, pretrain then fine-tune)
baselines/
  icecuber/         patches, Makefile and driver for the E11 baseline
  re-arc/           driver that generates verified RE-ARC tasks (fair pool, pretraining data)
tools/            arc_job.py (prepare and grade batches), paired and clustered bootstrap comparisons,
                  hint training, prior building, Hodel-solution analyses
experiments/      one folder per experiment: pre-registration, summary, per-task results and grades
reports/          hodel-primitive-counts.csv (arc-dsl primitive use in the 320 train-pool solvers)
examples/         two toy tasks for a quick end-to-end check
```

## Getting the data

No ARC data is included. To reproduce runs, download:

- **ARC-AGI-1**: `git clone https://github.com/fchollet/ARC-AGI` and use `data/training/` (400 tasks).
- **ARC-AGI-2** (only for the ARC-2-only pool of E11): `git clone https://github.com/arcprize/ARC-AGI-2`.
- **RE-ARC** (fair pool and E26b pretraining data): `git clone https://github.com/michaelhodel/re-arc`
  (the experiments used commit e5b7f1d). Remove the two matplotlib import lines from its `utils.py` if
  matplotlib is not installed.
- **arc-dsl** (only for the Hodel-solution analyses): `git clone https://github.com/michaelhodel/arc-dsl`.

The task ids of each pool are the ids in the grade files, for example:

```sh
mkdir -p jobs
python3 tools/arc_job.py ids experiments/e12-reasoner-v0/grades/train.json  > jobs/train-ids.txt
python3 tools/arc_job.py ids experiments/e12-reasoner-v0/grades/select.json > jobs/select-ids.txt
```

The fair pool is regenerated from the 40 select generators (seed 2026, 10 tasks of 4 train pairs + 1 test pair):

```sh
REARC_DIR=../re-arc python3 baselines/re-arc/rearc_gen.py --ids-file jobs/select-ids.txt \
    --per 10 --pairs 4 --seed 2026 --out jobs/fair
```

```sh
ls jobs/fair/out | sed 's/\.json$//' > jobs/fair-ids.txt
python3 tools/arc_job.py prepare jobs/fair/out jobs/fair-ids.txt jobs/fair-in
```

RE-ARC's generators draw from Python's `random`, so exact tasks depend on the RE-ARC commit and Python version.

## Building and running

Requirements: Rust (stable), Python 3.12, and [uv](https://docs.astral.sh/uv/) for the neural parts.

```sh
make build        # cargo build --release in reasoner/
make demo         # prepare, solve and grade the two toy tasks in examples/
make check        # re-derive the E13 and E23 comparisons from the committed grade files
make prior        # rebuild reasoner/src/prior.txt from the committed grade files
```

**One task:**

```sh
reasoner/target/release/reasoner solve path/to/task.json --cpu-budget 60 --node-budget 150000 --max-depth 4
```

**A pool**, with the settings used from E16 on (add `--prune 15 --max-depth 5 --hints-version 2` for the E22 best
setting):

```sh
python3 tools/arc_job.py prepare ../ARC-AGI/data/training jobs/train-ids.txt jobs/train/in
ARC_JOB_IN=jobs/train/in ARC_JOB_OUT=jobs/train \
  reasoner/target/release/reasoner job --cpu-budget 60 --node-budget 150000 --max-depth 4 --threads 8
python3 tools/arc_job.py grade ../ARC-AGI/data/training jobs/train-ids.txt jobs/train/out jobs/train/grade.json
python3 tools/compare_runs.py mine jobs/train/grade.json -- e16 experiments/e16-dsl-v1-part2/grades/part2-train.json
```

Other reasoner flags: `--best-first`, `--prior` (E17), `--probe` (E18), `--hints` (E19), `--prune K`,
`--prune-from D` (E20, E21), `--loo` (E15). See the header of `reasoner/src/main.rs`.

**Hint network** (E19, E22). Dreams come from the inputs of a prepared job folder:

```sh
ARC_JOB_IN=jobs/train/in ARC_JOB_OUT=jobs/dreams reasoner/target/release/reasoner dream --count 2000000 --seed 19
uv run python tools/train_hints.py jobs/dreams 6 reasoner/src/hints.json
# E22: --weighted --long dreams, plus real examples from tools/real_hint_examples.py
```

The weights are compiled into the binary, so rebuild after retraining.

**Test-time training** (E26). Both scripts read and write job folders like the reasoner:

```sh
uv sync
ARC_JOB_IN=jobs/train/in ARC_JOB_OUT=jobs/ttt uv run python ttt/ttt_solver.py job
REARC_DIR=../re-arc python3 baselines/re-arc/rearc_gen.py --ids-file experiments/e26-test-time-training/pretrain-generators.txt \
    --raw 300 --seed 26 --workers 8 --out jobs/pretrain
uv run python ttt/varc.py pretrain --data jobs/pretrain/out --out jobs/model.pt --steps 20000
ARC_JOB_IN=jobs/fair-in ARC_JOB_OUT=jobs/varc uv run python ttt/varc.py job --model jobs/model.pt
```

Pretraining needs a GPU (CUDA, ROCm or Apple MPS).

**icecuber baseline** (E11, macOS): see the build steps at the top of
`baselines/icecuber/Makefile.icecuber`, then run `baselines/icecuber/icecuber_driver.py` with `ICECUBER_BIN`,
`ARC_JOB_IN` and `ARC_JOB_OUT` set.

## References

- F. Chollet, *On the Measure of Intelligence*, 2019. [arXiv:1911.01547](https://arxiv.org/abs/1911.01547).
  Data: [ARC-AGI](https://github.com/fchollet/ARC-AGI), [ARC-AGI-2](https://github.com/arcprize/ARC-AGI-2),
  [arcprize.org](https://arcprize.org).
- M. Hodel, [arc-dsl](https://github.com/michaelhodel/arc-dsl): a domain-specific language for ARC with
  hand-written solvers for the 400 training tasks.
- M. Hodel, *Addressing the Abstraction and Reasoning Corpus via Procedural Example Generation* (RE-ARC), 2024.
  [arXiv:2404.07353](https://arxiv.org/abs/2404.07353), [code](https://github.com/michaelhodel/re-arc).
- J. S. Wind (icecuber), [ARC-solution](https://github.com/top-quarks/ARC-solution), winner of the 2020 Kaggle ARC
  challenge.
- M. Bober-Irizar and S. Banerjee, *Neural Networks for Abstraction and Reasoning*, 2024.
  [arXiv:2402.03507](https://arxiv.org/abs/2402.03507) (source of the 209/400 icecuber figure used in E11).
- K. Hu, A. Cy, L. Qiu, X. D. Ding, R. Wang, Y. E. Zhu, J. Andreas and K. He, *ARC Is a Vision Problem!* (VARC),
  2025. [arXiv:2511.14761](https://arxiv.org/abs/2511.14761), [code](https://github.com/lillian039/VARC).
- I. Liao and A. Gu, *ARC-AGI Without Pretraining* (CompressARC), 2025.
  [arXiv:2512.06104](https://arxiv.org/abs/2512.06104).
- E. Akyürek et al., *The Surprising Effectiveness of Test-Time Training for Abstract Reasoning*, 2024.
  [arXiv:2411.07279](https://arxiv.org/abs/2411.07279).
- S. Barke, H. Peleg and N. Polikarpova, *Just-in-Time Learning for Bottom-Up Enumerative Synthesis* (Probe),
  OOPSLA 2020. [doi:10.1145/3428295](https://doi.org/10.1145/3428295).
- K. Ellis et al., *DreamCoder: Growing Generalizable, Interpretable Knowledge with Wake-Sleep Bayesian Program
  Learning*, 2021. [arXiv:2006.08381](https://arxiv.org/abs/2006.08381).

## Licence

Apache-2.0 (see [LICENSE](LICENSE)). Third-party works used or patched are listed in
[THIRD_PARTY.md](THIRD_PARTY.md).
