# FieldLab — Cognitive Field Dynamics Laboratory

**FieldLab is Memorithm's research bench for testing whether field dynamics can serve as useful computational primitives for cognition, memory, retrieval and adaptive inference.**

The project begins from magnetic-field-inspired mathematics — attraction, repulsion, orientation, energy landscapes, hysteresis, resonance, metastability and collective transitions — without claiming that cognitive state is physically magnetic. Physical spintronic or skyrmion correspondence is a later research question, not an assumption.

> **Scientific rule:** every claimed benefit must survive a declared baseline, matched resource budget where applicable, reproducible protocol and explicit failure criterion. A lower energy is not automatically better cognition; a stable attractor is not automatically a correct answer.

## FL research series — primary roadmap

The **FL series is the authoritative line of development and experimentation for FieldLab**. Work advances this sequence rather than accumulating unrelated features. Negative results remain part of the evidence record and constrain the next series.

| Series | Research line | Status | Primary question | Exit criterion |
| --- | --- | --- | --- | --- |
| **FL-0** | **Mathematical sanity & deterministic field kernel** | ✅ Completed | Do state, energy, effective-field and integration primitives obey their declared invariants? | Tiny reference cases pass; deterministic replay holds; invalid states fail closed. |
| **FL-1** | **Associative recall & attractor memory** | ✅ Completed | Can field attractors recover corrupted memories, and how do they compare with retrieval baselines? | Exhaustive 7,551-case corruption campaign retained with reproducible evidence. |
| **FL-2** | **Competing hypotheses, signed coupling & frustration** | ✅ Completed | Do attraction and repulsion help resolve controlled contradictory evidence rather than merely creating instability? | Matched competition controls and analytic three-way frustration reference executed. |
| **FL-3** | **Hysteresis & context switching** | ✅ Completed | Can path dependence retain useful cognitive state without unacceptable lock-in? | Retention benefit and switching cost characterized under a frozen context-switch protocol. |
| **FL-4** | **CCOS field mapping** | 🟠 Next | Is CCOS causal pressure/heat usefully representable as a discrete field, and do extra field operators improve bounded context selection? | Fixed CCOS traces replayed; current CCOS vs field variants compared at identical budget while preserving auditability. |
| **FL-5** | **Resonance, perturbation & stochastic exploration** | ⚪ Planned | Can controlled perturbation improve basin escape, recall or ambiguity resolution? | Benefit exceeds matched no-noise and surrogate controls under the same compute envelope. |
| **FL-6** | **Sparse, low-rank & multiscale field scaling** | ⚪ Planned | Can useful dense interaction be retained without assuming scalable `O(N²)` coupling? | Approximation error and interaction/runtime scaling reported separately against a dense reference. |
| **FL-7** | **Learned fields & adaptive couplings** | ⚪ Planned | Which field parameters can be learned without obscuring reference semantics or provenance? | Explicit train/validation/test split; learned model beats fixed-rule baselines and remains inspectable. |
| **FL-8** | **Bounded cognitive tasks** | ⚪ Planned | Do validated field primitives improve memory/reasoning tasks rather than only toy energy objectives? | Pre-registered synthetic cognitive tasks pass matched-baseline criteria before LLM-agent claims are allowed. |
| **FL-9** | **Physical/hardware correspondence** | ⚪ Deferred | Do successful software operators map meaningfully onto published magnetic/spintronic dynamics or hardware? | Correspondence demonstrated operator-by-operator; mathematical resemblance alone is insufficient. |

### Direction of travel

