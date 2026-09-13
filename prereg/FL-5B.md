# FL-5B — Clean-cue-constrained basin escape gate

Status: **preregistered before FL-5B execution**; numeric grid, fixture, hard clean-cue filter and hypotheses frozen with the introducing executable.

## Question

Can a perturbation condition that first satisfies a hard clean-cue safety constraint on calibration still raise eligible wrong-basin target recovery above D0 on an untouched new holdout, and beat its own PERM surrogate?

## Motivation and anti-leakage relative to FL-5

FL-5's first gate (PR #17) supported H5-1 (holdout recovery rose under the calibration-selected OU amp `1.5` / θ `2.0`) but failed H5-2: amplitude `1.5` destroyed clean-cue recall on holdout. That qualitative constraint motivates a **strictly smaller amplitude grid**. FL-5B does **not**:

- inspect or reuse FL-5 holdout outcomes to choose amplitudes, θ, seeds or family;
- retune on the FL-5 holdout;
- reuse FL-5 case-identifier strings (a new namespace prevents the SHA-256 partition from leaking the previous holdout assignment).

Negative and mixed FL-5 outcomes remain constraints, not license to relax validity gates after seeing data.

## External perturbation reference

Identical NoiseLab pin as FL-5:

- repository: `Memorithm/NoiseLab`;
- commit: `8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e`;
- compatible seeded Gaussian / Ornstein–Uhlenbeck / temporal-permutation definitions only — FieldLab does not copy NoiseLab code.

PERM-* arms remain FieldLab temporal permutations of the same generated samples.

## Frozen system under test

Same FL-1 associative-memory field model path as FL-5:

1. construct the deterministic field model from the frozen Hebbian bank;
2. select a target and declared competing memory;
3. create a cue that falls into the competing/wrong basin under D0;
4. replay under each frozen perturbation condition;
5. decode only at the common terminal horizon.

Cases that do not enter the declared wrong basin under zero-noise D0 are ineligible and reported, never counted as escapes.

## Arms

Every eligible case uses the same integrator, time step and horizon:

- **D0 — deterministic reference:** no perturbation;
- **G — seeded Gaussian perturbation;**
- **OU — seeded Ornstein–Uhlenbeck perturbation;**
- **PERM-G — Gaussian surrogate** (same samples, frozen temporal permutation);
- **PERM-OU — OU surrogate** (same samples, frozen temporal permutation).

## Calibration / holdout separation

Reuse the frozen `fl5_partition` rule on the **exact UTF-8 FL-5B case-id bytes**:

1. SHA-256 over UTF-8 case-id;
2. **calibration** when first digest byte `< 0x80`;
3. **holdout** when first digest byte `>= 0x80`.

Holdout outcomes may not choose family, amplitude, OU θ, horizon or seed set.

## New case-id namespace

FL-5B case identifiers must not equal any FL-5 identifier string:

- wrong-basin: `fl5b|wb|t{target}|c{competitor}|k{variant}`
- clean-cue: `fl5b|clean|t{index}`

## Frozen calibration grid

Machine-readable constants in `crates/field-bench/src/bin/fl5b.rs`:

| Constant | Frozen value |
| --- | --- |
| `NODE_COUNT` | `8` |
| `FIELD_STEPS` | `128` |
| `FIELD_DT` | `0.05` |
| `FIELD_MOBILITY` | `1.0` |
| `CUE_TILT_RADIANS` | `0.15` |
| `AMPLITUDES` | `[0.0, 0.05, 0.1, 0.25, 0.5]` (zero retained as reference; all non-zero values strictly below the FL-5-destroying amp `1.5`) |
| `OU_THETAS` | `[0.5, 2.0, 8.0]` |
| `SEEDS` | `[1, 2, 3, 5, 8, 13, 21, 34]` (8 independent seeds) |
| `PERM_SEED` | `0x0F15_B005` |
| NoiseLab commit | `8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e` |

Non-zero amplitudes are evaluated for Gaussian and OU; OU additionally sweeps `OU_THETAS`. Surrogates reuse the same samples under the temporal permutation from `PERM_SEED`.

## Hard clean-cue filter and selection (calibration only)

Clean-cue safety tolerance matches FL-5 / H5-2: absolute clean-cue recall degradation versus D0 on the calibration clean-cue panel is at most **1 percentage point**. If the panel is too small for a one-point increment to be representable, use the stricter criterion of **zero additional errors**.

Selection procedure:

