# FieldLab — Cognitive Field Dynamics Laboratory

**FieldLab is Memorithm's research bench for testing whether field dynamics can serve as useful computational primitives for cognition, memory, retrieval and adaptive inference.**

The project begins from magnetic-field-inspired mathematics — attraction, repulsion, orientation, energy landscapes, hysteresis, resonance, metastability and collective transitions — without claiming that cognitive state is physically magnetic. Physical spintronic or skyrmion correspondence is a later research question, not an assumption.

> **Scientific rule:** every claimed benefit must survive a declared baseline, matched resource budget where applicable, reproducible protocol and explicit failure criterion. A lower energy is not automatically better cognition; a stable attractor is not automatically a correct answer.

## FL research series — primary roadmap

The **FL series is the authoritative line of development and experimentation for FieldLab**. Work advances this sequence rather than accumulating unrelated features. Negative results remain part of the evidence record and constrain the next series.

| Series | Research line | Status | Primary question | Exit criterion |
| --- | --- | --- | --- | --- |
| **FL-0** | **Mathematical sanity & deterministic field kernel** | ✅ Completed | Do state, energy, effective-field and integration primitives obey their declared invariants? | Tiny reference cases pass; deterministic replay holds; invalid states fail closed; CI executes the reference experiment. |
| **FL-1** | **Associative recall & attractor memory** | ✅ Completed | Can field attractors recover corrupted memories, and how do they compare with Hopfield and nearest-template retrieval? | Exhaustive 7,551-case campaign executed and retained with reproducible comparative evidence. |
| **FL-2** | **Competing hypotheses, signed coupling & frustration** | ✅ Completed | Do attraction and repulsion help resolve controlled contradictory evidence rather than merely creating instability? | Matched competition controls and analytic three-way frustration reference executed and retained. |
| **FL-3** | **Hysteresis & context switching** | ⚪ Next | Can path dependence retain useful cognitive state without unacceptable lock-in? | Retention benefit and switching cost jointly characterized; lock-in frontier measured. |
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
FL-1  attractor memory                         ✅
  ↓
FL-2  signed competition / frustration         ✅
  ↓
FL-3  hysteresis / temporal persistence        ← NEXT
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

## Latest completed result: FL-2 signed competition and frustration

FL-2 tested three matched two-hypothesis coupling conditions over 84 evidence cases each. All three conditions selected the evidence-favoured hypothesis correctly in every non-tie case, so **no accuracy advantage** was observed for repulsion. Its effect was instead a larger separation margin:

| Condition | Winner rate | Mean useful separation | Max tie polarization |
| --- | ---: | ---: | ---: |
| Repulsive `J=-0.5` | 100% | **0.829983940213** | 0 |
| Uncoupled `J=0` | 100% | 0.584645449903 | 0 |
| Attractive `J=+0.5` | 100% | 0.265158394256 | 0 |

The repulsive three-node triangle also converged to the analytic frustrated compromise: final energy `-1.5`, pairwise dot products within `2.78254e-7` of `-0.5`, and deterministic replay. The attractive control converged to full alignment and energy `-3.0`.

The bounded conclusion is **margin amplification plus stable frustrated compromise**, not improved reasoning accuracy. See [`prereg/FL-2.md`](prereg/FL-2.md), [`docs/FL-2-RESULT.md`](docs/FL-2-RESULT.md), and [`results/FL-2-signed-competition-frustration.json`](results/FL-2-signed-competition-frustration.json).

## Previous result: FL-1 associative recall

FL-1 executed every zero-to-four-bit corruption of three fixed orthogonal 16-symbol bipolar memories: **7,551 deterministic cases**.

| Method | Exact recoveries | Aggregate rate |
| --- | ---: | ---: |
| Continuous field dynamics | **6,723 / 7,551** | **89.0345650%** |
| Deterministic asynchronous Hopfield | 6,303 / 7,551 | 83.4723878% |
| Nearest-template retrieval | 7,342 / 7,551 | **97.2321547%** |

The narrow result is positive against the shared-coupling Hopfield baseline and negative against direct nearest-template retrieval. See [`docs/FL-1-RESULT.md`](docs/FL-1-RESULT.md).

## Foundation

- `field-core` — unit node states, signed coupling graph, external fields, energy and effective-field evaluation;
- `field-dynamics` — projected deterministic dynamics, explicit Euler and Heun reference integration;
- `field-memory` — bipolar memory banks, shared Hebbian coupling, cue encoding/decoding and declared retrieval baselines;
- `field-bench` — machine-readable FL experiment executables.

The current energy model remains deliberately narrow:

```text
E(M, x) = -Σ_i h_i(x)·m_i - Σ_(i,j) J_ij m_i·m_j
H_i_eff = -∂E/∂m_i
ṁ_i = η (I - m_i m_iᵀ) H_i_eff
```

FL-3 is reserved for explicit path dependence and hysteresis. Rotational dynamics and stochastic forcing remain separate later mechanisms.

## Relationship to the Memorithm ecosystem

**CCOS-Core remains stable.** FieldLab may consume and replay CCOS traces in FL-4, but exploratory FieldLab code must not silently change CCOS-Core semantics.

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
```

A comparative hypothesis may fail while its experiment remains scientifically valid. CI failures are reserved for invalid execution, broken invariants, failed reference gates or missing reproducibility evidence.

## Non-claims

FieldLab does **not** currently claim a new law of cognition, biological equivalence, physical magnetism in brains, AGI, universal superiority over Transformers, `O(N)` scaling, hardware speedup, or a working spintronic implementation. Those would each require separate evidence.

## License

FieldLab uses the same project license as SciRust: **PolyForm Noncommercial License 1.0.0**. See [`LICENSE.md`](LICENSE.md).

Required Notice: Copyright 2026 Tarek Zekriti (https://github.com/Memorithm/)
