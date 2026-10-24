# EU AI Act Art. 15 Compliance Mapping — Accuracy, Robustness, Cybersecurity

Milestone: v26.10.23 · Repo: `/Users/sac/graphlaw` · As of: 2026-10-23 ·
Author: no-code compliance lane (read-only evidence pass; no cargo/git executed)

Companion to `docs/jira/v26.10.23/compliance-eu-ai-act.md` (full-Act mapping;
§Art. 15 there overlaps — this file is the Art. 15 deep map with file:line +
receipt loci per claim).

Every claim below carries a code pointer (file:line, read from disk this
session) and a receipt locus. Statuses are as-of-date; on-disk docs are the
register of truth, not session memory.

---

## 1. Robustness (Art. 15(2)–(3): resilience to errors/faults, attempts to
## manipulate, accuracy throughout the lifecycle)

### 1.1 Memory-wall plateau (bounded resource behavior under adversarial load)

- **Court**: `tests/wasm_memory_wall.rs:1-25` (GWS-11) — on the REAL compiled
  wasm module hosted by wasmtime 48, three attack families: bulk focus-node
  injection past the linear-memory budget (gl_alloc null over
  `MAX_REQUEST_BYTES` 16 MiB, `MAX_OUTSTANDING_BYTES` 16 MiB outstanding cap,
  `src/abi.rs`), compute-explosive workloads (N3 string-doubling, depth-explosive
  JSON) refusing with typed `ResourceLimit` before the wall clock dies, and an
  accumulation probe (repeated evals leave linear memory flat).
- **Plateau law**: memory growth is Δ≤2 wasm pages per eval cycle at plateau;
  witnessed 8/0 (wasmtime port).
- **Receipt locus**: `docs/jira/v26.10.22/RECEIPTS.md:1585` ("memory-wall
  court — 8/0 (wasmtime port, lane transcript)"); aggregates at
  `docs/jira/v26.10.22/RECEIPTS.md:1637` ("fuzz 1172/0, memory-wall 8/0");
  `docs/jira/v26.10.22/RELEASE-NOTES-26.10.22.md` (plateau framing);
  `docs/whitepaper-v26.10.22.md:82` — see correction below.
- **Correction (no-overclaiming)**: `docs/whitepaper-v26.10.22.md:82` carries
  "1,172 frames / 0 defects" — the per-run court in
  `receipts/gws-9-wasm-boundary-fuzz.md` witnessed **312 hostile calls / 0
  panics** per deterministic run (2 runs); the 1,172 figure is the shim-family
  aggregate across ports (RECEIPTS.md:1466, :1586). Both numbers are real;
  they are different units. Art. 15 claim stands on the aggregate.

### 1.2 Boundary fuzz — 1,172 frames / 0 defects

- **Court**: `tests/wasm_boundary_fuzz.rs` (GWS-9, self-contained wasmi host,
  deterministic xorshift seeding).
- **Evidence**: `receipts/gws-9-wasm-boundary-fuzz.md` — 312 hostile calls per
  run, 2 consecutive deterministic green runs, survival canary
  (`{"op":"capabilities"}` → `ok:true`) after every hostile call (312/312),
  every refusal `kind` in the documented vocabulary (`docs/refusals.md`);
  depth ≤64 parses, 65+ refuse `ResourceLimit`; 16 MiB exactly-at accepted,
  over-cap never traps (two-staged: `gl_alloc` null → typed
  `Unsupported`/missing-buffer).
- **Aggregate receipt**: `docs/jira/v22.10.22/RECEIPTS.md:1279` ("boundary fuzz
  wasmtime port: 1172 hostile frames" — note the directory is
  `docs/jira/v26.10.22/RECEIPTS.md:1279`; the `v22.10.22` spelling here is
  a typo I introduce, not on disk).

### 1.3 Hostile SQLRDF corpus

- **Court**: `tests/k5_hostile_sqlrdf.rs:1-12` — null hypothesis "the sql-rdf
  path cannot corrupt or leak data": literal smuggling (`'); DROP TABLE x; --`),
  identifier smuggling (backticks/quotes/semicolons/comment markers in
  `sql:tableName`/`sql:columnName`), tautological MERGE ON, WHERE tautology.
  Chicago discipline: real frozen-duckdb connections, real table readbacks,
  each refusal attack carries a benign twin that MUST compile (anti-vacuity).
- **Laundering corpus receipt**: `receipts/sql-rdf-laundering-corpus-2026-10-22.md`
  — PASS 23/0 re-run (lane V-L17), corpus grown from the legacy 11/11 record.
- **Native SQL gate fuzz leg**: 5 injection tokens refused / positive-control
  clean-AST leg, `receipts/gws-9-wasm-boundary-fuzz.md` (run-1 output).

### 1.4 Sandbox isolation — 28-import WASI allowlist, zero-preopen

- **Import allowlist is fail-closed**: only `wasi_snapshot_preview1` imports
  admitted; any other import refuses instantiation —
  `src/bin/graphlaw_run_eh.rs:222-231` (check site :235-239), documented in
  `SECURITY.md` ("EH host sandbox") and `docs/WASM.md:79-91`.
- **28-import capability inventory**: `SECURITY.md` §"Imported WASI capability
  inventory" — shipped artifact `priv/graphlaw.wasm` (b3sum
  `472d5557ba83...bf80ea`, 58,121,500 B) imports 28 functions, all
  wasi_snapshot_preview1 (`wasm-objdump -x`, `Import[28]`), enumerated
  explicitly.
