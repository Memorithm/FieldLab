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

## Ordered transition traces

The bounded sequence contract is `fieldlab.boolean-transition-trace.v1`.

`evaluate_predicate_transition_trace` applies one already-declared predicate to an ordered sequence of explicit `FieldState` observations and records each adjacent transition. The implementation bounds the number of transitions before allocation and before evaluating the predicate. Zero- and one-observation sequences therefore produce an empty trace instead of inventing a transition.

The trace preserves temporal order. It is still descriptive observation infrastructure: it does not define a sampling interval, infer a regime boundary, choose a controller action, or establish physical time from an observation index.

## Exact transition summaries

The aggregate contract is `fieldlab.boolean-transition-summary.v1`.

`summarize_predicate_transition_trace` counts the four transition classes exactly and derives the total number of changed and stable transitions. It summarizes only the supplied trace; it does not reconstruct missing observations or treat a count as a rate without an independently declared observation cadence.

A high or low transition count is not, by itself, evidence of stability, bifurcation, cognition, hysteresis, or useful control.

## Exact dwell runs

The run reconstruction contract is `fieldlab.boolean-dwell-run.v1`.

`predicate_dwell_runs` reconstructs maximal contiguous Boolean runs from an explicit initial predicate value plus an ordered transition trace. It validates transition continuity and records each run's Boolean value, starting observation index and non-zero observation count. The reconstruction is exact for the supplied discrete observation sequence; an observation count is not automatically a physical residence time.

## Exact dwell summaries

The summary contract is `fieldlab.boolean-dwell-summary.v1`.

`summarize_predicate_dwell_runs` revalidates public `PredicateDwellRun` inputs before aggregation. A valid non-empty trajectory must start at observation zero, contain non-zero run lengths, be contiguous, and alternate Boolean values so adjacent runs are already maximal. Checked arithmetic is used for accumulated observation counts.

The summary reports exact observation count, transition count, False/True observation totals, False/True run counts and the longest observed dwell for each Boolean value. Longest dwell remains a descriptive observation count. It is not a preregistered stability threshold, attractor claim, debounce rule, hysteresis parameter, controller decision or bifurcation result.

## Explicit dwell-window censoring

The observation-boundary contract is `fieldlab.boolean-dwell-censoring.v1`.

`annotate_predicate_dwell_censoring` first revalidates the supplied maximal dwell
runs and then applies caller-declared left/right boundary knowledge. A boundary
may be declared `Complete` (for example a known initialization or terminal event)
or `ObservationCut` when the retained window can slice through a longer dwell.
Only the first run can be left-censored, only the final run can be right-censored,
and a single run may carry both annotations. Interior runs remain complete with
respect to the supplied observation sequence because observed transitions bound
them on both sides.

Censoring is never inferred from run length, transition count, field energy or
predicate value. The annotation does not estimate the unobserved duration and
must not be treated as a physical residence time. This distinction is required
before using bounded dwell observations in attractor-stability, basin-lifetime or
bifurcation experiments.

The censor-aware descriptive distribution contract is `fieldlab.boolean-censor-aware-dwell-distribution.v1`. `predicate_censor_aware_dwell_distribution` retains exact left/right/both-censored run counts while building run-length histograms **only** from runs known complete at both observation-window edges. Censored durations are not imputed and are not mixed into the complete-run histogram. This is still descriptive infrastructure: it is not a Kaplan–Meier estimator, a hazard model, an attractor-residence estimate or evidence that longer observed dwells imply stability.

The partial-identification contract `fieldlab.boolean-censor-aware-dwell-tail-bounds.v1` answers a narrower caller-declared question without fitting a survival model: for a positive observation-count threshold, how many complete dwell lengths are **definitely** at least that long, and how many **could** be at least that long given only the retained boundary-censoring facts? A censored run already observed beyond the threshold is definite; a censored run cut below it widens only the upper count. Complete short runs remain definitely below the threshold. These are exact count bounds over the supplied runs, not probabilities, Kaplan–Meier estimates, hazards, stationarity assumptions, attractor-lifetime estimates, or stability verdicts.

`fieldlab.boolean-censor-aware-dwell-tail-fraction-bounds.v1` preserves those bounds as exact count fractions over the observed run count for each Boolean value; an absent value has no empirical fraction rather than an invented zero. `fieldlab.boolean-censor-aware-dwell-tail-ordering.v1` then permits only a strict interval-order statement: `true` is definitely higher only when its exact lower bound is above the `false` upper bound, and conversely for `false`. Touching/overlapping intervals remain indeterminate and an absent Boolean value remains undefined. The comparison uses exact integer cross-products; it is not a probability, significance test, survival estimate, or field-stability/bifurcation verdict.

`fieldlab.boolean-censor-aware-dwell-tail-contrast-bounds.v1` retains a quantitative effect-size interval without introducing floating-point rounding: it reports exact bounds for `true_tail_fraction - false_tail_fraction` by subtracting the canonical partial-identification intervals. Signed rationals are reduced exactly, zero is canonicalized to `0/1`, and the contrast remains undefined when either Boolean value has no observed runs. A contrast interval crossing zero is deliberately left sign-indeterminate. This is descriptive empirical geometry over the declared observation window, not a significance test, survival-model estimate, attractor-lifetime estimate, or stability/bifurcation verdict.

`fieldlab.boolean-censor-aware-dwell-tail-profile.v1` evaluates those same exact bounds and strict orderings over a caller-declared, strictly increasing threshold grid. The grid must be fixed by the experiment protocol before result interpretation; the helper never searches for a favorable threshold and rejects empty, duplicate or reordered grids. A stable ordering across several supplied thresholds is descriptive robustness evidence only, not independent replicated evidence, a p-value, a survival model, or an attractor/stability/bifurcation verdict.

`fieldlab.boolean-censor-aware-dwell-tail-contrast-profile.v1` applies the exact true-minus-false contrast interval over a caller-declared, strictly increasing threshold grid. Each point is exactly the corresponding single-threshold contrast contract; absent Boolean values remain undefined and censoring-induced sign ambiguity remains explicit. The helper does not select, interpolate, smooth or multiplicity-adjust thresholds, so a favorable interval at one point cannot be promoted to independent evidence or a stability claim.

## Exact `{-1,+1}` spin encoding

The exact discrete encoding contract is `fieldlab.boolean-spin.v1` with the fixed convention:

- `-1.0 -> false`;
- `+1.0 -> true`.

`encode_spins` accepts only values whose IEEE-754 binary64 representation is exactly one of those two declared spins. It does not threshold or round continuous states: values such as `0.999`, signed zero, NaN and infinities fail closed with the offending index and raw bits. `decode_spins` is the inverse mapping and preserves bit order.

This contract is intended only for experiments whose state space has already been declared as exact Ising-like `{-1,+1}` values. Applying it to a continuous `FieldState` requires a separately declared observation/discretization rule; the exact-spin encoder itself cannot supply that missing semantics.

## Scientific boundary

These primitives are infrastructure for Boolean×Field experiments such as regime predicates, explicit switches, attractor-boundary encodings or Boolean controllers. They are **not** evidence that such a controller improves stability, cognition, speed, memory, energy or any other metric. Any scientific use must preregister predicate/encoding construction, observation cadence, baselines, decision rules and holdouts separately before measurements are interpreted.

Generic Boolean synthesis/equivalence remains owned by BooleanLab/SciRust as appropriate; FieldLab owns only the field-specific observation/representation boundary in this crate. Promotion of a more general primitive should occur only after repeated cross-project use justifies it.
