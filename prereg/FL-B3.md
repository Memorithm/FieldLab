# FL-B3 preregistration — Boolean collision audit on the FL-5G ambiguous panel

Status: **executed; outcomes recorded in docs/FL-B3-RESULT.md (HB3-0..HB3-4 supported).** Originally preregistered before implementation and outcome inspection.

`FL-B3` continues the orthogonal Boolean×Field subseries. It does **not** occupy,
rename, authorize, or change roadmap `FL-6`.

## Question

On the frozen nine-case FL-5F/FL-5G observable-input panel, do FL-B1 Boolean
observation codes (and a preregistered richer threshold bank) report the same
identifiability ceiling and contradictory-label collisions as the sign-input
audit, without reading target labels or feeding Boolean values into dynamics?

## Fixed foundations

- Parent: `main` after FL-B2.
- Panel construction (exact, before outcomes):
  1. clean `P0,P1,P2` with labels `0,1,2`;
  2. additional records `(P1,label=0)` and `(P2,label=0)` (label conflicts on identical signs);
  3. for target `1`, flip `P0` bits at indices `{6,7}` to `-1` (two cues);
  4. for target `2`, flip `P0` bits at indices `{4,5}` to `-1` (two cues).
  Total: 9 records. Case ids: `flb3|k{0..8}` in construction order above.
- Axial embedding: node `i` with sign `s` is `[s, 0]` via `NodeState::try_unit(..., 1e-12)`.
- Banks (frozen, not fit to labels):
  - **B1:** exact FL-B1.0 16-predicate bank (`±0.9` on component 0).
  - **RICH:** for each node `i=0..7`, predicates
    `(comp=0, thr=+0.9, AtLeast)`, `(comp=0, thr=-0.9, LessThan)`,
    `(comp=0, thr=+0.5, AtLeast)`, `(comp=0, thr=-0.5, LessThan)` — 32 predicates, node-major order.
- Observation-only: no Boolean feedback into energy, `H_eff`, integrator, or controller.

## Hypotheses

- **HB3-0 validity:** finite unit states, addressable predicates, exact semantic replay, 9 records.
- **HB3-1 B1 collisions exist:** B1 Boolean grouping of the 9 codes has at least one multi-member group.
- **HB3-2 conflict coverage:** every FL-5G contradictory-label sign group is contained in some B1 Boolean collision group (Boolean does not silently resolve label conflicts).
- **HB3-3 ceiling match:** the equal-weight deterministic exact-label ceiling from B1 Boolean groups equals the sign-input ceiling `7/9` declared by FL-5G for this panel.
- **HB3-4 richer bank does not invent identity:** RICH Boolean ceiling remains `≤ 7/9` and RICH still collides on every contradictory-label sign group (no label-free separation of identical axial cues).

HB3-1..HB3-4 are scientific outcomes; failures must not fail CI when HB3-0 holds.

## Metrics

- ordered predicate definitions for B1 and RICH;
- per-case id, signs, label (labels applied only after code construction), B1 code, RICH code;
- B1/RICH group membership, collision groups, ceilings;
- sign-input groups and ceiling (recomputed with `group_observables`);
- whether each sign-conflict group is Boolean-covered;
- `max_norm_squared_error`, `replay_exact`.

## Anti-leakage / boundary

Do not retune thresholds, panel membership, or scoring after outcomes. A later slice may preregister explicit external context bits; FL-B3 does not add them. No FL-6 authorization, cognition, survival, hardware, biology, or LLM claims.