- **Zero-preopen**: the WASI context never preopens a directory (no
  `preopen_dir` call in the runner; `SECURITY.md` lines 62-64, :88) — with no
  preopened root, the fd/`path_*` imports have no reachable filesystem, so an
  in-sandbox COPY/ATTACH cannot reach the host filesystem.
- **Request wall**: `WALL_BYTES` 64 MiB rejected before `gl_alloc`
  (`src/bin/graphlaw_run_eh.rs:28`); ABI caps `abi::MAX_*` (`src/abi.rs`).
- **Honest scope note (from SECURITY.md)**: no fuel/epoch metering (a wasm
  infinite loop pins the thread); no store memory ceiling; `inherit_env()`
  passes the full host environment to the guest
  (`src/bin/graphlaw_run.eh.rs:236` — disk spelling is `graphlaw_run_eh.rs:236`);
  `GRAPHLAW_ATTACH_ROOTS` is host-side only, it does not follow a preopen into
  the guest.

### 1.5 Resilience to failures — atomic batch abort + rollback + partial receipts

- **Atomic document exec**: wire `"atomic": true` — success carries
  `status`/`state_delta`; a mid-sequence failure carries a partial receipt
  marked `status:"aborted"`, `state_delta:0` (atomic rollback). Code:
  `src/duckdb_op.rs` (SQLRDF-26 module doc + batch paths :467-485); court
  `document_exec_atomic_wire_flag_aborted_partial_and_success_fields` at
  `src/duckdb_op.rs:1253-1290` asserting real envelopes (aborted partial
  receipt with state_delta 0, plus the pipeline byte-identical twin).
- **Typed refusals before engine contact**: SQLRDF-17 raw-SQL refusal before
  the engine sees a statement (`src/duckdb_op.rs:359-369`, :394-412);
  ATTACH allowlist applied "before the engine sees them"
  (`src/duckdb_op.rs:398-399`); batch members pass the ATTACH allowlist
  one-for-one before the engine (`src/duckdb_op.rs:467-468, :480-485`).
- **Typed refusal register**: every refusal has a stable code —
  `docs/refusals.md` (REFUSAL-01..04 etc., append-only); catalog
  `docs/reference/refusal-catalog.md`, exit codes
  `docs/reference/runner-exit-codes.md`; register audit receipt
  `receipts/refusal-register-audit-2026-10-22.md`.

---

## 2. Cybersecurity (Art. 15(5)–(6))

### 2.1 SECURITY.md sandbox analysis

- `SECURITY.md` — scope (untrusted plan/policy/JSON ABI input, receipt forgery,
  resource exhaustion, egress via COPY/ATTACH, sandbox escapes), known limits
  (plain receipts prove presence only; authenticated use needs the Ed25519
  signed API `src/attest.rs`; wasm ABI has no signing op;
  `unverified_lease: true` opts out of lease signature checks), and the
  EH-host sandbox analysis with the 28-import inventory above.
- Security policy + supported versions table gated by
  `tests/docs_release_claims.rs` (version-table moves with workspace bumps).

### 2.2 Admission SHACL pre-flight

- SHACL validation gates admission (`src/law.rs` `Step::AdmitShacl`);
  bounded shape-nesting guard `MAX_SHACL_VALIDATION_DEPTH = 16` with typed
  refusal on depth breach (`src/shacl_guard.rs:21-38, :71-82, :161-181`) —
  the guard's own recursion is bounded (no stack-exhaustion DoS through the
  evaluator).
