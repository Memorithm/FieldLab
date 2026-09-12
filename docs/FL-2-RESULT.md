# FL-2 — Signed Competition & Frustration Result

Date: 2026-09-12

Protocol: [`prereg/FL-2.md`](../prereg/FL-2.md)

Evidence: [`results/FL-2-signed-competition-frustration.json`](../results/FL-2-signed-competition-frustration.json)

## Status

**VALID EXECUTION.** FL-0 and FL-1 regression gates were green, all FL-2 cases were finite and replayable, and both analytic three-node reference systems met the frozen numerical tolerances.

This is a small deterministic field experiment. It does not establish a general theory of reasoning, inhibition, or magnetic cognition.

## Part A — Two-hypothesis competition

All three coupling conditions selected the evidence-favoured hypothesis correctly in every one of the 80 non-tie cases. Signed repulsion therefore did **not** improve winner accuracy on this grid because the controls were already at 100%.

Its measurable effect was increased separation between the competing hypothesis states.

| Condition | Winner rate | Mean useful separation | Minimum useful separation | Max tie polarization |
| --- | ---: | ---: | ---: | ---: |
| Repulsive `J=-0.5` | 100% | **0.829983940213** | **0.006103795044** | 0 |
| Uncoupled `J=0` | 100% | 0.584645449903 | 0.000086637677 | 0 |
| Attractive `J=+0.5` | 100% | 0.265158394256 | 0.000015236695 | 0 |

For every condition, useful separation was monotonic on the declared evidence branches, energy was non-increasing, and exact ties remained unpolarized under the symmetric initialization.

The narrow supported statement is therefore: **on the frozen FL-2 evidence grid, repulsive coupling amplifies the state-space separation between already-correct competing hypotheses without changing their winner accuracy or creating false tie polarization.**

## Part B — Frustrated triangle

The equal repulsive triangle converged to the analytic 120-degree compromise:

```text
final energy = -1.500000000000
pairwise dots =
  -0.499999721746
  -0.500000132586
  -0.500000145668
max |dot + 0.5| = 2.78254e-7
```

The matched attractive triangle converged to full alignment:

```text
final energy = -3.000000000000
pairwise dots = [1, 1, 1]
```

Both trajectories replayed deterministically.

This establishes that the current projected field kernel can represent a small frustrated system whose stable minimum is a distributed compromise rather than a binary winner or an oscillatory failure. It does not establish that such a compromise is useful on a cognitive task.

## Frozen hypotheses

All four declared Part-A hypotheses evaluated `true`:

- H2-A1: repulsive condition preserved 100% winner correctness;
- H2-A2: repulsive mean separation exceeded the uncoupled control;
- H2-A3: repulsive mean separation exceeded the attractive control;
- H2-A4: exact ties were not falsely polarized beyond the declared tolerance.

Because all methods already achieved 100% winner correctness, these outcomes support a **margin-amplification** interpretation, not an accuracy-improvement claim.

## What is not supported

FL-2 does not establish:

- better classification/reasoning accuracy from repulsion;
- calibrated probabilistic confidence;
- robustness to noisy/asymmetric ties;
- useful behavior in larger frustrated graphs;
- hysteretic memory;
- CCOS benefit;
- scaling or efficiency advantages;
- physical magnetic equivalence.

FL-3 should therefore test temporal path dependence explicitly instead of tuning FL-2 after observing these results.

## Provenance

- source head: `f061c947f3b59f14d49edcd07e7cafb7162c25b3`
- CI run: `34695053966`
- retained artifact: `10298364188`
- artifact ZIP SHA-256: `012d6db7e06230833c0d2beee6436765abbee3df49eae313c335a15a462a0130`
- experiment fingerprint: `fnv1a64:f4dbeb6be77671e0`
