# FL-5G — axial stability and observable-input qualification result

## Scope

FL-5G is a bounded deterministic qualification of the existing anisotropy model and of the information content of the frozen nine-case FL-5F geometry panel. It is not a stochastic-efficacy experiment and does not authorize FL-6.

The retained campaign was executed from FieldLab source SHA `a64a6064554e34831e60a84510a8d341449053b3` using the already-preregistered `prereg/FL-5G.md` protocol. The canonical retained JSON is `results/FL-5G-axial-stability-observable-input.json`; its SHA-256 is `8368b7f4d804b48113c9884713a755684b7ce4c708671fb693bf40588ad006e1`.

The run completed with `protocol_valid = true`, exact deterministic replay, the declared 180 trajectories, derivative checks, numerical invariants, historical-reference reproduction and zero-lift E1(a=0) equivalence all passing.

## Qualified bounded observations

The preregistered bank-wide stiffness rule selected `a = 1.25`.

For the historical E0 model, the sufficient row lower bounds at the three axial patterns were `-0.5`, `-1.0` and `-1.0`. The minimum tested sign-direction Rayleigh values were approximately `0`, `-0.25` and `-0.25`; the latter two are explicit unstable tested directions for this declared directional set. Under E1 with `a = 1.25`, the sufficient row lower bounds became `0.75`, `0.25` and `0.25`, while the minimum tested sign-direction Rayleigh values became `1.25`, `1.0` and `1.0`. Maximum finite-difference second-derivative error remained below the preregistered `1e-5` tolerance.

Across the 90 E1 trajectories, no decoded exit occurred through the matched duration `T = 51.2` at any of the three frozen time steps (`0.05`, `0.025`, `0.0125`), and the final maximum axial angle was `0.0` in the retained output. This supports H5G-1 and H5G-2 for the declared fixture only; it is not a basin-volume, noise-benefit or general memory-stability result.

The historical E0 positive-tilt reference is reproduced. P0 has no exit through the measured horizon. P1 and P2 first exit at step 145 for `dt = 0.05`, giving the interval `[7.20, 7.25]`; the refined grids give `[7.225, 7.25]` at `dt = 0.025` and `[7.225, 7.2375]` at `dt = 0.0125`. These brackets satisfy the preregistered cross-grid tolerance and support H5G-3.

The observable-input audit retained 9 labeled records but only 7 distinct observable sign inputs. Two observable groups contain contradictory target labels. Under the equal-weight deterministic exact-label bound declared in the preregistration, at most 7 of the 9 records can be correct from those observables alone. This supports H5G-4 and confirms that changing case identifiers or partition hashes cannot manufacture label-identifiable examples from identical inputs.

H5G-5 is supported: replay and the declared numerical invariants passed. The maximum observed norm-squared error was approximately `4.44e-16`; the maximum per-step energy increases were on the order of `1e-14`, below the preregistered `1e-10` bound.

## Interpretation

All five FL-5G hypotheses are supported for this frozen fixture and protocol. The result is useful primarily in two ways:

1. the declared E1 anisotropy supplies a sufficient local axial-stability certificate and removes the specific deterministic exits observed under E0 over the measured horizon; and
2. the prior nine-case FL-5F geometry panel contains genuine label ambiguity at the observable-input level, so future noise gates must use distinguishable targets or explicit additional context rather than a fresh case namespace alone.

These results do not show that anisotropy improves corrupted-cue recovery, stochastic exploration, general cognitive performance, biological correspondence, hardware performance or scalable coupling. Strong anisotropy can also trap undesired states. No FL-6 promotion follows from FL-5G.

A subsequent perturbation experiment should therefore preregister a genuinely identifiable case set and retain direct residence/transition-time evidence rather than infer permanence from a single endpoint.
