# Battery Receipt — trust_attest (Ed25519 attestation courts, L3 meta leg)

Lane: EXECUTION (battery receipts), lease `receipts/2026-10-24-battery-*.md` +
ledger appends. Date: 2026-10-24. Leg target dir:
`/Users/sac/graphlaw/target-battery-meta`. Command of record:
`CARGO_TARGET_DIR=…/target-battery-meta cargo test --locked --test trust_attest`
(part of the L3 fresh build; `Finished \`test\` profile [unoptimized +
debuginfo] target(s) in 3m 18s`, verbatim at `/tmp/bat-L3.log:582`).

## Header

- HEAD: a00770b4998a55ed4ee397e3e2a1f5a880c51db0
- Dirty disclosure: working tree carries in-flight wave edits — 267
  `git status --porcelain` entries at receipt-write time (80 M-or-D tracked + 187 untracked,
  including in-flight `receipts/CHAIN.blake3` and vendor edits). Moving-tree
  disclosure: HEAD advanced during this lane (`966cec3` at dispatch -> a
  re-read `a00770b` at receipt write, concurrent wave landing); the
  logs were captured 11:15-11:35 local on the pre-advance tree, the pins
  were re-measured at write time. Verified as-is; no file outside the lease
  touched; no git mutations.
- Pin b3sums (measured this session, `/opt/homebrew/bin/b3sum`):
  - `courts/code-partition/axioms.blake3` = `2add10ce9ff782e33b75de099cba8dc8a114ab759855f93a7be85abb18581506`
  - `priv/graphlaw.wasm` = `58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a`
    (matches pin file `priv/graphlaw.wasm.blake3` and
    `registry/ARTIFACTS.blake3:13` byte-for-byte, 58,124,612 B)
- Witness-log digests (b3sum; logs are volatile — these digests preserve
  them; the battery driver's shell wrapper echoed success unconditionally,
  so each leg's observed shell exit code is 0 regardless of cargo's outcome —
  the captured logs are the evidence of record, not the leg exit codes):
  - `/tmp/bat-L1.log` b3sum = `7dec1d1f598a6f1861824f55927bec94e56a3f6785eb208559700bd0647c29b3` (732 lines)
  - `/tmp/bat-L2.log` b3sum = `5d5337166ad00196cb37ca555f58e116854707121e563dd503e10fb37c221029` (786 lines)
  - `/tmp/bat-L3.log` b3sum = `5f69751d4c360b8e9fa79948e2cea9017550019eaa6b12a5a4b8051f5383192d` (786 lines)
  - `/tmp/bat-L4.log` b3sum = `c2eedcb4d1f09d5c4a171bb5d02036ee9f4a7eee13c01ba6e6b5adc3bc025647` (699 lines)
- Splice provenance: the `Running tests/…` header and `test result:` line
  below are spliced mechanically from the digested `/tmp/bat-L3.log`;
  nothing in the fenced block is hand-typed.

## Scope law (attestation surface)

This receipt witnesses the EXISTING signature path only: Ed25519
(`src/attest.rs` — lease signing, receipt signing, `sign_bytes` /
`sign_receipt` verification surface, deterministic signing, byte-flip
refusals). It declares NO post-quantum scheme, NO new parameter set, NO new
modulus: the Sigma_PQC clause (GO-NO-GO §1.4 second formula) remains UNMET
and is quoted, not instantiated, by this receipt.

## Aggregate

| Court (target) | Tests | Result | Classification |
|---|---|---|---|
| trust_attest | 12 | 12/0 ok | PASS |
| **Total** | **12** | **12 passed; 0 failed** | 1 PASS · 0 CLASSIFIED · 0 SKIP |

## Per-court witness (mechanically spliced from `/tmp/bat-L3.log`)

```
     Running tests/trust_attest.rs (target-battery-meta/debug/deps/trust_attest-3e95c4f53b5658e8)

running 12 tests
test c_lease_signed_by_untrusted_key_or_edited_after_signing_is_refused ... ok
test scope_and_ceiling_still_bind_a_verified_lease ... ok
test a_hand_written_receipt_triples_pass_require_but_not_require_signed ... ok
test b_expired_signed_lease_is_refused_and_no_caller_field_can_revive_it ... ok
test issuer_clock_lead_beyond_the_skew_bound_is_refused ... ok
test signing_is_deterministic_and_attestation_json_round_trips ... ok
test d_tampering_a_signed_receipt_payload_fails_verification ... ok
test c_receipt_signed_by_an_untrusted_key_is_refused ... ok
test a_signed_receipt_recorded_in_state_satisfies_the_signed_gate ... ok
test subject_digest_is_part_of_the_signed_payload_and_survives_the_state ... ok
test every_single_byte_flip_of_a_lease_signature_is_refused ... ok
test every_single_byte_flip_of_a_receipt_signature_is_refused ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

Reading of the twelve oks (each name is court output, not prose):

- `signing_is_deterministic_and_attestation_json_round_trips` — real Ed25519
  sign, byte-deterministic, attestation JSON round-trips (artifact-
  instantiating; validity precondition for every attestation receipt).
- `every_single_byte_flip_of_a_lease_signature_is_refused` /
  `every_single_byte_flip_of_a_receipt_signature_is_refused` — full
  bit-flip sweep, both payload kinds (non-vacuity by exhaustion).
- `d_tampering_a_signed_receipt_payload_fails_verification` /
  `c_receipt_signed_by_an_untrusted_key_is_refused` /
  `c_lease_signed_by_untrusted_key_or_edited_after_signing_is_refused` —
  tamper + untrusted-key refusal paths.
- `b_expired_signed_lease_is_refused_and_no_caller_field_can_revive_it` —
  expiry binds and is caller-proof; `issuer_clock_lead_beyond_the_skew_bound_is_refused`
  — clock skew bound.
- `scope_and_ceiling_still_bind_a_verified_lease` — verification does not
  grant authority beyond the lease's scope/ceiling.
- `a_signed_receipt_recorded_in_state_satisfies_the_signed_gate` /
  `a_hand_written_receipt_triples_pass_require_but_not_require_signed` —
  the signed gate distinguishes signed from hand-written triples (positive +
  negative control pair).
- `subject_digest_is_part_of_the_signed_payload_and_survives_the_state` —
  subject identity is inside the signed payload.

## Honesty line: machine-courted vs prose-declared

- All twelve arms are machine-courted with real keypairs and real signature
  bytes; zero mocks (Chicago discipline).
- The PQC posture is prose-declared here and UNMET: §1.4's second formula is
  quoted as an open obligation; no scheme/parameter set/modulus is defined
  by this receipt. Ed25519 remains the shipped signature path.

## Falsifiers

1. Any non-`ok` `test result:` line for `trust_attest` on this HEAD with pin
   `58a44095…` falsifies this receipt.
2. A mutated verifier that accepts a flipped signature byte must flip the
   `every_single_byte_flip_*` courts red; if they still pass, the sweep is
   vacuous.
3. Relaxing the expiry check must flip
   `b_expired_signed_lease_is_refused_and_no_caller_field_can_revive_it` red.
4. Landing a PQC scheme without a new court + receipt does not flip the
   UNMET clause; the clause flips only on its own machine evidence, which
   does not exist today.

Closing b3sum of this receipt is reported in the lane's final report (a file
cannot embed its own digest); verify with
`b3sum /Users/sac/graphlaw/receipts/2026-10-24-battery-trust-attest.md`.
