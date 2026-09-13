# FL-5 Result — Noise-assisted basin escape and recall

FL-5 was executed only after [`prereg/FL-5.md`](../prereg/FL-5.md) froze the question, arms, partition rule, calibration selection order, hypotheses and (in the introducing PR) the numeric grid plus first-gate fixture.

## Protocol validity

The first gate is structurally valid:

- frozen 8-node / 3-pattern Hebbian bank matching the FL-1 associative-memory model path;
- 30 eligible wrong-basin cases (16 calibration / 14 holdout) and 3 disjoint clean cues (1 / 2);
- zero ineligible wrong-basin candidates under D0;
- identical integrator, `dt = 0.05`, mobility `1.0` and horizon `128` for D0 and all perturbed arms;
- seeded Gaussian and Ornstein–Uhlenbeck definitions compatible with NoiseLab `@8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e` (SplitMix64 + Box–Muller; exact OU transition), with PERM surrogates under a frozen temporal permutation;
- holdout excluded from condition selection;
- deterministic replay of an identical `(case, arm, seed)` tuple.

Provenance fingerprint: `fnv1a64:cf57d6bca3c1efa9`.

## Calibration selection

Lexicographic calibration-only selection chose:

| Field | Value |
| --- | --- |
| Family | OU |
| Amplitude | `1.5` |
| OU θ | `2.0` |
| Calibration target-recovery fraction | `0.0703125` |
| Calibration clean-cue degradation | `0.625` |
| Calibration median terminal target-angle error | `≈ 1.724` |

Zero-amplitude D0 remains the matched baseline and is not itself selectable as the non-zero condition.

## Holdout outcomes

| Arm | Target-recovery fraction | Clean-cue recall | Clean degradation vs D0 |
| --- | ---: | ---: | ---: |
| D0 | `0.0` | `1.0` | `0.0` |
| Selected OU (amp `1.5`, θ `2.0`) | `0.026785714285714284` | `0.25` | `0.75` |
| PERM-OU surrogate | `0.03571428571428571` | (reported in artifact) | (reported in artifact) |

## Hypothesis outcomes

- **H5-0 null — not supported.** Holdout recovery under the selected perturbation exceeds D0.
- **H5-1 escape — supported in this fixture.** Holdout target-recovery rises from `0` under D0 to `≈ 0.0268` under the selected OU condition.
- **H5-2 clean-cue safety — not supported.** Absolute holdout clean-cue degradation is `0.75`, far above the preregistered one-point / zero-additional-error tolerance for this small panel.
- **H5-3 structured-noise control — not supported.** OU was selected, but PERM-OU slightly exceeded the ordered OU arm on holdout target-recovery (`≈ 0.0357` vs `≈ 0.0268`), so temporal structure is not evidenced.
- **H5-4 reproducibility — supported.** Identical `(case, arm, seed)` replay matched.

Negative and mixed hypothesis outcomes are retained. They constrain continuation of FL-5 rather than authorizing retuning on holdout.

## Interpretation boundary

This gate shows only that, on one bounded FieldLab attractor fixture under matched simulation conditions, a preregistered OU perturbation can raise wrong-basin target recovery above D0 while failing the declared clean-cue safety and structured-noise controls. It does **not** establish stochastic resonance as a general cognitive principle, biological correspondence, physical magnetic noise benefit, lower hardware energy, or an LLM-level quality improvement. A later resonance claim would still require an interior response optimum under a frozen sweep, uncertainty across seeds, and evidence against edge-of-grid artifacts.

Machine-readable evidence: [`results/FL-5-noise-assisted-basin-escape.json`](../results/FL-5-noise-assisted-basin-escape.json).
