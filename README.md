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
| **FL-4** | **CCOS field mapping** | ✅ First external comparative gate completed through FL-4E | Is CCOS causal pressure/heat usefully representable as a discrete field, and do extra field operators improve bounded context selection? | Fixed or externally grounded CCOS traces replayed; native CCOS vs field variants compared at identical budget while preserving auditability. |
| **FL-5** | **Resonance, perturbation & stochastic exploration** | ✅ First gate + FL-5B + FL-5C + FL-5D shallow-boundary gate | Can controlled perturbation improve basin escape, recall or ambiguity resolution under clean-cue safety? | Benefit exceeds matched no-noise and surrogate controls under the same compute envelope; clean-cue and structured-noise controls reported. |
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
FL-4  CCOS field mapping                       ✅ FL-4A/4B/4C/4D/4E validated; wider external replication remains optional
  ↓
FL-5  resonance / noise-assisted exploration   ✅ first gate + FL-5B + FL-5C + FL-5D
  ↓
FL-6  scalable coupling structures
  ↓
FL-7  learned field parameters
  ↓
FL-8  bounded cognitive computation
  ↓
FL-9  optional physical correspondence
```

## Latest completed result: FL-5D shallow-boundary clean-cue-constrained escape

FL-5D preregistered a fourth FL-5 gate that changes the **dynamical regime**: a correlated shallow-boundary bank (`H=2/2/4`) with frozen eligibility `D0 ends in competitor AND target_basin_margin >= -0.40`, hard clean-cue filter, FL-5B/C amplitude grid, and new `fl5d|` namespace with partition-balanced clean IDs (3/3). Shallow construction succeeded (6 eligible; 2 cal / 4 holdout); 8 deep-locked terminals were reported separately. Calibration discarded the only substantial-recovery OU cell (`amp 0.5`, `θ 0.5`, recovery `0.375`) for clean-cue degradation and selected OU amplitude `0.05` / θ `2.0` among zero-degradation survivors. On the untouched FL-5D holdout, D0, selected OU and PERM-OU all recovered `0` targets; holdout clean recall stayed `1.0`. H5D-0, H5D-3 and H5D-4 were supported; H5D-1 and H5D-2 were not. No biology, hardware, LLM or FL-6 claim is made.

See [`docs/FL-5D-RESULT.md`](docs/FL-5D-RESULT.md), [`prereg/FL-5D.md`](prereg/FL-5D.md), and [`results/FL-5D-shallow-boundary-clean-cue-constrained-escape.json`](results/FL-5D-shallow-boundary-clean-cue-constrained-escape.json).

## Earlier completed result: FL-5C nonempty-holdout clean-cue basin escape

FL-5C preregistered a third FL-5 gate that keeps the hard clean-cue calibration filter and FL-5B amplitude grid, but freezes a new `fl5c|` namespace and a partition-balanced clean-cue identifier list (≥2 clean cases on each SHA-256 side; here 3/3) so H5C-3 is not vacuous. FL-5 / FL-5B recovery numbers were not used for tuning. Calibration discarded the only non-zero-recovery OU cell (`amp 0.5`, `θ 0.5`) for clean-cue degradation and selected OU amplitude `0.05` / θ `8.0` among zero-degradation survivors. On the untouched FL-5C holdout, D0, selected OU and PERM-OU all recovered `0` targets; holdout clean recall stayed `1.0` (degradation `0`). H5C-0, H5C-3 and H5C-4 were supported; H5C-1 and H5C-2 were not. No biology, hardware, LLM or FL-6 claim is made.

See [`docs/FL-5C-RESULT.md`](docs/FL-5C-RESULT.md), [`prereg/FL-5C.md`](prereg/FL-5C.md), and [`results/FL-5C-nonempty-holdout-clean-cue-basin-escape.json`](results/FL-5C-nonempty-holdout-clean-cue-basin-escape.json).

## Earlier completed result: FL-5B clean-cue-constrained basin escape

FL-5B preregistered a second FL-5 gate with a hard clean-cue calibration filter and a strictly smaller amplitude grid (`0, 0.05, 0.1, 0.25, 0.5`), under a new `fl5b|` case-id namespace so the SHA-256 partition cannot leak the FL-5 holdout. FL-5 holdout outcomes were not used for tuning. Calibration discarded the only non-zero-recovery OU cell (`amp 0.5`, `θ 0.5`) for clean-cue degradation and selected Gaussian amplitude `0.5` among zero-degradation survivors. On the untouched FL-5B holdout, D0, selected Gaussian and PERM-G all recovered `0` targets. H5B-0, H5B-2 (Gaussian caveat), H5B-3 (vacuous empty holdout clean panel) and H5B-4 were supported; H5B-1 was not. No biology, hardware, LLM or FL-6 claim is made.

See [`docs/FL-5B-RESULT.md`](docs/FL-5B-RESULT.md), [`prereg/FL-5B.md`](prereg/FL-5B.md), and [`results/FL-5B-clean-cue-constrained-basin-escape.json`](results/FL-5B-clean-cue-constrained-basin-escape.json).

## Earlier completed result: FL-5 noise-assisted basin escape

FL-5 executed the first noise-assisted basin-escape gate on a frozen 8-node / 3-pattern FL-1-style Hebbian bank. Calibration (holdout-blind) selected OU amplitude `1.5` with θ `2.0`. On untouched holdout wrong-basin cases, D0 recovered `0` targets while the selected OU arm recovered `≈ 2.68%`; the PERM-OU surrogate recovered `≈ 3.57%`. Clean-cue holdout recall fell from `1.0` under D0 to `0.25` under the selected arm, so H5-2 failed. H5-1 and H5-4 were supported; H5-0, H5-2 and H5-3 were not. No biology, hardware or LLM claim is made.

See [`docs/FL-5-RESULT.md`](docs/FL-5-RESULT.md). Prior FL-5* holdouts remain unused for FL-5D tuning.

## Earlier completed result: FL-4E external native-window focus hysteresis

FL-4E executed the first external comparative focus-ablation gate against the pinned `Memorithm/CCOS-Core@a3c4d7e03744430c74dc337463ff3e944b4933ad` runtime. Calibration and untouched holdout remained separate, the native working-set budget stayed fixed at 2,048 tokens, and two complete acquisitions were replay-identical.

The frozen calibration grid selected `theta = 0.01` using calibration only. On the untouched holdout, the memoryless baseline recorded 25 truth errors, 3 false switches and 4 switches; the hysteretic focus annotation recorded 22 truth errors, 1 false switch and 1 switch. Both arms had maximum observed transition latency 8 and maximum token count 2,042, within the 2,048-token budget.

All five preregistered FL-4E hypotheses were reported as supported for this declared workload. This result is scoped to the pinned runtime, corpus, schedules, budget and protocol. It does **not** establish downstream LLM-task improvement, universal cognitive advantage or physical magnetic equivalence, and it does not alter CCOS working-set membership.

See [`docs/FL-4E-RESULT.md`](docs/FL-4E-RESULT.md). FL-4E holdout remains unused for FL-5 tuning.

## Earlier results

### FL-4D — native temporal CCOS trace acquisition

FL-4D acquired time-ordered scored working-set snapshots from the actual pinned `Memorithm/CCOS-Core@a3c4d7e03744430c74dc337463ff3e944b4933ad` runtime on CCOS's real top-level `src/*.rs` corpus. The 24-observation calibration and 24-observation holdout schedules were frozen before execution.

Two complete acquisitions from fresh workspaces were semantically identical and shared the same raw SHA-256 (`4eabc57a1ba9147591d5b4ca63b62a0eaa25d4549d24bc4336bba061e2cfafe4`). Calibration produced 23 distinct snapshots out of 24 and holdout produced 24/24 distinct snapshots. Every native working-set recall stayed below the 2,048-token hard budget.

All five FL-4D hypotheses are supported: exact temporal replay, stimulus sensitivity, hard-budget preservation, finite native score availability and representation of all four frozen anchors.

See [`prereg/FL-4D.md`](prereg/FL-4D.md), [`docs/FL-4D-RESULT.md`](docs/FL-4D-RESULT.md), and [`results/FL-4D-native-temporal-ccos.json`](results/FL-4D-native-temporal-ccos.json).

### FL-4C — external CCOS runtime replay gate

FL-4C moved the evidence source outside FieldLab-authored fixtures. The workflow compiled the actual pinned CCOS runtime, executed CCOS's own campaign probe twice on its real source corpus, and imported the resulting JSON without reconstructing CCOS scoring inside FieldLab.

Both runs were semantically identical with SHA-256 `90d716fa04f029f1b9d8065a91d045075d46ef13938365103c87ffe5acba76f2`. The corpus contained 55 files and 471,024 unique source tokens. Every returned window respected 2,048 tokens and consumed about 0.43% of the corpus.

Four of five preregistered hypotheses were supported. H4-C1 was **not** supported: full direct-dependency coverage held for 3/5 selected anchors, while `external_memory.rs` retained 5/11 direct dependencies and `agent_session.rs` 4/6 at the fixed budget. This negative result remains part of the evidence record.

See [`prereg/FL-4C.md`](prereg/FL-4C.md), [`docs/FL-4C-RESULT.md`](docs/FL-4C-RESULT.md), and [`results/FL-4C-external-ccos.json`](results/FL-4C-external-ccos.json).

### FL-4B — hysteretic bounded working-set selection

FL-4A first established an exact semantic bridge to pinned CCOS scoring, failure propagation, working-set assembly and Q-Page evidence. FL-4B then added one explicit FL-3 relay operator and tested it under the same 48-token budget on separate calibration and holdout traces.

The calibration-only procedure selected `theta = 0.10`. On the untouched holdout trace:

| Condition | Missed relevant slots | Exact sets | Replacements | Max transition latency | Max tokens |
| --- | ---: | ---: | ---: | ---: | ---: |
| Pinned CCOS baseline | 12 | 52 / 64 | 32 | 0 | 48 |
| **FL-4B relay** | **0** | **64 / 64** | **8** | **0** | **48** |

All five FL-4B hypotheses are supported on this controlled fixture, deterministic replay is exact, and the identical budget is preserved. This is not yet a claim about real repositories or arbitrary CCOS workloads.

See [`prereg/FL-4.md`](prereg/FL-4.md), [`prereg/FL-4B.md`](prereg/FL-4B.md), [`docs/FL-4B-RESULT.md`](docs/FL-4B-RESULT.md), and [`results/FL-4B-hysteretic-working-set.json`](results/FL-4B-hysteretic-working-set.json).

### FL-4A — lossless CCOS semantic field mapping

On the fixed FL-4A fixture, the independent field representation matched the pinned CCOS reference exactly: score error `0`, propagated-pressure error `0`, Q-Page error `0`, identical ordered working-set selection, identical 92-token use and deterministic replay. FL-4A is an equivalence gate, not an improvement claim.

### FL-3 — hysteresis and context switching

FL-3 introduced an explicit two-threshold relay/hysteron as an ablatable field bias. The frozen deterministic fixture contained 80 observations, 3 genuine context transitions and 8 isolated contradictory pulses. Threshold `0.30` rejected all eight contradictory pulses and removed the 16 false context changes of the memoryless response while adding two observations of switching latency. The lock-in-frontier hypothesis was not supported and remains recorded as a negative result.

See [`prereg/FL-3.md`](prereg/FL-3.md), [`docs/FL-3-RESULT.md`](docs/FL-3-RESULT.md), and [`results/FL-3-hysteresis-context.json`](results/FL-3-hysteresis-context.json).

### FL-2 — signed competition and frustration

All matched coupling conditions selected the evidence-favoured hypothesis correctly in every non-tie case. Repulsion therefore showed no accuracy advantage in that fixture, but increased mean useful separation from `0.584645449903` (uncoupled) to `0.829983940213`. The repulsive three-node triangle converged to the analytic 120° frustrated compromise with final energy `-1.5` and pairwise dot-product error below `2.8e-7`.

See [`docs/FL-2-RESULT.md`](docs/FL-2-RESULT.md).

### FL-1 — associative recall

Across 7,551 deterministic corruptions: continuous field dynamics recovered `6,723` cases (`89.0345650%`), matched asynchronous Hopfield recovered `6,303` (`83.4723878%`), and nearest-template retrieval recovered `7,342` (`97.2321547%`). The field method therefore beat the shared-coupling Hopfield baseline but not direct template access.

See [`docs/FL-1-RESULT.md`](docs/FL-1-RESULT.md).

## Foundation

- `field-core` — unit node states, versioned conservative energy models, signed scalar couplings, operator-valued couplings, local anisotropy and effective-field evaluation;
- `field-dynamics` — projected deterministic dynamics, explicit Euler and Heun reference integration over the common `FieldModel` contract;
- `field-memory` — bipolar memory banks, shared Hebbian coupling, cue encoding/decoding and retrieval baselines;
- `field-hysteresis` — explicit deterministic two-threshold relay/hysteron;
- `field-ccos-map` — lossless reference/field mapping for pinned CCOS scoring, pressure propagation, working-set selection and Q-Page evidence;
- `field-bench` — machine-readable FL experiment executables.

### Energy-model versions

The historical model used by FL-0 through FL-3 is now explicitly named **FL-E0**:

```text
E0(M, x) = -Σ_i h_i(x)·m_i -Σ_{ {i,j}∈E } J_ij m_i·m_j
H_i_eff  = -∂E0/∂m_i
ṁ_i     = η (I - m_i m_iᵀ) H_i_eff
```

The pair sum is over **unique undirected edges**. It is not a sum over every ordered matrix index, so no `1/2` factor is required. `CouplingGraph` rejects duplicate pairs, including reversed `(j,i)` duplicates, to make that convention executable rather than implicit.

**FL-E1** is a conservative extension that keeps exactly the same projected dynamics while permitting operator-valued pair interactions and symmetric local anisotropy:

```text
E1(M, x) = -Σ_i h_i(x)·m_i
           -Σ_{ {i,j}∈E } m_iᵀ K_ij m_j
           -1/2 Σ_i m_iᵀ A_i m_i
```

Every E0 model embeds into E1 through `K_ij = J_ij I` and `A_i = 0`. The preregistered FL-E1 validation campaign checks that equivalence before accepting the additional expressivity. FL-E2 then established controlled direct transport and two-edge composition for the tested orthogonal operators. Its strongest scalar-chain insufficiency hypothesis was false on one composition fixture (`best E0 cosine = 0.734158103318`), and that negative result is retained unchanged.

See [`docs/ENERGY-MODELS.md`](docs/ENERGY-MODELS.md), [`prereg/FL-E1.md`](prereg/FL-E1.md), and [`prereg/FL-E2.md`](prereg/FL-E2.md).

Hysteresis remains an explicit operator rather than being hidden in numerical inertia. Rotational/non-conservative dynamics, stochastic forcing and higher-order interactions remain separate later mechanisms so their effects can be ablated independently.

## Relationship to the Memorithm ecosystem

**CCOS-Core remains stable.** FL-4 may consume and replay CCOS traces, but exploratory FieldLab code must not silently change CCOS-Core semantics.

**TDI supplies evidence discipline.** Mature architectural or information-theoretic claims should be promoted to TDI only after FieldLab exploratory protocols stabilize.

**NoiseLab supplies perturbation methodology.** FL-5 / FL-5B / FL-5C / FL-5D record NoiseLab `@8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e` and use compatible seeded Gaussian / OU / temporal-permutation definitions without copying NoiseLab code.

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
cargo run -p field-bench --bin fl4
cargo run -p field-bench --bin fl4b
cargo run -p field-bench --bin fl4c -- <probe-a.json> <probe-b.json> <ccos-commit>
cargo run -p field-bench --bin fl4d -- <trace-a.json> <trace-b.json> <ccos-commit>
cargo run -p field-bench --bin fl5
cargo run -p field-bench --bin fl5b
cargo run -p field-bench --bin fl5c
cargo run -p field-bench --bin fle1
cargo run -p field-bench --bin fle2
```

A comparative hypothesis may fail while its experiment remains scientifically valid. CI failures are reserved for invalid execution, broken invariants, failed reference gates or missing reproducibility evidence.

## Non-claims

FieldLab does **not** currently claim a new law of cognition, biological equivalence, physical magnetism in brains, AGI, universal superiority over Transformers, `O(N)` scaling, hardware speedup, or a working spintronic implementation. Those would each require separate evidence.

## License

FieldLab uses the same project license as SciRust: **PolyForm Noncommercial License 1.0.0**. See [`LICENSE.md`](LICENSE.md).

Required Notice: Copyright 2026 Tarek Zekriti (https://github.com/Memorithm/)
