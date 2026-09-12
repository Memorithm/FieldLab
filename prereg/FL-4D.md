# FL-4D — Native temporal CCOS trace acquisition gate

Status: **frozen before FL-4D execution**.

## Question

Can FieldLab acquire a deterministic time-ordered sequence of scored working-set snapshots from the actual pinned CCOS runtime on CCOS's real source tree, without reconstructing CCOS scoring inside FieldLab?

FL-4D is an acquisition gate. It does **not** yet claim that hysteresis improves the external trace. A comparative field operator is allowed only after this gate produces replayable native temporal evidence.

## External source of truth

The external runtime remains fixed to the same source used by FL-4C:

- repository: `Memorithm/CCOS-Core`;
- commit: `a3c4d7e03744430c74dc337463ff3e944b4933ad`;
- runtime: `ccos` built at that commit with its required `llm` feature;
- corpus: the checkout's real top-level `src/*.rs` files;
- working-set budget: `2048` tokens;
- failure propagation depth: `3`.

FieldLab may orchestrate MCP calls and serialize returned metadata. It must not reimplement CCOS score calculation, causal propagation, region construction or ranking.

## Frozen anchor set

FL-4C selected five anchors using the pinned external campaign selector. FL-4D freezes the first four by that already-declared dependency-count/URI ordering:

- **A** = `src/external_memory.rs` (11 direct flat-file dependencies in FL-4C);
- **B** = `src/agent_session.rs` (6);
- **C** = `src/migrate.rs` (5);
- **D** = `src/region_metrics.rs` (4).

This choice is made after FL-4C and before FL-4D execution. It deliberately includes the two anchors for which H4-C1 failed, so the temporal trace retains the hard-budget stress cases rather than selecting only easy anchors.

## Session construction

Calibration and holdout are separate fresh CCOS workspaces.

For each workspace:

1. ingest every pinned top-level `src/*.rs` file in sorted path order;
2. verify the CCOS integrity surface before the first stimulus;
3. apply the frozen stimulus sequence one event at a time;
4. after each stimulus, call native `signal_failure(node, depth=3)` and then native `recall(strategy=working_set, budget=2048)`;
5. serialize only native response metadata needed for later comparison: stimulus anchor, step index, `affected`, window `tokens`, and for every returned item its `uri`, `score` and `kind` in native order;
6. do not serialize item source content into the FieldLab trace because content size is not part of this acquisition question.

## Frozen temporal schedules

The schedules contain stable blocks, isolated contradictory pulses and genuine context changes. No schedule element may be changed after looking at FL-4D output.

Calibration, 24 observations:

```text
A A A B A A A A  B B B C B B B B  C C C D C C C C
```

Holdout, 24 observations:

```text
C C A C C C C C  A A D A A A A A  D D B D D D D D
```

The single off-block symbols are perturbation pulses. The starts of the second and third 8-observation blocks are declared true context changes.

## Replay protocol

The complete calibration+holdout acquisition is run twice from fresh workspaces against the same pinned checkout.

Strict semantic equality is required for the two complete trace documents. Numeric tolerance, item reordering, field deletion and score rounding are forbidden.

Both raw trace documents are retained independently and SHA-256 hashed.

## Protocol validity

FL-4D fails closed when any of the following is false:

- the external commit is exactly the pinned CCOS commit;
- both acquisition runs exit successfully;
- both calibration and holdout contain exactly 24 observations;
- every observation corresponds to the preregistered stimulus at that step;
- every native recall reports `tokens <= 2048`;
- every returned score is finite;
- every returned URI and kind is non-empty;
- each `affected` count is non-negative;
- strict semantic equality holds between complete run A and run B;
- both raw trace SHA-256 values are recorded.

These are evidence-integrity requirements. They do not require a field operator to win.

## Descriptive hypotheses

The following are reported but do not control process exit, except H4-D1 which duplicates the replay validity gate:

- **H4-D1 — deterministic native temporal replay:** the two complete acquisitions are exactly equal;
- **H4-D2 — stimulus sensitivity:** at least one anchor change changes the native ordered working-set snapshot or its scores;
- **H4-D3 — hard-budget preservation:** every native snapshot remains within 2048 tokens;
- **H4-D4 — scored evidence availability:** every retained item exposes a finite native score suitable for a later operator ablation;
- **H4-D5 — anchor diversity:** each of A/B/C/D appears in at least one retained snapshot as a `file:` item or through an item URI belonging to that source file.

A negative descriptive result remains valid evidence.

## Interpretation boundary

FL-4D is still a controlled scripted workload, not organic agent usage. Its purpose is narrower: establish an externally grounded temporal input on real code using the actual CCOS runtime. If valid, the next FL-4 experiment may calibrate an explicit FieldLab operator on the calibration trace and evaluate it once on the untouched holdout trace at the same 2048-token budget.

## Stop condition

FL-4D stops when the two raw native temporal acquisitions, provenance hashes, typed FieldLab import, descriptive hypotheses and all existing FieldLab regressions have completed on the same PR head.
