# GL28-4: EU High-Risk AI Attestation Exporter (policy-gated)

- **Id**: GL28-4
- **Title**: EU High-Risk AI attestation exporter — signed conformance-claim
  document per release
- **Status**: SPEC (design; no implementation on disk)
- **Conformance target**: EU AI Act Arts. 9/10/12/14/15 evidence bundle,
  machine-checkable; every claim row re-verifiable from the artifact alone
- **Size**: M, **policy-gated** (see §8)
- **Inputs read**: `docs/reference/conformance_claim.md`,
  `src/attest.rs`, `registry/ARTIFACTS.blake3`,
  `registry/capability-registry.json`, `docs/jira/v26.10.28/TAG`

---

## 0. Landscape facts that shape the design

- `docs/reference/conformance_claim.md` is the court-to-article
  matrix. Its standing law is already receipt-gated: a count without a live
  run in the window is `UNVERIFIED`; a row promotes to `ALIVE` only via a
  dated receipt with embedded output. The exporter inherits this law — it
  packages standing, it never invents it.
- `src/attest.rs` provides the exact signing primitives to reuse:
  `SigningKey::from_seed_hex`, `VerifyingKey`/`key_id()`, `TrustedKeys`,
  `sign_bytes(&SigningKey, payload) -> Attestation`,
  `verify_bytes`, and `Attestation { payload_blake3, key_id, signature }`
  with strict canonical JSON `to_json`/`from_json`
  (`src/attest.rs:346-433`). No new crypto is written; the exporter is a
  payload-shaping module over `attest.rs`.
- The release pin surface already exists:
  `registry/ARTIFACTS.blake3` (24 lines, format
  `<kind> <blake3> <bytes> <name>`, per-release rows plus historical pins),
  `registry/capability-registry.json` carries
  `"registry_blake3": "blake3:6e1103673508ade20f2276e93c7c948dd4f7c31689a25
  a325fac4a308856035d"` (line 2461) — the
  capability digest. Note: 6e110367... is the **registry** digest, not a
  separate "capability digest" file; the exporter cites it as
  `registry_blake3` and the court resolves it against the registry file
  itself.
- `docs/jira/v26.10.28/TAG` records the v26.10.28 cut: commit
  `82814ac4f398274e64aeb47d2448d04f89cfefcd`, release workflow run
  40797877715 success. The exporter's `release` input is exactly this
  shape: tag name + commit SHA + release-workflow receipt pointer.
- `graphlaw-verify` (`./target/debug/graphlaw-verify`) already verifies
  attestations end to end and is court-pinned by `tests/trust_attest.rs`
  (12/12, receipt `receipts/trust-attest-2026-10-22.md`). The exporter
  reuses the verification path, not a new verifier.
- SPC receipts land as JSON with embedded Cpk
  (`receipts/spc-2026-10-28T171756.json`, Cpk 14.4149). The exporter cites
  receipt paths + digests, never re-derives Cpk.

## 1. Definition

One **attestation exporter** run per release produces one signed document:

```
attestation-v<version>.md        (human/consumable claims doc)
attestation-v<version>.blake3    (detached sidecar, blake3 of the .md body)
attestation-v<version>.attest    (Ed25519 attestation over the sidecar bytes)
```

The `.attest` file is a serialized `Attestation` whose `payload_blake3` is
the blake3 of the `.md` file bytes; the sidecar file holds that same digest
in the registry pin format so `blake3 <file>` workflows work unchanged.

## 2. Inputs (all resolved at export time, all digest-pinned in the doc)

| Input | Source | Pinned in doc as |
|---|---|---|
| Conformance claim table | `conformance_claim.md` court rows | `claims_doc_blake3` |
| Capability/registry digest | `capability-registry.json` `registry_blake3` | `registry_blake3` |
| SPC receipts | latest `receipts/spc-*.json` | `spc_receipt_*_blake3` per file |
| Artifact pin | `ARTIFACTS.blake3` wasm row | `artifacts_pin_blake3` (digest of pin file) |
| Release tag | `docs/jira/v table` + `git tag --points-at HEAD` | `release_tag`, `release_commit` |
| Court receipts | `receipts/*.md` cited per claim row | `receipt_*_blake3` per file |

Law: **the exporter resolves inputs at export time and re-derives every
digest it prints.** It does not copy a digest from a previous document; a
stale digest is a defect (no-overclaiming: counts/digests recalled from
memory are stale by default).

## 3. Output shape (claims_and_evidence pattern)

Mirrors ash_graphlaw's `documentation/reference/claims_and_evidence.md`
role: per-article sections, one row per claim, each row carrying
claim / court / receipt / falsifier / **standing at export time**.

```markdown
# graphlaw EU High-Risk AI Attestation v26.10.28

Standing header (read first)
  - export_utc, exporter version, standing vocabulary
  - ALIVE = live run in this export window (receipt digests below)
  - UNVERIFIED = binary exists, no live run in this window

Evidence surface
  release_tag: v26.10.28
  release_commit: 82814ac4f398274e64aeb47d2448d04f89cfefcd
  claims_doc_blake3: blake3:...
  registry_blake3: blake3:6e1103673508ade2...
  artifacts_pin_blake3: blake3:...
  spc_receipt_blake3: blake3:...

## Art. 9 — Risk management
- Claim: ...                (from conformance_claim.md)
- Court: tests/...          (cargo test --test <name>)
- Receipt: receipts/...     (blake3:...)
- Standing at export: ALIVE | UNVERIFIED
- Falsifier: ...

(repeat per article: 9, 10, 12, 14, 15)

## Signature
key_id: <VerifyingKey::key_id>
payload_blake3: blake3:...   (blake3 of this document's bytes)
signature: <hex Ed25519>
```

