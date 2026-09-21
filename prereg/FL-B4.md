# FL-B4 preregistration — complete-left censor-aware dwell contrast

Status: **executed; outcomes in docs/FL-B4-RESULT.md (HB4-0/1/3 supported; HB4-2/4 not).** Originally preregistered before implementation and outcome inspection.

`FL-B4` continues the orthogonal Boolean×Field subseries. It does **not** occupy,
rename, authorize, or change roadmap `FL-6`.

## Motivation

FL-B2 used dual `ObservationCut` boundaries on E1 stress-probe relaxations and
observed **zero** determinate True/False dwell-tail orderings: short opposite-value
runs under left/right cuts only widened upper bounds, producing indeterminate
overlap. FL-B4 tests the declared remedy that does **not** retune FL-B2: keep the
same dynamics, predicates, probes, cadence, and threshold grid, but declare the
**left** window edge `Complete` because the initial state is a known constructed
probe (not an unknown ongoing dwell), while the right edge remains `ObservationCut`.

## Question

Under a frozen observation window with left `Complete` and right `ObservationCut`,
do FL-B1 predicate bits on FL-5G E1 axial stress-probe relaxations produce at least
one determinate censor-aware True/False dwell-tail ordering on the frozen threshold
grid, and do memory signatures / bit roles then separate relative to CONST-FALSE,
without Boolean feedback into dynamics?

## Fixed foundations

Identical to FL-B2 except boundary declaration and case-id namespace:

- Memories P0/P1/P2; E1 stiffness `a = 1.25`; FL-B1.0 16-predicate bank;
- Stress radius `0.50`; Heun `dt = 0.05`, mobility `1.0`, 64 steps → 65 observations;
- Threshold grid `[2,4,8,16,32]`; primary `τ* = 8`;
- Probes: all nodes × both directions (48 trajectories);
- Case ids: `flb4|p{p}|pred{k}|n{j}|d{minus|plus}`;
- Left boundary: `ObservationBoundary::Complete`;
- Right boundary: `ObservationBoundary::ObservationCut`;
- Observation-only; no Kaplan–Meier / survival claims.

## Hypotheses

- **HB4-0 validity:** protocol shape, finite metrics, norm tolerance, exact replay.
- **HB4-1 determinate contrast exists:** ≥1 primary case×threshold yields
  `true-definitely-higher` or `false-definitely-higher`.
- **HB4-2 memory separation:** three memory signatures at `τ*` pairwise distinct.
- **HB4-3 baselines:** CONST-FALSE determinate hits = 0 and FL-B4 determinate hits
  strictly greater; PERM-PRED fails signature distinctness or role separation.
- **HB4-4 bit-role separation:** for ≥1 memory, ALIGNED vs ANTI modal multisets at
  `τ*` differ.

HB4-1..HB4-4 may fail without failing CI when HB4-0 holds.

## Anti-leakage / boundary

Do not retune FL-B2 after the fact; FL-B4 is a new namespace. No FL-6 authorization,
cognition, hardware, biology, LLM, or survival-model claims.
