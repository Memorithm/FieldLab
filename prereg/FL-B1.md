# FL-B1 preregistration — Boolean observability of stable field memories

Status: **preregistered before implementation and outcome inspection**.

`FL-B1` is an orthogonal Boolean×Field subseries. It does **not** occupy, rename, authorize, or change roadmap `FL-6`, which remains reserved for sparse, low-rank, and multiscale field scaling.

## Question

Can a fixed, explicit Boolean observation map over continuous FieldLab states distinguish the already-qualified stable E1 memories without reading target labels or modifying the field dynamics?

FL-B1 is an observability experiment. It does **not** introduce a Boolean controller and it does not feed Boolean values back into `FieldState`, the energy, or `H_eff = -∂E/∂m`.

## Fixed foundations

FL-B1 starts from current `main` after FL-5G and the `fieldlab.boolean-predicate.v1` bridge. It reuses the stable E1 memory fixtures qualified by FL-5G. Historical FL-0 through FL-5G experiments and evidence are immutable inputs, not tuning sets.

The field-to-Boolean boundary is `field-boolean::ComponentThresholdPredicate` / `evaluate_predicates`. Every predicate must explicitly declare node, component, finite threshold, and relation. No threshold may be inferred from the state being evaluated.

## Primary hypotheses

- **HB1-0 validity:** every evaluated state is finite and valid under the existing FieldLab state contract; Boolean evaluation returns no typed address error; complete campaign replay is exact.
- **HB1-1 code separation:** distinct qualified E1 memory templates have distinct Boolean codewords under the frozen predicate bank.
- **HB1-2 local robustness:** all declared single-node perturbations at radii `0.01`, `0.05`, `0.10`, and `0.25` radians preserve the clean Boolean codeword. Radius `0.50` is a preregistered stress observation and is reported but does not determine HB1-2 support.
- **HB1-3 collision diagnosis:** if two target labels are observationally indistinguishable under the frozen predicate bank, FL-B1 reports the collision rather than resolving it with target identity or post-hoc predicates.
- **HB1-4 information comparison:** the Boolean code retains non-zero discriminative information relative to a declared constant-code baseline, while using only the frozen predicate outputs. This is a scoped fixture result, not a compression or cognition claim.

HB1-1 through HB1-4 are scientific outcomes. Their failure must not make protocol-valid execution fail CI.

## Frozen FL-B1.0 fixture

This section freezes the exact first executable protocol before any FL-B1 outcome is inspected.

### Clean memories

Use the three eight-node axial memories already used by FL-5G, in this exact order:

1. `P0 = [+1,+1,+1,+1,+1,+1,+1,+1]`
2. `P1 = [+1,+1,+1,+1,+1,+1,-1,-1]`
3. `P2 = [+1,+1,+1,+1,-1,-1,+1,+1]`

A clean node with sign `s ∈ {-1,+1}` is represented exactly as the two-dimensional unit state `[s, 0]`. Clean and perturbed nodes are constructed with `NodeState::try_unit(..., 1.0e-12)`; **`1.0e-12` is the frozen construction tolerance for FL-B1.0**. Clean case identifiers are `flb1|clean|p0`, `flb1|clean|p1`, and `flb1|clean|p2`. Labels `0`, `1`, and `2` are attached only after Boolean and CONTINUOUS-BITWISE observations have been constructed.

### Predicate bank

The bank contains exactly **16 predicates** and is ordered by node index, then by predicate kind. For every node `i = 0..7`, append:

1. `(node=i, component=0, threshold=+0.9, relation=AtLeast)`;
2. `(node=i, component=0, threshold=-0.9, relation=LessThan)`.

The constant `0.9` is a declared protocol margin. It is not estimated from evaluated FL-B1 states, trajectories, calibration outcomes, or target labels. No predicate addresses component `1` in FL-B1.0.

The expected semantic role is symmetric axial occupancy: a sufficiently positive axial component satisfies the first predicate, a sufficiently negative axial component satisfies the second, and the margin region satisfies neither. This semantic description is not an outcome claim.

Duplicate `(node, component, threshold bits, relation)` tuples are forbidden. Predicate order is part of the protocol and therefore part of every codeword.

### Deterministic perturbation panel

The frozen radius grid uses **zero-based stable indices** with this exact mapping:

- `r0 = 0.01` radians;
- `r1 = 0.05` radians;
- `r2 = 0.10` radians;
- `r3 = 0.25` radians;
- `r4 = 0.50` radians.

For every clean memory, every radius, every node `j = 0..7`, and each direction `d ∈ {-1,+1}`, create exactly one single-node angular perturbation. All nodes except `j` remain at angle `0`. Node `j` uses angle `q = d * radius`; a node with axial sign `s` is constructed as `[s*cos(q), s*sin(q)]` and is passed to `NodeState::try_unit(..., 1.0e-12)`. No target label, decoder result, Boolean result, or previous perturbation outcome participates in this construction.

This yields exactly `3 × 5 × 8 × 2 = 240` perturbed states. Perturbation identifiers are `flb1|perturb|p{pattern}|r{radius_index}|n{node}|d{minus|plus}`, where `radius_index` is exactly `0..4` according to the mapping above. The index, not decimal formatting, is the stable identifier component.

