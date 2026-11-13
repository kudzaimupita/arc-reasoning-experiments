# Third-party material

Nothing below is vendored in this repository. These are the outside works the code builds on, patches, or reads.

| Work | Licence | How it is used here |
|---|---|---|
| [ARC-AGI-1](https://github.com/fchollet/ARC-AGI) and [ARC-AGI-2](https://github.com/arcprize/ARC-AGI-2) task data | Apache-2.0 | Not included. Task ids appear in result files. Download instructions are in the README. |
| [icecuber / ARC-solution](https://github.com/top-quarks/ARC-solution) by Johan Sokrates Wind, commit 9407072 | MIT | Baseline in E11. `baselines/icecuber/patches/` are patches against its source (diffs contain a few lines of the original code); `Makefile.icecuber` is adapted from its Makefile. |
| [RE-ARC](https://github.com/michaelhodel/re-arc) by Michael Hodel, commit e5b7f1d | MIT | `baselines/re-arc/rearc_gen.py` imports its generators and verifiers to make the fair pool (E23) and the pretraining data (E26b). Its `utils.py` needs its two matplotlib import lines removed to run without matplotlib. |
| [arc-dsl](https://github.com/michaelhodel/arc-dsl) by Michael Hodel | MIT | Read as text by `tools/hodel_primitive_counts.py` and `tools/missing_abilities.py` to count which primitives its hand-written solvers use (`reports/hodel-primitive-counts.csv`). Nothing from it is run or copied. |

The reasoner's DSL (`reasoner/src/dsl.rs`) is written from scratch in Rust. Its operation list was chosen with the
help of the arc-dsl usage counts above, but it shares no code with arc-dsl or icecuber.
