# FL-B4 result — complete-left censor-aware dwell contrast

Status: **qualified mixed fixture result**.

This result records the executable campaign from [`prereg/FL-B4.md`](../prereg/FL-B4.md). It is observation-only. It does not feed Boolean values into field dynamics and does not authorize FL-6.

## Provenance

- compact summary: [`results/FL-B4-complete-left-dwell-contrast-summary.json`](../results/FL-B4-complete-left-dwell-contrast-summary.json);
- lean cases: [`results/FL-B4-complete-left-dwell-contrast-cases.json`](../results/FL-B4-complete-left-dwell-contrast-cases.json);
- full semantic report SHA-256: `b25a7d443e7d1b5c76521dd2b8167d7b87383a2a2501c958aec58d9e98285333`;
- exact semantic replay succeeded; max norm-squared error remained far below `1e-10`.

## Protocol delta versus FL-B2

Identical E1 stress-probe panel, FL-B1 predicates, cadence, and threshold grid. The only preregistered change is the left observation boundary: `Complete` (known constructed initial probe) with right still `ObservationCut`.

## Observed result

Across 768 cases × 5 thresholds:

- `3600` `undefined-absent-value` (single-value windows);
- `224` `true-definitely-higher`;
- `16` `indeterminate-overlap`.

So **determinate contrasts exist** once the left edge is declared complete. CONST-FALSE still yields zero determinate hits. Memory signatures at `τ*=8` are **not** pairwise distinct, and ALIGNED/ANTI modal multisets do not differ on this aggregation.

Preregistered outcomes:

- HB4-0 protocol validity: **supported**;
- HB4-1 determinate contrast exists: **supported**;
- HB4-2 memory-signature separation: **not supported**;
- HB4-3 baselines / non-artifact structure: **supported**;
- HB4-4 bit-role separation: **not supported**.

## Interpretation boundary

FL-B4 confirms the FL-B2 failure-mode diagnosis: dual `ObservationCut` prevented determinate True/False orderings on this panel, while a Complete left boundary unlocks determinate `true-definitely-higher` cells without Boolean feedback into dynamics. Signature-level memory/role separation still fails under the FL-B2-style modal aggregation. No FL-6, cognition, survival, hardware, biology, or LLM claim follows.
