# How to sign, verify and require receipts

## Sign and verify a file
<!-- cap: bin:graphlaw-attest -->

```python
from graphlaw import attest
pub = attest.pubkey(SEED_HEX)
attest.sign(SEED_HEX, "artifact.bin", out="artifact.att.json")
attest.verify("artifact.bin", "artifact.att.json", [pub]).ok     # exit 0; 1 = REFUSED
```

## Require a signed receipt inside `law`

```python
rc = gl.law(data, [steps.rdfs()])["receipts"][0]
doc = attest.receipt_ntriples(rc, attest.sign_receipt(SEED_HEX, rc))
gl.law({"text": doc, "dialect": "ntriples"},
       [steps.require_signed_receipt("derive:rdfs", [pub])])
```

## Receipt refusals
<!-- cap: receipt_reason:unattested -->
<!-- cap: receipt_reason:bad_signature -->
<!-- cap: receipt_reason:untrusted_key -->

| `details.reason` | Cause |
|---|---|
| `unattested` | receipt recorded without an attestation |
| `untrusted_key` | signed by a key not in `trusted_keys` |
| `bad_signature` | receipt fields changed after signing |

## Registry digest check
<!-- cap: bin:graphlaw-registry -->

```python
from graphlaw.registry import run_registry_cli
assert run_registry_cli("--check").returncode == 0
```

## See Also

[Govern with law steps](govern-with-law-steps.md)
