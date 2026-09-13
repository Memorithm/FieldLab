# FL-5 — Noise-assisted basin escape and recall

Status: **preregistered before FL-5 execution**; first-gate constants and fixture frozen with the introducing executable.

## Question

Can a controlled stochastic perturbation improve escape from a deliberately wrong attractor and subsequent recovery of the declared target memory under the same deterministic field model, integration horizon and evaluation budget, without degrading clean-cue recall beyond a preregistered tolerance?

FL-5 begins the resonance / perturbation series only after FL-0 through FL-4 established deterministic field semantics, associative recall, signed competition, hysteresis and an externally grounded CCOS comparison. It does not reuse any FL-4 holdout for tuning.

## External perturbation reference

Noise-generation semantics and resonance terminology are anchored to the current NoiseLab source of truth at:

- repository: `Memorithm/NoiseLab`;
- commit: `8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e`;
- reusable reference surfaces: `gaussian_white_noise`, `ornstein_uhlenbeck_path`, `detect_interior_response_peak`, and NoiseLab's evidence/provenance discipline.

FieldLab does not copy NoiseLab implementation code in this gate. FL-5 uses compatible seeded perturbation definitions and records the exact NoiseLab revision. Any later shared primitive that becomes generally useful should be promoted through SciRust rather than maintained as duplicate logic.

## Frozen system under test

The first FL-5 gate uses the already-qualified FL-1 associative-memory field model. No learned coupling is introduced.

For each retained memory bank:

1. construct the same deterministic field model used by the FL-1 reference path;
2. select a target memory and a declared competing memory;
3. create a cue that deterministically falls into the competing/wrong basin under the no-noise reference trajectory;
4. replay the identical cue under each frozen perturbation condition;
5. decode only at the common terminal horizon.

Cases that do not enter the declared wrong basin in the zero-noise reference are not eligible for this basin-escape experiment and are reported separately rather than silently replaced.

## Arms

Every eligible case is evaluated under the same integrator, time step and horizon:

- **D0 — deterministic reference:** no perturbation;
- **G — seeded Gaussian perturbation:** independent zero-mean perturbation with frozen amplitude;
- **OU — seeded Ornstein-Uhlenbeck perturbation:** temporally correlated zero-mean perturbation with frozen amplitude and correlation parameter;
- **PERM-G — Gaussian surrogate control:** the same generated Gaussian samples reassigned by a frozen temporal permutation;
- **PERM-OU — OU surrogate control:** the same generated OU samples reassigned by a frozen temporal permutation.

The surrogate arms test whether an observed effect depends on temporal structure rather than merely on the perturbation sample distribution. No energy, hardware or biological interpretation is attached to these arms.

## Calibration / holdout separation

Eligible cases are split deterministically by stable case identifier before any perturbation result is inspected.

The partition function is frozen as follows:

1. encode the case identifier as its exact UTF-8 byte sequence;
2. compute SHA-256 over those bytes;
3. assign **calibration** when the first digest byte is `< 0x80`;
4. assign **holdout** when the first digest byte is `>= 0x80`.

The executable `field-bench --bin fl5_partition` materializes this rule and emits the exact case identifier, full SHA-256 digest and assigned partition. The experiment executable must preserve the same case-identifier bytes in its result artifact; changing case-ID serialization after outcomes are observed is prohibited and requires a preregistration revision.

Holdout outcomes may not choose perturbation family, amplitude, OU correlation parameter, horizon or seed set.

## Frozen calibration grid

The first implementation exposes the grid as explicit machine-readable constants in `crates/field-bench/src/bin/fl5.rs` before execution:

| Constant | Frozen value |
| --- | --- |
| `NODE_COUNT` | `8` |
| `FIELD_STEPS` | `128` |
| `FIELD_DT` | `0.05` |
| `FIELD_MOBILITY` | `1.0` |
| `CUE_TILT_RADIANS` | `0.15` |
| `AMPLITUDES` | `[0.0, 0.5, 1.0, 1.5, 2.0]` (exactly zero retained as reference) |
| `OU_THETAS` | `[0.5, 2.0, 8.0]` |
| `SEEDS` | `[1, 2, 3, 5, 8, 13, 21, 34]` (8 independent seeds) |
| `PERM_SEED` | `0x0F15_05ED` |
| NoiseLab commit | `8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e` |

Perturbation families on the grid are Gaussian and OU. Non-zero amplitudes are evaluated for both families; OU additionally sweeps `OU_THETAS`. Surrogate arms reuse the same samples under the frozen temporal permutation derived from `PERM_SEED`.

Calibration selects one non-zero condition lexicographically by:

