# Refusals Register Reconciliation — lane L72, v26.10.22

Subject: `/Users/sac/graphlaw` @ main (uncommitted working tree; coordinator owns git).
Receipt hash of the register after edit:
`76946732e9cc2fa9bc6084660f3c2f8c21587893781ca10df1cfa5446fdecd14  docs/refusals.md` (historical)
(shasum -a 256, this session).

## Method

Enumerated every refusal surface on disk, grepped each against
`docs/refusals.md`, appended missing kernel entries, added a separated
harness section, and verified the appended v26.10.22 table's discriminant
numbers against `src/sql_ast.rs`.

## Reconciliation matrix

| # | Surface (disk) | Evidence (grep, real exit 0) | In register? | Action |
|---|---|---|---|---|
| 1 | `RefusalCode` enum, `src/sql_ast.rs:94` (`MalformedAst=1`, `UnimplementedStatement=2`, `ScopeIntegrity=3`, `Capability=4`, `InvalidIdentifier=16`, `MissingProperty=17`, `RecursionLimitExceeded=18`, `MalformedList=19`, `UnsupportedTerm=20`, `MalformedNode=21`, `LiteralNotInt=36`, `LiteralNotFloat=37`) | `grep -n "LiteralNotInt = \|LiteralNotFloat = \|MalformedAst = " src/sql_ast.rs` → lines 96/118/121 | REFUSAL-01..04 + L4 table | L4 table verified CORRECT (see below); no change |
| 2 | `refusal_arm` consts incl. `JOIN_TYPE_IRI: u16 = 37` (`src/sql_ast.rs:65`) | same grep, line 65 | L4 table row 3 | Verified CORRECT; no change |
| 3 | `EvaluatorError::NoCanonicalGraph` (`src/duckdb_eval.rs:91`, raised at :542) | grep → enum + Display + raise site + courts (`tests/gws4_ipc_injection_court.rs:169`, `tests/query_cache_court.rs:56`) | MISSING | APPENDED as `NO-CANONICAL-GRAPH` (kernel section, L72) |
| 4 | `attest::AttestError` (`BadSignature`/`UnknownKey`/`Malformed`, `#[non_exhaustive]`, `src/attest.rs:39`) | grep + `cargo test --test trust_attest` → **10 passed; 0 failed** | reasons only (under `ReceiptRefused`) | APPENDED as `ATTEST-REFUSED` (kernel section, L72) |
| 5 | dialect `RefusalKind` / `ReasonClass` (`src/dialect.rs:129/213`) | grep | Present (`RefusalKind` section + `ResourceLimit`/`Refused` rows) | no change |
| 6 | duckdb_op refusals (`src/duckdb_op.rs`: `RefusalKind::DuckDb`, `duckdb_chunk_rows`/`duckdb_chunk_bytes` limits, `DUCKDB_MEMORY_LIMIT`) | grep `RefusalKind::` → :61 | Present (`DuckDb` row) | no change |
| 7 | admission-seal `ReceiptError(String)` (`src/duckdb_eval.rs:83`) | grep | Present (`ReceiptRefused`/receipt reasons) | no change |
| 8 | `build.rs:31` `WASI_PATCH_BUILD_REFUSED` | grep → :31 | MISSING | APPENDED in new harness section |
| 9 | `scripts/mutation_court.sh:65` `MUTATION-COURT-DIRTY-TREE` (exit 6) | grep → :61-68, `exit 6` | MISSING | APPENDED in harness section |
| 10 | `tests/common/mod.rs:55/66/78` `GRAPHLAW-WASM-BUILD-FAILED` + `BLOCKED-LINK` | grep → :55,66,78 | MISSING | APPENDED in harness section |
| 11 | `scripts/doc_staleness_gate.sh` exits 1/2 (rules R1–R4) | read script header + ran it (below) | MISSING | APPENDED as `DOC-STALENESS-FINDINGS` |
| 12 | `MANIFEST-REFUSED` / `DISK-REFUSED` | repo-wide grep: only `docs/jira/v26.10.22/designs/rules-pruner-design.md:114` — DESIGN-STAGE, not implemented | MISSING | APPENDED in harness section, marked DESIGN-STAGE |

