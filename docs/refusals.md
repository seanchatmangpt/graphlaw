# Refusals

Every refusal is typed and leaves no child state. In Rust these are `LawError` variants; over the
JSON ABI (feature `abi`, `src/abi.rs`) the response carries the refusal plus a `details` object
whose `code` field is the stable discriminator.

## Codes (`details.code`)

| code | Rust | `details` fields | What to do |
|---|---|---|---|
| `PlanRefused` | `LawError::PlanRefused` | `index`, `action`, `unmet` | Action `index` (or `<goal>` at one past the last) lacks the listed N-Quads lines. Fix the plan or the start state. |
| `ReceiptRequired` | `LawError::ReceiptRequired` | `step` | Run the step, then `receipt::record` its receipt into the state before the gate. |
| `LeaseRefused` | `LawError::LeaseRefused` | `reason` (`expired`, `out_of_scope`, `ceiling`), `lease_id`, `step` | Issue a lease that is current, covers the step, and has a high enough ceiling. |
| `NotAdmitted` | `LawError::NotAdmitted` | `violations[]`: `focus`, `path`, `component`, `message`, `severity` | Fix the data or the shapes; each entry is one SHACL result. |
| `ResourceLimit` | `RefusalKind::ResourceLimit` | `limit`, `observed`, `max` (`n3_iterations` has `max` only) | Shrink the request. Limits: `request_bytes`, `json_depth`, `atoms_per_field`, `plan_actions`, `policy_entries`, `n3_iterations` (constants `abi::MAX_*`). |
| `Refused` | `LawError::Refused` | `kind` | Upstream engine or routing refusal; see kinds below. |
| `PolicyRefused` | `policy::PolicyRefused` | `policy_kind`, `state`, `action` | See policy kinds below. |

## `RefusalKind` (inside `Refused`)

`NotSemanticContent` (empty, HTML or HTTP error body), `Ambiguous` (add a dialect hint),
`EngineRejected` (the owning engine rejected the document), `Unsupported` (valid but unusable here,
e.g. blank nodes in plan atoms), `ResourceLimit`.

## Policy kinds (`policy_kind`)

`MissingEntry` (reachable non-goal state has no policy entry), `InventedOutcome` (outcome not in
the problem's transitions), `BadMass` (outcome probabilities do not sum correctly), `DeadEnd`,
`Malformed`.
