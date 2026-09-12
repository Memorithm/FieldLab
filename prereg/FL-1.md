# FL-1 — Associative Recall & Attractor Memory

Status: frozen exploratory protocol for the first associative-memory campaign.

## Question

Can the FieldLab continuous field dynamics recover corrupted stored patterns, and how does that recovery compare with deterministic asynchronous Hopfield dynamics and nearest-template retrieval when all methods use the same fixed pattern bank?

This protocol tests computational behavior only. It does not claim biological memory, physical magnetism, novelty, or superiority outside the declared finite experiment.

## Fixed pattern bank

Three bipolar patterns of width 16 are fixed in code:

1. all `+1`;
2. eight `+1` followed by eight `-1`;
3. alternating blocks of four `+1` and four `-1`.

These three patterns are pairwise orthogonal under the ordinary dot product.

## Coupling rule

The field model and Hopfield baseline share the same symmetric Hebbian weights:

```text
J_ij = (1 / N) * sum_mu p_mu[i] p_mu[j],  i != j
J_ii = 0
```

No method may receive a separately tuned coupling matrix.

## Field representation and dynamics

Each bipolar cue symbol `s_i in {-1,+1}` is encoded as the two-dimensional unit vector

```text
m_i = [s_i cos(theta), sin(theta)]
```

with fixed `theta = 0.15` radians. The positive second component is a declared deterministic symmetry-breaking representation choice: an exactly collinear bipolar state would otherwise have zero tangent velocity under the current projected dissipative dynamics.

The field system uses:

- Heun integration;
- `dt = 0.05`;
- mobility `eta = 1.0`;
- 128 integration steps;
- no external field;
- no stochastic forcing;
- no learned parameters.

Decoding is the sign of the first vector component.

## Baselines

### Deterministic asynchronous Hopfield

Uses the same Hebbian matrix, updates nodes in ascending index order, retains a symbol when its local field is exactly zero, and stops at convergence or 32 sweeps.

### Nearest-template retrieval

Uses Hamming distance to the three stored templates. Ties are resolved by fixed pattern-bank order. This baseline has direct template access and therefore acts as a strong retrieval reference rather than a matched dynamical model.

## Corruption campaign

For every stored pattern, enumerate every bit-flip mask with Hamming weight 0 through 4.

Total declared cases:

```text
3 * (C(16,0) + C(16,1) + C(16,2) + C(16,3) + C(16,4))
= 3 * 2517
= 7551
```

No random sampling is used.

## Metrics

For every corruption level and in aggregate, report:

- exact recovery count and rate for field dynamics;
- exact recovery count and rate for Hopfield;
- exact recovery count and rate for nearest-template retrieval.

Also report:

- deterministic replay equality on a fixed four-bit-corrupted sentinel;
- zero-corruption recovery integrity;
- whether field aggregate exact recall exceeds Hopfield;
- whether field aggregate exact recall exceeds nearest-template retrieval;
- whether field exact recall remains 100% through three flips.

## Scientific interpretation

The experiment is valid even if every comparative hypothesis is false.

A green CI means only that the protocol executed completely and reproducibly. It does **not** mean that FieldLab outperformed a baseline.

The primary comparative question is whether continuous field dynamics exceed the deterministic Hopfield baseline under the shared coupling matrix. Nearest-template retrieval is reported independently and may remain stronger because it directly retains all templates.

## Protocol validity gate

The executable exits non-zero only if the evidence is structurally invalid:

1. total case count is not exactly 7551;
2. any method fails an uncorrupted stored pattern;
3. deterministic sentinel replay differs.

Comparative hypothesis failure does not fail CI.

## Evidence boundary

FL-1 does not authorize:

- changing the pattern bank after viewing results;
- parameter sweeps to rescue a failed comparative result;
- claims about capacity scaling;
- claims about long-sequence memory;
- CCOS integration;
- stochastic or resonant dynamics;
- learned couplings.

Those belong to later FL series or a separately frozen successor protocol.
