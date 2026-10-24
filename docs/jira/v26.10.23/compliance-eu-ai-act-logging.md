# EU AI Act Compliance Mapping — Art. 12 (Record-Keeping) and Art. 11 (Technical Documentation)

Milestone window: `v26.10.23`. Subject: `/Users/sac/graphlaw` at HEAD `7e92175`
(main). Prepared 2026-10-23. All file:line pointers verified against the tree
on disk at writing time; the tree is a Rust workspace (cargo), so the
witnessing courts named in §4/§5 run via `cargo test --test <target>`.

Scope note: this is a self-assessed mapping of GraphLaw's existing
receipt/logging surface to the EU AI Act's record-keeping and technical
documentation articles. It is not a notified-body assessment. Conformance
status is claimed per-requirement and each claim cites its witnessing court
(a real `cargo test` target or build).

---

## 1. Article 12 — Record-Keeping (automatic logging of events)

Art. 12 requires that high-risk AI systems automatically record events
("logs") while operating, with the log functionality contributing to
traceability of the system's functioning. The mechanism-level requirements
mapped here: (a) automatic recording over the lifetime of the system,
(b) tamper-resistance / integrity, (c) traceability of the system
functioning, including to inputs and to the data governing operation.

### 1.1 Automatic event recording — ExecutionReceipt

- **Mechanism**: every admitted statement execution mints an
  `ExecutionReceipt` — `src/duckdb_eval.rs:283` (`pub struct ExecutionReceipt`),
  carrying `query_hash: [u8; 32]` (line 290), `rows_emitted: usize` (line 294),
  and the compiled-plan-cache epoch the receipt was served at (line 300).
  Serialization of the receipt fields to the wire form is at
  `src/duckdb_eval.rs:337`/`339` (`query_hash`, `rows_emitted` entries).
- **Transaction-level records**: multi-statement runs produce aggregate
  records with `rows_emitted: Vec<usize>` (line 419), `state_delta: u64`
  (line 433, with rollback semantics documented at lines 402–424: on member
  failure every member effect is undone), and the member count as
  `state_delta` (line 1182).
- **Automatic over the system lifetime**: receipts are minted as a
  side effect of execution, not by an operator action; sequence execution
  mints them per statement (`src/duckdb_eval.rs:1121`,
  `rows_emitted` collection at 1144; transaction gating of state_delta at
  line 1160). Mutating members bump the epoch at their own execution point
  (`src/duckdb_eval.rs:1100`, 1195) so cached-state reuse is traceable to
  the epoch it was valid at.
- **Status**: CONFORMANT at mechanism level (automatic, per-event,
  input-bound records exist and are wire-serialized).
- **Witnessing courts**: `cargo test --test chicago_validation` (Chicago
  receipt/execution courts over real frozen-duckdb connections, per repo
  CLAUDE.md); `cargo test --test duckdb_statement_kinds` (35 PEG statement
  kinds on real frozen-duckdb, receipt per statement).

### 1.2 Traceability to inputs (the law that governed the run)

- **Mechanism**: the receipt binds to the exact admitted law, not a label:
  the sql-rdf receipt hash is `BLAKE3(Canon(G_AST) || P_params)` computed via
  `purrdf::try_canonicalize` (repo CLAUDE.md, Conventions), with the AST
  canonical-graph anchor folded into the hash at `src/duckdb_eval.rs:671`
  (`h_ast` from the canonical graph) and the parameter buffer hashed at
  `src/duckdb_eval.rs:516` (`hash_param_buffer`) / 613 (`blake3::Hasher`
  fold). The admission seal check before execution is at
  `src/duckdb_eval.rs:1216` (`compiled.verify_seal()`), definition at
  `src/sql_ast/ast.rs:645` (`pub fn verify_seal(&self) -> bool`).
- **Admission-before-execution**: `src/sql_ast.rs:1743` — "the admission
  seal: `CompiledQuery::verify_seal` must hold before" execution. A tampered
  or mutated plan flips the seal (unit-level falsifiers:
  `src/sql_ast.rs:3850`, 3857, 3863 — sql mutation, canon-bytes mutation,
  and doc mutation each make `verify_seal()` false; params re-binding is
  explicitly lawful and unsealed, line 3879).