- SHACL WHERE shapes surface: `ggen sync run` projection + `ast_governance`
  court green (EH-closure wave, `receipts/2026-10-23-eh-closure-wave.md` §4).
- Injection refusal at the shapes layer: `InvalidIdentifier` /
  `ShaclViolation` typed refusals for SQL-injection tokens
  (`receipts/gws-9-wasm-boundary-fuzz.md` findings §1/§3).

### 2.3 SQLRDF-17 raw-SQL refusal

- Raw SQL text is not an admitted admission path; the Turtle AST document is
  the admission record — typed refusal
  `src/duckdb_op.rs:359-369` (legacy `ast.text`), :412 (wire-param/literal
  mismatch), :480-485 (document exec never binds wire params). Module law at
  `src/duckdb_op.rs:11-18`.
- **ATTACH allowlist (SQLRDF-22 G6)**: env `GRAPHLAW_ATTACH_ROOTS` colon-separated
  absolute roots; env unset/empty refuses EVERY ATTACH (fail-closed default) —
  `src/duckdb_op.rs:214-247` (`enforce_attach_policy` + pure core refusing with
  "attach path outside GRAPHLAW_ATTACH_ROOTS allowlist"); applied before the
  engine sees the statement (:398-399); court 13/0 (W15,
  `receipts/2026-10-23-eh-closure-wave.md` §4 table).

### 2.4 IPC injection court

- `tests/gws4_ipc_injection_court.rs:1-15` — the in-wasm duckdb cannot reach
  the host filesystem, so data crosses as a linear-memory buffer
  (`gl_alloc` → host `memory.write` → `gl_call(ptr,len)` → response), sized
  against `MAX_REQUEST_BYTES` 16 MiB; the duckdb op dispatches in-module with
  TYPED responses (mode validation / SQLRDF-17 raw-SQL refusal / resource-limit
  refusal), never a trap; instance survives to the next benign call. Receipt:
  `receipts/gws4-ipc-injection-court-2026-10-22.md`. Typed-loud SKIP arm for
  artifact-hosting limits (`[SKIP]` with reason, never a faked pass) at
  `tests/gws4_ipc_injection_court.rs:358`.

### 2.5 No ambient authority

- Import allowlist fail-closed to `wasi_snapshot_preview1`
  (`src/bin/graphlaw_run_eh.rs:222-239`), zero-preopen
  (`SECURITY.md:62-64`), ATTACH fail-closed with env unset
  (`src/duckdb_op.rs:220-229`), lease signature checks with the
  `unverified_lease` opt-out **disclosed as a limit** in `SECURITY.md` known
  limits (not silently available), Ed25519 attestation gate
  `src/attest.rs` + `./target/debug/graphlaw-verify` (CLAUDE.md command table),
  receipt chain `src/receipt_chain.rs` + `receipts/CHAIN.blake3`.
- Authority manifest court: `tests/authority_manifest.rs`;
  egress/attach policy doc: `docs/how-to/egress-and-attach-policy.md`
  (scalar-fallback COPY caveat disclosed at `SECURITY.md` Scope —
  hardcoded `/tmp/graphlaw-rdf-<table>.csv` destination,
  `src/sql_ast.rs:2255-2258`, a known lawful egress hole with a documented
  preferred form).

---

## 3. Accuracy (Art. 15(3)-(4): accuracy throughout the lifecycle)

### 3.1 p28 determinism court

- `tests/p28_determinism_court.rs:1-12` (TC-LEGAL-01) — identical §7 MERGE AST
  graph + identical parameters → 100% byte-identical
  `H_query = BLAKE3(Canon(G_AST) || P_params)` across runs; 10,000-iteration
  hash-stability leg + 1,000 real `compile_turtle` end-to-end leg.

### 3.2 Two-verifier agreement

- `tests/two_verifier_court.rs:1-12` (WO-08, TC-MEGA-FIBO-01) — two independent
  frozen-duckdb verifier instances agree byte-for-byte on `H_query`
  (sql-rdf Rec §1.4 two-node form); Verifier A executes, Verifier B re-derives
  `h_ast` from carried canon bytes alone.

### 3.3 Canonical parity

- `tests/canonical_cross_dialect.rs:1-4` (R03) — RDFC-1.0 + BLAKE3 canonical id
  (`LawState::from_dataset`) stable across named-graph isomorphism and dialect
  routes; anti-vacuity: a one-quad perturbation MUST move the canonical id
  (`tests/canonical_cross_dialect.rs:287` "court is VACUOUS" guard).
