# FL-5D Result — Shallow-boundary clean-cue-constrained escape

FL-5D was executed only after [`prereg/FL-5D.md`](../prereg/FL-5D.md) froze the shallow-boundary construction rule (`target_basin_margin >= -0.40` with D0 terminal competitor), amplitude grid, hard clean-cue calibration filter, new `fl5d|` case-id namespace, hypotheses and a partition-balanced clean-cue identifier list (identifier hashes only). FL-5 / FL-5B / FL-5C recovery numbers were not used to pick amplitudes, θ, seeds or to drop IDs after seeing recoveries. The FL-5 deep bank was not used as a silent fallback.

## Protocol validity

The gate is structurally valid:

- frozen **correlated** 8-node / 3-pattern Hebbian bank (`H=2/2/4`), distinct from the FL-5 / FL-5B / FL-5C deep bank;
- new case-id namespace `fl5d|…` (prior FL-5* identifier strings not reused);
- **shallow fixture constructible:** 6 shallow-eligible wrong-basin cases (2 calibration / 4 holdout) under the frozen margin rule;
- 8 deep-locked competitor terminals and 8 other ineligible candidates reported separately and not counted as escapes;
- 6 disjoint clean cues under the frozen `CLEAN_CUE_IDS` list (**3 calibration / 3 holdout**);
- identical integrator, `dt = 0.05`, mobility `1.0` and horizon `128` for D0 and all perturbed arms;
- amplitude grid `[0.0, 0.05, 0.1, 0.25, 0.5]` with 8 seeds; all non-zero amplitudes strictly below `1.5`;
- seeded Gaussian and Ornstein–Uhlenbeck definitions compatible with NoiseLab `@8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e`, with PERM surrogates under frozen temporal permutation `PERM_SEED = 0x0F15_B005`;
- hard clean-cue filter applied on calibration only before selection;
- holdout excluded from condition selection;
- both partition sides non-empty for clean cues and for shallow-eligible wrong-basin cases;
- deterministic replay of an identical `(case, arm, seed)` tuple.

Provenance fingerprint: `fnv1a64:897cd90b007d5ae3`.

## Calibration selection

Calibration clean-cue tolerance used the H5-2 rule. With 3 clean cues × 8 seeds = 24 trials, the minimum representable degradation exceeds one percentage point, so the stricter **zero additional errors** criterion applied (`tolerance = 0`).

One OU condition (`amp 0.5`, `θ 0.5`) produced non-zero calibration target-recovery (`0.375`) but failed the hard filter (`clean degradation ≈ 0.417`). All other non-zero-recovery cells also violated clean-cue safety; zero-recovery cells with zero clean degradation survived.

Among survivors (all with calibration target-recovery `0`), lexicographic selection chose:

| Field | Value |
| --- | --- |
| Family | OU |
| Amplitude | `0.05` |
| OU θ | `2.0` |
| Calibration target-recovery fraction | `0.0` |
| Calibration clean-cue degradation | `0.0` |
| Calibration median terminal target-angle error | `≈ 0.696` |

Zero-amplitude D0 remains the matched baseline and is not itself selectable as the non-zero condition. The hard filter was **not** relaxed after seeing holdout.

## Holdout outcomes

| Arm | Target-recovery fraction | Clean-cue recall | Clean degradation vs D0 |
| --- | ---: | ---: | ---: |
| D0 | `0.0` | `1.0` (3 holdout clean cases) | `0.0` |
| Selected OU (amp `0.05`, θ `2.0`) | `0.0` | `1.0` | `0.0` |
| PERM-OU surrogate | `0.0` | (reported in artifact) | (reported in artifact) |

Holdout clean-cue panel size is **3** (non-empty on both sides by construction). H5D-3 is therefore evaluated on a real holdout clean panel.

## Hypothesis outcomes

- **H5D-0 null — supported.** The selected condition does not increase holdout target-recovery over D0 (both `0.0`).
- **H5D-1 safe shallow escape — not supported.** Shallow construction succeeded and a filter-surviving condition was selected, but holdout recovery did not rise above D0.
- **H5D-2 structured-noise control — not supported.** OU was selected, but ordered OU holdout recovery did not exceed PERM-OU (both `0.0`).
- **H5D-3 holdout clean-cue — supported.** Holdout clean panel non-empty (3 cases); recorded degradation `0.0` against tolerance `0.0`.
- **H5D-4 reproducibility — supported.** Identical `(case, arm, seed)` replay matched.

Negative and mixed outcomes are retained. They constrain continuation of FL-5 rather than authorizing retuning on this holdout, relaxing the clean-cue filter, or starting FL-6.

## Interpretation boundary

This gate shows only that, on one bounded **shallow-boundary** FieldLab fixture under matched simulation conditions, a hard clean-cue calibration filter, and non-empty clean-cue panels on both partition sides, no selected safe condition raised wrong-basin holdout recovery above D0 — even though shallow construction succeeded and the only calibration cell with substantial recovery failed clean-cue safety. It does **not** establish stochastic resonance, biological correspondence, physical magnetic noise benefit, lower hardware energy, or an LLM-level quality improvement. It also does not authorize FL-6.

Machine-readable evidence: [`results/FL-5D-shallow-boundary-clean-cue-constrained-escape.json`](../results/FL-5D-shallow-boundary-clean-cue-constrained-escape.json).
