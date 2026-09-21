# FL-B3 result — Boolean collision audit on the FL-5G ambiguous panel

Status: **qualified fixture result**.

This result records the executable campaign from [`prereg/FL-B3.md`](../prereg/FL-B3.md). It is observation-only. It does not feed Boolean values into field dynamics and does not authorize FL-6.

## Provenance

- retained machine report: [`results/FL-B3-identifiable-boolean-collision-audit.json`](../results/FL-B3-identifiable-boolean-collision-audit.json);
- full report SHA-256 `e78c8ef71064047ffe6b5b80bf690e08640797eee8fe982ae6ffa6e91476d250`; exact semantic replay succeeded;
- protocol validity HB3-0 supported.

## Frozen protocol executed

The nine-case FL-5F/FL-5G observable panel was reconstructed exactly as preregistered (three clean axial memories, two contradictory-label reuses of `P1`/`P2` with label `0`, and four single-bit corruptions of `P0`). Axial embeddings used `[s, 0]` unit nodes. Two frozen banks were evaluated without label-tuned thresholds: the FL-B1.0 16-predicate bank and a richer 32-predicate threshold bank (`±0.9` and `±0.5` on component 0).

## Observed result

Sign-input audit reproduced FL-5G: 7 distinct inputs, 2 contradictory-label groups, deterministic ceiling `7/9`.

Under the FL-B1 bank and the richer bank:

- Boolean collision groups existed;
- every contradictory-label sign group was covered by a Boolean collision group (Boolean observation did not silently resolve label conflicts);
- Boolean deterministic ceilings matched the sign ceiling at `7/9`;
- the richer bank did not invent identity among identical axial cues (ceiling remained `7/9`; conflict coverage remained complete).

Preregistered outcomes: HB3-0 through HB3-4 supported on this frozen panel.

## Interpretation boundary

FL-B3 shows that, for this axial panel, Boolean thresholds neither hide nor repair the FL-5G observable-input collisions, and that merely enriching fixed thresholds without new declared context does not create identifiable targets from identical cues. Future stochastic gates that need identifiable targets must add explicit declared context or restrict to unambiguous corruptions. No Boolean control, cognition, survival, hardware, biology, LLM, or FL-6 claim is made.
