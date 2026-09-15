# FL-B1 result — frozen Boolean observability fixture

Status: **qualified fixture result**.

This result records the first executable campaign from [`prereg/FL-B1.md`](../prereg/FL-B1.md). It is an observability result for the frozen FieldLab fixture only. It does not introduce Boolean feedback into the field dynamics and it does not authorize FL-6.

## Provenance

- merged implementation commit: `198725ec32d833cd9b8036ccabfc1574d286f462`
- exact qualified pull-request head: `ab3f558bad73080e2f0c83607580fd46ab943ec3`
- GitHub Actions pull-request checkout revision embedded in the semantic report: `2849655bcdcee79bfa23f3b9167114253045681f`
- workflow run: `34941348489`
- retained artifact: `10385013578`, `fieldlab-flb1-2849655bcdcee79bfa23f3b9167114253045681f`
- artifact ZIP digest: `sha256:d4bfeced7799cb79cbc228b0237415b01617fe9bfb97f1438924f085f47614d6`
- semantic report SHA-256: `05ea50cd555c7f7953ee9ea4ad05aab439defd39ff010f2481e163f862ee3bca`
- the two complete campaign executions and the retained canonical report were byte-identical.

The Actions checkout revision is the synthetic pull-request merge revision used by `github.sha`; it is not the pull-request head SHA. Both identities are retained so the evidence is not silently attributed to the wrong commit.

The compact machine-readable result retained in the repository is [`results/FL-B1-boolean-observability.json`](../results/FL-B1-boolean-observability.json).

## Frozen protocol executed

The campaign used the three preregistered eight-node clean memories, the fixed ordered bank of 16 `fieldlab.boolean-predicate.v1` component-threshold predicates, and the five frozen perturbation radii `0.01`, `0.05`, `0.10`, `0.25`, and `0.50` radians. Each radius contains 48 deterministic single-node perturbations, for 240 perturbation cases total.

The construction tolerance was `1e-12`; campaign validity required maximum norm-squared error at most `1e-10`. The observed maximum was `1.1102230246251565e-16`. Exact semantic replay succeeded.

## Observed result

The three clean memories produced distinct Boolean codewords. Their minimum pairwise Hamming distance was `4`, and no clean Boolean collision was observed.

The declared HB1-2 local-robustness radii are `0.01`, `0.05`, `0.10`, and `0.25`. At each of these four radii, all `48/48` perturbations retained the corresponding clean Boolean codeword. HB1-2 is therefore supported on the preregistered local panel.

The preregistered `0.50` radius is a stress observation outside the HB1-2 decision rule. At that radius, `0/48` perturbations retained the clean codeword. All three clean labels first changed code at radius index `4` (`0.50` radians). This stress failure is retained as part of the result and constrains any robustness interpretation.

On the three-case clean fixture, empirical label entropy was `1.584962500721156` bits. Conditional entropy given the frozen Boolean code was `0`, giving the same `1.584962500721156`-bit empirical mutual information on this finite panel. The declared constant-code baseline produced `0` bits. This is only a descriptive finite-panel quantity.

The preregistered outcome fields were therefore:

- HB1-1 clean-code separation: supported;
- HB1-2 declared local robustness through radius `0.25`: supported;
- HB1-3 clean Boolean collision count: `0`;
- HB1-4 non-zero empirical information relative to CONST on the frozen three-case fixture: supported.

## Interpretation boundary

The result shows that this fixed Boolean observation map preserves the declared distinctions on these three stable memories and all preregistered local perturbations through `0.25` radians. It also shows a sharp failure of exact code retention at the `0.50` stress radius under this predicate margin.

It does **not** establish general memory-system observability, cognition, compression, speed, reduced memory traffic, energy benefit, hardware advantage, physical magnetic equivalence, model-quality improvement, or novelty. A richer or learned predicate bank requires a new preregistered subseries rather than retuning FL-B1 after outcome inspection.
