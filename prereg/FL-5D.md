# FL-5D — Shallow-boundary clean-cue-constrained escape gate

Status: **preregistered before FL-5D execution**; shallow-boundary construction rule, amplitude grid, hard clean-cue filter, new `fl5d|` case-id namespace, partition-balanced clean-cue identifier list (identifier hash only) and hypotheses frozen with the introducing executable.

## Question

On a **shallow-boundary** associative fixture — D0 wrong-basin cues that remain near a basin boundary (low continuous target-basin margin), not deep locked competitors — can a hard-clean-cue-filtered perturbation raise holdout recovery above D0 without exceeding clean-cue tolerance, and beat its PERM surrogate?

## Motivation and anti-leakage relative to FL-5 / FL-5B / FL-5C

FL-5 / FL-5B / FL-5C used the same deep-attractor 8-node / 3-pattern Hebbian bank. On that bank every condition that recovered also failed clean-cue safety, and every safe condition was null versus D0. FL-5D therefore changes the **dynamical regime**, not the amplitude hunt:

- constructs a correlated shallow-boundary bank/cue fixture under a frozen quantitative shallow-eligibility rule;
- keeps the hard clean-cue calibration filter and the amplitude grid already constrained by FL-5's qualitative amp-`1.5` clean-cue failure (`[0, 0.05, 0.1, 0.25, 0.5]`);
- introduces a new case-id namespace `fl5d|…` so the SHA-256 partition cannot leak prior holdout assignments;
- freezes, before any dynamics, a clean-cue identifier list whose SHA-256 first-byte partition yields ≥2 clean cases on each side.

FL-5D does **not**:

- inspect or reuse FL-5 / FL-5B / FL-5C recovery numbers to choose amplitudes, θ, seeds, family or to drop IDs after seeing recoveries;
- retune on prior holdouts;
- reuse FL-5 / FL-5B / FL-5C case-identifier strings;
- relax the hard clean-cue filter after seeing holdout;
- silently fall back to the old deep bank if shallow construction fails;
- authorize FL-6.

## External perturbation reference

Identical NoiseLab pin as FL-5 / FL-5B / FL-5C:

- repository: `Memorithm/NoiseLab`;
- commit: `8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e`;
- compatible seeded Gaussian / Ornstein–Uhlenbeck / temporal-permutation definitions only — FieldLab does not copy NoiseLab code.

PERM-* arms remain FieldLab temporal permutations of the same generated samples.

## Frozen system under test

Same FL-1 associative-memory field model path (Hebbian `PatternBank` → `EnergyModel` → projected Heun integration):

1. construct the deterministic field model from the frozen shallow-boundary bank;
2. select a target and declared competing memory;
3. create a cue that falls into the competing/wrong basin under D0 **and** satisfies the frozen shallow-boundary eligibility rule below;
4. replay under each frozen perturbation condition;
5. decode only at the common terminal horizon.

## Frozen shallow-boundary eligibility

Define continuous angles via the same mean node-wise angle used by FL-5*:

- `target_angle = mean_angle(terminal_state, encode(target))`
- `competitor_angle = mean_angle(terminal_state, encode(competitor))`
- `target_basin_margin = competitor_angle - target_angle`

Interpretation: deeply locked competitor trajectories have large negative margin; trajectories near the target/competitor boundary have margin near zero.

A wrong-basin candidate is **shallow-eligible** iff, under matched zero-noise D0:

1. terminal decoded pattern equals the declared competitor; **and**
2. `target_basin_margin >= SHALLOW_MARGIN_MIN` with frozen `SHALLOW_MARGIN_MIN = -0.40`.

Cases that end in the competitor but with `target_basin_margin < -0.40` are **deep-locked ineligible** for this gate and are reported separately. Cases that do not end in the declared competitor under D0 are also ineligible and reported. Deep-locked and other ineligible cases are never counted as escapes and never used to fall back to the FL-5 deep bank.

If the frozen construction rule yields **zero** shallow-eligible cases (or either partition lacks shallow-eligible wrong-basin cases), that is a **valid-negative construction** result: report it and stop the gate without silent deep-bank fallback.

## Arms

Every eligible case uses the same integrator, time step and horizon:

- **D0 — deterministic reference:** no perturbation;
- **G — seeded Gaussian perturbation;**
- **OU — seeded Ornstein–Uhlenbeck perturbation;**
- **PERM-G — Gaussian surrogate** (same samples, frozen temporal permutation);
- **PERM-OU — OU surrogate** (same samples, frozen temporal permutation).

## Calibration / holdout separation

Reuse the frozen `fl5_partition` rule on the **exact UTF-8 FL-5D case-id bytes**:

1. SHA-256 over UTF-8 case-id;
2. **calibration** when first digest byte `< 0x80`;
3. **holdout** when first digest byte `>= 0x80`.

