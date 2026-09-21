# FL-B2 preregistration — censor-aware dwell-tail contrast on E1 axial memories

Status: **executed; outcomes recorded in docs/FL-B2-RESULT.md (negative on HB2-1..HB2-4).** Originally preregistered before implementation and outcome inspection.

`FL-B2` continues the orthogonal Boolean×Field subseries. It does **not** occupy,
rename, authorize, or change roadmap `FL-6`, which remains reserved for sparse,
low-rank, and multiscale field scaling and stays blocked as an efficacy progression
until an FL-5* gate shows qualified benefit under stable horizon semantics.

## Question

Under a frozen observation window with explicit left/right `ObservationCut`
censoring, do the FL-B1.0 predicate bits observed along FL-5G E1 relaxations of the
three axial memories (P0/P1/P2, bank-wide stiffness `a = 1.25`) produce determinate
censor-aware True/False dwell-tail contrast orderings — or contrast profiles on a
frozen threshold grid — that separate memories and/or bit roles relative to declared
baselines, without feeding Boolean values back into the field dynamics?

## Fixed foundations

- Parent line: current `main` after FL-5G evidence retention and the censor-aware
  dwell-tail contrast-profile contract (`fieldlab.boolean-censor-aware-dwell-tail-contrast-profile.v1`).
- Memories: the three eight-node axial patterns already used by FL-5G / FL-B1, in order:
  1. `P0 = [+1,+1,+1,+1,+1,+1,+1,+1]`
  2. `P1 = [+1,+1,+1,+1,+1,+1,-1,-1]`
  3. `P2 = [+1,+1,+1,+1,-1,-1,+1,+1]`
- Energy: FL-E1 operator lift of the Hebbian graph with bank-wide stiffness
  `a = max(0, -min_s L_s(0)) + 0.25`, which equals `1.25` on this frozen bank.
- Predicate bank: **exactly** the FL-B1.0 bank of 16
  `fieldlab.boolean-predicate.v1` component-threshold predicates
  (node `i=0..7`, component `0`, thresholds `+0.9`/`AtLeast` then `-0.9`/`LessThan`).
- Observation-only: Boolean outputs never enter `FieldState`, the energy, `H_eff`,
  the integrator, or any controller. No Kaplan–Meier, hazard, or survival-model claim.

## Frozen FL-B2.0 observation protocol

### Probes and trajectories

For every memory `p ∈ {0,1,2}`, every node `j ∈ {0..7}`, and every direction
`d ∈ {-1,+1}`, construct the initial state by the FL-B1 single-node angular chart at
**stress radius** `r = 0.50` radians (the FL-B1.0 stress radius known to leave the
predicate margin). All other nodes remain at angle `0`. This yields
`3 × 8 × 2 = 48` trajectories.

Integrate the E1 model with projected Heun, `dt = 0.05`, mobility `1.0`, for
`STEPS = 64` steps, retaining the state at every integer step including the initial
state. Thus each trajectory has `OBSERVATION_COUNT = 65` field observations and
covers simulated time `[0, 3.2]`.

Case identifiers use the fresh namespace
`flb2|p{p}|pred{k}|n{j}|d{minus|plus}` with predicate index `k ∈ {0..15}` in the
FL-B1.0 bank order.

### Censoring and dwell-tail analysis

For each `(memory, probe, predicate)`:

1. Evaluate the predicate on the ordered observation sequence.
2. Build the transition trace and exact dwell runs with the existing
   `field-boolean` contracts.
3. Declare **both** window edges as `ObservationBoundary::ObservationCut`
   (the retained window is not claimed to start at a known reset or end at a known
   terminal Boolean event).
4. Evaluate the exact censor-aware dwell-tail **contrast profile** on the frozen
   threshold grid `THRESHOLDS = [2, 4, 8, 16, 32]` observation counts.
5. Evaluate the exact censor-aware dwell-tail **ordering** at each threshold on that
   same grid.

