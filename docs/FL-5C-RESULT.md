# FL-5C Result — Non-empty holdout clean-cue basin escape

FL-5C was executed only after [`prereg/FL-5C.md`](../prereg/FL-5C.md) froze the question, amplitude grid, hard clean-cue calibration filter, new `fl5c|` case-id namespace, hypotheses and a **partition-balanced clean-cue identifier list** (identifier hashes only; no dynamics / recovery leakage). FL-5 and FL-5B recovery numbers were not used to pick amplitudes, θ, seeds or to drop IDs after seeing recoveries.

## Protocol validity

The gate is structurally valid:

- frozen 8-node / 3-pattern Hebbian bank matching the FL-1 associative-memory model path;
- new case-id namespace `fl5c|…` (FL-5 / FL-5B identifier strings not reused);
- 30 eligible wrong-basin cases (13 calibration / 17 holdout) and 6 disjoint clean cues under the frozen `CLEAN_CUE_IDS` list (**3 calibration / 3 holdout**);
- zero ineligible wrong-basin candidates under D0;
- identical integrator, `dt = 0.05`, mobility `1.0` and horizon `128` for D0 and all perturbed arms;
- amplitude grid `[0.0, 0.05, 0.1, 0.25, 0.5]` with 8 seeds; all non-zero amplitudes strictly below `1.5`;
- seeded Gaussian and Ornstein–Uhlenbeck definitions compatible with NoiseLab `@8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e`, with PERM surrogates under frozen temporal permutation `PERM_SEED = 0x0F15_B005`;
- hard clean-cue filter applied on calibration only before selection;
- holdout excluded from condition selection;
- both partition sides non-empty for clean cues (≥2 each; here 3/3) and for eligible wrong-basin cases;
- deterministic replay of an identical `(case, arm, seed)` tuple.

Provenance fingerprint: `fnv1a64:6244a7162146701a`.

## Calibration selection

Calibration clean-cue tolerance used the H5-2 rule. With 3 clean cues × 8 seeds = 24 trials, the minimum representable degradation exceeds one percentage point, so the stricter **zero additional errors** criterion applied (`tolerance = 0`).

One OU condition (`amp 0.5`, `θ 0.5`) produced non-zero calibration target-recovery (`≈ 0.0288`) but failed the hard filter (`clean degradation ≈ 0.583`). All other grid cells had zero clean-cue degradation and therefore survived.

Among survivors (all with calibration target-recovery `0`), lexicographic selection chose:

| Field | Value |
| --- | --- |
| Family | OU |
| Amplitude | `0.05` |
| OU θ | `8.0` |
| Calibration target-recovery fraction | `0.0` |
| Calibration clean-cue degradation | `0.0` |
| Calibration median terminal target-angle error | `≈ 1.241` |

Zero-amplitude D0 remains the matched baseline and is not itself selectable as the non-zero condition. The hard filter was **not** relaxed after seeing holdout.

## Holdout outcomes

| Arm | Target-recovery fraction | Clean-cue recall | Clean degradation vs D0 |
| --- | ---: | ---: | ---: |
| D0 | `0.0` | `1.0` (3 holdout clean cases) | `0.0` |
| Selected OU (amp `0.05`, θ `8.0`) | `0.0` | `1.0` | `0.0` |
| PERM-OU surrogate | `0.0` | (reported in artifact) | (reported in artifact) |

Holdout clean-cue panel size is **3** (non-empty on both sides by construction of the frozen identifier list). H5C-3 is therefore evaluated on a real holdout clean panel, not vacuously.

## Hypothesis outcomes

- **H5C-0 null — supported.** The selected condition does not increase holdout target-recovery over D0 (both `0.0`).
- **H5C-1 safe escape — not supported.** A filter-surviving condition was selected, but holdout recovery did not rise above D0.
- **H5C-2 structured-noise control — not supported.** OU was selected, but ordered OU holdout recovery did not exceed PERM-OU (both `0.0`).
- **H5C-3 holdout clean-cue — supported.** Holdout clean panel non-empty (3 cases); recorded degradation `0.0` against tolerance `0.0`.
- **H5C-4 reproducibility — supported.** Identical `(case, arm, seed)` replay matched.

Negative and mixed outcomes are retained. They constrain continuation of FL-5 rather than authorizing retuning on this holdout, relaxing the clean-cue filter, or starting FL-6.

## Interpretation boundary

This gate shows only that, on one bounded FieldLab attractor fixture under matched simulation conditions, a hard clean-cue calibration filter, and non-empty clean-cue panels on both partition sides, no selected safe condition raised wrong-basin holdout recovery above D0. It does **not** establish stochastic resonance, biological correspondence, physical magnetic noise benefit, lower hardware energy, or an LLM-level quality improvement. It also does not authorize FL-6.

Machine-readable evidence: [`results/FL-5C-nonempty-holdout-clean-cue-basin-escape.json`](../results/FL-5C-nonempty-holdout-clean-cue-basin-escape.json).
