# sig-tamper-receipt (K2, Lane M06)

## Defect class

Receipt/lease signature tamper coverage in `tests/trust_attest.rs` was
1-of-64 bytes (`sig[0] ^= 1` only); the lease signature path had zero
signature-byte coverage.

## Design

Two exhaustive single-byte-flip guards, appended to `tests/trust_attest.rs`
(extension only, lease respected):

- `every_single_byte_flip_of_a_receipt_signature_is_refused` — signs a real
  receipt, asserts the decoded signature is 64 bytes, then for each byte index
  0..64 flips exactly that bit-0 (`^= 1`) and asserts
  `attest::verify_receipt` returns `Err(AttestError::BadSignature)`. Asserts
  the refusal count is exactly 64.
- `every_single_byte_flip_of_a_lease_signature_is_refused` — signs a real
  lease, first proves the unmutated `SignedLease` authorizes
  (`transition_authorized` succeeds) so any later refusal is attributable to
  the flip, then repeats the 64-byte exhaustive flip asserting
  `LeaseReason::BadSignature` from the state gate each time. Asserts the
  count is 64.

## Non-vacuity

A verify that accepted any flipped byte would fail the test directly
(wrong-typed result or failing assert), so no separate mutation court is
needed for this guard. Runtime is ~90ms for the whole 12-test binary.

## Verification

```
CARGO_TARGET_DIR=target-m6 cargo test --test trust_attest
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```
