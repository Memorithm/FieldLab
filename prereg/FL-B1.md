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
- **HB1-2 local robustness:** perturbations inside a preregistered local neighborhood of a qualified E1 memory preserve its Boolean codeword at the declared rate.
- **HB1-3 collision diagnosis:** if two target labels are observationally indistinguishable under the frozen predicate bank, FL-B1 reports the collision rather than resolving it with target identity or post-hoc predicates.
- **HB1-4 information comparison:** the Boolean code retains non-zero discriminative information relative to a declared constant-code baseline, while using only the frozen predicate outputs. This is a scoped fixture result, not a compression or cognition claim.

HB1-1 through HB1-4 are scientific outcomes. Their failure must not make protocol-valid execution fail CI.

## Predicate-bank construction

The predicate bank must be frozen in source before outcome inspection.

For this first experiment:

1. Predicate addresses are selected from declared node/component coordinates of the qualified FL-5G E1 fixture.
2. Thresholds are fixed constants written in the executable/preregistration update before the campaign is run; they may not be computed from evaluated memory values, trajectories, calibration outcomes, or target labels.
3. Relations are explicit `AtLeast` or `LessThan` values.
4. Predicate order is part of the protocol and therefore part of the Boolean codeword.
5. Duplicate predicates are forbidden.
6. No target label, nearest-template identity, expected answer, FL-5G collision label, or decoded state may be an input to predicate evaluation.

A later experiment may preregister a calibration/holdout procedure for predicate synthesis. FL-B1 deliberately does not do so: it first qualifies the observation machinery without learned or tuned thresholds.

## Cases

Use the exact stable E1 memory templates qualified by FL-5G. Each case receives a fresh `flb1|...` identifier. Labels are used only after Boolean evaluation to score separation/collisions.

For robustness, generate deterministic local perturbations in the tangent space of each node, renormalize through the existing FieldLab state machinery, and evaluate a frozen radius grid. The implementation must declare the radius grid and deterministic direction construction in source before retaining results. No stochastic perturbation is required for FL-B1.

## Baselines

1. **CONST:** same all-false codeword for every state. This establishes zero discriminative information.
2. **FROZEN-BOOL:** the preregistered explicit predicate bank.
3. **CONTINUOUS-BITWISE (diagnostic only):** flatten each evaluated clean terminal `FieldState` in canonical node order and component order and use the exact sequence of `f64::to_bits()` values as the grouping key. No tolerance, decoded label, template identity, digest, or learned mapping participates in this key. This baseline is used only to report whether a Boolean collision joins clean states that are distinct under exact continuous-state representation. It is not an optimization oracle for FROZEN-BOOL and it does not reuse the discrete FL-5G `observable_panel` cue-sign grouping.

The implementation must report both the Boolean collision groups and the CONTINUOUS-BITWISE groups before applying labels. A Boolean collision is classified as **Boolean-induced on this finite panel** only when its members occupy more than one CONTINUOUS-BITWISE group. This phrase is descriptive for the declared fixture only; it is not a population or information-theoretic sufficiency claim.

## Metrics

Report at minimum:

- number of predicates and code width;
- codeword for every clean qualified memory;
- number and membership of clean Boolean-code collisions;
- number and membership of clean CONTINUOUS-BITWISE groups;
- pairwise Hamming distances between clean codewords;
- minimum clean Hamming distance;
- per-radius code retention rate under deterministic local perturbations;
- first radius at which each memory changes code, when observed;
- label purity of each Boolean-code group;
- empirical label entropy `H(Y)` and conditional entropy `H(Y|B)` on the declared finite fixture, with mutual information `I(Y;B)=H(Y)-H(Y|B)` reported as a descriptive finite-panel quantity only;
- exact replay equality for the complete machine-readable report.

Entropy uses base-2 logarithms and empirical frequencies over the declared finite case panel. No population/generalization interpretation is permitted.

## Validity gates

The run is protocol-valid only if:

- the repository revision and schema are recorded;
- the predicate bank is non-empty, finite, duplicate-free, and unchanged between repeats;
- all node/component addresses are valid for every evaluated state;
- no evaluated state contains a non-finite component;
- every perturbed node satisfies the existing normalization tolerance;
- CONTINUOUS-BITWISE keys are constructed only from the canonical full clean state, before labels are consulted;
- both complete runs produce exactly equal semantic reports;
- scientific hypothesis failures remain represented as data rather than converted into execution failures.

## Anti-leakage rules

After the first campaign result is inspected, do not change predicate addresses, thresholds, relations, order, perturbation radii, directions, scoring rules, case membership, or the CONTINUOUS-BITWISE grouping inside FL-B1. Any improved predicate construction is a new preregistered subseries (for example FL-B2) with a fresh namespace and, if tuning is introduced, a fresh calibration/holdout partition.

## Interpretation boundary

A positive FL-B1 result would show only that a fixed Boolean observation map can preserve useful distinctions for the declared stable FieldLab fixture. It would not establish that Boolean state replaces continuous field dynamics, attention, neural computation, physical magnetism, or general memory systems. It would not establish speed, energy, storage, cognition, LLM-quality, or hardware advantages.

A negative result is equally retained: it would quantify where component-threshold Boolean observations lose distinctions or robustness and would motivate a separately preregistered richer observation algebra rather than post-hoc repair.
