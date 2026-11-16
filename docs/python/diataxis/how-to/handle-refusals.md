# How to handle refusals

`GraphLawError` carries the ABI error object: `kind`, `message`, `dialect`, `engine` and
`details` (`code` plus code-specific fields). An agent should branch on `kind` and
`details.code`, never on message text.

## Kinds
<!-- cap: refusal_kind:NotSemanticContent -->
<!-- cap: refusal_kind:Ambiguous -->
<!-- cap: refusal_kind:EngineRejected -->
<!-- cap: refusal_kind:Unsupported -->
<!-- cap: refusal_kind:ResourceLimit -->

| `kind` | Agent response |
|---|---|
| `NotSemanticContent` | input is empty/HTML/an HTTP error: re-fetch |
| `Ambiguous` | resend with `hint` |
| `EngineRejected` | the owning engine rejected the document: fix the text |
| `Unsupported` | malformed or unusable request: fix the request |
| `ResourceLimit` | split the work; see [limits](stay-inside-the-limits.md) |

## Codes (`details.code`)
<!-- cap: refusal_code:Refused -->
<!-- cap: refusal_code:ResourceLimit -->
<!-- cap: refusal_code:PlanRefused -->
<!-- cap: refusal_code:ReceiptRequired -->
<!-- cap: refusal_code:ReceiptRefused -->
<!-- cap: refusal_code:LeaseRefused -->
<!-- cap: refusal_code:NotAdmitted -->
<!-- cap: refusal_code:DuckDb -->
<!-- cap: refusal_code:UnverifiedLeaseRefused -->
<!-- cap: refusal_code:PolicyRefused -->

`Refused` (a `law` step surfaced an upstream kind), `ResourceLimit`, `PlanRefused`,
`ReceiptRequired`, `ReceiptRefused`, `LeaseRefused`, `UnverifiedLeaseRefused`, `NotAdmitted`,
`DuckDb`, `PolicyRefused`.

## Policy kinds
<!-- cap: policy_kind:MissingEntry -->
<!-- cap: policy_kind:InventedOutcome -->
<!-- cap: policy_kind:BadMass -->
<!-- cap: policy_kind:DeadEnd -->
<!-- cap: policy_kind:Malformed -->

`MissingEntry`, `InventedOutcome`, `BadMass`, `DeadEnd`, `Malformed` (in `details.policy_kind`).

## See Also

[Refusal register](../../../refusals.md) | [Playbook](../../../refusal-playbook.md)