The implementation reports `max_norm_squared_error = max(abs(dot(node,node) - 1))` across every node of every evaluated clean or perturbed state. **The frozen FL-B1.0 numerical validity tolerance is `1.0e-10`: protocol validity requires `max_norm_squared_error <= 1.0e-10`.** This validity metric is separate from the `1.0e-12` construction tolerance. Any non-finite component is also a protocol failure.

## Predicate-bank construction rules

The predicate bank is frozen by the preceding section before outcome inspection.

1. Thresholds may not be computed from evaluated memory values, trajectories, calibration outcomes, or target labels.
2. Relations remain explicit `AtLeast` or `LessThan` values.
3. Predicate order is stable and caller-owned.
4. Duplicate predicates are forbidden.
5. No target label, nearest-template identity, expected answer, FL-5G collision label, or decoded state may be an input to predicate evaluation.
6. Any change to the 16 predicates is a new preregistered subseries, not an FL-B1.0 retune.

A later experiment may preregister a calibration/holdout procedure for predicate synthesis. FL-B1 deliberately does not do so: it first qualifies the observation machinery without learned or tuned thresholds.

## Cases

Use only the exact clean and perturbation cases frozen above. Labels are used only after Boolean and CONTINUOUS-BITWISE observation keys have been constructed to score separation, collisions, purity, and empirical finite-panel information.

No stochastic perturbation is part of FL-B1.0.

## Baselines

1. **CONST:** same all-false codeword for every clean state. This establishes zero discriminative information.
2. **FROZEN-BOOL:** the 16-predicate bank frozen above.
3. **CONTINUOUS-BITWISE (diagnostic only):** flatten each evaluated clean `FieldState` in canonical node order and component order and use the exact sequence of `f64::to_bits()` values as the grouping key. No tolerance, decoded label, template identity, digest, or learned mapping participates in this key. This baseline is used only to report whether a Boolean collision joins clean states that are distinct under exact continuous-state representation. It is not an optimization oracle for FROZEN-BOOL and it does not reuse the discrete FL-5G `observable_panel` cue-sign grouping.

The implementation must report both the Boolean collision groups and the CONTINUOUS-BITWISE groups before applying labels. A Boolean collision is classified as **Boolean-induced on this finite panel** only when its members occupy more than one CONTINUOUS-BITWISE group. This phrase is descriptive for the declared fixture only; it is not a population or information-theoretic sufficiency claim.

## Metrics

Report at minimum:

- predicate schema, predicate count, ordered predicate definitions, and code width;
- codeword for every clean qualified memory;
- number and membership of clean Boolean-code groups and collisions;
- number and membership of clean CONTINUOUS-BITWISE groups;
- pairwise Hamming distances between clean codewords;
- minimum clean Hamming distance;
- per-radius code retention count and rate over the 48 perturbations at each radius;
- first radius at which each memory changes code, when observed;
- label purity of each Boolean-code group;
- empirical label entropy `H(Y)` and conditional entropy `H(Y|B)` on the declared finite clean fixture, with mutual information `I(Y;B)=H(Y)-H(Y|B)` reported as a descriptive finite-panel quantity only;
- CONST-baseline `I(Y;B)` computed over the same clean fixture;
- maximum norm-squared error over evaluated clean and perturbed nodes;
- exact replay equality for the complete machine-readable semantic report.

Entropy uses base-2 logarithms and empirical frequencies over the declared finite clean case panel. No population/generalization interpretation is permitted.

## Validity gates

The run is protocol-valid only if:

- the repository revision and predicate schema are recorded;
- the clean fixture contains exactly the three frozen patterns and the perturbation panel exactly 240 cases;
- the predicate bank contains exactly the 16 frozen, finite, duplicate-free predicates in the declared order and is unchanged between repeats;
- all node/component addresses are valid for every evaluated state;
- no evaluated state contains a non-finite component;
- every node was constructed through `NodeState::try_unit(..., 1.0e-12)` and the campaign-wide `max_norm_squared_error` is at most `1.0e-10`;
- radius identifiers use exactly the frozen zero-based `r0..r4` mapping;
- CONTINUOUS-BITWISE keys are constructed only from the canonical full clean state, before labels are consulted;
- both complete runs produce exactly equal semantic reports;
- scientific hypothesis failures remain represented as data rather than converted into execution failures.

## Anti-leakage rules

After the first campaign result is inspected, do not change predicate addresses, thresholds, relations, order, perturbation radii or their stable indices, directions, construction/validity tolerances, scoring rules, case membership, HB1-2 criterion, or the CONTINUOUS-BITWISE grouping inside FL-B1. Any improved predicate construction is a new preregistered subseries (for example FL-B2) with a fresh namespace and, if tuning is introduced, a fresh calibration/holdout partition.

## Interpretation boundary

A positive FL-B1 result would show only that a fixed Boolean observation map can preserve useful distinctions for the declared stable FieldLab fixture. It would not establish that Boolean state replaces continuous field dynamics, attention, neural computation, physical magnetism, or general memory systems. It would not establish speed, energy, storage, cognition, LLM-quality, or hardware advantages.

A negative result is equally retained: it would quantify where component-threshold Boolean observations lose distinctions or robustness and would motivate a separately preregistered richer observation algebra rather than post-hoc repair.
