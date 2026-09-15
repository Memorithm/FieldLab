# Boolean × Field bridge

`field-boolean` is a narrow, versioned bridge between declared field representations and Boolean values. It does not replace the field model, alter the energy, change `H_eff = -∂E/∂m`, or assign physical meaning to a threshold or bit.

## Component predicates

The field observation contract is `fieldlab.boolean-predicate.v1`.

Each `ComponentThresholdPredicate` records all information needed to evaluate one bit:

- node index;
- component index;
- finite scalar threshold;
- explicit relation (`>=` or `<`).

`evaluate_predicates` preserves caller order and returns one Boolean value per predicate. Thresholds are never inferred from the evaluated state. Invalid node/component addresses and non-finite thresholds fail closed with typed errors.

## Predicate transitions

The two-observation transition contract is `fieldlab.boolean-transition.v1`.

`evaluate_predicate_transition` applies the same already-declared predicate to an explicit previous and current `FieldState`, then returns exactly one of `StableFalse`, `Rising`, `Falling`, or `StableTrue`. `evaluate_predicate_transitions` preserves the caller-declared predicate order for a batch. The previous state is evaluated before the current state, and any invalid field address fails closed through the existing typed predicate errors.

This transition surface is observation-only. It does not infer thresholds, rank transitions, select a regime, mutate the field, add hysteresis, or authorize a Boolean controller. A scientific experiment that uses a transition to switch dynamics must separately declare and preregister the switching rule and its baselines.

## Exact `{-1,+1}` spin encoding

The exact discrete encoding contract is `fieldlab.boolean-spin.v1` with the fixed convention:

- `-1.0 -> false`;
- `+1.0 -> true`.

`encode_spins` accepts only values whose IEEE-754 binary64 representation is exactly one of those two declared spins. It does not threshold or round continuous states: values such as `0.999`, signed zero, NaN and infinities fail closed with the offending index and raw bits. `decode_spins` is the inverse mapping and preserves bit order.

This contract is intended only for experiments whose state space has already been declared as exact Ising-like `{-1,+1}` values. Applying it to a continuous `FieldState` requires a separately declared observation/discretization rule; the exact-spin encoder itself cannot supply that missing semantics.

## Scientific boundary

These primitives are infrastructure for Boolean×Field experiments such as regime predicates, explicit switches, attractor-boundary encodings or Boolean controllers. They are **not** evidence that such a controller improves stability, cognition, speed, memory, energy or any other metric. Any scientific use must preregister predicate/encoding construction, baselines, decision rules and holdouts separately before measurements are interpreted.

Generic Boolean synthesis/equivalence remains owned by BooleanLab/SciRust as appropriate; FieldLab owns only the field-specific observation/representation boundary in this crate. Promotion of a more general primitive should occur only after repeated cross-project use justifies it.
