# How to stay inside the resource limits

Limits are in `capabilities()["limits"]` and `registry/capability-registry.json`. Exceeding one is
a typed `ResourceLimit` refusal naming `details.limit`.
<!-- cap: limit:hooks_max_firings -->
<!-- cap: limit:hooks_max_rounds -->
<!-- cap: limit:hooks_max_state_quads -->
<!-- cap: limit:max_atoms_per_field -->
<!-- cap: limit:max_json_depth -->
<!-- cap: limit:max_outstanding_alloc_bytes -->
<!-- cap: limit:max_plan_actions -->
<!-- cap: limit:max_plan_total_atoms -->
<!-- cap: limit:max_policy_entries -->
<!-- cap: limit:max_request_bytes -->
<!-- cap: limit:n3_max_backward_depth -->
<!-- cap: limit:n3_max_backward_solutions_per_goal -->
<!-- cap: limit:n3_max_derived_facts -->
<!-- cap: limit:n3_max_iterations -->
<!-- cap: limit:n3_max_match_steps -->
<!-- cap: limit:n3_max_term_bytes -->
<!-- cap: limit:n3_max_term_depth -->
<!-- cap: limit:n3_max_total_bytes -->
<!-- cap: limit:n3_max_total_steps -->

| Limit | Value |
|---|---|
| `hooks_max_firings` | 10000 |
| `hooks_max_rounds` | 64 |
| `hooks_max_state_quads` | 1000000 |
| `max_atoms_per_field` | 10000 |
| `max_json_depth` | 64 |
| `max_outstanding_alloc_bytes` | 16777216 |
| `max_plan_actions` | 1000 |
| `max_plan_total_atoms` | 100000 |
| `max_policy_entries` | 100000 |
| `max_request_bytes` | 16777216 |
| `n3_max_backward_depth` | 32 |
| `n3_max_backward_solutions_per_goal` | 1024 |
| `n3_max_derived_facts` | 500000 |
| `n3_max_iterations` | 4000000 |
| `n3_max_match_steps` | 200000 |
| `n3_max_term_bytes` | 65536 |
| `n3_max_term_depth` | 64 |
| `n3_max_total_bytes` | 268435456 |
| `n3_max_total_steps` | 50000000 |

The gate probes the cheap ones (frame size, JSON depth, plan actions, atoms, policy entries, N3
term size and depth). The rest are registered gaps with a stated reason in
`docs/python/diataxis/coverage.json`; the gate prints them on every run.

## See Also

[Handle refusals](handle-refusals.md)
