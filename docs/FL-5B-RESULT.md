# FL-5B Result — Clean-cue-constrained basin escape

FL-5B was executed only after [`prereg/FL-5B.md`](../prereg/FL-5B.md) froze the question, smaller amplitude grid, hard clean-cue calibration filter, new case-id namespace, hypotheses and fixture. FL-5 holdout outcomes were not used to pick amplitudes, θ or seeds; the qualitative FL-5 constraint that amp `1.5` destroyed clean-cue recall only motivated declaring a strictly smaller grid in the prereg before execution.

## Protocol validity

The gate is structurally valid:

- frozen 8-node / 3-pattern Hebbian bank matching the FL-1 associative-memory model path;
- new case-id namespace `fl5b|…` (FL-5 identifier strings not reused);
- 30 eligible wrong-basin cases (13 calibration / 17 holdout) and 3 disjoint clean cues (3 / 0 under the frozen SHA-256 partition);
- zero ineligible wrong-basin candidates under D0;
- identical integrator, `dt = 0.05`, mobility `1.0` and horizon `128` for D0 and all perturbed arms;
- amplitude grid `[0.0, 0.05, 0.1, 0.25, 0.5]` with 8 seeds; all non-zero amplitudes strictly below `1.5`;
- seeded Gaussian and Ornstein–Uhlenbeck definitions compatible with NoiseLab `@8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e`, with PERM surrogates under frozen temporal permutation `PERM_SEED = 0x0F15_B005`;
- hard clean-cue filter applied on calibration only before selection;
- holdout excluded from condition selection;
- deterministic replay of an identical `(case, arm, seed)` tuple.

Provenance fingerprint: `fnv1a64:7007dec62d4076d1`.

## Calibration selection

Calibration clean-cue tolerance used the H5-2 rule. With 3 clean cues × 8 seeds = 24 trials, the minimum representable degradation exceeds one percentage point, so the stricter **zero additional errors** criterion applied (`tolerance = 0`).

One OU condition (`amp 0.5`, `θ 0.5`) produced non-zero calibration target-recovery (`≈ 0.0577`) but failed the hard filter (`clean degradation ≈ 0.583`). All other grid cells had zero clean-cue degradation and therefore survived.

Among survivors (all with calibration target-recovery `0`), lexicographic selection chose:

| Field | Value |
| --- | --- |
| Family | Gaussian |
| Amplitude | `0.5` |
| OU θ | none |
| Calibration target-recovery fraction | `0.0` |
| Calibration clean-cue degradation | `0.0` |
| Calibration median terminal target-angle error | `≈ 1.358` |

Zero-amplitude D0 remains the matched baseline and is not itself selectable as the non-zero condition. The hard filter was **not** relaxed after seeing holdout.

## Holdout outcomes

| Arm | Target-recovery fraction | Clean-cue recall | Clean degradation vs D0 |
| --- | ---: | ---: | ---: |
| D0 | `0.0` | n/a (0 holdout clean cases) | `0.0` |
| Selected Gaussian (amp `0.5`) | `0.0` | n/a (0 holdout clean cases) | `0.0` |
| PERM-G surrogate | `0.0` | (reported in artifact) | (reported in artifact) |

Under the frozen `fl5b|` case-id namespace, all three clean cues hashed into calibration, so the holdout clean-cue panel is empty. H5B-3 is therefore vacuously within tolerance (zero trials; zero recorded degradation) and is reported with that limitation explicit.

## Hypothesis outcomes

- **H5B-0 null — supported.** The selected condition does not increase holdout target-recovery over D0 (both `0.0`).
- **H5B-1 safe escape — not supported.** A filter-surviving condition was selected, but holdout recovery did not rise above D0.
- **H5B-2 structured-noise control — supported on the Gaussian caveat path.** Gaussian was selected; ordered Gaussian vs PERM-G is not interpreted as temporal-structure evidence.
- **H5B-3 holdout clean-cue — supported vacuously.** Holdout clean panel empty under the frozen partition; recorded degradation `0.0` against tolerance `0.0`.
- **H5B-4 reproducibility — supported.** Identical `(case, arm, seed)` replay matched.

Negative and mixed outcomes are retained. They constrain continuation of FL-5 rather than authorizing retuning on this holdout or relaxing the clean-cue filter.

## Interpretation boundary

This gate shows only that, on one bounded FieldLab attractor fixture under matched simulation conditions and a hard clean-cue calibration filter with a strictly smaller amplitude grid, no selected safe condition raised wrong-basin holdout recovery above D0. It does **not** establish stochastic resonance, biological correspondence, physical magnetic noise benefit, lower hardware energy, or an LLM-level quality improvement. It also does not authorize FL-6.

Machine-readable evidence: [`results/FL-5B-clean-cue-constrained-basin-escape.json`](../results/FL-5B-clean-cue-constrained-basin-escape.json).
