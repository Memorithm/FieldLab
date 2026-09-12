# FL-1 — Associative Recall & Attractor Memory Result

Date: 2026-09-12

Protocol: [`prereg/FL-1.md`](../prereg/FL-1.md)

Evidence: [`results/FL-1-associative-recall.json`](../results/FL-1-associative-recall.json)

## Status

**VALID EXECUTION.** The exhaustive campaign completed all 7,551 declared cases, the zero-corruption integrity gate passed, and the deterministic replay sentinel matched exactly.

This is an exploratory finite-pattern result. It is not a capacity-scaling result and does not establish superiority over associative-memory methods in general.

## Main result

| Method | Exact recoveries | Aggregate exact-recall rate |
| --- | ---: | ---: |
| Continuous FieldLab dynamics | 6,723 / 7,551 | **0.890345650** |
| Deterministic asynchronous Hopfield | 6,303 / 7,551 | **0.834723878** |
| Nearest-template retrieval | 7,342 / 7,551 | **0.972321547** |

Under the frozen shared Hebbian coupling matrix, the continuous field dynamics recovered **420 more cases** than the deterministic asynchronous Hopfield baseline. It did **not** beat nearest-template retrieval, which has direct access to the stored templates and is therefore a stronger but structurally different retrieval reference.

## Corruption frontier

| Flipped bits | Cases | Field | Hopfield | Nearest template |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 3 | 100% | 100% | 100% |
| 1 | 48 | 100% | 100% | 100% |
| 2 | 360 | 100% | 100% | 100% |
| 3 | 1,680 | **100%** | 90.7142857% | 100% |
| 4 | 5,460 | **84.8351648%** | 80.0000000% | 96.1721612% |

The field method was exact for every tested corruption through three flipped bits. Its advantage over Hopfield appears first at three flips and persists at four flips. At four flips, nearest-template retrieval remains substantially stronger.

## What is supported

Within this exact fixed bank of three orthogonal 16-symbol bipolar patterns, fixed Hebbian weights, fixed cue representation, fixed deterministic integration policy and exhaustive zero-to-four-bit corruption set:

- the continuous field dynamics are reproducible;
- the continuous field dynamics outperform the declared asynchronous Hopfield baseline in aggregate exact recall;
- the continuous field dynamics remain perfect through three flipped bits;
- the continuous field dynamics do not outperform direct nearest-template retrieval.

## What is not supported

FL-1 does not establish:

- a universal advantage over Hopfield networks;
- a memory-capacity law;
- an advantage at larger pattern counts or widths;
- an advantage over optimized nearest-neighbour or modern associative-memory methods;
- a computational-efficiency advantage;
- physical magnetic equivalence;
- a CCOS benefit.

The next series must therefore study a different mechanism rather than tuning FL-1 after observing this result.

## Provenance

- source head: `37920212af7b8d4280f6b61fbd83a498066e5270`
- CI run: `34694622340`
- retained artifact: `10298044011`
- artifact ZIP SHA-256: `60b7a70837ba78e12780f3780dbb9e5819cdd82558d6a8389a5ba8dabc6a34e6`
- experiment fingerprint: `fnv1a64:bd77cc7f608935b6`