1. highest target-recovery fraction among eligible wrong-basin cases;
2. lowest clean-cue degradation measured on a separate clean-cue calibration panel;
3. lowest median terminal target-angle error;
4. lowest perturbation amplitude;
5. Gaussian before OU only as the final deterministic tie-break.


## Frozen first-gate fixture

The first FL-5 executable uses a small FL-1-style Hebbian associative-memory bank rather than the full FL-1 7,551-case campaign:

- patterns (8 bipolar symbols):
  - `P0 = [+1,+1,+1,+1,+1,+1,+1,+1]`
  - `P1 = [+1,+1,+1,+1,-1,-1,-1,-1]`
  - `P2 = [+1,+1,-1,-1,+1,+1,-1,-1]`
- **wrong-basin panel:** for every ordered pair `(target, competitor)` with `target != competitor`, include the exact competitor cue and each single differing-bit flip of the competitor toward the target (`k0..k4`). Case ids are `wb|t{target}|c{competitor}|k{variant}`. Only trajectories whose zero-noise terminal decode equals the declared competitor are eligible; ineligible ids are reported and never counted as escapes.
- **clean-cue panel:** exact stored patterns with ids `clean|t{index}`, partitioned independently and required to be disjoint from the wrong-basin panel.
- Partitioning uses the frozen `fl5_partition` SHA-256 rule on the exact UTF-8 case-id bytes.

This fixture is sized for CI while still producing a non-empty eligible wrong-basin set under D0.

## Metrics

Report calibration and holdout separately, with per-seed distributions rather than seed-averaged values only:

- eligible wrong-basin cases;
- target recoveries / eligible cases;
- competing-memory terminal selections;
- unresolved terminal states;
- terminal target-angle error;
- first-passage time into the target basin when it occurs;
- fraction of trajectories leaving the initial wrong basin;
- clean-cue recall accuracy on a separate panel;
- clean-cue terminal target-angle error;
- perturbation family, amplitude, OU parameter and seed;
- exact FieldLab commit and NoiseLab reference commit.

Energy may be reported only as the FieldLab model's declared mathematical energy along the trajectory. It is not a physical energy-consumption measurement.

## Preregistered hypotheses

- **H5-0 (null):** on untouched holdout wrong-basin cases, the calibration-selected perturbation does not increase target-recovery fraction over D0.
- **H5-1 (escape):** on untouched holdout wrong-basin cases, the calibration-selected perturbation increases target-recovery fraction over D0.
- **H5-2 (clean-cue safety):** absolute holdout clean-cue recall degradation versus D0 is at most 1 percentage point. If the panel is too small for a one-point increment to be representable, the executable must use the stricter criterion of zero additional errors.
- **H5-3 (structured-noise control):** if OU is selected, it must outperform its PERM-OU surrogate on target-recovery fraction; if Gaussian is selected, its result must not be interpreted as temporal-structure evidence because Gaussian and PERM-G have the same intended independence structure.
- **H5-4 (reproducibility):** rerunning an identical `(case, arm, seed)` tuple produces the identical retained trajectory/result under the deterministic software environment used by the experiment.

H5-1 may fail while the experiment remains valid. A failure is retained as evidence and constrains continuation of FL-5.

## Anti-leakage and validity gates

The experiment fails closed if any of the following occurs:

- holdout data influences grid selection;
- the zero-noise D0 arm differs in model, integrator, time step or horizon from a perturbed arm;
- an ineligible case is silently counted as a successful escape;
- perturbation seeds are not recorded;
- a non-finite state or metric is produced;
- clean-cue and wrong-basin panels overlap when the executable declares them independent;
- the NoiseLab reference commit is omitted or differs from the frozen value without a new preregistration revision;
- an arm receives additional integration steps or compute budget solely because it is noisy.

## Interpretation boundary

A positive H5-1 result would show only that a declared stochastic perturbation can improve escape/recovery on this bounded FieldLab attractor workload under matched simulation conditions. It would not establish stochastic resonance as a universal cognitive principle, biological correspondence, physical magnetic noise benefit, lower hardware energy, or an LLM-level quality improvement.

The first FL-5 result must retain negative arms and surrogate controls. A later resonance claim additionally requires an interior response optimum under a frozen sweep, uncertainty across seeds and evidence that an apparent optimum is not simply an edge-of-grid effect.

## Stop condition

The first FL-5 gate stops after the executable, frozen constants, deterministic calibration/holdout split, zero-noise baseline, seeded perturbation arms, surrogate controls, clean-cue safety panel and reproducible result artifact are all green on one exact PR head. No downstream FL-6/FL-7 claim is required for this gate.
