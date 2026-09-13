# FL-5F — deterministic basin lifetime and horizon stability

## Status

Preregistered before implementation and before any FL-5F trajectory is executed or inspected.

FL-5E exposed a prerequisite problem for further stochastic claims: the H=2/2/4 correlated associative-memory fixture changes its decoded state under the deterministic D0 dynamics when the horizon is extended from 128 to 256 steps. FL-5F therefore removes noise completely and characterizes the deterministic residence times, transitions, angular geometry and energy trajectory of the benchmark itself.

FL-5F is a characterization gate. It performs no calibration, no holdout selection, no parameter fitting and no perturbation search.

## Frozen research question

For the current H=2/2/4 three-pattern field-memory fixture, how long do exact stored cues and the six shallow wrong-basin cues retain their declared decoded basin under uninterrupted deterministic projected-Heun dynamics, and what common time horizon remains semantically stable enough for later basin-escape comparisons?

## Fixed model and dynamics

- Field model: current FL-E0 / FL-1-style associative `PatternBank` energy model.
- Patterns:
  - `P0 = ++++++++`
  - `P1 = ++++++--`
  - `P2 = ++++--++`
- Node count: `8`.
- State dimension: `2`.
- Cue tilt: `0.15` rad.
- Integrator: projected Heun.
- `dt = 0.05`.
- mobility `= 1.0`.
- No stochastic forcing.
- No external field changes.
- One uninterrupted trajectory per case from step `0` through step `1024`.
- Complete campaign executed twice and required to replay exactly.

## Frozen cases

All case identifiers use a new `fl5f|...` namespace.

### Exact clean cues

- `fl5f|clean|t0` from exact `P0`, expected basin `P0`.
- `fl5f|clean|t1` from exact `P1`, expected basin `P1`.
- `fl5f|clean|t2` from exact `P2`, expected basin `P2`.

### Wrong-basin cues

Reuse only the six **cue geometries** that FL-5E declared before execution, under new identifiers:

- `fl5f|wb|t0|c1|k0`: target P0, declared competitor P1, cue `++++++--`.
- `fl5f|wb|t0|c2|k0`: target P0, declared competitor P2, cue `++++--++`.
- `fl5f|wb|t1|c0|k1`: target P1, competitor P0, cue `++++++-+`.
- `fl5f|wb|t1|c0|k2`: target P1, competitor P0, cue `+++++++-`.
- `fl5f|wb|t2|c0|k1`: target P2, competitor P0, cue `++++-+++`.
- `fl5f|wb|t2|c0|k2`: target P2, competitor P0, cue `+++++-++`.

No FL-5E outcome is used to add, remove or alter cases.

## Frozen observation horizons

Detailed state classification is computed at every integration step. The retained checkpoint table is frozen at:

`H = {0, 16, 32, 64, 96, 128, 160, 192, 224, 256, 320, 384, 512, 768, 1024}`.

The full per-step decoder is used for exact transition times; checkpoints are not used to approximate an event that occurred between them.

## State classification

At every step:

1. decode the bipolar state using the existing `PatternBank::decode_state` semantics;
2. classify it as `P0`, `P1`, `P2`, or `other` if the decoded vector equals none of the three stored patterns;
3. compute mean angular distance to every stored template;
4. record nearest-template index and top-two angular gap;
5. record model energy;
6. verify all numeric values are finite and every node remains unit-normalized within the existing numerical tolerance.

The decoded label and nearest-template label are reported separately and must not be silently substituted for one another.

## Residence and transition metrics

For each clean case report:

- first step whose decoded label differs from the expected target (`first_clean_exit`), or null through 1024;
- first return to the target after an exit, if any;
- total decoded-label transitions;
- run-length encoded decoded segments;
- decoded and nearest-template labels at every frozen checkpoint;
- target angular error at each checkpoint;
- final label and final target angle.

For each wrong-basin case report:

- first step decoded as the declared competitor;
- first step after competitor entry that leaves that competitor;
- first step decoded as the declared target;
- first return to competitor after leaving, if any;
- total decoded-label transitions;
- run-length encoded decoded segments;
- decoded and nearest-template labels at every frozen checkpoint;
- target and competitor angular errors at each checkpoint;
- final label and final target/competitor angles.

## Aggregate stability quantities

Define:

- `clean_common_stable_through`: the greatest integer step `h <= 1024` such that every exact clean case is decoded as its expected target at every step `0..=h`.
- `wrong_common_competitor_through`: the greatest integer step `h <= 1024` such that every wrong-basin case, once it has first entered its declared competitor, remains in that competitor through `h`. If a case has not entered its competitor by step 128, the protocol is invalid.
- `benchmark_common_stable_through = min(clean_common_stable_through, wrong_common_competitor_through)`.

Also report the largest frozen checkpoint in `H` not exceeding `benchmark_common_stable_through`. This is a descriptive candidate horizon for later preregistration, not an automatic retuning of prior experiments.

## Reference gates and preregistered hypotheses

- **H5F-0 — 128-step reference gate:** all exact clean cases decode to their target and all six wrong-basin cases decode to their declared competitor at step 128. Failure means the FL-5E fixture cannot be reproduced and invalidates FL-5F.
- **H5F-1 — long-horizon clean permanence:** every exact clean cue remains decoded as its target through step 1024.
- **H5F-2 — long-horizon wrong-basin permanence:** after entering its declared competitor, every wrong-basin trajectory remains in that competitor through step 1024.
- **H5F-3 — 128→256 classification invariance:** all nine cases have the same decoded label at steps 128 and 256.
- **H5F-4 — deterministic replay:** the complete 0..1024 campaign, including every checkpoint and transition segment, is exactly replay-identical.

H5F-1/2/3 are deliberately strict. Their failure is not a protocol failure; it quantifies metastability/horizon dependence.

## Protocol validity

FL-5F is valid iff:

- all nine frozen case IDs are present exactly once;
- all case IDs begin with `fl5f|`;
- H5F-0 passes;
- all required checkpoints are present;
- all numeric metrics are finite;
- unit-norm validation passes throughout;
- energy evaluation succeeds throughout;
- complete replay is exact.

Comparative hypotheses H5F-1/2/3 may fail while `protocol_valid` remains true.

## Exit criterion for the gate

FL-5F exits with one of two conclusions:

1. a non-zero `benchmark_common_stable_through` and corresponding frozen checkpoint are identified for a later, separately preregistered stochastic comparison; or
2. no practically useful common stable horizon exists, in which case the current H=2/2/4 bank is unsuitable as a basin-escape benchmark and the fixture/model must be redesigned before further FL-5 efficacy claims.

FL-5F does not itself authorize FL-6, stochastic-resonance claims, biological claims, hardware claims or LLM-task claims.
