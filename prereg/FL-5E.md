# FL-5E — ambiguity-gated strong perturbation

## Status

Preregistered before implementation and before any FL-5E outcome is inspected.

FL-5 through FL-5D established a persistent trade-off on the declared attractor fixtures: perturbation strong enough to produce any basin escape also degraded clean-cue recall, whereas clean-safe global perturbation did not recover targets. FL-5E tests one narrower mechanism: keep the strong perturbation amplitude available, but activate it only after an intrinsic, target-agnostic ambiguity gate says the settled state lies near a competing attractor boundary.

This is not an amplitude retune of FL-5D and does not authorize FL-6 by itself.

## Frozen research question

Can a two-stage, target-agnostic ambiguity gate preserve clean stored memories while selectively exposing ambiguous wrong-basin states to a strong OU perturbation, thereby breaking the global-noise safety/escape trade-off observed in FL-5B/C/D?

## Fixed provenance and fixture

- Noise process semantics remain anchored to NoiseLab commit `8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e`.
- Field model: the current FL-E0 / FL-1-style 8-node associative dynamics.
- Correlated memory bank: the FL-5D `H=2/2/4` three-pattern bank.
- Integrator: projected Heun, `dt=0.05`, mobility `1.0`.
- Stage A deterministic settling horizon: 128 steps.
- Stage B matched continuation horizon: 128 steps for every arm, including D0.
- Seeds: `[1,2,3,5,8,13,21,34]`.
- New case-id namespace only: `fl5e|...`; no FL-5/5B/5C/5D case identifier is reused.
- Partition rule remains the frozen SHA-256 rule: calibration iff first digest byte `< 0x80`, otherwise holdout.

## Intrinsic ambiguity gate

After Stage A, compute the angular distance from the settled field state to every stored template. Sort those distances. Define

`ambiguity_gap = second_best_angle - best_angle`.

This quantity does not use the declared target label. Small values mean that two stored attractors are similarly compatible with the settled state.

The Stage B perturbation gate is

`gate_on iff ambiguity_gap <= tau`.

Frozen threshold grid:

`tau ∈ {0.05, 0.10, 0.20, 0.35}` radians.

No target-vs-competitor margin may be used by the gate, by selection, or by runtime perturbation.

## Perturbation grid

FL-5D showed that global OU amplitude `0.5`, theta `0.5` can produce substantial calibration recovery but fails clean-cue safety. FL-5E therefore deliberately includes strong perturbation while testing whether gating isolates its cost.

Frozen Stage B grid:

- OU amplitudes: `{0.25, 0.50}`;
- OU theta: `{0.5, 2.0}`;
- gate threshold `tau`: `{0.05, 0.10, 0.20, 0.35}`.

For every `(amplitude, theta, tau)` candidate, evaluate:

1. `GATED-OU`: perturb only when the Stage-A ambiguity gate is on;
2. `PERM-GATED-OU`: same sampled OU sequence but with the frozen temporal permutation control;
3. `GLOBAL-OU`: same amplitude/theta, always on during Stage B;
4. matched `D0`: no perturbation during either stage.

All arms receive exactly 256 integration steps.

## Cases

Wrong-basin panel uses the six FL-5D shallow-regime cue geometries but new `fl5e|wb|...` identifiers, producing a fresh deterministic calibration/holdout assignment. Clean panel uses exact stored patterns under six new `fl5e|clean|...` identifiers.

Required construction invariants before outcome evaluation:

- at least two clean cases in calibration and at least two in holdout;
- at least two wrong-basin cases in calibration and at least two in holdout;
- clean and wrong-basin identifiers are disjoint;
- Stage-A ambiguity is finite for every case;
- at least one wrong-basin case gates on for some preregistered `tau` in each partition;
- no clean case may be forced to gate off by special-case code; clean gating must follow the same ambiguity rule.

A construction failure invalidates the experiment rather than permitting case or threshold changes.

## Calibration-only selection

First apply a hard clean-cue safety filter on calibration:

- gated clean recall must equal D0 clean recall;
- gated clean errors may not exceed D0 clean errors.

Among surviving GATED-OU candidates select lexicographically:

1. highest calibration target-recovery fraction;
2. highest fraction leaving the original wrong basin;
3. lowest median target-angle error;
4. lowest gate duty cycle on clean cases;
5. lowest amplitude;
6. highest theta;
7. lowest tau.

The holdout is then evaluated exactly once with the selected `(amplitude, theta, tau)`. No FL-5E holdout value may influence selection or subsequent retuning.

## Metrics

Report separately for calibration and holdout:

- target recovery fraction;
- competing-terminal fraction;
- unresolved-terminal fraction;
- fraction leaving the original wrong basin;
- median target-angle error;
- mean first-passage time when defined;
- clean recall fraction;
- clean degradation versus D0;
- gate-on fraction for wrong-basin cases;
- gate-on fraction for clean cases;
- selected amplitude/theta/tau;
- exact replay equality across repeated deterministic campaign execution.

## Preregistered hypotheses

- **H5E-0 — selective gate:** on holdout, the selected gate activates on at least one wrong-basin trial and on fewer clean trials than wrong-basin trials.
- **H5E-1 — safe escape:** selected GATED-OU has strictly higher holdout target recovery than matched D0 while preserving D0 clean recall exactly.
- **H5E-2 — gating benefit:** selected GATED-OU preserves clean recall better than the matched GLOBAL-OU arm without lower wrong-basin target recovery.
- **H5E-3 — temporal structure:** selected GATED-OU has holdout target recovery strictly greater than PERM-GATED-OU. Failure forbids a structured-noise/resonance claim.
- **H5E-4 — reproducibility:** complete campaign replay is exact and all reported metrics are finite.

A valid negative result remains evidence. In particular, failure of H5E-1 means the current FL-5 family has not shown a clean-safe basin-escape mechanism and FL-6 must not be justified from FL-5 efficacy.

## Interpretation boundary

FL-5E tests a target-agnostic state-dependent perturbation policy on one controlled associative-memory fixture. It does not establish stochastic resonance as a general cognitive principle, biological equivalence, physical magnetic noise benefit, hardware energy reduction, LLM quality improvement, or universal retrieval improvement.
