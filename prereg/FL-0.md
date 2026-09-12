# FL-0 — Mathematical Sanity & Deterministic Field Kernel

Status: **preregistered bootstrap protocol**

## Question

Do the minimal FieldLab state, signed-coupling, energy, effective-field and projected integration primitives satisfy their declared mathematical and reproducibility invariants on tiny controlled systems?

FL-0 is a correctness gate. It does not test cognition, learning, scaling, hardware or novelty.

## Frozen implementation scope

FL-0 permits only:

- finite-dimensional real node vectors;
- unit-norm state construction;
- a finite graph with signed scalar pair couplings;
- optional external vector fields;
- energy `E = -Σ_i h_i·m_i - Σ_(i,j) J_ij m_i·m_j`;
- effective-field evaluation;
- projected dissipative dynamics;
- explicit Euler and Heun reference integration;
- deterministic execution without stochastic forcing.

No learned parameters, GPU kernels, CCOS mutation path, noise-assisted mechanism or physical-magnetism claim is in scope.

## Reference cases

### FL-0-A — attractive pair

Initial state:

```text
m0 = [1, 0]
m1 = [0, 1]
J01 = +1
h0 = h1 = [0, 0]
```

Reference integration:

```text
integrator = Heun
dt = 0.01
mobility = 1.0
steps = 100
```

Acceptance:

1. final energy is strictly lower than initial energy;
2. final dot product `m0·m1 > 0.9`;
3. maximum unit-norm error is `<= 1e-12` after normalization;
4. identical input/configuration produces an exactly equal replayed `FieldState` under the same declared platform/toolchain contract.

### FL-0-B — repulsive pair

Same configuration except `J01 = -1`.

Acceptance:

1. final energy is strictly lower than initial energy;
2. final dot product `m0·m1 < -0.9`;
3. maximum unit-norm error is `<= 1e-12`;
4. exact replay equality holds under the same declared platform/toolchain contract.

## Validation failures

The implementation must fail closed for at least the following malformed inputs:

- empty state/vector;
- zero vector where normalization is requested;
- non-finite values;
- dimension mismatch;
- node-count mismatch;
- out-of-bounds coupling endpoints;
- self-coupling in the bootstrap graph representation;
- invalid integration time step or mobility.

## Numerical policy

FL-0 uses IEEE-754 `f64`. Unit state is restored by deterministic normalization after each explicit integration update. This protocol does **not** claim cross-architecture bitwise identity. Replay equality is scoped to the same implementation, toolchain and platform contract.

A future series may compare integration methods and error control. FL-0 only verifies that the declared reference implementation behaves coherently on tiny cases.

## Decision rule

FL-0 passes only if all unit tests pass and both executable reference cases pass all acceptance criteria. Any failed criterion is reported as a failed FL-0 result; it must not be weakened after observing the result without creating a new protocol revision.

## Evidence artifact

`cargo run -p field-bench` prints a machine-readable JSON result containing the integration configuration, deterministic provenance fingerprint, per-case measurements and overall pass/fail state. CI retains this output as the FL-0 artifact.
