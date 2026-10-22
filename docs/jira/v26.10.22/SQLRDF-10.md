# SQLRDF-10 — Section 1.4 post-quantum identity is unmet

- **Id:** SQLRDF-10
- **Title:** Section 1.4 post-quantum identity is unmet
- **Status:** Open

## Conformance target

The second formula in section 1.4 of the pasted sql-rdf Recommendation, the conformance text for v26.10.22:

`ΔS_admitted + Φ_lease ≡ R_receipt (mod Σ_PQC)`

The conformance text names `Σ_PQC` and stops. This ticket does not define it.

## Current behavior

GraphLaw signs with Ed25519. [`src/attest.rs`](../../../src/attest.rs) `sign_bytes` (line 267) signs an arbitrary payload and records `payload_blake3`, `key_id`, and the signature. `sign_receipt` (line 378) signs the canonical law-receipt payload through `sign_bytes`. [`tests/trust_attest.rs`](../../../tests/trust_attest.rs) courts that path (`sign_receipt`, `verify_receipt`, trusted keys, and a bad signature).

The tree has no ML-DSA implementation. [`docs/refusals.md`](../../refusals.md) has no conformance note that `Σ_PQC` is unmet. Receipts attached to SQL execution today are the digests in SQLRDF-09’s current behavior (`ExecutionReceipt.query_hash` and `state_receipt`), unsigned by either formula.

## Required behavior

[`docs/refusals.md`](../../refusals.md) states explicitly that `Σ_PQC` is unmet.

A signature over the SQLRDF-09 digest uses the Ed25519 path already in the crate (`sign_bytes` / `sign_receipt` in [`src/attest.rs`](../../../src/attest.rs)). This ticket adds no signature scheme.

## Acceptance

`cargo test --test trust_attest`.
