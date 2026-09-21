# FL-B2 result — censor-aware dwell-tail contrast on E1 axial memories

Status: **qualified negative fixture result**.

This result records the first executable campaign from [`prereg/FL-B2.md`](../prereg/FL-B2.md). It is an observation-only Boolean×Field result for the frozen FieldLab E1 axial fixture. It does not introduce Boolean feedback into the field dynamics and it does not authorize FL-6.

## Provenance

- implementation branch campaign executed against local source revision recorded in the machine report's `source_revision` field at run time;
- compact retained summary: [`results/FL-B2-censored-dwell-contrast-summary.json`](../results/FL-B2-censored-dwell-contrast-summary.json);
- lean per-case ordering panel: [`results/FL-B2-censored-dwell-contrast-cases.json`](../results/FL-B2-censored-dwell-contrast-cases.json);
- full pretty-printed semantic report SHA-256: `4a4032fc843963ec6d15607957273d6faca75fd2b2cccd18c3105832795cad20`;
- exact semantic replay succeeded (`replay_exact = true`);
- campaign-wide `max_norm_squared_error = 4.440892098500626e-16` (≤ `1e-10`).

## Frozen protocol executed

The campaign reused the three FL-5G / FL-B1 axial memories and the FL-B1.0 16-predicate bank. Bank-wide E1 stiffness was `a = 1.25`. For each memory, each node, and each direction, a single-node angular stress probe at radius `0.50` was integrated with projected Heun (`dt = 0.05`, mobility `1.0`, 64 steps → 65 observations). Both observation-window edges were declared `ObservationCut`. Censor-aware dwell-tail contrast profiles and orderings were evaluated on the frozen grid `[2, 4, 8, 16, 32]` with primary signature threshold `τ* = 8`. Case identifiers used the `flb2|…` namespace. Boolean values were never fed back into dynamics.

Panel shape: `48` trajectories × `16` predicates = `768` primary cases.

## Observed result

Exactly `48/768` cases produced two dwell runs (the ALIGNED predicate on the perturbed node, which starts below the `0.9` margin at radius `0.50` and becomes true under E1 relaxation). The remaining `720/768` cases produced a single Boolean value for the whole window.

On the dual-run cases, every grid threshold returned `indeterminate-overlap`: with left/right `ObservationCut` censoring, the short initial false run widens only the false upper bound, so the true and false partial-identification intervals touch or overlap and never separate. On the single-run cases, every threshold returned `undefined-absent-value` because one Boolean value is absent.

Aggregate ordering counts over all case×threshold cells: `3600` undefined-absent-value and `240` indeterminate-overlap. Determinate True/False orderings: `0`.

Memory signatures at `τ* = 8` were therefore identical all-`undefined-absent-value` 16-tuples. ALIGNED and ANTI modal multisets did not differ. The CONST-FALSE baseline also produced zero determinate orderings.

Preregistered outcomes:

- HB2-0 protocol validity: **supported**;
- HB2-1 determinate contrast exists: **not supported**;
- HB2-2 memory-signature separation: **not supported**;
- HB2-3 baselines / non-artifact structure: **not supported** (no FL-B2 determinate surplus over CONST-FALSE);
- HB2-4 bit-role separation: **not supported**.

## Interpretation boundary

The negative result is retained. On this frozen E1 stress-probe panel, censor-aware True/False dwell-tail contrasts do not become determinate under dual `ObservationCut` boundaries when trajectories contain at most one Boolean change. Short censored opposite-value runs prevent strict interval separation, and stable bits yield absent-value undefined contrasts.

This does **not** refute the Boolean observation map from FL-B1, nor does it establish that richer probes, complete-boundary declarations, or multi-crossing observation schedules cannot produce determinate contrasts. Those require a new preregistered slice. It does not authorize Boolean control, cognition claims, survival/Kaplan–Meier semantics, hardware benefit, biological correspondence, LLM quality, or FL-6.

FL-B2 remains orthogonal to roadmap FL-6.
