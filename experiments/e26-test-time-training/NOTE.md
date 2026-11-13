# Experiment 26: test-time training from scratch (development result, 12 Nov 2026)

Status: **stopped at the development check.** It does not work as built, so there was no cloud run.

## Method (`ttt/ttt_solver.py`)
Per task, a residual fully convolutional network (width 64, 8 dilated 3×3 blocks, about 0.3M parameters) is trained
from scratch on the task's own train pairs. Training uses the 8 flips and rotations, with random recolouring off
(it breaks colour-specific rules), 1,500 AdamW steps and 1 CPU thread. Phase 1 covers tasks whose outputs are the
same size as their inputs.

Honesty checks:
1. Train on all pairs but the last, and answer only if the held-out pair is predicted exactly.
2. At least 5 of the 8 views agree on each test answer.

## Development data (train pool only)
- **First version** (answers whenever the network reproduces all train pairs): 2 right, 2 wrong out of 4 smoke tasks.
  The network memorises train pairs without learning the rule.
- **With the honesty checks, on the first 48 same-size train tasks** (median 16 CPU-s each):
  - held-out pair wrong on 47;
  - 1 answered, and that answer was wrong.

## Reading
A small network trained from scratch on 2 to 4 examples does not learn transferable rules. It fits pixels, not
concepts. The published small-network results that work bring priors from somewhere:
- **VARC:** a vision transformer **pretrained** on ARC training tasks and RE-ARC, then fine-tuned per task. About
  38 H100-hours of pretraining (claimed).
- **CompressARC:** no pretraining, but a carefully designed equivariant architecture and a description-length
  objective, run for about 20 GPU-minutes per task (claimed).

Our naive version has neither. As built, Phase 1 adds nothing.

## Options
1. **VARC-style:** pretrain the network on the 320 train-pool tasks plus RE-ARC variants from **train-pool
   generators only** (the fair pool's select generators are excluded), then test-time train per task. GPU needed,
   roughly $50 to $200 at cloud H100 or RTX 6000 Ada prices.
2. **CompressARC-style:** reproduce its architecture and objective. No pretraining, but about 20 GPU-minutes per
   task: about $50 to $150 for the 400-task fair pool.
3. **Stop the neural route** and return to the Hodel-corpus option.
