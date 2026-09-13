# FL-5E result — ambiguity-gated strong perturbation

FL-5E tested whether the safety/escape trade-off observed in FL-5B/C/D could be broken by making strong perturbation **state dependent** rather than global. The protocol was frozen in `prereg/FL-5E.md` before implementation and before any FL-5E outcome was inspected.

The gate is target-agnostic: after 128 deterministic settling steps it measures the angular gap between the best and second-best stored templates. Strong OU perturbation is enabled only when that intrinsic ambiguity gap is below a calibration-selected threshold. Every arm then receives a matched additional 128 steps.

## Selected calibration condition

The calibration-only procedure selected:

- OU amplitude: `0.25`;
- OU theta: `2.0`;
- ambiguity threshold `tau`: `0.35` rad.

On calibration, D0 and the selected gated arm both recovered `1/3` wrong-basin targets. Both also retained the same clean-cue recall (`0.5`). The selected gate activated on all calibration wrong-basin trials and half of the calibration clean trials. Its median target-angle error was smaller (`0.6982042441` versus `0.7103778934` for D0), but that geometric improvement did not change terminal recovery.

## Untouched holdout

| Arm | Target recovery | Left original wrong basin | Clean recall | Wrong gate/noise duty | Clean gate/noise duty | Median target angle |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| D0 | 0.333333 | 0.333333 | 0.25 | 0 | 0 | 0.710378 |
| Selected GATED-OU | 0.333333 | 0.333333 | 0.25 | 1.00 | 0.75 | 0.689126 |
| PERM-GATED-OU | 0.333333 | 0.333333 | 0.25 | 1.00 | 0.75 | 0.708241 |
| GLOBAL-OU | 0.333333 | 0.333333 | 0.25 | 1.00 | 1.00 | 0.689126 |

The selected gated arm therefore **did not improve terminal target recovery over D0**. It also did not outperform the temporal-permutation control, and on the declared terminal metrics it did not outperform global OU. The gate was selective in the preregistered narrow sense (`1.00` wrong-basin duty versus `0.75` clean duty), but that selectivity was too weak to create an efficacy advantage.

The gated and global arms shared the same median target-angle improvement relative to D0, while terminal recovery remained unchanged. This is evidence of a geometric displacement, not evidence of successful basin escape.

## Hypotheses

- **H5E-0 selective gate:** supported.
- **H5E-1 safe escape:** **not supported**.
- **H5E-2 gating benefit:** **not supported**.
- **H5E-3 temporal structure:** **not supported**.
- **H5E-4 reproducibility:** supported.

The full campaign replay was exact and every declared protocol validity check passed.

## New finding: the deterministic horizon is itself unstable

FL-5E also exposed a more fundamental issue that was not visible in FL-5D. FL-5D evaluated 128 field steps total. FL-5E deliberately uses a two-stage matched budget: 128 deterministic settling steps followed by another 128 steps for **every** arm, including D0.

At the 128-step Stage-A boundary all six preregistered wrong-basin cases still ended in their declared competitor. However, over the second deterministic stage D0 itself recovered `1/3` of the holdout cases. More importantly, exact clean-cue recall under D0 on the FL-5E holdout was only `0.25` at the 256-step endpoint, despite those clean cues decoding correctly at Stage A.

This means the H=2/2/4 fixture does not behave as a set of indefinitely stable discrete attractors over the longer horizon used by FL-5E. The apparent basin classification is therefore horizon-dependent. FL-5E remains a valid negative experiment under its preregistered rules, because all arms share the same horizon and clean safety was defined relative to matched D0. But the result blocks a clean interpretation of further stochastic tuning as "basin escape" until deterministic basin lifetimes are characterized.

## Consequence for the FL roadmap

The next useful FL-5 gate should therefore **not** be another noise-amplitude sweep. It should first characterize the deterministic residence time and transition structure of the current attractor bank across multiple frozen horizons, including exact stored cues and shallow wrong-basin cues. Only after a stable evaluation horizon or an explicit metastable-state definition is established should another perturbation policy be judged for escape efficacy.

This result does not establish stochastic resonance, biological equivalence, physical magnetic noise benefit, hardware energy reduction, LLM quality improvement or universal retrieval improvement.

Machine-readable retained evidence: `results/FL-5E-ambiguity-gated-strong-perturbation.json`.