Section order and row fields are fixed; the court below parses this exact
shape. A row without all five fields (claim/court/receipt/standing/
falsifier) is a malformed document — refused, not repaired.

## 4. Signing (reuse `src/attest.rs` — no new crypto)

- Payload = exact bytes of the generated `.md` document.
- `sign_bytes(&SigningKey, payload_bytes)` -> `Attestation`; serialize with
  `Attestation::to_json` (strict canonical form) into
  `attestation-v<version>.attest`.
- The signer key is a **release key**: `SigningKey::from_seed_hex` seeded
  from `GRAPHLAW_ATTEST_SEED` (env; loud failure if absent — a default
  seed would let anyone sign). `key_id()` is embedded; the pubkey is
  published in the release notes (same posture as `graphlaw-verify`).
- Verification is the existing path: `Attestation::from_json` ->
  `verify_bytes` -> recomputing blake3 of the `.md` bytes. No new verify
  CLI; `graphlaw-verify`-equivalent logic is exercised by the court.

## 5. Exporter module shape

- File: `src/attestation_export.rs` (host crate), re-exported from `lib.rs`.
- Entry: `pub fn export(release: &ReleaseRef, key: &SigningKey)
  -> Result<Exported, AttestError>` where `ReleaseRef` carries
  `{tag, commit, claims_doc_path}` and `Exported` carries the three files
  as byte buffers plus the `Attestation`.
- Deterministic rendering: same inputs -> byte-identical output (fixed
  section order, fixed field order, RFC 3339 export_utc — no local time,
  no locale). Byte-identity is court-asserted (deterministic-renderer law,
  same rule as generated projections).
- Unreadable/missing input (claims doc, registry, pin file, receipt) ->
  typed `AttestError::MissingInput(path)` — a loud failure, never a
  silent omission of a section.

## 6. The exporter court (`tests/attestation_export.rs`)

New court file. Every claim row re-checkable, so the court verifies the
artifact, not the exporter's self-report:

1. **Shape**: parse the exported `.md`; every row has all five fields;
   article sections 9/10/12/14/15 all present.
2. **Digest honesty**: recompute blake3 of the `.md` bytes; must equal
   `payload_blake3` in both sidecar and `.attest`; recompute each cited
   input digest (claims doc, registry_blake3 against the registry file
   bytes, pin file, each receipt) — any mismatch is FAIL.
3. **Signature**: `Attestation::from_json` + `verify_bytes` with the
   release `TrustedKeys` entry; forged-payload flip (tamper one byte of
   the `.md`, verify must fail) — the anti-vacuity inversion.
4. **Standing honesty**: a row citing a receipt whose blake3 does not
   resolve on disk is `UNVERIFIED` in the doc AND the court asserts the
   doc says `UNVERIFIED` for it — the exporter must downgrade, not
   fabricate. (Flip: delete a cited receipt temp fixture; exporter output
   must downgrade the row and the court must still pass, proving the
   downgrade path is real.)
5. **Determinism**: run `export` twice; byte-identical `.md` +
   `.attest` (modulo the keypair, which is caller-supplied).
6. **Refusal**: missing registry file -> `MissingInput`, exit-typed; the
   court asserts the typed error, not a panic.

Court rung: `cargo test --test attestation_export`.

## 7. Anti-vacuity

`scripts/mutation_court.sh` gains one mutation case: flip a standing token
`ALIVE` -> `UNVERIFIED` in the exporter's renderer source (or invert the
digest-honesty check). The exporter court must flip. Exporter cases are
serialized against other lanes per the existing mutation-court solo rule.

## 8. Policy-gate note (scope boundary, load-bearing)

**Streaming to an external ledger is out of scope.** No publication, no
network write, no ledger adapter, no Notified Body submission transport in
this milestone. The exporter produces an **artifact** —
`attestation-v<version>.{md,blake3,attest}` — and stops. What a Notified
Body consumes is the artifact plus the published release pubkey; the
transport decision (portal upload, email, ledger anchor) is a policy
decision taken outside the repo, gated on this spec landing. Rationale:
actuation-vs-admission separation — the repo admits (signs, pins, courts)
evidence; it does not actuate its distribution.

## 9. Acceptance

- [ ] `src/attestation_export.rs` + court
      `tests/attestation_export.rs` exist and
      `cargo test --test attestation_export` passes with embedded output
      receipted under `receipts/`.
- [ ] Court flip-witness: the forged-payload and missing-receipt cases
      each demonstrably flip (mutation court case added).
- [ ] One real export for v26.10.28: the three files exist, the court
      verifies them, receipt under `receipts/attestation-v26.10.28.md`
      with real `blake3` + verify output.
- [ ] `docs/reference/conformance_claim.md` gains a pointer from
      each article section to the per-release attestation artifact.
