# FL-E1 preregistration — operator-valued conservative field model

Status: frozen before CI execution of the FL-E1 campaign.

## Purpose

FL-E1 tests a strictly bounded extension of the historical FL-E0 energy model. It does not replace or reinterpret FL-0 through FL-3 results.

FL-E0 is defined over unique undirected edges:

```text
E0(M, x) = -Σ_i h_i(x)·m_i - Σ_{ {i,j}∈E } J_ij m_i·m_j
```

FL-E1 adds operator-valued pair interactions and symmetric local quadratic anisotropy:

```text
E1(M, x) = -Σ_i h_i(x)·m_i
           -Σ_{ {i,j}∈E } m_iᵀ K_ij m_j
           -1/2 Σ_i m_iᵀ A_i m_i
```

For each pair term, the effective fields are:

```text
H_i += K_ij m_j
H_j += K_ijᵀ m_i
```

For each symmetric local term:

```text
H_i += A_i m_i
```

Projected dissipative dynamics remain unchanged:

```text
ṁ_i = η (I - m_i m_iᵀ) H_i_eff
```

Rotational/non-conservative dynamics, stochastic forcing and higher-order interactions are explicitly out of scope for this experiment.

## Invariant correction

`CouplingGraph` declares FL-E0 pairs to be undirected and unique. The constructor must therefore reject both repeated `(i,j)` and reversed `(j,i)` duplicates. This closes an ambiguity that could otherwise double-count one physical edge.

## Fixed protocol

Executable: `cargo run -p field-bench --bin fle1`

Integration parameters for dynamic witnesses:

- integrator: projected Heun;
- `dt = 0.01`;
- mobility `η = 1.0`;
- `256` steps;
- numerical equivalence tolerance `1e-10`.

### Gate E1-G0 — duplicate-edge rejection

A graph containing `(0,1)` and `(1,0)` must fail with `DuplicateCoupling { source: 0, target: 1 }`.

### Gate E1-G1 — exact FL-E0 embedding

For three fixed two-dimensional states and five fixed scalar couplings `[-1.25, -0.5, 0, 0.75, 1.5]`, construct FL-E1 by the declared embedding

```text
K_ij = J_ij I
A_i = 0
```

with identical external fields.

Pass criteria:

- maximum absolute energy error `<= 1e-10`;
- maximum absolute effective-field component error `<= 1e-10`.

Failure means FL-E1 is not a valid conservative extension of FL-E0 and the experiment stops.

### Gate E1-G2 — cross-axis expressivity witness

Use two initially aligned states `(1,0)` with zero external field.

FL-E0 control: `J = 1`. Because the scalar coupling field is radial at this initial state, projected motion should remain zero.

FL-E1 condition:

```text
K = [[0, -1],
     [1,  0]]
```

This operator maps the partner's x-direction into a tangent y-direction. Both conditions receive the same dimension, node count, integrator, mobility, step size and step count.

Pass criteria:

- FL-E0 total state displacement `<= 1e-10`;
- FL-E1 total state displacement `> 0.5`;
- FL-E1 energy decrease `> 0.9`;
- deterministic replay is bitwise identical.

This is an expressivity witness only. It is not a cognitive-performance claim.

### Gate E1-G3 — local anisotropy witness

Use one state initialized at 45 degrees and

```text
A = [[2, 0],
     [0, 0]]
```

with no external field and no pair couplings.

Pass criteria:

- absolute x-axis alignment increases from its initial value;
- final absolute x-axis alignment `> 0.99`;
- energy decrease `> 0.4`.

## Interpretation rules

Passing FL-E1 establishes only that:

1. the historical scalar model is embedded consistently;
2. operator-valued conservative couplings can represent cross-component transformations unavailable to scalar `J_ij` couplings under the same state representation;
3. symmetric local quadratic terms can impose explicit directional preferences;
4. duplicate undirected FL-E0 edges fail closed.

It does not establish improved memory, reasoning, learning, scaling, biological plausibility or physical magnetic correspondence. Those require separate FL-series experiments and matched baselines.