Matrix counts: 12 surfaces enumerated; 5 register additions (2 kernel,
3 harness + 1 design-stage row); 1 design-stage surface with no on-disk
implementation; 0 renumberings; 0 CONFIRMED discriminant-number defects.

## Item 3 — L4 table code verification (register correction check)

Appended v26.10.22 table vs disk:

| Register claim | Disk (`src/sql_ast.rs`) | Verdict |
|---|---|---|
| `LITERAL_NOT_INT (RefusalCode = 36)` | `LiteralNotInt = 36` (line 118) | MATCH |
| `LITERAL_NOT_FLOAT (RefusalCode = 37)` | `LiteralNotFloat = 37` (line 121) | MATCH |
| `JOIN_TYPE_IRI (refusal_arm = 37)` | `pub const JOIN_TYPE_IRI: u16 = 37` (line 65) | MATCH |

All three match: **no wrong-number defect; nothing corrected in place.**
The existing "Appended internal discriminants" table is left byte-identical.

## Embedded command output (receipt body)

- `grep -n "LiteralNotInt = \|LiteralNotFloat = \|MalformedAst = \|JOIN_TYPE_IRI: u16" src/sql_ast.rs`
  → `65: pub const JOIN_TYPE_IRI: u16 = 37;` / `96: MalformedAst = 1,`
  / `118: LiteralNotInt = 36,` / `121: LiteralNotFloat = 37,` / `exit=0`
- `grep -n "REFUSAL_04_CAPABILITY_REFUSAL" src/tier.rs` → `25: pub const REFUSAL_04_CAPABILITY_REFUSAL: &str = "REFUSAL-04";`
- `bash scripts/doc_staleness_gate.sh` → `doc_staleness_gate: 61 finding(s)`, **exit 1**.
  All 61 findings are PRE-EXISTING, in other files (`docs/wasm-integration.md`,
  `docs/jira/v26.10.22/RECEIPTS.md`, designs/, mega/) — zero findings in
  `docs/refusals.md`. The gate can exit 1, so it is not vacuous; the findings
  belong to other lanes. (As-of 2026-10-22, this session.)
- `cargo test --test trust_attest` → `test result: ok. 10 passed; 0 failed; 0 ignored; ... finished in 0.08s`
- `cargo test --test gws4_ipc_injection_court` → `test result: FAILED. 1 passed; 2 failed`
  — **PRE-EXISTING environment defect, not session-introduced**: the
  frozen-duckdb wasm cache holds a host archive
  (`/Users/sac/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/libduckdb.a is not a wasm32 archive ... rebuild with WASI_SDK_PATH set`),
  so the wasm-linked tests panic at `frozen-duckdb-sys/build.rs:97`. This is
  the `GRAPHLAW-WASM-BUILD-FAILED (BLOCKED-LINK)` harness refusal firing in
  the wild — it validates harness row 10 above. 1 of 3 tests (the
  fail-closed refusal logic) passed. Recovery is environmental (rebuild the
  wasm archive with `WASI_SDK_PATH`), outside L72 file ownership.
- Register row grep: `grep -n "NO-CANONICAL-GRAPH\|ATTEST-REFUSED\|WASI_PATCH_BUILD_REFUSED\|MUTATION-COURT-DIRTY-TREE\|GRAPHLAW-WASM-BUILD-FAILED\|BLOCKED-LINK\|MANIFEST-REFUSED\|DISK-REFUSED\|DOC-STALENESS" docs/refusals.md` → 7 rows, `exit 0`.

## Files touched

- `docs/refusals.md` — appended only (append-in-place; no renumbering, no
  status hand-edits): 2 kernel rows (`NO-CANONICAL-GRAPH`, `ATTEST-REFUSED`)
  + 1 new "Harness refusal surfaces" section (5 rows incl. the design-stage
  pair). Σ_PQC section untouched: unquoted here except by pointer — the
  register's §1.4 second formula stays UNMET; no scheme, parameter set, or
  modulus defined.