Holdout outcomes may not choose family, amplitude, OU θ, horizon or seed set.

## New case-id namespace

FL-5D case identifiers must not equal any FL-5 / FL-5B / FL-5C identifier string:

- wrong-basin: `fl5d|wb|t{target}|c{competitor}|k{variant}`
- clean-cue: `fl5d|clean|t{index}|a{alias}` (exact stored-pattern cues; alias diversifies identifiers for partition balance)

## Frozen clean-cue identifier list (identifier-only; precomputed before dynamics)

Before any trajectory integration, the following **exact** six identifiers are frozen so that each stored pattern appears once on each partition side (≥2 clean cases per side; here 3 / 3):

| Case id | Target pattern | Partition (first SHA-256 byte) |
| --- | --- | --- |
| `fl5d|clean|t0|a1` | P0 | calibration (`0x6b`) |
| `fl5d|clean|t1|a0` | P1 | calibration (`0x5a`) |
| `fl5d|clean|t2|a0` | P2 | calibration (`0x4f`) |
| `fl5d|clean|t0|a0` | P0 | holdout (`0xa8`) |
| `fl5d|clean|t1|a1` | P1 | holdout (`0xfe`) |
| `fl5d|clean|t2|a1` | P2 | holdout (`0xba`) |

Cue content for each row is the exact bipolar stored pattern for that target index. Aliases do not alter the cue. This list was chosen from identifier hashes alone — not from recovery outcomes.

## Frozen calibration grid

Machine-readable constants in `crates/field-bench/src/bin/fl5d.rs`:

| Constant | Frozen value |
| --- | --- |
| `NODE_COUNT` | `8` |
| `FIELD_STEPS` | `128` |
| `FIELD_DT` | `0.05` |
| `FIELD_MOBILITY` | `1.0` |
| `CUE_TILT_RADIANS` | `0.15` |
| `SHALLOW_MARGIN_MIN` | `-0.40` |
| `AMPLITUDES` | `[0.0, 0.05, 0.1, 0.25, 0.5]` (zero retained as reference; all non-zero values strictly below `1.5`; grid reused from FL-5B/FL-5C declaration, already constrained by FL-5 qualitative amp-`1.5` failure, **not** by FL-5D holdout) |
| `OU_THETAS` | `[0.5, 2.0, 8.0]` |
| `SEEDS` | `[1, 2, 3, 5, 8, 13, 21, 34]` (≥8 independent seeds) |
| `PERM_SEED` | `0x0F15_B005` |
| NoiseLab commit | `8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e` |
| `CLEAN_CUE_IDS` | the six frozen identifiers above |

Non-zero amplitudes are evaluated for Gaussian and OU; OU additionally sweeps `OU_THETAS`. Surrogates reuse the same samples under the temporal permutation from `PERM_SEED`.

## Hard clean-cue filter and selection (calibration only)

Clean-cue safety tolerance matches FL-5 / H5-2: absolute clean-cue recall degradation versus D0 on the calibration clean-cue panel is at most **1 percentage point**. If the panel is too small for a one-point increment to be representable, use the stricter criterion of **zero additional errors**.

Selection procedure (executed only when shallow construction succeeds):

1. **Hard filter:** discard every non-zero condition whose calibration clean-cue degradation exceeds the tolerance above.
2. If **no** non-zero condition survives, that is a **valid negative result**. Report it. Do **not** relax the filter after seeing holdout.
3. Among survivors, select lexicographically by:
   1. highest target-recovery fraction among shallow-eligible wrong-basin calibration cases;
   2. lowest median terminal target-angle error;
   3. lowest perturbation amplitude;
   4. Gaussian before OU as the final deterministic tie-break.

Zero-amplitude D0 is never selectable as the non-zero condition.

## Frozen fixture (shallow-boundary bank)

Distinct from the FL-5 / FL-5B / FL-5C deep bank. Patterns are more correlated so D0 wrong-basin terminals can remain near basin boundaries:

- patterns (8 bipolar symbols):
  - `P0 = [+1,+1,+1,+1,+1,+1,+1,+1]`
  - `P1 = [+1,+1,+1,+1,+1,+1,-1,-1]`
  - `P2 = [+1,+1,+1,+1,-1,-1,+1,+1]`
- pairwise Hamming distances: `H(P0,P1)=2`, `H(P0,P2)=2`, `H(P1,P2)=4`
- **wrong-basin candidates:** for every ordered pair `(target, competitor)` with `target != competitor`, include the exact competitor cue and each single differing-bit flip of the competitor toward the target (`k0..`). Apply the frozen shallow-eligibility rule; only shallow-eligible cases enter the escape panel.
- **clean-cue panel:** exactly the six frozen `CLEAN_CUE_IDS` above (exact stored patterns), required disjoint from wrong-basin ids, with ≥2 cases on each partition side.

