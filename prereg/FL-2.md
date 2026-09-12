# FL-2 — Signed Competition & Frustration

Status: frozen exploratory protocol.

## Question

Do signed repulsive couplings provide useful separation between competing hypotheses under controlled contradictory evidence, and can the same dynamics settle a genuinely frustrated three-way system into a stable compromise rather than oscillating or collapsing?

This protocol tests a small deterministic field model. It does not claim a general reasoning mechanism, biological equivalence, or a universal benefit of inhibition/repulsion.

## Part A — Two-hypothesis evidence competition

Two unit-vector hypothesis states, `A` and `B`, begin from the same neutral orientation `[0, 1]`.

For each case, evidence is represented as external fields

```text
h_A = [c + d, 0]
h_B = [c - d, 0]
```

where:

- `c` is common-mode support shared by both hypotheses;
- `d` is differential evidence, with `d > 0` favouring A and `d < 0` favouring B.

Fixed grid:

```text
c ∈ {0.0, 0.25, 0.5, 1.0}
d ∈ {-1.0, -0.9, ..., -0.1, 0.0, 0.1, ..., 0.9, 1.0}
```

Total cases per coupling condition: `4 * 21 = 84`.

### Coupling conditions

All conditions use exactly the same state size, evidence fields, integrator and number of steps.

1. **Repulsive signed field:** `J_AB = -0.5`.
2. **Uncoupled control:** `J_AB = 0.0`.
3. **Attractive/unsigned control:** `J_AB = +0.5`.

The attractive condition is intentionally not described as a cognitively sensible competitor; it is a control for the effect of reversing the sign of the interaction.

### Dynamics

- Heun integration;
- `dt = 0.02`;
- mobility `eta = 1.0`;
- 256 steps;
- no stochastic forcing;
- no learned parameters.

### Metrics

For each case define

```text
margin = (A_x - B_x) / 2
```

For `d != 0`, winner correctness requires `sign(margin) = sign(d)`.

Report per coupling condition:

- winner correctness over all non-tie cases;
- mean useful separation `mean(sign(d) * margin)` over non-tie cases;
- minimum useful separation over non-tie cases;
- maximum absolute tie polarization `|margin|` over `d = 0` cases;
- whether useful separation is non-decreasing with `|d|` independently on the positive and negative branches for every fixed `c`;
- final energy relative to initial energy for every case.

### Comparative hypotheses

- **H2-A1:** repulsive coupling preserves 100% winner correctness on the declared grid.
- **H2-A2:** repulsive coupling has larger mean useful separation than the uncoupled control.
- **H2-A3:** repulsive coupling has larger mean useful separation than the attractive control.
- **H2-A4:** repulsive coupling does not create false polarization on exact ties beyond `1e-12`.

Failure of any comparative hypothesis remains a valid result.

## Part B — Three-way frustrated triangle

Three unit-vector states are connected by equal pairwise repulsive couplings:

```text
J_01 = J_12 = J_20 = -1.0
```

with zero external field.

Initial vectors are fixed and deliberately asymmetric:

```text
m_0 = angle(0°)
m_1 = angle(70°)
m_2 = angle(210°)
```

The analytic minimum of the declared planar equal-coupling energy is the 120-degree compromise, for which every pairwise dot product is `-1/2` and total interaction energy is `-3/2`.

A matched attractive control uses `J = +1.0` on the same triangle and same initial vectors; its expected minimum is alignment with pairwise dot products `+1`.

### Dynamics

- Heun integration;
- `dt = 0.02`;
- mobility `eta = 1.0`;
- 512 steps;
- no stochastic forcing;
- no external field.

### Metrics and analytic gates

For the repulsive triangle report:

- final energy;
- all three pairwise dot products;
- maximum deviation from `-0.5`;
- energy error relative to `-1.5`;
- deterministic replay equality.

For the attractive control report analogous pairwise-dot and energy diagnostics against `+1` alignment / energy `-3.0`.

Reference tolerances:

```text
repulsive pairwise dot max error <= 1e-6
repulsive energy error <= 1e-6
attractive pairwise dot max error <= 1e-6
attractive energy error <= 1e-6
```

These tolerances are numerical validation targets for this fixed integration policy, not universal physical tolerances.

## Protocol validity gate

The FL-2 executable exits non-zero only if the evidence is structurally invalid:

1. the declared case count is not executed;
2. deterministic replay fails;
3. a non-finite state or metric is produced;
4. the analytic three-node reference cases fail their declared numerical tolerances.

Comparative Part-A hypotheses do not control CI success.

## Evidence boundary

FL-2 does not authorize post-result parameter sweeps, stochastic rescue, hysteresis, CCOS integration, learned couplings, scaling claims, or physical magnetic claims.

The next series remains FL-3 hysteresis/context switching unless FL-2 exposes a correctness defect that must first be repaired.
