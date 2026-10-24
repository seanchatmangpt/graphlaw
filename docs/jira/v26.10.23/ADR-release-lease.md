# ADR: Release Lease — Attest-Signed Release Authority (replacing RELEASE_HOLD)

- **Status**: Proposed
- **Date**: 2026-10-23
- **Milestone**: v26.10.23
- **Scope**: gate semantics of `.github/workflows/release.yml`, `publish-npm.yml`, `js.yml`;
  reuse of `src/attest.rs` and `src/law.rs` vocabulary. This ADR is prose only — no code
  changes are made by it.

## Context

### The current gate: a one-bit console variable

`RELEASE_HOLD` is a repository **variable** (not a secret, not a signed object). Three
workflows read it:

- `.github/workflows/release.yml:42` binds `RELEASE_HOLD: ${{ vars.RELEASE_HOLD }}`; at
  lines 74–75 it prints `RELEASE_HOLD is true: skipping tag and publish` and skips tag +
  GitHub release.
- `.github/workflows/publish-npm.yml:49,160` gates jobs/steps on
  `vars.RELEASE_HOLD != 'true'`.
- `.github/workflows/js.yml:33,114,142,171,196` mirrors the same condition so a hold
  freezes the JS surface too.

`release.yml`'s own header comment (lines 3–14, 32–35) documents the ambiguity directly:
a green run with nothing to release is normal, and the workflow is "NOT locally
reproducible end to end" because it acts via `GH_TOKEN`. The gate is a single bit, and
the ambiguity is documented — a green run can mean "nothing to release" or "held, and
the plan step declined to say so loudly".

### The replacement already exists in-tree

Every primitive needed for a signed gate is shipped and court-tested:

- **Deterministic Ed25519 attestations**: `src/attest.rs` — `SigningKey` /
  `VerifyingKey` with deterministic `key_id()` (BLAKE3 of the 32-byte public key,
  `attest.rs:231`); `Attestation` canonical JSON `{"key_id","payload_blake3","signature"}`
  (`attest.rs:338-352`); `TrustedKeys` set keyed by `key_id` (`attest.rs:236-295`);
  no network, no clock.
- **Lease vocabulary**: `src/law.rs` — `Lease` (`law.rs:264`), `SignedLease`
  (`law.rs:350`) with `authorize` semantics; `LawState::apply_signed` /
  step-admission under lease check (`law.rs:865-870`).
- **Golden vectors** for the attestation surface in `tests/abi_manifest_court.rs`
  (native golden vectors, `tests/abi_manifest_court.rs:13-15,96,204-208`).

Nothing needs to be invented; the decision composes existing capital.

## Decision

### 1. Hold becomes an attest-signed release lease

A release is authorized by an attestation over the release triple
**(version, commit, chain_head)**:

```json
{
  "kind": "release-lease",
  "version": "MAJOR.MINOR.PATCH",
  "commit": "<full sha of the head being released>",
  "chain_head": "<blake3 digest of receipts/CHAIN.blake3 at the released commit>",
  "not_before": "<rfc3339>",
  "expires_at": "<rfc3339>"
}
```

The `chain_head` binding matters: a lease signed over a stale chain head is refused, so
a lease cannot be replayed across a receipt-chain re-baseline (see Rejected, below).

`release.yml`'s plan step is extended to verify, in order:

1. signature verifies against the payload (deterministic, no clock);
2. `key_id ∈ TrustedKeys` (the trusted set is repo configuration, reviewed by diff);
3. `not_before <= now < expires_at` and the triple matches the plan step's own
   `work == 'yes'` computation.

Any failure is a typed refusal, recorded in the run log — not a silent skip. The one-bit
green-skip ambiguity disappears: either a valid lease authorizes the release, or the run
fails loudly with the refusal class.

### 2. Stronger form: timed signed objection window

After the attest-signed release receipt lands, publication waits **N hours** and proceeds
unless a **root-key release-objection attestation** lands in that window:

- 999 unattended releases cost nothing — the lease is minted by the same automation that
  would have run anyway.
- The wrong release costs exactly one signature: the objector signs an objection over
  `(version, commit)` with a root key; the release job sees it and refuses.
- Silence is machine-verifiable evidence: absence of a matching objection attestation
  within the window is checkable by the workflow itself, not by a human remembering to
  look.

This converts "hold" (a bit someone must remember to set) into "objection" (a signed act
someone must perform to stop a release) — the asymmetric cost structure the RELEASE_HOLD
bit cannot express.

### 3. Pin-by-copy becomes derive-then-write (companion decision)

`pin-by-copy` of artifacts across release steps is replaced by **derive-then-write**,
gated on the digest verify passing: the downstream step re-derives the artifact and
writes it only if its blake3 digest matches the attested digest. This is the same
signature-verify-then-act shape as the lease; the gate is the digest check, and the
write is refused (typed) on mismatch. It rides the existing
`registry/ARTIFACTS.blake3` digest discipline rather than introducing a second
mechanism.

## Consequences

### Positive

- Release authority becomes a signed, expiring, replay-resistant object rather than a
  mutable console bit.
- Green-run ambiguity is eliminated: the workflow either acts under a valid lease or
  refuses with a typed class.
- Objection-window form gives root-key holders a real veto with one signature and zero
  per-release cost in the quiet case.

### Preserved (already-implemented boundary)

- **crates.io stays local.** The typed refusal that keeps crates.io publishing out of
  CI is preserved unchanged; the release lease authorizes tag + GitHub release, not
  crate publication.
- **Chain re-baseline automation is deliberately rejected.** Receipt-chain re-baselines
  (`receipts/CHAIN.blake3`) keep their quiesced-regen flow; the timing of a re-baseline
  is the human-judgment residue and is not automated by this ADR. The lease's
  `chain_head` field is the machine-checkable boundary: leases signed over a stale
  chain head are refused, so the automation never needs to *decide* when to re-baseline.

### Costs / risks

- `TrustedKeys` maintenance becomes release-critical repo configuration; a leaked root
  key authorizes releases. Mitigation: expiry on every lease + the objection window.
- The objection window adds N hours of latency to every release. Acceptable for this
  repo's release cadence; N is repo configuration.

## Migration

1. **Mint release keys** (offline): one root `SigningKey`, per-machine publish keys if
   needed; record `TrustedKeys` hex in repo config (reviewed by diff).
2. **Extend `release.yml` plan step** with the three verification checks (signature,
   key membership, validity window + triple match) reading the lease from the run
   environment; typed refusal classes appended to `docs/refusals.md` (append-only).
3. **Add the objection-window step**: release receipt lands → wait N hours → verify no
   objection attestation → publish. N defaults conservatively (e.g. 24h) and shrinks
   with operating history.
4. **Flip workflows off RELEASE_HOLD** once lease-gated runs are witnessed: remove
   `vars.RELEASE_HOLD` conditions from `release.yml`, `publish-npm.yml`, `js.yml` in one
   reviewed diff.
5. **Retire the variable**: delete `RELEASE_HOLD` from repo variables; the bit is dead,
   the signed lease is the only gate.

## See Also

- `docs/refusals.md` — typed refusal register (append-only codes)
- `src/attest.rs`, `src/law.rs` — attestation and lease vocabulary
- `.github/workflows/release.yml` — the gated workflow
