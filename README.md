# FieldLab — Cognitive Field Dynamics Laboratory

**FieldLab is Memorithm's research bench for testing whether field dynamics can serve as useful computational primitives for cognition, memory, retrieval and adaptive inference.**

The project begins from magnetic-field-inspired mathematics — attraction, repulsion, orientation, energy landscapes, hysteresis, resonance, metastability and collective transitions — without claiming that cognitive state is physically magnetic. Physical spintronic or skyrmion correspondence is a later research question, not an assumption.

> **Scientific rule:** every claimed benefit must survive a declared baseline, matched resource budget where applicable, reproducible protocol and explicit failure criterion. A lower energy is not automatically better cognition; a stable attractor is not automatically a correct answer.

## FL research series — primary roadmap

The **FL series is the authoritative line of development and experimentation for FieldLab**. Work advances this sequence rather than accumulating unrelated features. Negative results remain part of the evidence record and constrain the next series.

| Series | Research line | Status | Primary question | Exit criterion |
| --- | --- | --- | --- | --- |
| **FL-0** | **Mathematical sanity & deterministic field kernel** | ✅ Completed bootstrap | Do state, energy, effective-field and integration primitives obey their declared invariants? | Tiny reference cases pass; deterministic replay holds; invalid states fail closed; CI executes the reference experiment. |
| **FL-1** | **Associative recall & attractor memory** | 🟠 Active | Can field attractors recover corrupted memories, and how do they compare with Hopfield and nearest-template retrieval? | Frozen exhaustive corruption campaign executes with reproducible evidence and explicit comparative outcomes. |
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
FL-0  mathematical correctness                 ✅
  ↓
FL-1  attractor memory                         🟠
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

## Current frontier: FL-1 associative recall

FL-1 introduces the first actual memory task. Three fixed bipolar patterns of width 16 are stored with a shared Hebbian coupling matrix. Every corruption mask containing zero through four flipped bits is enumerated exactly, for **7,551 deterministic cases**.

Three retrieval mechanisms are compared:

- **Field dynamics** — each cue bit becomes a tilted two-dimensional unit vector and evolves for 128 Heun steps under the continuous projected field;
- **Hopfield** — deterministic asynchronous updates using exactly the same Hebbian matrix;
- **Nearest-template** — direct Hamming-distance retrieval with deterministic tie-breaking, retained as a strong template-access reference.

The FL-1 protocol is frozen in [`prereg/FL-1.md`](prereg/FL-1.md). A green experiment means the evidence is complete and reproducible; it does **not** imply that the field method wins.

## FL-0 foundation

FL-0 established the smallest falsifiable field kernel:

- `field-core` — unit node states, signed coupling graph, external fields, energy and effective-field evaluation;
- `field-dynamics` — projected deterministic dynamics, explicit Euler and Heun reference integration;
- `field-bench` — machine-readable experiment executables;
- `prereg/FL-0.md` — frozen mathematical sanity protocol.

Its initial energy model is deliberately narrow:

```text
E(M, x) = -Σ_i h_i(x)·m_i - Σ_(i,j) J_ij m_i·m_j
```

with effective field

```text
H_i_eff = -∂E/∂m_i
```

and norm-preserving dissipative motion approximated by

```text
ṁ_i = η (I - m_i m_iᵀ) H_i_eff
```

FL-1 adds `field-memory`, which owns bipolar pattern validation, Hebbian coupling construction, cue encoding/decoding and declared retrieval baselines. Later anisotropy, explicit repulsion terms, hysteresis, rotational dynamics and stochastic forcing remain separate ablatable operators.

## Relationship to the Memorithm ecosystem

**CCOS-Core remains stable.** FieldLab may consume and replay CCOS traces in FL-4, but exploratory FieldLab code must not silently change CCOS-Core semantics.

**TDI supplies evidence discipline.** Mature architectural or information-theoretic claims should be promoted to TDI only after FieldLab exploratory protocols stabilize.

**NoiseLab supplies perturbation methodology.** FL-5 should reuse compatible seeded noise/process definitions and resonance controls rather than duplicate them.

**SciRust supplies reusable mathematics.** General-purpose graph, ODE/SDE, spectral, sparse/low-rank and deterministic simulation primitives that mature in FieldLab should move upstream rather than become permanent duplicate infrastructure.

## Reproduce the current experiments

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p field-bench --bin field-bench
cargo run -p field-bench --bin fl1
```

FL-0 exits non-zero if a mathematical acceptance criterion fails. FL-1 exits non-zero only when its evidence protocol is structurally invalid; a negative comparative result remains a valid scientific result.

## Non-claims

FieldLab does **not** currently claim a new law of cognition, biological equivalence, physical magnetism in brains, AGI, universal superiority over Transformers, `O(N)` scaling, hardware speedup, or a working spintronic implementation. Those would each require separate evidence.

## License

FieldLab uses the same project license as SciRust: **PolyForm Noncommercial License 1.0.0**. See [`LICENSE.md`](LICENSE.md).

Required Notice: Copyright 2026 Tarek Zekriti (https://github.com/Memorithm/)
