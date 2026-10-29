# conformance_claim

> Court-to-EU-AI-Act article matrix for graphlaw (`/Users/sac/graphlaw`).
> Mirrors the claim-register role of ash_graphlaw's
> `documentation/reference/claims_and_evidence.md`.

## Standing header (read first)

Every court count on this page is **receipt-gated**: a count without a live
run against the current tree is `UNVERIFIED`, by the no-overclaiming law
(counts recalled from receipts or memory are stale by default). The test
binaries named below were verified to exist on disk as of 2026-10-28. To
re-verify any row, run its binary and embed the real output in a dated
receipt under `receipts/` before citing the count as standing.

Standing vocabulary: `ALIVE` (live run in this window, output embedded),
`UNVERIFIED` (binary exists, no live run in this window). No row on this
page is `ALIVE`; a row is promoted only by a dated receipt.

## Art. 9 — Risk management

- Claim: adversarial and boundary courts over the admission law.
- Court: `tests/chicago_validation.rs` (cited 12/12).
- Receipt: `receipts/chicago-validation-2026-10-22.md`.
- Falsifier: a validation court passes on a mutated/adversarial law, or the
  count drifts on a live rerun.

- Claim: N3 budget/cliff is typed-refused before any evaluation.
- Court: `tests/n3_budget_cliff_court.rs` (cited 10/10).
- Receipt: binary verified on disk; no dated receipt yet — `UNVERIFIED`.
- Falsifier: a budget cliff admits instead of typed-refusing.

## Art. 10 — Data and supply-chain governance

- Claim: registry artifacts match their blake3 digests.
- Court: `tests/registry_artifacts.rs` (cited 22/22).
- Receipt: `receipts/registry-courts-2026-10-22.md`.
- Falsifier: a mutated registry artifact passes the digest court.

- Claim: the capability digest covers the ABI surface.
- Court: `tests/capabilities_digest.rs` (cited 6/6).
- Receipt: binary verified on disk; no dated receipt yet — `UNVERIFIED`.
- Falsifier: a capability change leaves the digest court green without a
  regen.

- Claim: vendored crates stay on a pinned fork (deny.toml).
- Court: `deny.toml` (repo root) + cargo-deny run.
- Receipt: `receipts/2026-10-25-n3-upstreaming-purrdf.md` (purrdf-n3 3.0.1
  fork swap, `vendor/purrdf-n3`).
- Falsifier: a vendored crate drifts from its pin and no court fails.

## Art. 12 — Record-keeping

- Claim: proof leases bind the evidence hash (`pe:sourceDigest`).
- Court: `tests/n3_proof_lease_court.rs` (cited 4/4).
- Receipt: binary verified on disk; no dated receipt yet — `UNVERIFIED`.
- Falsifier: a lease with a tampered `pe:sourceDigest` still admits.

- Claim: Ed25519-signed receipts verify end to end via
  `./target/debug/graphlaw-verify`.
- Court: `tests/trust_attest.rs` (cited 12/12).
- Receipt: `receipts/trust-attest-2026-10-22.md`.
- Falsifier: a forged or tampered receipt passes `graphlaw-verify`.

## Art. 14 — Human oversight

- Claim: the op ABI manifest is closed and pinned; unlisted ops are
  refused.
- Court: `tests/abi_manifest_court.rs` (cited 10/10).
- Receipt: binary verified on disk; no dated receipt yet — `UNVERIFIED`.
- Falsifier: an unlisted op reaches the engine without a manifest refusal.

- Claim: the graphlaw_ex BEAM turnstile fails closed — a malformed engine
  response decodes to a typed refusal
  (`:capability_response_undecodable`), never a crash.
- Court: `graphlaw_ex/test/` (law/capability suites).
- Receipt: no dated graphlaw_ex receipt yet — `UNVERIFIED`.
- Falsifier: a malformed response decodes instead of typed-refusing.

## Art. 15 — Robustness

- Claim: N3 proof pipeline end-to-end courts hold per stage.
- Court: `tests/n3_proof_pipeline_court.rs` (cited 5/5).
- Receipt: binary verified on disk; no dated receipt yet — `UNVERIFIED`.
- Falsifier: a pipeline stage corrupts output and the court stays green.

- Claim: forged leases and chain-hash tampering are rejected.
- Courts: `tests/k6_hostile_lease.rs`, `tests/trust_attest.rs`
  (cited 13/13); the chain-recompute forgery check lives at
  `tests/bcbs_aggregation_court.rs:411`.
- Receipt: binaries verified on disk; no dated 13/13 receipt yet —
  `UNVERIFIED`.
- Falsifier: a forged lease or tampered chain hash is admitted.

- Claim: SPC gate on the receipt fold path holds Cpk >= 1.33.
- Court: `scripts/spc_court.sh`.
- Receipt: `receipts/spc-2026-10-28T171756.json` (Cpk 14.4149),
  `receipts/spc-2026-10-28T165319.json` (Cpk 6.0979).
- Falsifier: the fold path drifts so Cpk < 1.33 and the gate still exits 0.

## Verification recipe (per row)

```bash
cargo test --test <court_name>          # host rung
bash scripts/spc_court.sh               # SPC rung
```

A row is `ALIVE` only with the real command output embedded in a dated
receipt. A loud SKIP is not a pass; a gate that cannot exit 1 is the
failure.
