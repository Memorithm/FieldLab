# FL-4C — External CCOS runtime replay/provenance gate

Status: **frozen before result inspection**.

## Question

Can FieldLab consume a machine-generated measurement produced by the actual pinned CCOS runtime on a non-FieldLab source corpus, preserve its provenance exactly, and reproduce the measurement deterministically before any later field operator is evaluated on external CCOS data?

FL-4C is a **bridge/replay gate**, not an improvement experiment. It deliberately does not apply FL-4B hysteresis yet. Its role is to prevent later FL-4 experiments from silently replacing CCOS behavior with a FieldLab reconstruction or a hand-authored pseudo-trace.

## External source of truth

The external runtime and workload are fixed to:

- repository: `Memorithm/CCOS-Core`;
- commit: `a3c4d7e03744430c74dc337463ff3e944b4933ad`;
- runtime: the `ccos` binary built from that commit;
- measurement program: `scripts/ccos_campaign_probe.py` from that same commit;
- measured corpus: that checkout's real top-level `src/*.rs` files;
- recall budget: `2048` tokens;
- failure propagation depth: `3`.

FieldLab must not copy or reimplement the CCOS scoring, failure propagation, region construction, or recall ranking for this gate. The CCOS process itself produces the raw evidence through its MCP interface.

## Anchor rule

No anchor name is selected after seeing recall results.

The unmodified CCOS campaign probe chooses anchors by its own deterministic rule when `--anchors` is omitted:

1. parse direct flat-file `use crate::...` dependencies from each top-level `src/*.rs`;
2. retain files with at least two such dependencies;
3. sort candidates by `(direct_dependency_count, uri)` descending;
4. keep the first five.

This rule is part of the pinned external script and therefore part of FL-4C provenance.

## Repetition protocol

The complete CCOS probe is executed **twice**, each time allowing the pinned script to create fresh per-anchor workspaces and ingest the same pinned source tree.

The two raw JSON reports are retained independently as `probe-a.json` and `probe-b.json`.

Replay equality is strict semantic JSON equality after parsing. No numeric tolerance, field deletion, anchor reordering, or post-hoc normalization is permitted. The `crate_src` string is identical by construction because both runs use the same checkout path.

## Imported fields

FieldLab validates and reports, without changing values:

- `files`;
- `all_src_tokens`;
- `budget`;
- `depth`;
- `duplication_factor`;
- the ordered anchor map;
- per-anchor direct dependencies;
- per-anchor dependencies present in the returned window;
- per-anchor `affected` count;
- per-anchor `window_tokens`;
- per-anchor `% all-src`;
- per-anchor noise files.

The FieldLab result also records the pinned CCOS commit and SHA-256 hashes of both raw reports.

## Protocol validity

FL-4C fails closed when any of the following is false:

- the checked-out CCOS commit equals the preregistered commit exactly;
- both CCOS probe processes exit successfully;
- both reports parse as JSON objects;
- semantic replay equality holds exactly;
- `files > 0` and `all_src_tokens > 0`;
- the report states `budget = 2048` and `depth = 3`;
- at least one anchor is present;
- every anchor has at least two declared direct dependencies (as required by the external selector);
- every `window_tokens` value is `<= 2048`;
- all numeric report values used by FieldLab are finite and non-negative where applicable.

These are evidence-integrity requirements only.

## Descriptive / scientific hypotheses

The following do **not** control process exit. A false value is retained as a valid negative result.

- **H4-C1 — direct-dependency coverage:** every selected external anchor has all of its declared direct dependencies represented in the 2048-token CCOS window.
- **H4-C2 — nontrivial causal pressure:** every selected anchor reports `affected > 1` after failure signalling.
- **H4-C3 — bounded external recall:** every selected anchor's returned window consumes less than the entire unique `src/` token count.
- **H4-C4 — low-noise majority:** for a strict majority of selected anchors, the number of noise files is no greater than the number of covered direct dependencies.
- **H4-C5 — deterministic external replay:** the two complete reports are semantically identical.

## Interpretation boundary

A positive FL-4C gate establishes only that FieldLab can acquire and verify reproducible evidence from the actual pinned CCOS runtime on CCOS's real source tree. It does not establish that FL-4B hysteresis improves real CCOS sessions, that the probe workload is organic agent usage, or that FieldLab should modify CCOS-Core.

If FL-4C is valid, the next experiment may use the retained native score/window snapshots as externally grounded inputs for an operator comparison. If the pinned probe does not expose enough temporal information for that comparison, the correct next step is to add a separate trace-acquisition gate rather than fabricate a session history.

## Stop condition

FL-4C stops when both external probe runs, provenance checks, FieldLab import validation, machine-readable evidence, hypotheses, and all existing FieldLab regressions have completed on the same PR head.