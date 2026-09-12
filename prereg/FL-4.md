# FL-4 — CCOS Field Mapping

Status: **FL-4A frozen before result inspection**.

## Pinned CCOS reference

FL-4A maps the semantics of `Memorithm/CCOS-Core` at commit:

`a3c4d7e03744430c74dc337463ff3e944b4933ad`

No FieldLab code is inserted into CCOS-Core in this stage.

The pinned semantics used here are:

1. node score

   `clamp(base*w_base + failure*w_failure + recency*w_recency + ln(max(access,1))*w_access + centrality*w_centrality + state_bias - distrust, 0, 1)`

   where `state_bias = 0` for Stable, `+0.3` for Working, `-1.0` for Orphan and `distrust = (1-clamp(trust,0,1))*w_trust`;

2. default scoring weights

   `w_base=.15`, `w_failure=.50`, `w_recency=.30`, `w_access=.05`, `w_centrality=0`, `w_trust=0`, `failure_decay=.8`, `failure_fanout=6`;

3. directed failure propagation

   `p(target) += p(source) * edge_weight * failure_decay^depth * fanout_damp`

   with `fanout_damp = min(1, failure_fanout / max(out_degree, failure_fanout))`, clamping node pressure to `[0,1]`, always applying the current hop and recursing only when the added pressure exceeds the paging floor;

4. public working-set ordering is deterministic by descending causal score, then ascending URI, before budget assembly;

5. dual evidence is already explicit in CCOS Q-Pages:

   `belief = (support - contradiction)/(support + contradiction + 1)`

   `conflict = 2*sqrt(support*contradiction)/(support + contradiction + 1)`.

FL-4A treats these as the baseline semantics. It does **not** reinterpret every CCOS edge as magnetic attraction or repulsion.

## Field representation

FL-4A asks only whether the current CCOS state can be represented as a deterministic field without semantic loss.

### Scalar activation field

For node `i`, define the scalar activation potential

`phi_i = sum_k source_{i,k}`

where the sources are exactly the CCOS score contributions (base, failure, recency, access, optional centrality, lifecycle bias and provenance distrust). The observable activation is `clamp(phi_i,0,1)`.

This is deliberately an identity mapping, not a novelty claim. Passing it establishes a compatibility layer on which later non-CCOS field operators can be ablated safely.

### Causal pressure field

Failure pressure is represented as an ordered emission process over the directed CCOS graph. Each traversal emits

`delta = phi_failure(source) * edge_weight * decay^depth * fanout_damp`.

The traversal order is preserved because the pinned CCOS recursive implementation is order-sensitive when multiple paths accumulate into the same node. FL-4A does **not** silently replace it with a commutative diffusion equation.

### Belief/conflict field

A Q-Page maps to a two-component state

`q = [belief, conflict]`.

This preserves CCOS's existing signed direction and orthogonal conflict magnitude. Later FL-4 experiments may compare vector-field frustration against this baseline; they must not claim that `Contradicts` edges were previously unsigned.

## Deterministic fixtures

### A — score equivalence

A fixed eight-node fixture spans:

- Stable, Working and Orphan lifecycle states;
- failure relevance from zero to high pressure;
- multiple recency/access levels;
- trust from 0.25 to 1.0;
- in-degrees from zero upward.

Two scoring profiles are executed:

- exact CCOS defaults;
- a fixed extended profile enabling in-degree centrality and provenance distrust, solely to exercise the off-by-default terms.

For every node, the independent CCOS-reference score and scalar-field activation must agree within `1e-12`.

### B — failure propagation equivalence

A fixed directed graph contains both a sparse causal chain and a hub with out-degree greater than the default fan-out threshold. Origin pressure is `0.95`. The paging floor is `0.10`; max depth is `3`.

The independent CCOS-reference recursion and scalar-field emission implementation must agree on every final node pressure within `1e-12`.

### C — bounded working-set equivalence

The same fixture is assigned deterministic contents/token costs. Reference CCOS-like assembly and field-derived assembly use the same token budget, score ordering and URI tie-break. The selected URI sequence and total token estimate must be exactly equal.

### D — Q-Page equivalence

A fixed grid of support/contradiction pairs includes zero evidence, one-sided evidence, balanced conflict and asymmetric conflict. The independent Q-Page reference and `[belief, conflict]` field mapping must agree within `1e-12`.

## Structural hypotheses

These are evidence-validity requirements for FL-4A, not performance claims:

- **H4-A1 score equivalence:** all mapped activation scores match the pinned CCOS formula.
- **H4-A2 propagation equivalence:** all mapped failure pressures match the pinned CCOS recursion.
- **H4-A3 working-set equivalence:** the same budget yields the same ordered selection.
- **H4-A4 Q-Page equivalence:** belief/conflict coordinates match CCOS formulas.
- **H4-A5 replay:** repeating FL-4A yields bit-identical machine-readable measurements.

If an equivalence check fails, FL-4A fails closed. We do not proceed by loosening semantics post-hoc.

## What FL-4A does not test

FL-4A does not test whether fields improve CCOS. It establishes a semantic bridge only.

After FL-4A passes, FL-4B may preregister optional operators on matched traces:

- signed/frustrated vector coupling versus native Q-Page conflict;
- FL-3 hysteresis as an optional anti-thrash retention operator;
- changed working-set quality under an identical token budget.

Those operators must remain outside CCOS-Core until they beat the native CCOS baseline under replayable evidence.
