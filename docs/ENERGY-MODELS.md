# FieldLab energy-model contract

FieldLab now separates the historical scalar energy kernel from experimental extensions instead of treating one formula as the complete field paradigm.

## FL-E0 — historical scalar model

FL-0 through FL-3 remain attached to the original scalar pair model:

```text
E0(M, x) = -Σ_i h_i(x)·m_i - Σ_{ {i,j}∈E } J_ij m_i·m_j
H_i_eff  = -∂E0/∂m_i
ṁ_i     = η (I - m_i m_iᵀ) H_i_eff
```

The edge sum is over unique undirected edges. It is not a sum over all ordered matrix indices, so a factor `1/2` is neither required nor used. `CouplingGraph` enforces this convention by rejecting repeated and reversed duplicate edges.

The Rust type remains `EnergyModel` for API compatibility; `FlE0EnergyModel` is an explicit alias for code that wants to declare the model version.

## FL-E1 — conservative operator extension

FL-E1 adds two independently ablatable conservative terms:

```text
E1(M, x) = -Σ_i h_i(x)·m_i
           -Σ_{ {i,j}∈E } m_iᵀ K_ij m_j
           -1/2 Σ_i m_iᵀ A_i m_i
```

`K_ij` is a finite square operator matching the node-state dimension. Its effective-field contribution is

```text
H_i += K_ij m_j
H_j += K_ijᵀ m_i
```

The transpose on the second endpoint is required by the scalar energy definition. An arbitrary `K_ij` may mix representation components even when the historical scalar model cannot.

`A_i` is a finite symmetric local matrix. Symmetry is validated because

```text
E_A = -1/2 m_iᵀ A_i m_i
```

has the declared effective field `A_i m_i` only when the antisymmetric part is absent. Non-symmetric local matrices fail closed rather than silently changing the derivative convention.

The Rust implementation is `OperatorEnergyModel`, with `OperatorCoupling` and `LocalAnisotropy` terms.

## Exact E0 embedding

Every FL-E0 model has a declared embedding into FL-E1:

```text
K_ij = J_ij I
A_i = 0
```

`OperatorEnergyModel::from_e0` performs that conversion. FL-E1 validation requires energy and effective fields to agree numerically with FL-E0 on fixed fixtures before any new operator behavior is accepted.

## Dynamics contract

`field-dynamics` consumes the `FieldModel` trait. Both E0 and E1 therefore use the same projected Euler/Heun reference integrators and the same unit-state constraint. This prevents a new energy term from being confounded with an integrator change.

The present dynamics remain purely dissipative projected gradient flow. The following mechanisms are deliberately not folded into E1:

- non-conservative/rotational dynamics;
- stochastic forcing and resonance;
- explicit hysteresis state;
- higher-order interactions;
- learned or time-varying couplings.

Those mechanisms require separate operators, preregistered experiments and ablations.

## Scientific interpretation

FL-E1 is an expressivity extension, not evidence of better cognition. A new operator is useful only after a later matched experiment demonstrates a task-level benefit over E0 and relevant non-field baselines.

See `prereg/FL-E1.md` and the `fle1` experiment executable for the frozen validation protocol.
