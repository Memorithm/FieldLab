# FL-4B result — hysteretic bounded working-set selection

FL-4B tested one previously validated FieldLab operator — the FL-3 two-threshold relay — on top of the lossless FL-4A mapping of pinned CCOS working-set semantics. The protocol was frozen in `prereg/FL-4B.md` before result inspection.

## Result

The calibration-only procedure selected `theta = 0.10`. No holdout retuning was performed.

| Holdout condition | Missed relevant slots | Exact sets | Exact accuracy | Replacements | Max transition latency | Max tokens |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Pinned CCOS baseline | 12 | 52 / 64 | 0.8125 | 32 | 0 | 48 |
| FL-4B relay, `theta=0.10` | 0 | 64 / 64 | 1.0000 | 8 | 0 | 48 |

The relay therefore removed all 12 one-slot misses caused by the declared disturbance pulses and reduced working-set replacements from 32 to 8 on the holdout trace, while retaining zero-observation recovery latency on the declared true transitions and never exceeding the same 48-token budget.

The calibration grid is informative rather than hidden: `theta=0.05` behaved like the memoryless baseline, while every tested threshold from `0.10` through `0.25` produced zero calibration misses and seven replacements. The preregistered tie-break therefore selected the smallest such threshold, `0.10`.

## Hypotheses

All five preregistered comparative hypotheses are supported on this controlled fixture:

- H4-B1 holdout quality: supported;
- H4-B2 anti-thrash: supported;
- H4-B3 bounded switching cost: supported;
- H4-B4 exact budget preservation: supported;
- H4-B5 calibration transfer: supported.

The protocol also replayed exactly, selected exactly three equal-cost items at every observation, and stayed within the 48-token budget throughout.

## Interpretation boundary

This is evidence for a narrow claim: an explicit hysteresis relay can stabilize this controlled CCOS-like bounded working-set trace without sacrificing the declared transition responsiveness or budget. It is not evidence yet that the same operator improves real repository traces, arbitrary CCOS workloads or LLM context quality.

Accordingly FL-4 remains in progress. The next FL-4 gate should replay recorded or otherwise externally grounded CCOS traces with the same baseline/variant separation before any operator is proposed for CCOS-Core.

Machine-readable evidence: `results/FL-4B-hysteretic-working-set.json`.
