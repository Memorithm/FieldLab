# FL-5H — identifiable E1 residence-aware stochastic gate

Status: preregistered before implementation and before any FL-5H outcome is inspected.
Parent FieldLab revision: `fa36d6e` (post FL-B4 / FL-B3 / FL-B2 on main).

## Motivation and boundary

FL-5G showed that the nine-case FL-5F geometry panel has only seven distinct
observable sign inputs and two contradictory-label groups (deterministic ceiling
`7/9`). FL-B3 confirmed Boolean thresholds neither hide nor repair that ceiling.
FL-5F / FL-5E showed that endpoint permanence is horizon-dependent. FL-B4 showed
that Complete-left / ObservationCut-right unlocks determinate dwell contrasts.

FL-5H is therefore the first stochastic gate that:

1. uses **only identifiable** observable inputs (no contradictory labels under the
   sign-input audit; respect the 7/9 ceiling by dropping conflict geometries rather
   than renaming cases);
2. records **censor-aware residence / first-passage / transition** evidence with
   Complete-left (known constructed cue) and ObservationCut-right semantics;
3. prefers the FL-5G E1 axially-stable fixture (`a = 1.25`) with explicit horizon
   semantics.

This gate does **not** authorize FL-6. No biology, hardware, or LLM claim is made.
Prior FL-5* holdouts are not used for tuning.

## Frozen research question

Under the FL-5G E1 (`a = 1.25`) H=2/2/4 bank, with a preregistered identifiable
wrong-basin panel and matched-budget OU perturbation selected under hard clean-cue
safety, does selected OU improve holdout target recovery over matched D0 while
preserving clean recall, and do Complete-left residence metrics distinguish arms?

If E1 axial stability makes target escape impossible over the declared horizon for
all arms, that negative outcome is retained evidence; the model is **not** switched
after seeing outcomes.

## Fixed provenance and fixture

- Noise process semantics: NoiseLab commit `8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e`
  (compatible seeded OU / temporal-permutation definitions; no NoiseLab code copy).
  PERM surrogates remain FieldLab-side.
- Patterns (H=2/2/4): `P0=++++++++`, `P1=++++++--`, `P2=++++--++`.
- Energy: FL-E1 operator lift of the Hebbian graph with local axial anisotropy
  `A_i = diag(a, 0)`, bank-wide `a = 1.25` (FL-5G stiffness rule output).
- Integrator: projected Heun, `dt = 0.05`, mobility `1.0`.
- Horizon: **256** matched integration steps for every arm (`T = 12.8` time units).
  Left observation boundary: `Complete` (cue is a known constructed initial state).
  Right observation boundary: `ObservationCut` at step 256. Do not interpret the
  endpoint as permanence beyond the cut.
- Cue encoding tilt: `0.15` rad (historical).
- Seeds: `[1, 2, 3, 5, 8, 13, 21, 34]`.
- PERM seed: `0x0F15_5008`.
- Namespace: `fl5h|…` only. No FL-5 / 5B / 5C / 5D / 5E / 5F / 5G case identifier is reused.
- Partition: SHA-256 of case id; calibration iff first digest byte `< 0x80`, else holdout.

## Identifiable case set (frozen)

### Clean cues (exact stored patterns)

Frozen identifiers (chosen for partition balance before outcomes):

- `fl5h|clean|t0|a0`, `fl5h|clean|t0|a1`
- `fl5h|clean|t1|a0`, `fl5h|clean|t1|a1`
- `fl5h|clean|t2|a1`, `fl5h|clean|t2|a5`

Target index is the `t*` field. Expected partition: 3 calibration / 3 holdout.

### Wrong-basin cues (unambiguous single-bit corruptions of P0 only)

Drop the FL-5G conflict geometries (full `P1`/`P2` cues labeled as target `0`).
Retain only the four single-bit flips of `P0` toward `P1`/`P2` that FL-5G already
treated as distinct observables:

| Case id | Cue (signs) | Target | Competitor |
| --- | --- | --- | --- |
| `fl5h|wb|t1|c0|bit6` | `++++++-+` | P1 | P0 |
| `fl5h|wb|t1|c0|bit7` | `+++++++-` | P1 | P0 |
| `fl5h|wb|t2|c0|bit4` | `++++-+++` | P2 | P0 |
| `fl5h|wb|t2|c0|bit5` | `+++++-++` | P2 | P0 |

Expected partition: 2 calibration / 2 holdout.

### Identifiability validity gate

