# FL-4E — External native-window focus hysteresis ablation

Status: **frozen before FL-4E execution**.

## Question

On a new externally grounded temporal workload produced by the actual pinned CCOS runtime, can explicit hysteresis improve the stability and truth-tracking of the active focus chosen from a native 2,048-token CCOS working set, without inventing scores for nodes that are absent from that window?

FL-4E is the first comparative external operator ablation in the FL-4 series. It follows FL-4D, which established that native CCOS working-set traces on real CCOS source code are deterministic, stimulus-sensitive and scored.

## External source of truth

The runtime remains fixed to:

- repository: `Memorithm/CCOS-Core`;
- commit: `a3c4d7e03744430c74dc337463ff3e944b4933ad`;
- binary: `ccos` built at that commit with its required `llm` feature;
- corpus: the checkout's real top-level `src/*.rs` files;
- native working-set budget: `2048` tokens;
- failure propagation depth: `3`.

Calibration and holdout are separate fresh CCOS workspaces. All source files are ingested in sorted path order before the first stimulus.

## Frozen candidate anchors

The same real high-degree anchors used by FL-4D remain in scope:

- **A** = `src/external_memory.rs`;
- **B** = `src/agent_session.rs`;
- **C** = `src/migrate.rs`;
- **D** = `src/region_metrics.rs`.

This deliberately retains the FL-4C hard-budget stress cases A and B.

## No-imputation rule

FL-4E may use an anchor score only when the exact `file:<path>` item for that anchor is present in the native `working_set` response at that observation.

If an anchor is absent, no zero, floor, extrapolated, replayed, lexical or reconstructed score may be substituted for it.

The hysteretic policy may retain its current focus only while that focus remains present in the current native window. If the current focus disappears, the policy must immediately fall back to the highest-ranked currently visible candidate anchor.

This rule is the central observability constraint of FL-4E.

## New frozen workloads

FL-4E does not reuse the inspected FL-4D holdout. Each block has one declared persistent truth anchor; isolated off-anchor stimuli are contradictory pulses. True context changes occur only at block boundaries.

Calibration, 32 observations:

```text
A A C A A B A A | C D C C A C C C | B B A B D B B B | D C D D D A D D
```

Truth by block:

```text
A A A A A A A A | C C C C C C C C | B B B B B B B B | D D D D D D D D
```

Holdout, 32 observations:

```text
B D B B B A B B | D D C D B D D D | A C A A D A A A | C B C D C C C C
```

Truth by block:

```text
B B B B B B B B | D D D D D D D D | A A A A A A A A | C C C C C C C C
```

No stimulus, truth label or block boundary may change after result inspection.

## Native trace acquisition

At every observation:

1. call native `signal_failure(file:<stimulus>, depth=3)`;
2. call native `recall(strategy=working_set, budget=2048)`;
3. retain native `tokens` and native item order;
4. for each returned item retain only `uri`, `score`, and `kind` in the machine-readable trace.

The complete calibration+holdout acquisition is run twice from fresh workspaces. Strict semantic equality is required. The two raw trace SHA-256 hashes are retained.

## Baseline focus policy

For each observation, scan the native working-set items in their returned order and retain exact file nodes whose URI is one of `file:A`, `file:B`, `file:C`, or `file:D`.

The **memoryless baseline focus** is the first such visible candidate. If none of the four file nodes is visible, baseline focus is unresolved for that observation and counts as a focus error.

No other CCOS item is discarded from the native window; this focus is an annotation over the unchanged bounded context.

## Hysteretic focus policy

Let `f(t-1)` be the previous hysteretic focus and let `c(t)` be the current memoryless candidate.

- If there is no visible candidate, focus is unresolved.
- If there is no previous focus, choose `c(t)`.
- If `f(t-1)` is absent from the current native window, choose `c(t)` immediately.
- If `f(t-1) == c(t)`, retain it.
- Otherwise both have native visible scores in the same window. Switch to `c(t)` only when

```text
score(c(t)) - score(f(t-1)) >= theta
```

  otherwise retain `f(t-1)`.

This is a discrete Schmitt-style persistence operator over native CCOS evidence. It does not modify CCOS scores or window membership.

## Frozen calibration grid

The candidate thresholds are fixed before FL-4E execution:

```text
0.00, 0.01, 0.02, 0.04, 0.08, 0.16
```

The scale is informed only by the already-completed FL-4D evidence, where visible file-score differences were of order hundredths. No FL-4E calibration or holdout value has been observed when this grid is frozen.

Threshold selection uses calibration only and is lexicographic:

1. fewest focus errors against calibration truth;
2. fewest false switches during isolated contradictory pulses;
3. fewest total focus switches;
4. lowest maximum transition latency after a true block boundary;
5. smallest threshold.

The selected threshold is frozen before holdout evaluation inside the experiment executable. No holdout retuning is permitted.

## Metrics

For baseline and hysteretic policies, report separately on calibration and holdout:

- focus errors: observations where focus != declared truth;
- unresolved observations;
- false switches: focus changes when declared truth did not change;
- total focus switches;
- maximum transition latency after true block changes;
- observations in which the selected focus is actually present in the native window;
- maximum native window tokens.

The native CCOS window itself is never enlarged or rewritten. Therefore the comparative operator has exactly the same 2,048-token source context as the baseline at every observation.

## Protocol validity

FL-4E fails closed if any of the following is false:

- external commit equals the pinned CCOS commit;
- both acquisitions exit successfully;
- calibration and holdout each contain exactly 32 observations;
- the frozen stimulus sequences match exactly;
- every native window reports `tokens <= 2048`;
- every retained score is finite;
- item order and values replay exactly between the two complete acquisitions;
- every chosen threshold is from the frozen grid;
- hysteretic focus is never retained when its exact file node is absent from the current native window.

Scientific hypotheses below do not control process exit.

## Preregistered hypotheses

- **H4-E1 — holdout truth tracking:** hysteretic holdout focus errors are lower than the memoryless baseline.
- **H4-E2 — anti-thrash:** hysteretic holdout false switches are lower than the memoryless baseline.
- **H4-E3 — bounded transition cost:** hysteretic maximum holdout transition latency is at most baseline latency + 1 observation.
- **H4-E4 — exact budget/observability:** every native window stays within 2,048 tokens and every resolved hysteretic focus is an actually visible native file node.
- **H4-E5 — calibration transfer:** the threshold chosen without holdout access satisfies both H4-E1 and H4-E2 on holdout.

A negative hypothesis is retained as valid evidence.

## Interpretation boundary

FL-4E tests whether explicit temporal persistence improves **focus selection within an unchanged native bounded context**. It does not yet claim that rewriting CCOS window membership improves downstream LLM task success. A positive result would justify a later matched experiment in which the selected focus drives a native bounded `around` recall; a negative result would constrain or terminate this hysteresis line for CCOS integration.

## Stop condition

FL-4E stops when the two native acquisitions, exact replay proof, calibration-only threshold selection, untouched holdout comparison, hypotheses and all existing FieldLab regressions complete on the same PR head.