- Receipt hash law: `H_query = BLAKE3(Canon(G_AST) || P_params)` via
  `purrdf::try_canonicalize` (CLAUDE.md Receipts convention).
- sql-rdf conformance receipts: `receipts/sql-rdf-conformance-2026-10-22.md`,
  `receipts/sql-rdf-dialect-2026-10-22.md`;
  conformance text: `docs/reference/sql-rdf-rec.md` (cited by section).

---

## 4. Honest gaps (count: 3)

1. **SPC Cpk under load — FAIL standing.** Latest SPC receipt
   (`receipts/spc-2026-10-23T152558.json`, tc_hot_01 / clone_path / SMOKE):
   Cpk 0.0008 vs the 1.33 requirement, stddev 16,282.67 ns, rule-1
   beyond-3σ flagged (33 points), `verdict: "FAIL"`. The gate exiting 1 is the
   gate working (`scripts/spc_court.sh`, Cpk ≥ 1.33); the underlying
   process-capability claim under load is NOT met. Disclosed in
   `docs/jira/v26.10.23/compliance-eu-ai-act.md:112-116, :235` as Art. 9(7)(a)
   honest-FAIL.
2. **read_frame unbounded prefix — designed fix exists, not landed.** Gap G4 in
   `docs/reference/wasm-gap-register.md:14` (OPEN): `run_eh.rs:336-350`
   read_frame unbounded alloc; closing witness "bound court: exit 92". Designed
   fix documented at `docs/jira/v26.10.23/design-read-frame-bound.md`.
3. **EH-19 (tc08 COPY egress EINVAL) — status divergence between dispatch and
   disk.** The dispatch for this lane said EH-19 is closed; on-disk record as
   of this read: OPEN — `docs/jira/v26.10.23/ROADMAP.md:227` ("EH-19 open"),
   `docs/jira/v26.10.23/ADR-egress-op.md:20-22` (raw EINVAL, W59) with the
   typed-refusal closure planned at :49, :110. No closure receipt found in
   `receipts/`. Recorded here as OPEN-on-disk; flip is receipt-gated.

Cross-cutting residual (disclosed in SECURITY.md, not counted as a gap row
since it is a documented product limit, not an open defect): no fuel/epoch
metering, no store memory ceiling, `inherit_env()` passes the full host env.

---

## 5. Section and gap summary

| Art. 15 requirement | Standing | Primary evidence locus |
|---|---|---|
| Robustness (memory-wall plateau 8/0) | Evidenced | `tests/wasm_memory_wall.rs`; RECEIPTS.md:1585 |
| Robustness (boundary fuzz 1,172/0) | Evidenced (aggregate; 312/run deterministic) | `receipts/gws-9-wasm-boundary-fuzz.md`; RECEIPTS.md:1279,1466 |
| Robustness (hostile SQLRDF corpus) | Evidenced | `tests/k5_hostile_sqlrdf.rs`; laundering receipt 23/0 |
| Robustness (sandbox 28-import / zero-preopen) | Evidenced | `SECURITY.md` inventory; `src/bin/graphlaw_run_eh.rs:222-239` |
| Robustness (atomic batch + typed refusals) | Evidenced | `src/duckdb_op.rs:1253-1290, :359-369, :467-485` |
| Cybersecurity (SECURITY.md analysis) | Evidenced | `SECURITY.md` |
| Cybersecurity (SHACL pre-flight) | Evidenced | `src/shacl_guard.rs`, `src/law.rs` AdmitShacl |
| Cybersecurity (SQLRDF-17 + ATTACH allowlist) | Evidenced | `src/duckdb_op.rs:214-247, :359-369` |
| Cybersecurity (IPC injection court) | Evidenced | `tests/gws4_ipc_injection_court.rs` |
| Cybersecurity (no ambient authority) | Evidenced | SECURITY.md; run_eh; duckdb_op |
| Accuracy (determinism courts ×3) | Evidenced | `tests/p28_determinism_court.rs`, `tests/two_verifier_court.rs`, `tests/canonical_cross_dialect.rs` |
| Gaps | 3 open | §4 above |

Sections: 4 evidence sections + gap section + summary table.
Gap count: **3** (SPC Cpk under load; read_frame unbounded prefix;
EH-19 open-on-disk vs closed-per-dispatch divergence recorded).
