# FL-5G — axial memory stability and observable-input qualification

Status: preregistered before implementation or new trajectory inspection.
Parent FieldLab revision: `9a403aa1bdd16d62f47434a838966ad7699a032d`.

## Motivation and boundary

FL-5F retained a first deterministic clean-cue exit at step 145 for the H=2/2/4 bank. A sign-decoded endpoint and zero tangent gradient do not certify a local energy minimum. This gate uses the existing FL-E1 anisotropy, without changing FL-E0, any historical experiment, or CCOS-Core.

A second limitation is informational: an identical observable cue, model and random stream cannot require different target labels. Renaming case IDs or changing their hashes does not create independent examples. This gate diagnoses contradictory labels and duplicate observables; it does not silently repair past partitions or claim a new independent holdout.

## Frozen energy and analytic test

Three patterns of width 8: `++++++++`, `++++++--`, `++++--++`. Hebbian J has exactly the existing zero diagonal and unique-edge convention, J_ij = sum_mu s_mu_i s_mu_j / 8.

With zero external field, use

E_a = -sum_{i<j} J_ij m_i dot m_j - a/2 sum_i m_ix^2.

Near an exact axial pattern s, parameterize m_i(q_i) = s_i (cos q_i, sin q_i). The tangent Hessian is

B_ii = sum_{j!=i} J_ij s_i s_j + a;
B_ij = -J_ij s_i s_j.

Every exact axial pattern is stationary. Define L_s(a) = min_i (B_ii - sum_{j!=i}|B_ij|). Strict positivity is a sufficient local-minimum certificate, not a necessary condition. Nonpositive L alone is inconclusive. A negative tested Rayleigh quotient proves an unstable direction. Exhaustively test the 256 normalized sign directions v_i in {-1,+1}/sqrt(8), but do not call their minimum the smallest eigenvalue.

Freeze a single bank-wide stiffness from geometry only:

a = max(0, -min_s L_s(0)) + 0.25.

No stiffness sweep or trajectory-based selection is permitted. Test both a=0 (historical E0) and this E1 a. Check stationarity and second directional derivatives directly against the implemented energy, with central differences at epsilon=1e-3 and absolute tolerance 1e-5; stationary tangent-field tolerance 1e-12. E1(a=0) must match E0 energy/fields within 1e-12. The exact mathematical certificate is subject to floating-point evaluation, not an interval-arithmetic or proof-assistant certificate.

## Frozen deterministic trajectories

At each of the three patterns, test two full-node transverse tilts +/-0.15 radians (same cue as historical encoding for positive tilt), plus eight single-node angular perturbations +0.01 radians. Ten initial states per pattern, thirty per model, no random forcing. Run both models at dt=0.05, 0.025, 0.0125 to the same simulated duration T=51.2 (1024, 2048, 4096 steps). Thus 180 trajectories. Keep first decoded exit as a bracket [(k-1)dt,k dt], null if right-censored at T; final maximum angular error to the exact axial target; maximum norm-squared error; maximum per-step energy increase; and full-state SHA-256 terminal fingerprints. Repeat the entire campaign; equality is required.

Do not interpret smaller dt as extra simulated time. For the historical positive-tilt E0 trajectories, report whether exit brackets agree within 0.10 time units across all dt; missing exits must be reported, not imputed. Require reproduction of the FL-5F positive-tilt reference at dt=.05 (P0 no exit through 1024, P1/P2 first exit 145).

## Observable-label diagnostic

Audit the nine FL-5F geometries (three clean cues, six wrong-basin cues). Group by the exact eight input signs, excluding target, competitor, case ID and clean/wrong kind from the observable key. Report all label-conflict groups, distinct observable count and the best possible deterministic exact-label accuracy bound: sum_g max_y count(g,y) / total records. This bound is for equal weighting of these records and identical model/context; it is not a cognitive performance claim. Add a generic pure grouping helper and regression tests so later benches can check actual inputs before partitioning.

## Hypotheses and execution gates

H5G-1: all three E1 axial targets have positive sufficient curvature bounds and pass derivative checks.
H5G-2: all declared E1 perturbed cues preserve their decoded target until T at every dt, with final max angular error <= 1e-3.
H5G-3: the historical E0 positive-tilt departure is robust to the frozen time-step refinement (exit brackets within 0.10 time units).
H5G-4: at least one observable-input group in the prior nine-case panel has contradictory labels; the diagnostic reports rather than hides it.
H5G-5: complete deterministic replay and numerical invariants pass.

Execution validity requires the frozen case set, finite metrics, max norm-squared error <= 1e-10, max energy increase <= 1e-10, derivative checks, a=0 equivalence, FL-5F reference reproduction and exact replay. A scientific hypothesis may be false without making a correctly executed experiment invalid. A local-minimum certificate alone does not prove basin volume, correct corrupted-cue recovery, useful noise, superiority over nearest-template retrieval, or scalable cognitive performance. Strong anisotropy may also trap undesired states. No FL-6 promotion follows.

## Reuse and next decision

Package the curvature and observable-group checks as small reusable Rust helpers, not a replacement runtime. They are candidates for NoiseLab experiment qualification and later SciRust upstreaming, but no dependency/provenance change in those repositories is part of this gate. A subsequent noise experiment must use genuinely distinguishable targets (explicit extra context or restricted unambiguous corruption), actual observable-group partitions and new evidence; a fresh namespace alone is insufficient.

Method reference: Manopt, Helpful tools / Hessian check, https://www.manopt.org/tools.html (accessed 2026-09-14). The model-specific Hessian and row-bound calculation above are derived from the declared energy.