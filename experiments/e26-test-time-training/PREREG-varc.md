# Experiment 26b: VARC-style pretraining plus per-task fine-tuning (pre-registered 2026-11-12, before any training)

## Data
- **Pretraining:** 86,400 RE-ARC examples, 300 from each of 288 **train-pool** generators. Generated in an isolated
  job (`rearc-pre-2`, exit 0, seed 26) and validated with a strict JSON and grid check.
- **Dev:** 320 tasks, 10 from each of 32 train-pool generators held out by hash
  (`experiments/e26-test-time-training/dev-generators.txt`). These are not in the pretraining data (job
  `rearc-dev-1`, seed 2627).
- **Fair pool (Experiment 23):** 400 tasks from the 40 select generators. No select generator is in pretraining
  or dev. Inputs are uploaded only after pretraining has finished.

## Model and procedure
`ttt/varc.py` at the commit that adds this file:
- **Model:** a pair encoder makes a task vector; a 10-block dilated convolutional decoder with FiLM conditioning;
  width 160; 30×30 canvas with an "outside" class.
- **Pretraining:** 20,000 steps, batch 64, AdamW at 1e-3 with a one-cycle schedule, bf16. Each episode has 2 to 4
  context pairs and 1 query, with a random flip or rotation and a colour permutation.
- **Per task:** 100 fine-tuning steps on leave-one-out episodes from the task's own train pairs, with the last pair
  held back.
- **Honesty checks:** the held-back pair must be predicted exactly, and at least 5 of the 8 views must agree on each
  test answer. Otherwise no answer is given.

## Machine
DigitalOcean `gpu-mi350x1-288gb-spot` (AMD, ROCm build of PyTorch) in ric1, with:
- a provider firewall: SSH in from one address only, and no outbound traffic after setup;
- an auto-destroy timer of 5 hours (at most $15).

Outputs are pulled back and graded on the Mac:
- fair pool: by the strict grader;
- dev: against the validated full tasks, with a script.

## Hypotheses (fair pool, cluster bootstrap by generator, 10,000 resamples, seed 12)
- **H1:** VARC solves at least one task from a generator the reasoner (E23 control) never solves. This is the first
  sign of a new rule family on unseen data.
- **H2:** union (reasoner answer where it has one, else VARC) minus reasoner alone: CI lower bound above 0.
- **H3, precision:** VARC's wrong share among its answers is at most 10%.

Secondary: VARC alone against the reasoner; dev-set results; per-task GPU seconds.

## What would change the plan
- H1 and H2 hold: the neural route adds rule families. Scale it (more data, longer pretraining, larger model) and
  combine it with search.
- Both fail: report it, and see whether the dev set shows the same thing. If so, the model or data is too small
  for transfer.
