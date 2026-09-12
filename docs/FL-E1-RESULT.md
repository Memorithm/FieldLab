# FL-E1 result — conservative energy-model extension

Protocol: `energy-model-extension-v1`

Preregistration: [`prereg/FL-E1.md`](../prereg/FL-E1.md)

Machine-readable evidence: [`results/FL-E1-energy-model.json`](../results/FL-E1-energy-model.json)

## Result

The preregistered FL-E1 validation gates passed.

| Gate | Observed result | Criterion | Status |
| --- | ---: | ---: | --- |
| Duplicate undirected-edge guard | rejected | must reject `(0,1)` + `(1,0)` | pass |
| E0→E1 maximum energy error | `1.665334536938e-16` | `<= 1e-10` | pass |
| E0→E1 maximum field-component error | `0` | `<= 1e-10` | pass |
| E0 control state displacement | `0` | `<= 1e-10` | pass |
| E1 operator state displacement | `1.074576914742` | `> 0.5` | pass |
| E1 operator energy decrease | `0.999928531015` | `> 0.9` | pass |
| E1 deterministic replay | identical | required | pass |
| Anisotropy initial x alignment | `0.707106781187` | reference | — |
| Anisotropy final x alignment | `0.999982132594` | `> 0.99` and greater than initial | pass |
| Anisotropy energy decrease | `0.499964265508` | `> 0.4` | pass |

`protocol_valid = true`.

## What this establishes

The historical scalar FL-E0 model embeds numerically into FL-E1 through `K_ij = J_ij I` and `A_i = 0` on the fixed validation fixtures. The maximum observed energy discrepancy is at floating-point roundoff scale and the effective-field components match exactly on those fixtures.

The fixed cross-axis witness also demonstrates a capability that scalar `J_ij` coupling does not provide for the same state representation and initial condition. With two states initially aligned to the x-axis, the scalar E0 control remains stationary under projected dynamics, while the fixed operator

```text
K = [[0, -1],
     [1,  0]]
```

produces non-zero tangent motion and lowers the declared E1 energy. This is an expressivity result, not a task-performance result.

The local symmetric anisotropy witness drives a state initialized at 45 degrees toward the declared x-axis preference while lowering energy, validating the implemented `-1/2 mᵀ A m` term and its `A m` effective field on this fixture.

## What this does not establish

FL-E1 does not establish improved associative recall, reasoning accuracy, scaling, learned representations, biological plausibility or physical magnetic correspondence. Those claims require separate preregistered comparisons against FL-E0 and relevant non-field baselines.

The experiment also does not justify folding rotational/non-conservative dynamics, stochastic forcing, explicit hysteresis or higher-order interactions into FL-E1. Those remain separate mechanisms so that future effects can be attributed by ablation.
