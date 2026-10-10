# Refusals

Every refusal is typed and leaves no child state. In Rust these are `LawError` variants; over the
JSON ABI (feature `abi`, `src/abi.rs`) the response carries the refusal plus a `details` object
whose `code` field is the stable discriminator. The envelope is described in the
[ABI reference](abi-reference.md).

Refusals raised by the engines themselves (bad syntax, unknown query form) carry no `details`;
only the codes below do. Decoders must treat `details` as optional and unknown codes as open (see
the [capability registry](capability-registry.md)).

## Codes (`details.code`)

| code | Rust | `details` fields | What to do |
|---|---|---|---|
| `PlanRefused` | `LawError::PlanRefused` | `index`, `action`, `unmet`, `violated_absent` | Action `index` (or `<goal>` at one past the last) lacks the N-Quads lines in `unmet`, or has the forbidden lines in `violated_absent` (`pre_not` / `goal_not`, PDDL `(not p)`) present. Fix the plan or the start state. |
| `ReceiptRequired` | `LawError::ReceiptRequired` | `step` | Run the step, then `receipt::record` its receipt into the state before the gate. |
| `ReceiptRefused` | `LawError::ReceiptRefused` | `step`, `reason` (`unattested`, `bad_signature`, `untrusted_key`) | Raised by `require-signed-receipt`. `unattested`: the receipt has no attestation; `bad_signature`: the signature does not verify; `untrusted_key`: the signing key is not in the step's `trusted_keys`. |
| `LeaseRefused` | `LawError::LeaseRefused` | `reason`, `lease_id`, `step` | See lease reasons below. |
| `UnverifiedLeaseRefused` | ABI request check | none | A `law` request carried an unsigned `lease` without `"unverified_lease": true`. Supply `signed_lease` + `trusted_keys`, or opt in explicitly (with `now_unix`). The envelope `kind` is `Unsupported`. |
| `NotAdmitted` | `LawError::NotAdmitted` | `violations[]`: `focus`, `path`, `component`, `message`, `severity` | Fix the data or the shapes; each entry is one SHACL result. The envelope `kind` is `EngineRejected`, dialect `Turtle`, engine `PurRdf`. |
| `ResourceLimit` | `RefusalKind::ResourceLimit` | `limit`, `observed`, `max` (N3 limits always carry `max`) | Shrink the request. Limits: `request_bytes`, `json_depth`, `atoms_per_field`, `plan_actions`, `policy_entries` (constants `abi::MAX_*`), plus the N3 reasoner limits `n3_iterations`, `n3_match_steps`, `n3_term_bytes`, `n3_total_bytes`, `n3_derived_facts` (constants `law::N3_*`). |
| `Refused` | `LawError::Refused` | `kind` | Upstream engine or routing refusal raised inside a `law` step; `kind` is a `RefusalKind` name (below). |
| `PolicyRefused` | `policy::PolicyRefused` | `policy_kind`, `state`, `action` | See policy kinds below. |

## `Refused {kind}`

`kind` names the upstream `RefusalKind` that a `law` step surfaced. Refusals from top-level ops
(not inside `law`) put the same kind in `error.kind` and have no `details`.

## `RefusalKind`

`NotSemanticContent` (empty, HTML or HTTP error body), `Ambiguous` (add a dialect hint),
`EngineRejected` (the owning engine rejected the document), `Unsupported` (valid but unusable
here, for example blank nodes in plan atoms, or a malformed ABI request), `ResourceLimit`.

## Lease reasons (`LeaseRefused.reason`)

| reason | Meaning |
|---|---|
| `expired` | Strict expiry: `now >= expires_unix`. |
| `out_of_scope` | The step name is not in the lease `scope`. |
| `ceiling` | The lease ceiling is below the step's required ceiling. |
| `bad_signature` | The lease attestation does not verify. Signed leases only. |
| `untrusted_key` | The attestation key is not in `trusted_keys`. Signed leases only. |
| `clock_skew` | `issued_unix` leads the verifier's clock by more than `max_skew_secs` (default 60). Signed leases only. |

Required ceilings per step are in the [ABI reference](abi-reference.md#13-law).

## Receipt reasons (`ReceiptRefused.reason`)

`unattested`, `bad_signature`, `untrusted_key`.

## Policy kinds (`policy_kind`)

`MissingEntry` (reachable non-goal state has no policy entry), `InventedOutcome` (outcome not in
the problem's transitions), `BadMass` (outcome probabilities do not sum correctly), `DeadEnd`,
`Malformed`. `state` and `action` name where the policy failed.

## Aggregate boundary

Law-kernel materialization (N3/Datalog closure) refuses SPARQL 1.1 aggregates by design: a law
kernel is one executable law and must be total and deterministic over the closure, so
`GROUP_CONCAT`/`SAMPLE` in a query handed to the law path is a refusal, not a result.
GraphLaw's `Step` enum (`src/law.rs`) offers no aggregate-capable step — the kernel surface is
`AdmitShacl`, `DeriveN3`, `Hooks`, `Plan`, receipt gates, and entailment closures only — so an
aggregate simply has no admitted path through a `law` step. (Historically the praxis
law-materialization engine, vendored in ggen as `crates/praxis-graphlaw` until its retirement
in ggen commit `8e92df31f`, 2026-10-10, surfaced this as a typed
`SPARQL query planning refused (unsupported construct)` error from its `build_for_aggregate`
planner; that crate is gone and the boundary now lives in the step surface itself.)

Aggregate projections (`GROUP_CONCAT`, `SAMPLE`, non-`COUNT` aggregation over groups) belong to
the generation/projection layer instead, where the full SPARQL 1.1 engine (oxigraph in the
ggen sync path) has standing and determinism is enforced by projection replay, not by the kernel.

Rule: if a gate or query needs an aggregate, run it in the projection layer and admit the
projected artifact; do not push it through a `law` step expecting a kernel answer.

## See Also

- [ABI reference](abi-reference.md)
- [Capability registry](capability-registry.md)