```text
FL-0  mathematical correctness                 ✅
  ↓
FL-1  attractor memory                         ✅
  ↓
FL-2  signed competition / frustration         ✅
  ↓
FL-3  hysteresis / temporal persistence        ✅
  ↓
FL-4  CCOS field mapping                       ← NEXT
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

## Latest completed result: FL-3 hysteresis and context switching

FL-3 introduced an explicit two-threshold relay/hysteron as an ablatable field bias. The protocol was frozen before result inspection. The deterministic fixture contains 80 observations, 3 genuine context transitions and 8 isolated contradictory pulses.

| Condition | Total errors | Contradiction errors | False context changes | Mean switch latency | Lock-in events |
| --- | ---: | ---: | ---: | ---: | ---: |
| Memoryless | 8 | 8 | 16 | 0 | 0 |
| relay 0.10 | 8 | 8 | 16 | 0 | 0 |
| relay 0.20 | 11 | 8 | 16 | 1 | 0 |
| **relay 0.30** | **6** | **0** | **0** | 2 | 0 |
| relay 0.40 | 9 | 0 | 0 | 3 | 0 |
| relay 0.50 | 9 | 0 | 0 | 3 | 0 |
| relay 0.60 | 9 | 0 | 0 | 3 | 0 |

Within this fixture, threshold `0.30` provides the best preregistered total-error trade-off: it rejects all eight contradictory pulses and removes the 16 false context changes of the memoryless response while adding two observations of switching latency. Higher thresholds retain disturbance rejection but increase transition cost.

Three preregistered hypotheses are supported in this fixture: useful retention, disturbance rejection and a positive switching cost. The lock-in-frontier hypothesis is **not supported**: no tested threshold exceeds the preregistered `>3` observation lock-in boundary. This negative result is retained unchanged.

See [`prereg/FL-3.md`](prereg/FL-3.md), [`docs/FL-3-RESULT.md`](docs/FL-3-RESULT.md), and [`results/FL-3-hysteresis-context.json`](results/FL-3-hysteresis-context.json).

## Earlier results

### FL-2 — signed competition and frustration

All matched coupling conditions selected the evidence-favoured hypothesis correctly in every non-tie case. Repulsion therefore showed no accuracy advantage in that fixture, but increased mean useful separation from `0.584645449903` (uncoupled) to `0.829983940213`. The repulsive three-node triangle converged to the analytic 120° frustrated compromise with final energy `-1.5` and pairwise dot-product error below `2.8e-7`.

See [`docs/FL-2-RESULT.md`](docs/FL-2-RESULT.md).

### FL-1 — associative recall

Across 7,551 deterministic corruptions: continuous field dynamics recovered `6,723` cases (`89.0345650%`), matched asynchronous Hopfield recovered `6,303` (`83.4723878%`), and nearest-template retrieval recovered `7,342` (`97.2321547%`). The field method therefore beat the shared-coupling Hopfield baseline but not direct template access.

See [`docs/FL-1-RESULT.md`](docs/FL-1-RESULT.md).

## Foundation

- `field-core` — unit node states, signed coupling graph, external fields, energy and effective-field evaluation;
- `field-dynamics` — projected deterministic dynamics, explicit Euler and Heun reference integration;
- `field-memory` — bipolar memory banks, shared Hebbian coupling, cue encoding/decoding and retrieval baselines;
- `field-hysteresis` — explicit deterministic two-threshold relay/hysteron;
- `field-bench` — machine-readable FL experiment executables.

The current field energy remains deliberately narrow:

```text
E(M, x) = -Σ_i h_i(x)·m_i - Σ_(i,j) J_ij m_i·m_j
H_i_eff = -∂E/∂m_i
ṁ_i = η (I - m_i m_iᵀ) H_i_eff
```

Hysteresis is kept as an explicit operator rather than hidden in numerical inertia. Rotational dynamics and stochastic forcing remain separate later mechanisms.

## Relationship to the Memorithm ecosystem

**CCOS-Core remains stable.** FL-4 may consume and replay CCOS traces, but exploratory FieldLab code must not silently change CCOS-Core semantics.

**TDI supplies evidence discipline.** Mature architectural or information-theoretic claims should be promoted to TDI only after FieldLab exploratory protocols stabilize.

**NoiseLab supplies perturbation methodology.** FL-5 should reuse compatible seeded noise/process definitions and resonance controls rather than duplicate them.

**SciRust supplies reusable mathematics.** General-purpose graph, ODE/SDE, spectral, sparse/low-rank and deterministic simulation primitives that mature in FieldLab should move upstream rather than become permanent duplicate infrastructure.

## Reproduce experiments

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p field-bench --bin field-bench
cargo run -p field-bench --bin fl1
cargo run -p field-bench --bin fl2
cargo run -p field-bench --bin fl3
```

A comparative hypothesis may fail while its experiment remains scientifically valid. CI failures are reserved for invalid execution, broken invariants, failed reference gates or missing reproducibility evidence.

## Non-claims

FieldLab does **not** currently claim a new law of cognition, biological equivalence, physical magnetism in brains, AGI, universal superiority over Transformers, `O(N)` scaling, hardware speedup, or a working spintronic implementation. Those would each require separate evidence.

## License

FieldLab uses the same project license as SciRust: **PolyForm Noncommercial License 1.0.0**. See [`LICENSE.md`](LICENSE.md).

Required Notice: Copyright 2026 Tarek Zekriti (https://github.com/Memorithm/)
