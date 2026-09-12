# FL-4B — Hysteretic bounded working-set selection

Status: **frozen before result inspection**.

## Question

Can the already-validated FL-3 hysteresis operator reduce bounded-context thrash when applied on top of the lossless FL-4A CCOS field mapping, without changing the token budget or silently changing CCOS-Core semantics?

FL-4B is a controlled FieldLab experiment. No code is inserted into `Memorithm/CCOS-Core`.

## Baseline

The baseline is the pinned CCOS score/working-set semantics established by FL-4A at:

`Memorithm/CCOS-Core@a3c4d7e03744430c74dc337463ff3e944b4933ad`

For FL-4B the score profile deliberately isolates one scalar source so the temporal operator can be tested without confounding score terms:

- `w_base = 1.0`;
- every other node-score weight is `0`;
- all nodes are `Stable`;
- equal content cost: 64 characters = 16 estimated tokens;
- eight candidate files;
- budget = 48 tokens, therefore exactly three equal-cost items can fit.

At each observation the baseline calls the normal CCOS-like working-set assembly: descending score, URI tie-break, identical budget.

## Hysteretic field variant

Each candidate node owns one `field-hysteresis::Relay`.

For raw scalar activation `s_i(t)`, relay input is

`u_i(t) = s_i(t) - 0.5`.

The relay uses symmetric thresholds `[-theta, +theta]` and retains its previous state inside the deadband. Initial state is determined only from the first observed score (`Positive` when `s_i(0) >= 0.5`, otherwise `Negative`). Ground-truth relevance is never used to initialize or update the operator.

The adjusted FieldLab score is

`s'_i(t) = clamp(s_i(t) + g * r_i(t), 0, 1)`

with fixed gain

`g = 0.12`

and `r_i(t) in {-1,+1}` the relay state. Selection then uses the same FL-4A field working-set assembler and the same 48-token budget.

No other field operator, signed coupling, Q-Page modification or learned parameter is active in FL-4B.

## Threshold calibration

The preregistered threshold grid is:

`theta in {0.05, 0.10, 0.15, 0.20, 0.25}`.

Thresholds are evaluated only on the calibration trace. The selected threshold minimizes, in order:

1. total missed relevant slots;
2. total working-set replacements (churn);
3. smaller `theta` as deterministic final tie-break.

The selected threshold is then frozen and evaluated on the holdout trace without retuning.

## Deterministic traces

Both traces contain eight candidate files and exactly three relevant files per observation. Ground truth is used only for evaluation.

### Calibration trace

64 observations, split into four 16-observation regimes. Relevant sets are:

1. observations `0..15`: `{A,B,C}`;
2. observations `16..31`: `{D,E,F}`;
3. observations `32..47`: `{B,F,G}`;
4. observations `48..63`: `{A,G,H}`.

Steady-state evidence is `0.80` for relevant nodes and `0.20` for irrelevant nodes.

At each true regime transition, leaving nodes receive `0.35` and entering nodes `0.65` on the transition observation; persistent nodes retain `0.80`; all others retain `0.20`. Subsequent observations use the steady-state values.

Twelve one-observation disturbance pulses occur at observations:

`{4,8,12,20,24,28,36,40,44,52,56,60}`.

Each pulse lowers one currently relevant node to `0.42` and raises one currently irrelevant node to `0.58`. The affected pair is fixed by the executable fixture and does not depend on runtime outcomes.

### Holdout trace

64 observations with different regime memberships, transition positions and disturbance pairs:

1. observations `0..13`: `{A,D,H}`;
2. observations `14..29`: `{B,C,G}`;
3. observations `30..46`: `{A,E,F}`;
4. observations `47..63`: `{C,F,H}`.

Steady-state evidence remains `0.80` / `0.20`.

At true transitions, leaving nodes receive `0.34` and entering nodes `0.66` for one observation.

Twelve holdout disturbance pulses occur at:

`{3,7,11,18,22,26,34,38,42,51,55,59}`.

Each lowers one relevant node to `0.41` and raises one irrelevant node to `0.59`, using fixed preregistered node pairs.

## Metrics

For baseline and hysteretic variant report:

- total missed relevant slots over all observations;
- exact-set accuracy: observations where selected set equals ground truth;
- total replacements between consecutive working sets;
- maximum true-transition recovery latency in observations;
- maximum and total token use;
- exact deterministic replay.

One replacement is counted as one selected item leaving and one new item entering; equivalently half the symmetric-difference size between consecutive equal-cardinality sets.

Transition recovery latency is the number of observations from the truth-set transition observation until the first exact working-set match. A match on the transition observation has latency `0`.

## Protocol validity

The executable fails closed only when evidence validity is broken:

- any selected working set exceeds 48 tokens;
- any observation does not return exactly three candidates;
- the chosen threshold is not one of the preregistered grid values;
- any reported metric is non-finite where applicable;
- replay is not exact.

Comparative hypotheses do **not** control process exit; a false hypothesis remains a valid negative result.

## Comparative hypotheses

- **H4-B1 — holdout quality:** hysteresis yields strictly fewer missed relevant slots than the CCOS baseline on the holdout trace.
- **H4-B2 — anti-thrash:** hysteresis yields strictly fewer working-set replacements than baseline on the holdout trace.
- **H4-B3 — bounded switching cost:** hysteresis maximum holdout transition latency is at most one observation greater than baseline.
- **H4-B4 — exact budget preservation:** both methods stay within the identical 48-token budget for every observation.
- **H4-B5 — calibration transfer:** the threshold selected only on calibration satisfies both H4-B1 and H4-B2 on holdout without retuning.

## Non-claims

A positive FL-4B result would establish only that this explicit relay can improve stability/selection quality on these controlled bounded traces. It would not establish superiority on real software repositories, LLM contexts, arbitrary CCOS workloads, biological cognition, or magnetic hardware.

## Stop condition

FL-4B stops when the calibration grid, frozen-threshold holdout evaluation, machine-readable evidence and deterministic replay have completed, while FL-0 through FL-4A plus FL-E1/FL-E2 remain regression-clean.