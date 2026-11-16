# How to govern a graph with law steps and leases

`gl.law(data, steps, **lease_kwargs)` runs steps in order, one receipt per transition. Builders
live in `graphlaw.steps`.

## The ten steps

| Step | Builder | Ceiling |
|---|---|---|
| `shacl` | `steps.shacl(shapes)` | observe |
| `n3` | `steps.n3(rules)` | construct |
| `rdfs` | `steps.rdfs()` | construct |
| `owl-rl` | `steps.owl_rl()` | construct |
| `sql` | `steps.sql()` | observe |
| `hooks` | `steps.hooks(pack)` | construct |
| `plan` | `steps.plan(actions, goal)` | select |
| `record-receipts` | `steps.record_receipts()` | - |
| `require-receipt` | `steps.require_receipt(name)` | observe |
| `require-signed-receipt` | `steps.require_signed_receipt(name, keys)` | observe |

<!-- cap: step:shacl -->
<!-- cap: step:n3 -->
<!-- cap: step:rdfs -->
<!-- cap: step:owl-rl -->
<!-- cap: step:sql -->
<!-- cap: step:hooks -->
<!-- cap: step:require-signed-receipt -->

## Signed leases

```python
from graphlaw import attest
sl = attest.sign_lease(SEED_HEX, dict(id="L1", holder="agent", ceiling="construct",
                                       scope=["derive:rdfs"], expires_unix=2**40))
gl.law(data, [steps.rdfs()], signed_lease=sl, trusted_keys=[attest.pubkey(SEED_HEX)])
```

Scope names are the step receipt names: `admit:shacl`, `derive:n3`, `derive:rdfs`,
`derive:owl-rl`, `admit:sql`, `derive:hooks`, `admit:plan`, `admit:require-receipt`,
`admit:require-signed-receipt`.

## Ceilings
<!-- cap: lease_ceiling:observe -->
<!-- cap: lease_ceiling:select -->
<!-- cap: lease_ceiling:construct -->

`observe < select < construct`. A lease whose ceiling is below a step's requirement refuses it
with reason `ceiling`.

## Lease refusals
<!-- cap: lease_reason:expired -->
<!-- cap: lease_reason:out_of_scope -->
<!-- cap: lease_reason:ceiling -->
<!-- cap: lease_reason:bad_signature -->
<!-- cap: lease_reason:untrusted_key -->
<!-- cap: lease_reason:clock_skew -->

| `details.reason` | Cause |
|---|---|
| `expired` | `now >= expires_unix` |
| `out_of_scope` | step name not in `scope` |
| `ceiling` | lease ceiling below the step's |
| `bad_signature` | lease edited after signing |
| `untrusted_key` | signer not in `trusted_keys` |
| `clock_skew` | `issued_unix` ahead of the module clock by > `max_skew_secs` |

An unsigned `lease` is refused (`UnverifiedLeaseRefused`) unless `unverified_lease=True` with
`now_unix`; that form proves nothing about the issuer.

## See Also

[Receipts and attestation](receipts-and-attestation.md) | [Refusals](../../../refusals.md)