Before outcome evaluation, run `group_observables` on `(cue_signs, target_index)`
for all retained labeled cases (clean + wrong-basin). Require:

- `conflicting_groups == 0`;
- `deterministic_maximum_correct == records`;
- distinct observable count equals the number of unique cue sign vectors.

A conflict fails the **protocol**, not a scientific hypothesis. Case-id renaming to
manufacture identity is forbidden.

Construction also requires ≥2 clean and ≥1 wrong-basin cases on each partition side,
disjoint clean/wrong identifiers, and finite Stage-0 (initial) decoded labels.

## Perturbation grid and arms

OU-only grid (Gaussian omitted; E1 axial stiffness is the fixture under test):

- amplitudes: `{0.05, 0.10, 0.25, 0.50, 1.00}`;
- OU theta: `{0.5, 2.0, 8.0}`.

For every candidate `(amplitude, theta)` evaluate on calibration:

1. `OU` — additive OU forcing each step;
2. matched `D0` — no forcing (shared across candidates);
3. after selection only: `PERM-OU` on holdout — same OU samples with frozen temporal permutation.

All arms receive exactly 256 steps.

## Calibration-only selection

Hard clean-cue safety on calibration:

- selected OU clean recall must equal D0 clean recall within the representable
  one-trial tolerance `1 / n_clean_trials` (same rule as FL-5B/C/D).

Among survivors, select lexicographically:

1. highest calibration target-recovery fraction;
2. highest fraction leaving the original wrong basin (competitor);
3. lowest median terminal target-angle error;
4. lowest amplitude;
5. highest theta.

Holdout is evaluated **once** with the selected `(amplitude, theta)`. No FL-5H
holdout value may influence selection. No prior FL-5* holdout may be used for tuning.

If no candidate survives clean-cue safety, report `none_survived_clean_cue_filter`
and skip scientific escape claims; protocol may still be valid.

## Residence / transition metrics (Complete-left)

For every trajectory, with initial decoded label known at step 0 (Complete left):

- `initial_decoded_label` (pattern index or `other`);
- `first_leave_initial_step` — first step whose decoded label differs from initial,
  or null if right-censored;
- `first_target_hit_step` — first step decoded as declared target, or null;
- `first_competitor_leave_step` — first step decoded ≠ competitor (wrong-basin only);
- `transition_count` — number of decoded-label changes over steps `1..256`;
- `terminal_decoded_label`;
- `right_censored_no_target_hit` boolean;
- run-length encoding of decoded segments (compact summary in JSON).

Aggregate mean first-passage time only over uncensored target hits; report
right-censor counts separately. Do not impute censored times.

## Metrics (calibration and holdout separately)

- target recovery fraction;
- competing-terminal fraction;
- left-wrong-basin fraction;
- clean recall fraction and clean degradation vs D0;
- median terminal target-angle error;
- mean first-passage time (uncensored only) and right-censor count;
- mean `first_leave_initial_step` among uncensored leaves;
- mean transition count;
- selected amplitude/theta;
- exact replay equality of the full campaign.

## Preregistered hypotheses

- **H5H-0 — protocol validity:** identifiability gate, partition minima, finite
  metrics, max norm-squared error ≤ `1e-10` on D0 probes used for construction
  checks, exact campaign replay, NoiseLab pin recorded.
- **H5H-1 — safe escape:** selected OU has strictly higher holdout target recovery
  than matched D0 **and** preserves D0 clean recall within tolerance. **May fail**
  while the protocol remains valid (including the case where E1 traps all arms).
- **H5H-2 — temporal structure:** selected OU holdout target recovery strictly
  greater than PERM-OU. Failure forbids a structured-noise/resonance claim.
- **H5H-3 — identifiable inputs:** retained panel has zero observable-label
  conflicts under the sign-input audit (also part of H5H-0).
- **H5H-4 — residence evidence:** every evaluated trajectory records Complete-left
  residence fields listed above; right-censor counts are reported.
- **H5H-5 — reproducibility:** complete campaign replay is exact.

A scientific hypothesis may be false without invalidating a correctly executed
experiment. No FL-6 promotion follows from FL-5H regardless of outcome.

## Deliverables

- `prereg/FL-5H.md` (this document);
- `crates/field-bench/src/bin/fl5h.rs`;
- `results/FL-5H-identifiable-e1-residence-stochastic-gate.json`;
- `docs/FL-5H-RESULT.md`;
- README update.

## Non-claims

No biology, hardware, LLM, survival-model, Kaplan–Meier, cognitive efficacy, or
FL-6 authorization claim.
