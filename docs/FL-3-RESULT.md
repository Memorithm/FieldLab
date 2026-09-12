# FL-3 Result — Hysteresis & Context Switching

FL-3 was executed only after [`prereg/FL-3.md`](../prereg/FL-3.md) froze the operator, fixture, thresholds, gain, metrics and hypotheses.

## Protocol validity

The deterministic campaign is structurally valid:

- 80 observations;
- 3 genuine context changes;
- 8 isolated contradictory pulses;
- relay thresholds from 0.10 through 0.60 match their declared descending/ascending switching points;
- exact deterministic replay holds;
- FL-0, FL-1 and FL-2 regressions remained green.

Provenance fingerprint: `fnv1a64:906b07d4ebaa6fce`.

## Context-task results

| Condition | Total errors | Contradiction errors | Transition errors | False context changes | Mean switch latency | Lock-in events |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| memoryless | 8 | 8 | 0 | 16 | 0 | 0 |
| relay 0.10 | 8 | 8 | 0 | 16 | 0 | 0 |
| relay 0.20 | 11 | 8 | 3 | 16 | 1 | 0 |
| **relay 0.30** | **6** | **0** | 6 | **0** | 2 | 0 |
| relay 0.40 | 9 | 0 | 9 | 0 | 3 | 0 |
| relay 0.50 | 9 | 0 | 9 | 0 | 3 | 0 |
| relay 0.60 | 9 | 0 | 9 | 0 | 3 | 0 |

The threshold `0.30` is the best condition under the preregistered total-error metric in this fixture. It rejects all eight isolated contradictory pulses and removes the 16 false context changes produced by the memoryless response, while paying two observations of latency at each genuine transition.

The higher thresholds retain the disturbance rejection but increase the switching cost to three observations and produce nine transition errors in total. They do not cross the preregistered lock-in boundary (`latency > 3`).

## Hypothesis outcomes

- **H3-A1 useful retention — supported in this fixture.** Threshold 0.30 reduces total errors from 8 to 6.
- **H3-A2 disturbance rejection — supported in this fixture.** Thresholds 0.30–0.60 eliminate all eight isolated contradictory-pulse errors.
- **H3-A3 switching cost — supported in this fixture.** Every threshold that fully rejects the contradictory pulses has positive switching latency.
- **H3-A4 lock-in frontier — not supported by this fixture.** No tested threshold produces a preregistered lock-in event.

The negative H3-A4 result is retained rather than redefining the lock-in threshold after inspection.

## Interpretation boundary

FL-3 demonstrates one deterministic retention/switching trade-off for an explicit two-threshold relay coupled as a field bias. It does not establish that hysteresis is generally optimal for cognition, that 0.30 is transferable to other tasks, or that biological cognition uses this mechanism.

The result is nevertheless useful for the next FieldLab step: FL-4 can map CCOS causal pressure/working-set traces into field variables while treating hysteresis as an optional, independently ablatable context-retention operator rather than an implicit implementation detail.

Machine-readable evidence: [`results/FL-3-hysteresis-context.json`](../results/FL-3-hysteresis-context.json).