1. **Hard filter:** discard every non-zero condition whose calibration clean-cue degradation exceeds the tolerance above.
2. If **no** non-zero condition survives, that is a **valid negative result**. Report it. Do **not** relax the filter after seeing holdout.
3. Among survivors, select lexicographically by:
   1. highest target-recovery fraction among eligible wrong-basin calibration cases;
   2. lowest median terminal target-angle error;
   3. lowest perturbation amplitude;
   4. Gaussian before OU as the final deterministic tie-break.

Zero-amplitude D0 is never selectable as the non-zero condition.

## Frozen fixture

Same bounded FL-1-style Hebbian bank as FL-5, with the new case-id namespace:

- patterns (8 bipolar symbols):
  - `P0 = [+1,+1,+1,+1,+1,+1,+1,+1]`
  - `P1 = [+1,+1,+1,+1,-1,-1,-1,-1]`
  - `P2 = [+1,+1,-1,-1,+1,+1,-1,-1]`
- **wrong-basin panel:** for every ordered pair `(target, competitor)` with `target != competitor`, include the exact competitor cue and each single differing-bit flip of the competitor toward the target (`k0..`). Only D0-terminal-competitor trajectories are eligible.
- **clean-cue panel:** exact stored patterns, partitioned independently, required disjoint from wrong-basin ids.

Sized for CI while producing a non-empty eligible wrong-basin set under D0.

## Metrics

Report calibration and holdout separately, with per-seed distributions:

- eligible wrong-basin cases;
- target recoveries / eligible cases;
- competing-memory terminal selections;
- unresolved terminal states;
- terminal target-angle error;
- first-passage time into the target basin when it occurs;
- fraction of trajectories leaving the initial wrong basin;
- clean-cue recall accuracy on a separate panel;
- clean-cue terminal target-angle error;
- whether any condition survived the hard clean-cue filter;
- selected family / amplitude / OU θ (or explicit none);
- perturbation seeds;
- exact FieldLab commit and NoiseLab reference commit.

## Preregistered hypotheses

- **H5B-0 (null):** either no non-zero condition survives the calibration clean-cue filter, or the selected condition does not increase untouched-holdout target-recovery fraction over D0.
- **H5B-1 (safe escape):** at least one non-zero condition survives the calibration clean-cue filter, and the selected condition increases untouched-holdout target-recovery fraction over D0.
- **H5B-2 (structured-noise control):** if a condition is selected and it is OU, holdout target-recovery under ordered OU must exceed PERM-OU; if Gaussian is selected, the result must not be interpreted as temporal-structure evidence (Gaussian and PERM-G share intended independence structure) — H5B-2 is then recorded as supported only for the Gaussian non-structure caveat path.
- **H5B-3 (holdout clean-cue):** if a condition is selected, absolute holdout clean-cue recall degradation versus D0 is at most the same tolerance used in the hard filter (≤1 percentage point, or zero additional errors if the panel is too small). If none survive the filter, H5B-3 is not supported.
- **H5B-4 (reproducibility):** rerunning an identical `(case, arm, seed)` tuple produces the identical retained trajectory/result under the deterministic software environment.

H5B-1 may fail while the experiment remains valid. A failure is retained as evidence and constrains continuation of FL-5.

## Anti-leakage and validity gates

Fail closed if any of the following occurs:

- holdout data influences grid selection or filter relaxation;
- FL-5 holdout outcomes are used to pick amplitudes / θ / seeds;
- D0 differs in model, integrator, time step or horizon from a perturbed arm;
- an ineligible case is silently counted as a successful escape;
- perturbation seeds are not recorded;
- a non-finite state or metric is produced;
- clean-cue and wrong-basin panels overlap when declared independent;
- the NoiseLab reference commit is omitted or differs from the frozen value without a new preregistration revision;
- an arm receives additional integration steps or compute budget solely because it is noisy;
- FL-5 case-identifier strings are reused.

## Interpretation boundary

A positive H5B-1 result would show only that a perturbation surviving a hard clean-cue calibration filter can still raise wrong-basin recovery on this bounded FieldLab fixture under matched simulation conditions, and (when OU is selected and H5B-2 holds) that the ordered temporal structure mattered relative to its PERM surrogate. It would not establish stochastic resonance as a universal cognitive principle, biological correspondence, physical magnetic noise benefit, lower hardware energy, or an LLM-level quality improvement.

## Stop condition

FL-5B stops after the executable, frozen constants, hard clean-cue filter, deterministic calibration/holdout split under the new case-id namespace, zero-noise baseline, seeded perturbation arms, surrogate controls, reproducible result artifact and documentation are green on one exact PR head. No FL-6 claim is required or authorized by this gate.
