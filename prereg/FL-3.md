# FL-3 — Hysteresis & Context Switching

Status: **frozen before result inspection**.

## Question

Can explicit path dependence preserve useful context through weak contradictory evidence without producing unacceptable lock-in when the true context changes?

FL-3 does not treat slow numerical relaxation as hysteresis. It introduces an explicit deterministic relay (hysteron) with an internal state and separate lower/upper switching thresholds. The relay may bias a field node, while a matched memoryless field receives the same raw evidence without the relay bias.

## Mechanism

A relay state `q ∈ {-1,+1}` updates from scalar evidence `u` using thresholds `l < h`:

- if `u <= l`, set `q = -1`;
- if `u >= h`, set `q = +1`;
- otherwise retain the previous `q`.

The symmetric FL-3 cases use `l=-theta`, `h=+theta`.

For the field-coupled task, one two-dimensional unit field node is reinitialized to the same neutral state for every observation. This intentionally isolates memory in the relay rather than in numerical integration. The external field is

`H = [u + g*q, 0]`

for hysteretic conditions and

`H = [u, 0]`

for the memoryless control. Here `g=0.35` is fixed before execution.

The node is relaxed with the existing deterministic Heun integrator for 32 steps (`dt=0.05`, mobility `1.0`) and decoded by the sign of its first component.

## Part A — Analytic relay loop

For each symmetric threshold

`theta ∈ {0.10, 0.20, 0.30, 0.40, 0.50, 0.60}`

the input is swept from `+1.0` to `-1.0` and back in increments of `0.05`.

Structural checks:

1. the descending switch must occur in `[-theta-0.05, -theta+1e-12]`;
2. the ascending switch must occur in `[theta-1e-12, theta+0.05]`;
3. the measured loop width must be positive;
4. replay must be exact.

These checks validate the declared hysteresis semantics; they are CI gates.

## Part B — Deterministic context task

The true context contains four 20-observation segments:

`+1 -> -1 -> +1 -> -1`.

At each genuine transition, evidence ramps toward the new context with magnitudes

`0.10, 0.20, 0.30, 0.40, 0.55, 0.70`

for the first six observations. All later stable observations use magnitude `0.55`, except two isolated contradictory pulses per segment at offsets 9 and 14, where evidence has magnitude `0.25` with the wrong sign.

Thus there are exactly:

- 80 observations;
- 3 genuine context changes;
- 8 isolated contradictory pulses.

No random noise is used in FL-3.

## Conditions

1. **memoryless**: raw evidence only (`g=0`), no relay;
2. relay thresholds `theta = 0.10, 0.20, 0.30, 0.40, 0.50, 0.60`, each with fixed bias gain `g=0.35`.

All field integration parameters are identical.

## Metrics

For every condition report:

- total classification errors;
- errors on isolated contradictory pulses;
- errors inside the six-observation transition ramps;
- false context changes outside genuine transition ramps;
- switch latency for each of the three genuine context changes;
- mean switch latency;
- number of lock-in events.

A lock-in event is preregistered as a genuine transition whose first correct decoded context occurs more than 3 observations after the transition begins, or never occurs inside the six-observation ramp.

## Comparative hypotheses

These are scientific outcomes and do **not** gate CI.

- **H3-A1 — useful retention:** at least one nonzero threshold has fewer total errors than the memoryless field.
- **H3-A2 — disturbance rejection:** at least one threshold eliminates all eight isolated contradictory-pulse errors.
- **H3-A3 — switching cost:** every threshold that eliminates all contradictory-pulse errors has positive mean switch latency.
- **H3-A4 — lock-in frontier:** at least one larger threshold produces more lock-in events than the best lower-threshold condition.

A failure of any hypothesis remains a valid result.

## Evidence validity gates

CI fails only if:

- the relay analytic loop violates its declared threshold bounds;
- the context fixture does not contain exactly 80 observations, 3 true transitions and 8 contradictory pulses;
- output contains non-finite values;
- replay differs;
- the FL-0/FL-1/FL-2 regression chain fails.

Comparative performance does not control CI success.

## Non-claims

FL-3 cannot establish biological hysteresis, general cognitive superiority, optimal thresholds, or robustness outside this deterministic fixture. It characterizes one explicit path-dependent operator and its retention/switching frontier.