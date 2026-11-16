# Tutorial: admit a plan and verify its receipts

You will admit a two-action robot plan with `law`, record its receipts, and gate on one.
<!-- cap: op:law -->
<!-- cap: step:plan -->
<!-- cap: step:record-receipts -->
<!-- cap: step:require-receipt -->

## 1. State and plan

```python
from graphlaw import GraphLaw, steps
A = "<urn:p:robot> <urn:p:at> <urn:p:a> .\n"
B = "<urn:p:robot> <urn:p:at> <urn:p:b> .\n"
C = "<urn:p:robot> <urn:p:at> <urn:p:c> .\n"
plan = steps.plan(
    [dict(name="a-b", pre=A, add=B, **{"del": A}),
     dict(name="b-c", pre=B, add=C, **{"del": B})],
    goal=C)
```

## 2. Admit, record, require

```python
with GraphLaw() as gl:
    out = gl.law({"text": A, "dialect": "ntriples"},
                 [plan, steps.record_receipts(), steps.require_receipt("plan-action")])
    for r in out["receipts"]:
        print(r["step"], r["parent"][:14], "->", r["child"][:14])
```

Every action is one receipt (`parent`, `child`, `added`, `authority`, `revision`). A plan whose
precondition is unmet is refused at that action with `PlanRefused`.

## 3. Verify a receipt store offline
<!-- cap: bin:graphlaw-verify -->

`graphlaw-verify` replays a plan against an on-disk signed receipt store:

```python
from graphlaw import verify_receipts
res = verify_receipts(start="start.nt", plan="plan.json", receipts="receipts/",
                      trusted_keys=[PUBKEY_HEX])
assert res.admitted             # exit 0; exit 1 is a typed refusal
```

## See Also

[Govern with law steps](../how-to/govern-with-law-steps.md) |
[Receipts and attestation](../how-to/receipts-and-attestation.md)
