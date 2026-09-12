# FieldLab — Cognitive Field Dynamics Laboratory

**FieldLab is Memorithm's research bench for testing whether field dynamics can serve as useful computational primitives for cognition, memory, retrieval and adaptive inference.**

The project begins from magnetic-field-inspired mathematics — attraction, repulsion, orientation, energy landscapes, hysteresis, resonance, metastability and collective transitions — without claiming that cognitive state is physically magnetic. Physical spintronic or skyrmion correspondence is a later research question, not an assumption.

> **Scientific rule:** every claimed benefit must survive a declared baseline, matched resource budget, reproducible protocol and explicit failure criterion. A lower energy is not automatically better cognition; a stable attractor is not automatically a correct answer.

## FL research series — primary roadmap

The **FL series is the authoritative line of development and experimentation for FieldLab**. Work should advance this sequence rather than accumulate unrelated features. Exploratory code may support a later series, but a series is promoted only when its protocol, controls and evidence boundary are explicit.

| Series | Research line | Status | Primary question | Exit criterion |
| --- | --- | --- | --- | --- |
| **FL-0** | **Mathematical sanity & deterministic field kernel** | 🟠 Active bootstrap | Do the state, energy, effective-field and integration primitives obey their declared invariants? | Analytic/tiny reference cases pass; deterministic replay holds; invalid states fail closed; numerical policy documented. |
| **FL-1** | **Associative recall & attractor memory** | ⚪ Planned | Can field attractors recover corrupted memories better than declared retrieval baselines at matched state size? | Pre-registered recall/corruption metrics and baselines executed with reproducible evidence. |
| **FL-2** | **Competing hypotheses, signed coupling & frustration** | ⚪ Planned | Do attraction and repulsion help resolve controlled contradictory evidence rather than merely creating instability? | Separation/calibration benefit measured against unsigned/diffusion controls; oscillation and failure regimes reported. |
| **FL-3** | **Hysteresis & context switching** | ⚪ Planned | Can path dependence retain useful cognitive state without unacceptable lock-in? | Retention benefit and switching cost jointly characterized; lock-in frontier measured. |
| **FL-4** | **CCOS field mapping** | ⚪ Planned | Is CCOS causal pressure/heat usefully representable as a discrete field, and do extra field operators improve bounded context selection? | Fixed CCOS traces replayed; current CCOS vs field variants compared at identical token budget while preserving auditability. |
| **FL-5** | **Resonance, perturbation & stochastic exploration** | ⚪ Planned | Can controlled perturbation improve basin escape, recall or ambiguity resolution? | Benefit exceeds matched no-noise and surrogate controls under the same compute envelope. |
| **FL-6** | **Sparse, low-rank & multiscale field scaling** | ⚪ Planned | Can the useful part of dense interaction be retained without assuming scalable `O(N²)` coupling? | Approximation error and interaction/runtime scaling reported separately against a dense reference. |
| **FL-7** | **Learned fields & adaptive couplings** | ⚪ Planned | Which field parameters can be learned without obscuring reference semantics or provenance? | Explicit train/validation/test split; learned model beats fixed-rule baselines and remains inspectable. |
| **FL-8** | **Bounded cognitive tasks** | ⚪ Planned | Do validated field primitives improve memory/reasoning tasks rather than only toy energy objectives? | Pre-registered synthetic cognitive tasks pass matched-baseline criteria before LLM-agent claims are allowed. |
| **FL-9** | **Physical/hardware correspondence** | ⚪ Deferred | Do successful software operators map meaningfully onto published magnetic/spintronic dynamics or hardware? | Correspondence demonstrated operator-by-operator; mathematical resemblance alone is insufficient. |

### Direction of travel

```text
FL-0  mathematical correctness
  ↓
FL-1  attractor memory
  ↓
FL-2  signed competition / frustration
  ↓
FL-3  hysteresis / temporal persistence
  ↓
FL-4  CCOS field mapping
  ↓
FL-5  resonance / noise-assisted exploration
  ↓
FL-6  scalable coupling structures
  ↓
FL-7  learned field parameters
  ↓
FL-8  bounded cognitive computation
  ↓
FL-9  optional physical correspondence
```

Negative results do not break the roadmap. They constrain the next series and remain part of the evidence record.

## Current bootstrap: FL-0

FL-0 implements the smallest falsifiable kernel before any learned model, GPU optimization or active CCOS integration:

- `field-core` — unit/bounded node states, signed coupling graph, external fields, energy and effective-field evaluation;
- `field-dynamics` — projected deterministic dynamics, explicit Euler and Heun reference integration;
- `field-bench` — reproducible FL-0 executable and machine-readable result output;
- `prereg/FL-0.md` — frozen scope, controls and acceptance criteria;
- GitHub CI — format, lint, tests and execution of the FL-0 reference experiment.

The initial energy model is deliberately narrow:

```text
E(M, x) = -Σ_i h_i(x)·m_i - Σ_(i,j) J_ij m_i·m_j
```

with effective field

```text
H_i_eff = -∂E/∂m_i
```

and high-dimensional norm-preserving dissipative motion approximated by

```text
ṁ_i = η (I - m_i m_iᵀ) H_i_eff
```

Later anisotropy, explicit repulsion terms, hysteresis, rotational/non-conservative dynamics and stochastic forcing must be added as independently ablatable operators rather than hidden inside one monolithic update rule.

## Relationship to the Memorithm ecosystem

**CCOS-Core remains stable.** FieldLab may consume and replay CCOS traces in FL-4, but exploratory FieldLab code must not silently change CCOS-Core semantics.

**TDI supplies evidence discipline.** Mature architectural or information-theoretic claims should be promoted to TDI only after FieldLab exploratory protocols stabilize.

**NoiseLab supplies perturbation methodology.** FL-5 should reuse compatible seeded noise/process definitions and resonance controls rather than duplicate them.

**SciRust supplies reusable mathematics.** General-purpose graph, ODE/SDE, spectral, sparse/low-rank and deterministic simulation primitives that mature in FieldLab should move upstream rather than become permanent duplicate infrastructure.

## Reproduce FL-0

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p field-bench
```

The executable exits non-zero if an FL-0 acceptance criterion fails.

## Non-claims

FieldLab does **not** currently claim a new law of cognition, biological equivalence, physical magnetism in brains, AGI, universal superiority over Transformers, `O(N)` scaling, hardware speedup, or a working spintronic implementation. Those would each require separate evidence.

## License

FieldLab uses the same project license as SciRust: **PolyForm Noncommercial License 1.0.0**. See [`LICENSE.md`](LICENSE.md).

Required Notice: Copyright 2026 Tarek Zekriti (https://github.com/Memorithm/)
