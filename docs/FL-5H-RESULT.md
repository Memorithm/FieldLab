# FL-5H result — identifiable E1 residence-aware stochastic gate

Status: **qualified negative on escape efficacy; protocol valid**.

This result records the executable campaign from [`prereg/FL-5H.md`](../prereg/FL-5H.md).
It does not authorize FL-6.

## Provenance

- retained machine report: [`results/FL-5H-identifiable-e1-residence-stochastic-gate.json`](../results/FL-5H-identifiable-e1-residence-stochastic-gate.json);
- report SHA-256 `c26229ad84e02bafe5311202743ed897afc9e2c411f442ba334a0027bd25e941`;
- preregistration commit `ea9bb6e88c90f1978bb1757a8393c3598c7a0db6`;
- NoiseLab pin `8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e` (compatible defs; no code copy);
- energy: E1 axial anisotropy `a = 1.25` (FL-5G stiffness); horizon `256` steps (`T = 12.8`) with left `Complete` / right `ObservationCut`.

## Identifiability

The frozen panel drops the FL-5G conflict geometries (full `P1`/`P2` cues labeled as target `0`) and retains four unambiguous single-bit `P0` corruptions plus six exact cleans (`fl5h|…` namespace only).

Sign-input audit: **10** records, **0** conflicting groups, deterministic ceiling **10/10**. H5H-3 supported. Renaming alone was not used to invent identity.

## Calibration selection

Hard clean-cue filter on calibration retained candidates; lexicographic selection chose:

- OU amplitude `0.5`, theta `0.5`.

Calibration (selected OU): target recovery `0.1875`, left-wrong-basin `1.0`, clean recall `1.0` (matched D0 clean), mean uncensored first-passage `190`, right-censor fraction `0.8125`.

Calibration D0: target recovery `0`, clean recall `1.0`, right-censor `1.0`, mean transitions `0`.

## Untouched holdout

| Arm | Target recovery | Left wrong basin | Clean recall | Right-censor (no target) | Mean transitions |
| --- | ---: | ---: | ---: | ---: | ---: |
| D0 | 0.0 | 1.0 | 1.0 | 1.0 | 0.0 |
| Selected OU | 0.0 | 1.0 | 0.583 | 0.9375 | 0.825 |
| PERM-OU | 0.0 | 1.0 | 1.0 | 1.0 | (matched control) |

Selected OU did **not** improve holdout target recovery over D0 (both `0`). Clean recall degraded on holdout relative to D0 (`1.0 → 0.583`). PERM-OU also recovered `0` targets while preserving clean recall `1.0`.

Complete-left residence fields were recorded for every trajectory: under D0, wrong-basin cues remain in their initial decoded segment through the cut (no leave-initial event; full right-censor on target hits). Under selected OU, some leave-initial events appear (mean first leave ≈ `142.8` among uncensored leaves) without producing holdout target hits.

## Hypotheses

- **H5H-0 protocol validity:** supported.
- **H5H-1 safe escape:** **not supported**.
- **H5H-2 temporal structure:** **not supported**.
- **H5H-3 identifiable inputs:** supported.
- **H5H-4 residence evidence:** supported.
- **H5H-5 reproducibility:** supported (exact campaign replay).

## Interpretation

FL-5H shows that an identifiable panel plus Complete-left residence metrics can be executed under E1 `a = 1.25` without protocol failure. On this fixture and horizon, matched D0 never recovers wrong-basin targets (full right-censor), and calibration-selected OU that produces partial calibration recovery does not transfer to holdout recovery while harming holdout clean recall. The E1 axial-stability negative anticipated in the preregistration is therefore retained rather than patched by silent model switching.

No biology, hardware, LLM, survival-model, or FL-6 claim follows.
