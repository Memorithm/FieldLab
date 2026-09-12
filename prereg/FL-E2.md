# FL-E2 preregistration — relational transport and composition

## Status

Frozen before inspection of FL-E2 result output.

## Question

Does the conservative FL-E1 operator model provide reproducible relational transport that cannot be represented by the historical scalar FL-E0 coupling, while retaining matched controls on relations that FL-E0 can represent?

This experiment tests representational capability, not general cognitive superiority.

## Models

### FL-E0 baseline

`E0 = -Σ_i h_i·m_i - Σ_{ {i,j} } J_ij m_i·m_j`

A scalar pair coupling can only prefer alignment or anti-alignment between two unit vectors.

### FL-E1 operator model

`E1 = -Σ_i h_i·m_i - Σ_{ {i,j} } m_iᵀ K_ij m_j`

No local anisotropy is used in FL-E2. For an orthogonal target transform `T`, the experiment installs `K = Tᵀ`, so a source node anchored near `s` induces a target field aligned with `T s`.

## Shared numerical protocol

- state dimension: 2;
- deterministic Heun integration;
- `dt = 0.01`;
- mobility `η = 1`;
- source angles: `0°, 22.5°, ..., 337.5°` (16 cases per relation);
- fixed target initialization for direct transport: `17°`;
- all similarities are cosine similarities between unit vectors;
- all runs must be finite and deterministic.

The source is anchored by an external field. This is deliberate: FL-E2 asks whether a relation can be transported from a controlled source state, not whether the source itself should drift under reciprocal coupling.

## Experiment A — single-edge relational transport

### Relations

Six orthogonal transforms are tested:

1. identity `I`;
2. inversion `-I`;
3. rotation `+90°`;
4. rotation `-90°`;
5. reflection across the x axis;
6. coordinate swap / reflection across `y=x`.

Identity and inversion are positive controls because FL-E0 can represent them through positive and negative scalar couplings.

The other four are non-scalar controls: no `J I` can equal these transforms.

### FL-E1 condition

- source anchor magnitude: `4`;
- operator magnitude: `1`;
- steps: `1024`;
- pair operator: `K = Tᵀ`.

### Matched FL-E0 baseline

For every relation, execute the same source angles, source anchor and initial target state for each scalar coupling in the preregistered grid:

`J ∈ {-2, -1, -0.5, 0.5, 1, 2}`.

The baseline score reported for a relation is the highest mean target cosine over that complete grid. This intentionally gives E0 the benefit of the best scalar sign and magnitude in the declared search set.

### Metrics

For every relation:

- E1 mean target cosine;
- E1 minimum target cosine;
- E1 mean source fidelity `cos(m_source_final, s)`;
- E1 minimum source fidelity;
- best E0 mean target cosine and selected `J`;
- E1 energy non-increase;
- exact deterministic replay.

### Preregistered interpretation criteria

Protocol validity requires:

- all runs finite;
- deterministic replay for every E1 case;
- E1 energy never increases beyond `1e-10` tolerance between initial and final measurements;
- E1 mean target cosine `>= 0.999` for every relation;
- E1 minimum target cosine `>= 0.99` for every relation;
- E1 minimum source fidelity `>= 0.995`.

Comparative hypotheses:

- **H-E2-A1 control preservation:** best E0 mean target cosine `>= 0.999` for identity and inversion;
- **H-E2-A2 operator transport:** E1 mean target cosine `>= 0.999` for each of the four non-scalar relations;
- **H-E2-A3 scalar insufficiency on this balanced fixture:** best E0 mean target cosine `<= 0.05` for each of the four non-scalar relations.

Comparative hypotheses are reported individually. Structural/protocol failures fail the executable; a failed comparative hypothesis remains a valid negative scientific result.

## Experiment B — two-edge composition

The second experiment asks whether operator relations compose through an unconstrained intermediate field node rather than only on one pair.

### Chain

`source -> intermediate -> target`

The source is externally anchored. Intermediate and target have no external field.

- source anchor magnitude: `8`;
- steps: `2048`;
- intermediate initial angle: `17°`;
- target initial angle: `83°`.

For transform pair `(T1, T2)`:

- first coupling uses `K01 = T1ᵀ`;
- second coupling uses `K12 = T2ᵀ`;
- expected intermediate state is `T1 s`;
- expected target state is `T2 T1 s`.

### Composition fixtures

The preregistered transform pairs are:

1. `rotate(+90°)` then `reflect-x`;
2. `reflect-x` then `rotate(+90°)`;
3. `rotate(+45°)` then `rotate(+90°)`;
4. `swap` then `rotate(-45°)`.

None of the four composed target transforms is `+I` or `-I`.

### FL-E0 composition baseline

For each composition fixture, search every pair in the Cartesian grid

`(J01, J12) ∈ {-2, -1, -0.5, 0.5, 1, 2}²`

with otherwise identical initialization and source anchoring. Report the pair giving the highest mean final target cosine over all 16 source angles.

### Metrics

For every composition fixture:

- E1 mean/minimum intermediate cosine;
- E1 mean/minimum target cosine;
- E1 minimum source fidelity;
- best E0 mean target cosine and selected `(J01, J12)`;
- deterministic replay;
- finite execution.

### Preregistered interpretation criteria

Protocol validity requires:

- E1 mean intermediate cosine `>= 0.999`;
- E1 mean target cosine `>= 0.999`;
- E1 minimum target cosine `>= 0.99`;
- E1 minimum source fidelity `>= 0.995`;
- exact deterministic replay;
- all reported values finite.

Comparative hypotheses:

- **H-E2-B1 composition:** all four E1 composition fixtures satisfy the target criteria;
- **H-E2-B2 scalar-chain insufficiency on this balanced fixture:** the best E0 mean target cosine is `<= 0.05` for every composition fixture.

## Non-claims

A positive FL-E2 result would establish only that conservative operator-valued pair interactions can encode and compose these controlled orthogonal relations under the declared dynamics. It would not establish natural-language reasoning, learned relational structure, scaling advantage, biological equivalence, physical magnetism or superiority over neural architectures.

## Stop condition

FL-E2 stops when the complete direct and composition grids have executed, machine-readable evidence has been retained, hypotheses have been reported without post-hoc threshold changes, and FL-0 through FL-3 plus FL-E1 remain regression-clean.
