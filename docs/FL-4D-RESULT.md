# FL-4D result — native temporal CCOS trace acquisition

FL-4D acquired time-ordered scored working-set snapshots from the actual pinned CCOS runtime on CCOS's real source tree. The protocol and both 24-observation schedules were frozen in `prereg/FL-4D.md` before execution.

## Result

The gate is valid. Two complete acquisitions from fresh CCOS workspaces produced semantically identical trace documents and identical raw SHA-256 hashes:

`4eabc57a1ba9147591d5b4ca63b62a0eaa25d4549d24bc4336bba061e2cfafe4`

Each acquisition contained 24 calibration observations and 24 holdout observations over the fixed A/B/C/D anchor schedule, using native `signal_failure(depth=3)` followed by native `working_set` recall at a hard 2,048-token budget.

| Quantity | Calibration | Holdout |
| --- | ---: | ---: |
| Observations | 24 | 24 |
| Distinct snapshots | 23 | 24 |
| Token range | 1,317–2,038 | 1,709–2,027 |
| Affected-node range | 177–418 | 54–418 |

The snapshots are not a static replay of one ranking. Native scores and ordered windows change under the frozen stimulus schedule. For example, calibration begins with `external_memory.rs` as the highest-scored file under A; the isolated B pulse introduces `agent_session.rs` and changes the native ordering while preserving the hard budget.

## Hypotheses

All five preregistered FL-4D hypotheses are supported:

- **H4-D1 — deterministic native temporal replay:** supported;
- **H4-D2 — stimulus sensitivity:** supported;
- **H4-D3 — hard-budget preservation:** supported;
- **H4-D4 — scored evidence availability:** supported;
- **H4-D5 — anchor diversity:** supported.

## Interpretation

FL-4D closes the missing acquisition gap identified after FL-4C. FieldLab now has an externally grounded temporal evidence source produced by the real pinned CCOS runtime, on real CCOS code, with native scores, ordering, propagated failure counts and token usage.

The result does not yet establish that a field operator improves CCOS. It establishes that such a comparison can now be performed without substituting a FieldLab-authored score model for CCOS behavior.

The next experiment must keep the native 2,048-token CCOS baseline intact, calibrate any explicit field operator only on the FL-4D calibration trace, freeze its parameters, and evaluate once on the untouched FL-4D holdout trace. The negative FL-4C high-degree coverage cases remain in scope and must not be filtered out.

Machine-readable summary: `results/FL-4D-native-temporal-ccos.json`. The two full raw traces are retained by the successful FL-4D Actions run and identified by the SHA-256 above.