The primary signature threshold is `τ* = 8`. Thresholds are frozen before outcome
inspection; no data-driven threshold search is permitted.

Construction tolerance is `1e-12`. Campaign validity requires every retained state
to be finite with campaign-wide `max_norm_squared_error ≤ 1e-10`, exact semantic
replay of the machine-readable report, and the exact case cardinality
`48 trajectories × 16 predicates = 768` primary cases.

### Bit roles

On each memory, a predicate is **ALIGNED** when it is `true` on the clean axial
codeword under the FL-B1.0 bank, and **ANTI** when it is `false` on that clean
codeword. Role labels are computed from clean axial states only, before probe
outcomes are inspected for hypothesis scoring.

### Memory signature

For each memory and each predicate index, the **modal ordering** at `τ* = 8` is the
unique most frequent `PredicateDwellTailOrdering` across the 16 probes of that
memory; ties break to `IndeterminateOverlap`, then `UndefinedAbsentValue`, then
`TrueDefinitelyHigher`, then `FalseDefinitelyHigher` (declared lexicographic
tie-break, not a scientific preference). The memory signature is the 16-tuple of
modal orderings.

## Baselines

1. **CONST-FALSE:** for every case, replace the observed bit stream by the constant
   `false` stream of length 65 (single left-and-right-censored false dwell). This
   baseline cannot produce a True/False contrast.
2. **PERM-PRED:** build memory signatures exactly as above, but assign predicate
   index `k` the modal ordering belonging to predicate `(k+1) mod 16` on the same
   memory (cyclic permutation of bit roles). This breaks ALIGNED/ANTI role alignment
   while preserving marginal ordering frequencies.

## Primary hypotheses

- **HB2-0 validity:** protocol shape, finite metrics, norm tolerance, and exact
  semantic replay hold.
- **HB2-1 determinate contrast exists:** at least one primary case yields
  `TrueDefinitelyHigher` or `FalseDefinitelyHigher` at at least one grid threshold.
- **HB2-2 memory separation:** the three memory signatures at `τ* = 8` are pairwise
  distinct.
- **HB2-3 baselines:** CONST-FALSE has zero determinate orderings on the full panel,
  and the FL-B2 determinate-case count is strictly larger than CONST-FALSE's; the
  PERM-PRED signatures fail HB2-2-style pairwise distinctness **or** fail HB2-4-style
  role separation (so role/memory structure is not a permutation artifact).
- **HB2-4 bit-role separation:** for at least one memory, the multiset of modal
  orderings at `τ* = 8` over ALIGNED predicates differs from the multiset over ANTI
  predicates.

HB2-1 through HB2-4 are scientific outcomes. Their failure must not make
protocol-valid execution fail CI.

## Metrics to retain

- stiffness `a`, `dt`, steps, observation count, threshold grid, boundary declaration;
- per-case: case id, run count, transition count, censor annotations summary,
  ordering at each grid threshold, contrast profile points as exact signed rationals;
- aggregate determinate-ordering counts for FL-B2 and CONST-FALSE;
- memory signatures and PERM-PRED signatures;
- ALIGNED/ANTI modal multisets per memory;
- hypothesis booleans;
- `max_norm_squared_error` and `replay_exact`.

## Anti-leakage and interpretation boundary

Do not retune the radius, stiffness rule, predicate bank, threshold grid, cadence,
window length, baselines, or scoring rules inside FL-B2 after outcome inspection.
Richer codes, identifiable ambiguous-panel probes, or stochastic gates with
censor-aware residence recording require a new preregistered slice (for example
FL-B3 or FL-5H).

A positive result shows only that censor-aware Boolean dwell-tail contrasts on this
frozen E1 observation panel can separate declared memories/bit roles relative to the
declared baselines. It does not establish Boolean control benefit, cognition,
compression, hardware advantage, biological correspondence, LLM quality, survival
semantics, or FL-6 authorization. A negative or indeterminate result is retained.