Under the frozen rule this construction is expected to produce non-empty shallow-eligible wrong-basin sets on **both** partitions (and separately report deep-locked competitor terminals). If that expectation fails at execution, report valid-negative construction.

## Metrics

Report calibration and holdout separately, with per-seed distributions:

- shallow-eligible wrong-basin cases (counts per partition);
- deep-locked and other ineligible wrong-basin candidates (reported, not counted as escapes);
- target recoveries / shallow-eligible cases;
- competing-memory terminal selections;
- unresolved terminal states;
- terminal target-angle error;
- D0 target-basin margin for each wrong-basin candidate;
- first-passage time into the target basin when it occurs;
- fraction of trajectories leaving the initial wrong basin;
- clean-cue recall accuracy on the separate panel (counts per partition; both sides must be non-empty);
- clean-cue terminal target-angle error;
- whether shallow construction succeeded;
- whether any condition survived the hard clean-cue filter;
- selected family / amplitude / OU θ (or explicit none);
- perturbation seeds;
- exact FieldLab commit and NoiseLab reference commit.

## Preregistered hypotheses

- **H5D-0 (null):** either shallow construction fails, or no non-zero condition survives the calibration clean-cue filter, or the selected condition does not increase untouched-holdout target-recovery fraction over D0.
- **H5D-1 (safe shallow escape):** shallow construction succeeds, at least one non-zero condition survives the calibration clean-cue filter, and the selected condition increases untouched-holdout target-recovery fraction over D0.
- **H5D-2 (structured-noise control):** if a condition is selected and it is OU, holdout target-recovery under ordered OU must exceed PERM-OU; if Gaussian is selected, the result must not be interpreted as temporal-structure evidence (Gaussian and PERM-G share intended independence structure) — H5D-2 is then recorded as supported only for the Gaussian non-structure caveat path.
- **H5D-3 (holdout clean-cue):** if a condition is selected, absolute holdout clean-cue recall degradation versus D0 is at most the same tolerance used in the hard filter (≤1 percentage point, or zero additional errors if the panel is too small). Because both clean panels are required non-empty when construction succeeds, H5D-3 is **not** vacuously true due to an empty holdout clean panel. If none survive the filter or construction fails, H5D-3 is not supported.
- **H5D-4 (reproducibility):** rerunning an identical `(case, arm, seed)` tuple produces the identical retained trajectory/result under the deterministic software environment.

H5D-1 may fail while the experiment remains valid. A failure is retained as evidence and constrains continuation of FL-5. This gate does **not** authorize FL-6.

## Anti-leakage and validity gates

Fail closed if any of the following occurs:

- holdout data influences grid selection or filter relaxation;
- FL-5 / FL-5B / FL-5C recovery outcomes are used to pick amplitudes / θ / seeds or to drop IDs after seeing recoveries;
- D0 differs in model, integrator, time step or horizon from a perturbed arm;
- a deep-locked or otherwise ineligible case is silently counted as a successful escape;
- the executable silently falls back to the FL-5 deep bank after shallow construction failure;
- perturbation seeds are not recorded;
- a non-finite state or metric is produced;
- clean-cue and wrong-basin panels overlap when declared independent;
- when construction succeeds, either partition has fewer than 2 clean-cue cases, or either partition has zero shallow-eligible wrong-basin cases;
- the NoiseLab reference commit is omitted or differs from the frozen value without a new preregistration revision;
- an arm receives additional integration steps or compute budget solely because it is noisy;
- FL-5 / FL-5B / FL-5C case-identifier strings are reused.

## Interpretation boundary

A positive H5D-1 result would show only that, on one bounded **shallow-boundary** FieldLab fixture under matched simulation conditions with a hard clean-cue calibration filter and non-empty clean-cue panels on both partition sides, a filter-surviving perturbation can raise wrong-basin recovery above D0, and (when OU is selected and H5D-2 holds) that ordered temporal structure mattered relative to its PERM surrogate. It would not establish stochastic resonance as a universal cognitive principle, biological correspondence, physical magnetic noise benefit, lower hardware energy, or an LLM-level quality improvement. It does not authorize FL-6.

A valid-negative construction result shows only that the declared shallow-eligibility construction produced no usable shallow wrong-basin panel on this attempt — not that shallow regimes are impossible in general.

## Stop condition

FL-5D stops after the executable, frozen constants, frozen shallow-eligibility rule, frozen clean-cue identifier list, hard clean-cue filter, deterministic calibration/holdout split under the new case-id namespace, zero-noise baseline, seeded perturbation arms, surrogate controls, reproducible result artifact and documentation are green on one exact PR head. No FL-6 claim is required or authorized by this gate.
