# FL-5F — deterministic basin-lifetime and horizon-stability result

## Scope

FL-5F is a deterministic horizon audit. It does not test stochastic efficacy, noise-assisted escape, hardware performance, biological correspondence, LLM quality, or FL-6 scalability.

The qualified campaign was executed in GitHub Actions from PR head `53bae819cb42a633fd9447288d68bfdd0ca4eb51` and merged to `main` as `38f26de6b453332daf2d759548d08f5ecf483ca6`. The CI artifact `fieldlab-fl5f-eae00c9cf0b3d9f1d1ec53fb7ba212709614b2e6` has SHA-256 digest `0371bf17f0e3b92f3146261eb50eccbe706c774c61a6a5ad8c662c1f8cb4e8c8`.

Frozen constants for this gate were 8 nodes, maximum horizon 1024 steps, `dt = 0.05`, mobility `1.0`, cue tilt `0.15` radians, and checkpoints `[0,16,32,64,96,128,160,192,224,256,320,384,512,768,1024]`.

## Qualified aggregate result

The exact campaign output reported:

- `clean_common_stable_through = 144`
- `wrong_common_competitor_through = 144`
- `benchmark_common_stable_through = 144`
- `largest_stable_checkpoint = 128`
- `H5F-0 reference-128 gate = supported`
- `H5F-1 clean permanence = not supported`
- `H5F-2 wrong-basin permanence = not supported`
- `H5F-3 128→256 invariance = not supported`
- `H5F-4 deterministic replay = supported`
- all numeric values finite
- all unit-norm checks valid
- all checkpoints present
- frozen case set exact
- protocol valid

Two clean non-P0 fixtures remained in their declared basin through step 144 and transitioned at step 145. The two exact wrong-basin fixtures targeting P0 from P1/P2 likewise left their declared competitor and entered P0 at step 145. The remaining wrong-basin fixtures were already classified as P0 by step 61 and remained there through the measured horizon.

## Interpretation

The previously used 128-step endpoint is a valid reference checkpoint for this frozen fixture, but it is not a permanence guarantee. The common deterministic stability interval extends through step 144, while behavior at 160 and later checkpoints differs from the 128-step labels. Therefore a stochastic comparison that treats the 128-step terminal label as an indefinitely stable attractor would confound perturbation effects with deterministic horizon drift.

This is a negative result for permanence and 128→256 invariance, and it is retained as such. It does not establish that longer horizons are generally preferable, nor that the observed transition at step 145 generalizes beyond this exact bank, integrator, constants, and frozen case set.

## Consequence for FL-5

Future FL-5 stochastic gates must declare their horizon semantics explicitly. If a gate is intended to test escape from a stable basin rather than transient residence, it must either:

1. use a horizon/case set whose deterministic residence requirement is preregistered and verified before stochastic comparison, or
2. treat residence time / transition time itself as the response variable.

No FL-6 efficacy authorization follows from FL-5F.