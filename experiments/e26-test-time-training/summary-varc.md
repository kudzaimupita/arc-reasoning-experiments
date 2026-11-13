# Experiment 26b results: VARC-style pretraining plus fine-tuning (12 Nov 2026)

Pre-registration: `PREREG-varc.md`. Ran on DigitalOcean `gpu-mi350x1-288gb-spot` (AMD MI350X, PyTorch 2.13 ROCm 7.1,
about 151 bf16 TFLOPS) as an unprivileged user, with no outbound traffic after setup. The droplet ran about 30 minutes (about
$1.50) and was destroyed after the run.

## Pretraining
- **Model:** 4.02M parameters.
- **Run:** 20,000 steps of batch 64 in 609 s, on 86,400 RE-ARC examples from 288 train-pool generators.
- **Result:** loss fell from 0.39 to 0.16. The share of whole query grids predicted exactly in a training batch
  stayed at 0 to 5%, on generators the model was trained on.

## Results

| Pool | Tasks | Answered | Right | Statuses |
|---|---|---|---|---|
| Dev (32 held-out train-pool generators) | 320 | 0 | 0 | held-out wrong 314, views disagree 6 |
| **Fair pool (40 select generators)** | 400 | **0** | **0** | held-out wrong 397, views disagree 3 |

- **H1 (a new generator solved): not met.** No task was solved.
- **H2 (union beats reasoner): not met.** +0.0; the union is the reasoner's 63.
- **H3 (precision):** not applicable, since nothing was answered.

## Reading
The model never learned to produce whole grids exactly, even on the generators it was pretrained on (at most 5% of
batch grids). ARC needs every cell right, so a model at this level can't pass the held-out check on a new task.
What separates this from the published VARC:
- **Size and architecture:** a 4M convolutional net here, against an 18M vision transformer.
- **Training:** 10 GPU-minutes here, against about 38 H100-hours (claimed); roughly 200 times less compute.
- **Augmentation and test time:** VARC uses scale and translation augmentation, per-task embeddings and much
  longer test-time training, none of which this version has.

This rules out the cheap version, not the method.

## Cost to test the method properly
A faithful reproduction:
- a ViT about 18M in size;
- VARC's augmentations;
- tens of GPU-hours of pretraining, about $100 to $200 at cloud GPU prices;
- about 1 to 2 days of engineering and runs.

The published result (54.5% on ARC-1 eval, claimed) would be the target. Only its fair-pool score, and whether it
adds rule families beyond the reasoner, would count here.

## Trend check (12 Nov 2026, diagnostic, not a hypothesis test)
Does the cheap architecture improve with size and training? It ran on the same type of droplet (about 23 minutes,
about $1.20, destroyed after). The setup was the same as above, but:
- width 224 (7.28M parameters) and 22,000 steps (886 s);
- a new diagnostic, **dev exact**: the share of 256 held-out dev tasks (generators not in pretraining) whose test
  grid is predicted exactly from their train pairs, with one view and no fine-tuning.

The 4M model above scores **0.8%** on the same diagnostic.

| Step | Loss | Dev exact |
|---|---|---|
| 2,000 | 0.30 | 0.0% |
| 6,000 | 0.23 | 0.4% |
| 10,000 | 0.19 | 0.4% |
| 14,000 | 0.17 | 0.8% |
| 18,000 | n/a | n/a |
| 22,000 | 0.149 | **0.8%** |

Loss keeps falling, from 0.16 to 0.149 below the 4M run, but exact whole-grid accuracy on unseen generators stays
at 0.8% (2 of 256). It doesn't move with nearly twice the parameters and a little more training. **Scaling this
convolutional design is not a promising use of money.** A faithful VARC test would need its actual architecture and
recipe (a vision transformer, scale and translation augmentation, per-task embeddings), not more of this one.
