# FL-4C result — external CCOS runtime replay/provenance gate

FL-4C replaced FieldLab-authored CCOS-like fixtures with measurements emitted by the actual pinned CCOS runtime. The protocol was frozen in `prereg/FL-4C.md` before the external result was inspected.

## Result

The gate is valid. Two independent executions of `Memorithm/CCOS-Core@a3c4d7e03744430c74dc337463ff3e944b4933ad` produced semantically identical JSON and identical raw SHA-256 hashes:

`90d716fa04f029f1b9d8065a91d045075d46ef13938365103c87ffe5acba76f2`

The pinned CCOS campaign probe measured the repository's real top-level `src/*.rs` corpus:

| Quantity | Value |
| --- | ---: |
| Source files | 55 |
| Unique source tokens | 471,024 |
| Recall budget | 2,048 |
| Failure depth | 3 |
| Full-region duplication factor | 0.792 |
| Selected anchors | 5 |

Every selected anchor produced nontrivial propagated failure pressure and a bounded recall window of 2,046–2,048 tokens, only about 0.43% of the unique source corpus.

## Direct-dependency coverage

| Anchor | Direct deps | Covered | Affected nodes | Window tokens | Noise files |
| --- | ---: | ---: | ---: | ---: | ---: |
| `src/external_memory.rs` | 11 | 5 | 177 | 2,048 | 2 |
| `src/agent_session.rs` | 6 | 4 | 170 | 2,046 | 1 |
| `src/migrate.rs` | 5 | 5 | 54 | 2,047 | 1 |
| `src/region_metrics.rs` | 4 | 4 | 27 | 2,047 | 1 |
| `src/region_engine.rs` | 4 | 4 | 42 | 2,048 | 1 |

Three of five anchors retained every declared direct dependency. The two highest-degree anchors did not: `external_memory.rs` retained 5/11 and `agent_session.rs` retained 4/6. This is the preregistered negative result H4-C1 and is retained unchanged.

## Hypotheses

- **H4-C1 — direct-dependency coverage:** **not supported**;
- **H4-C2 — nontrivial causal pressure:** supported;
- **H4-C3 — bounded external recall:** supported;
- **H4-C4 — low-noise majority:** supported;
- **H4-C5 — deterministic external replay:** supported.

## Interpretation

FL-4C establishes the evidence bridge FieldLab needed: the actual pinned CCOS runtime can be compiled, executed on CCOS's real source tree, measured twice and imported into FieldLab with exact provenance and deterministic replay.

It also exposes a useful limit. A fixed 2,048-token window cannot simultaneously retain every direct dependency for the two most highly connected selected anchors. This is not hidden as a CI failure because it is a scientific outcome, not a protocol violation. Later field operators must therefore be judged under the same hard budget rather than against an impossible full-coverage assumption.

FL-4C is still not a temporal operator comparison. The external probe emits static anchor measurements. Before applying FL-4B hysteresis to external evidence, FieldLab needs a native temporal CCOS trace whose snapshots retain scores, selected items, token use and exact external provenance.

Machine-readable evidence:

- `results/FL-4C-probe-a.json`;
- `results/FL-4C-probe-b.json`;
- `results/FL-4C-external-ccos.json`.
