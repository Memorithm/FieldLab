# Boolean × Field bridge

`field-boolean` is a narrow, versioned bridge from an existing continuous `FieldState` to explicitly declared Boolean predicates. It does not replace the field model, alter the energy, change `H_eff = -∂E/∂m`, or assign physical meaning to a threshold.

The initial contract is `fieldlab.boolean-predicate.v1`.

Each `ComponentThresholdPredicate` records all information needed to evaluate one bit:

- node index;
- component index;
- finite scalar threshold;
- explicit relation (`>=` or `<`).

`evaluate_predicates` preserves caller order and returns one Boolean value per predicate. Thresholds are never inferred from the evaluated state. Invalid node/component addresses and non-finite thresholds fail closed with typed errors.

This is infrastructure for future Boolean×Field experiments such as regime predicates, explicit switches, attractor-boundary encodings or Boolean controllers. It is **not** evidence that such a controller improves stability, cognition, speed, memory, energy or any other metric. Any scientific use must preregister predicate construction, baselines, decision rules and holdouts separately before measurements are interpreted.

Generic Boolean synthesis/equivalence remains owned by BooleanLab/SciRust as appropriate; FieldLab owns only the field-specific observation boundary in this crate. Promotion of a more general primitive should occur only after repeated cross-project use justifies it.
