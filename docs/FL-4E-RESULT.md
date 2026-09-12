# FL-4E — External native-window focus hysteresis result

## Provenance

- FieldLab PR: `#11`
- FieldLab evaluated head: `2548cf9059188d7a890f7b0538774da4725d1f66`
- Pinned CCOS-Core: `a3c4d7e03744430c74dc337463ff3e944b4933ad`
- Protocol: `external-native-window-focus-hysteresis-v1`
- Native working-set budget: `2048` tokens
- CCOS depth: `3`
- Source corpus: `external/CCOS-Core/src`, `55` files
- GitHub Actions run: `34707251891`
- Evidence artifact: `fieldlab-fl4e-39c1e8bc21c491dc6ed25383f6ace3364c8e49a1`
- Artifact digest: `sha256:3759315a3011273711ae9d24db64b55da0980afc08c4fb82328a888f1083008c`
- Trace A SHA-256: `8fb464d962a70e53abeec9d357118b4ce66e0469f78197f40d0312a43c3f7025`
- Trace B SHA-256: `8fb464d962a70e53abeec9d357118b4ce66e0469f78197f40d0312a43c3f7025`

The two complete acquisitions were exactly replay-equal. The evaluator reported `protocol_valid = true`.

## Calibration

The frozen threshold grid was `[0.00, 0.01, 0.02, 0.04, 0.08, 0.16]`. Calibration selected `theta = 0.01` without using the holdout.

| Policy | Errors | Unresolved | False switches | Switches | Max transition latency | Max tokens |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Memoryless baseline | 25 | 1 | 6 | 7 | 8 | 2037 |
| Hysteretic, theta=0.01 | 24 | 1 | 0 | 0 | 8 | 2037 |

Thresholds `0.01` through `0.16` produced the same calibration metrics. The frozen selection rule chose `0.01`.

## Untouched holdout

| Policy | Errors | Unresolved | False switches | Switches | Max transition latency | Max tokens |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Memoryless baseline | 25 | 4 | 3 | 4 | 8 | 2042 |
| Hysteretic, theta=0.01 | 22 | 4 | 1 | 1 | 8 | 2042 |

For this declared workload, the hysteretic focus annotation reduced truth errors from `25` to `22` and false switches from `3` to `1`, while preserving the native CCOS window and remaining within the 2048-token budget. The experiment does not establish a downstream LLM-task improvement and does not alter CCOS working-set membership.

## Preregistered hypotheses

The machine-readable evaluator reported all five declared FL-4E hypotheses as supported on this experiment:

- `H4_E1_holdout_truth_tracking = true`
- `H4_E2_anti_thrash = true`
- `H4_E3_bounded_transition_cost = true`
- `H4_E4_exact_budget_observability = true`
- `H4_E5_calibration_transfer = true`

These outcomes are scoped only to the pinned runtime, corpus, schedules, budget and protocol above. They are not evidence of a universal cognitive advantage, magnetic equivalence, or general LLM improvement.

## Next gate

FL-4E closes the first external comparative focus-ablation gate. Any promotion beyond this result should use a new preregistered workload and preserve the separation between calibration and holdout. FieldLab can now either widen external replication for FL-4 or begin FL-5 only with explicit NoiseLab-compatible perturbation baselines and no reuse of this holdout for tuning.