- **Status**: CONFORMANT — logs trace to the governing inputs (canonical
  graph + parameters) by content hash, not by reference to mutable state.
- **Witnessing courts**: `cargo test --test sql_rdf_dialect` (SQL-in-RDF
  compile + admission courts), `cargo test --test chicago_validation`.

### 1.3 Tamper-resistance / integrity of logs

Three independent layers, each with a witnessed falsifier:

1. **Admission seal (per-execution)**: `verify_seal` at
   `src/sql_ast/ast.rs:645`, enforced pre-execution at `src/duckdb_eval.rs:1216`;
   the in-file tests at `src/sql_ast.rs:3836–3879` prove mutation flips it.
2. **Ed25519 attestation**: `src/attest.rs` — `sign_bytes` at line 417 binds
   an `Attestation` (struct line 328) to a canonical payload via
   `payload_blake3` + 64-byte Ed25519 signature; the tampered-receipt
   refusal path is documented at `src/attest.rs:577` ("including the
   tampered-receipt refusal"). Key types: `SigningKey::from_seed` (124),
   `VerifyingKey` (165). Determinism of signing is proven in doc-tests
   (lines 116–119).
   Witnessing court: `cargo test --test trust_attest` (Ed25519 attestation
   courts).
3. **Chain tamper detection**: `src/receipt_chain.rs` ports the chain rule
   `chain_hash(t+1) = BLAKE3(chain_hash(t) || frame_bytes(t+1))` verbatim
   (module docs lines 14–15). Tamper detection is staged at
   `src/receipt_chain.rs:777` (`chain_recompute` — tamper detection) and
   enforced via `record.verify_chain_against(claimed)` at line 933;
   `verify_chain` is exercised with a real tamper falsifier in
   `tests/receipt_chain_golden_test.rs:96–105` (forged v2 payload fails
   `verify_chain`).
   The on-disk ledger `receipts/CHAIN.blake3` (1,638 lines: per-file blake3
   digests + `chain_head d48a3a61…bb9f7bdf7`) is maintained by
   `scripts/receipt_chain_sync.sh` (computes b3sum per file, writes the
   chain), and is read by `scripts/doc_staleness_gate.sh:74–76` as a
   real on-disk authority.
   Witnessing court: `cargo test receipt_chain` targets
   (`tests/receipt_chain_golden_test.rs`) and
   `bash scripts/receipt_chain_sync.sh` as the ledger-maintenance gate.

### 1.4 Replay (third-party verifiability of the log)

- **Mechanism**: `graphlaw-verify` (`src/bin/graphlaw_verify.rs`, entry
  `fn main` at 197) replays `Plan::admit` from the start state offline,
  checks the receipt store equals the replayed chain (parent/child ids,
  step, authority, plan digest), and verifies every receipt's Ed25519
  attestation against trusted keys. No network. Prints `ADMITTED` /
  `REFUSED: <code>: <detail>`; exit codes 0/1/2 (module docs lines 1–12).
  This is the Art. 12(5)-style "logs enable retrospective reconstruction"
  surface, with typed refusals per `docs/refusals.md`.
- **Status**: CONFORMANT at mechanism level; see Gap G3 on key distribution.
- **Witnessing court**: `./target/debug/graphlaw-verify` end-to-end run plus
  the trust_attest courts.

---

## 2. Article 11 — Technical Documentation

Art. 11 requires technical documentation demonstrating the system's
compliance, drawn up before placing on the market. The existing doc surface:

| Art. 11 element | GraphLaw surface | Location |
|---|---|---|
| System architecture / design | Architecture doc | `/Users/sac/graphlaw/docs/ARCHITECTURE.md` |
| Capability surface (what the system may do) | Capability registry | `registry/capability-registry.json` + `docs/capability-registry.md` (generated, never hand-patched) |
| Operational description | Whitepaper | `/Users/sac/graphlaw/docs/whitepaper-v26.10.22.md` |
| Deployment/runtime description | WASM integration doc | `/Users/sac/graphlaw/docs/WASM.md`, `docs/wasm-integration.md` |
| Security controls | Security policy | `/Users/sac/graphlaw/SECURITY.md` |
| Conformance statement | sql-rdf Recommendation + conformance register | `/Users/sac/graphlaw/docs/reference/sql-rdf-rec.md`, `docs/reference/refusal-catalog.md`, `docs/reference/runner-exit-codes.md`, `docs/reference/wasm-gap-register.md` |
| Typed failure vocabulary | Refusal register (append-only, stable codes) | `/Users/sac/graphlaw/docs/refusals.md` |
| API stability commitments | API stability doc | `/Users/sac/graphlaw/docs/api-stability.md`, `docs/abi-reference.md` |
| Governance of doc truth | Doc staleness gate reading the receipt chain | `scripts/doc_staleness_gate.sh:74–76` |

Note: `docs/reference-generated/` artifacts are regenerated by their
generators, never patched — documentation integrity is enforced the same way
as the receipts (regeneration from the canonical graph, not hand edits).

Status: SUBSTANTIALLY PRESENT. The documentation set covers architecture,
capability surface, security, deployment, and a machine-checkable conformance
register with typed refusal codes.

---

## 3. Honest gaps

- **G1 — Unsigned transition receipts (`UNSIGNED-NO-KEY`).** 1,510 of 1,515
  transition receipts in `receipts/` carry an empty identity field: they
  exist but are not signed. Under `R=receipt(A)` a receipt without identity
  is telemetry, not a receipt (see
  `docs/jira/v26.10.23/design-dev-loop-self-hosting.md:25`, and ROADMAP.md:249).
  Art. 12 impact: the per-event records exist and are tamper-evident via
  CHAIN.blake3, but the cryptographic *identity* layer (Art. 12's
  attribution requirement at the strictest reading) is missing on the bulk
  ledger. Remediation direction: `receipt_v2` emission with signed identity
  going forward; the 1,510 historical rows are NOT backfilled (design
  decision recorded at design-dev-loop-self-hosting.md:130).
- **G2 — `receipt_v2` emission is Phase 2.** The v2 sealed-receipt format
  exists (attest doc-tests exercise `receipt_v2` at `src/attest.rs:778–788`,
  and `ReceiptEpochV2` exists in `src/receipt_chain.rs:601`), but
  emission-by-default on the dev loop is a Phase 2 item
  (design-dev-loop-self-hosting.md:146 — CHAIN.blake3 as fold head over the
  serialized stream is forward work). Until it lands, the chain head over
  the bulk ledger is digest-level (blake3 per file) rather than
  signature-level.
- **G3 — Trusted-key distribution is out-of-band.** `graphlaw-verify` takes
  `--trusted-key <hex>` from the caller; there is no in-repo PKI or key
  directory binding signing keys to principals. Acceptable for a library
  kernel, but the log-verification story is not self-contained.

Gap count: **3** (1 substantive at the strictest reading, 2 forward-work /
boundary items).

---

## 4. Verification commands (as-of-disk receipts)

Commands the mapping rests on (each is a real target in this tree):

```bash
cargo test --test chicago_validation
cargo test --test sql_rdf_dialect
cargo test --test duckdb_statement_kinds
cargo test --test trust_attest
cargo test --test receipt_chain_golden_test
bash scripts/receipt_chain_sync.sh   # ledger maintenance + head check
```

Embedding full output for each would duplicate the per-court receipts already
in `receipts/`; this document cites the courts rather than re-running them,
and marks itself as-of 2026-10-23 HEAD `7e92175`.

## 5. Status summary

| Art. 12 requirement | Status | Pointer |
|---|---|---|
| Automatic recording of events | CONFORMANT (mechanism) | `src/duckdb_eval.rs:283` (§1.1) |
| Traceability to inputs/law | CONFORMANT | §1.2, BLAKE3(Canon(G_AST) ‖ P_params) |
| Tamper-resistance (seal) | CONFORMANT | §1.3.1 |
| Tamper-resistance (signature) | CONFORMANT mechanism / GAP bulk ledger | §1.3.2 + G1 |
| Tamper-resistance (chain) | CONFORMANT (mechanism + ledger) | §1.3.3 |
| Replay / reconstruction | CONFORMANT (mechanism) | §1.4 |
| Art. 11 technical documentation | SUBSTANTIALLY PRESENT | §2 |
