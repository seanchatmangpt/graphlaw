# Release Notes — graphlaw 26.10.22 (FINAL)

Workspace version: `Cargo.toml:3` (`version = "26.10.22"`).
Source of truth: `CHANGELOG.md` — the [26.10.22] entry carries this release
(including the EH-closure/sequences/atomic-mode/window-CTE wave, merged from the
former Unreleased section at publish staging).
Cross-checked against `docs/jira/v26.10.22/RECEIPTS.md` (as-of 2026-10-22, and
re-verified on disk by the docs lane).

## Highlights

- **SQL-in-RDF compiler with typed refusals.** Declarative SQL-in-RDF compiler
  (`src/sql_ast.rs`): projection, aggregations (`COUNT`/`SUM`/`AVG`/`MIN`/`MAX`),
  joins, parameterized `MERGE INTO`, 35 statement classes extracted from the
  DuckDB 2.0 PEG. SHACL pre-flight (`validate_ast_shacl`, strict closed-world)
  rejects malformed ASTs, injection-shaped queries, and unbounded
  `WHERE`-less statements. Failures are typed, not stringly: the refusal register
  `REFUSAL-01..04` (`docs/refusals.md`) — malformed-AST, unimplemented-statement,
  scope-integrity, capability — with `AstError { code, ctx }` pure-data errors
  (Copy, no heap). Legacy raw-SQL arm deleted; `compile_turtle` is the sole
  admission path (`src/duckdb_op.rs:211-222`).
- **frozen-duckdb embedded, unconditionally.** `frozen-duckdb 2.0` is built into
  GraphLaw as the third admitted upstream engine with no feature gate. Vectorized
  evaluator (`src/duckdb_eval.rs`) reads `ValueRef` variants directly into typed
  RDF literals, scans Hive partitions, registers Parquet views. `builtin_eval_sql`
  bridges columnar results into PurRDF/N3 deduction state. The dependency is
  pinned with registry checksums (crates.io 2.0 series; note: this tree's
  manifest `[patch]` section redirects frozen-duckdb to a local checkout for
  local builds — see CHANGELOG [26.10.22] correction).
- **SPSC evidentiary ring.** `src/evidence_ring.rs`, optional feature
  `evidentiary-ring` (default-off). Never-block property fence (dev ratio leg +
  release absolute leg, witnessed 1.584 µs < 10 µs, receipt:
  `docs/jira/v26.10.22/designs/cold-path-ledger.md:91-95`) and ring-to-ledger
  joint court (max push 292 ns; drop gaps persisted exactly; RECEIPTS.md §8-9).
- **Single-unit wasm architecture.** One executable law runs on host and wasm:
  the same artifact admitting SQL-in-RDF inside `graphlaw-wasm`
  (`wasm/src/lib.rs`). Current state: the committed artifact `priv/graphlaw.wasm`
  was re-pinned at `332ea218…2c30c` (historical); the SHIPPED artifact is `58a44095…d2280a` (58,124,612 B, commit 499e759), RECEIPTS.md security-wave rows);
  the wasm32-wasip1 link of the successor edits was pending at receipt time.
  State at drafting time: RED on this host — a fresh lane-isolated boundary
  build aborts in `frozen-duckdb-sys/build.rs:101` ("WASI_SDK_PATH is required
  to link libc++ into wasm32-wasip1"); the committed `priv/graphlaw.wasm`
  artifact was the last good build at that checkpoint (historical); the shipped artifact is `58a44095…d2280a`.

### What is NOT in this release (loud boundaries)

- The `generated_kinds` test in `duckdb_statement_kinds` fails on any host
  lacking `/Users/sac/frozen-duckdb/target/wasm-build/` — a typed environmental
  failure, not a pass; the parse-parity surface it guards is VERIFIED-EXTERNAL
  21/21 (RECEIPTS.md §6).
- Full anti-vacuity sweep UNMET for the wave; per-court inversions remain local
  evidence (RECEIPTS.md §9).

## Breaking / Changed

- `compile_turtle` is the sole admission path — legacy `ast.text` raw-SQL is
  refused with a typed pointer to `compile_turtle`/`ast.turtle`
  (`src/duckdb_op.rs:211-222`).
- Sealed `CompiledQuery`: the legacy absent-field receipt-hash arm is gone;
  statement receipts key on `H_query = BLAKE3(Canon(G_AST) || P_params)` with
  canon-once carry (`CompiledQuery.canon_graph_bytes`, `src/sql_ast.rs:839`).
- No `duckdb` feature: embedded `frozen-duckdb 2.0` is unconditional (no opt-in).
- Pure-data typed refusals: `AstError { code: RefusalCode, ctx: RefusalContext }`
  replaces stringly compiler errors; `docs/refusals.md` code tables regenerated
  from the binary code space.
- Statement receipt hash is the TC-HOT-01 two-node form, witnessed green in
  `sql_rdf_dialect` 25/0 + `chicago_validation` 9/0 including a 5-variant
  cosmetic-invariance set (RECEIPTS.md SQLRDF-09 row).
- `frozen-duckdb` pinned to the crates.io 2.0 series with checksums (local
  `[patch]` override on this tree, see above).
- WASM artifact re-pinned `58a44095…d2280a` (58,124,612 B; superseded historical pin `332ea218…2c30c`).

### Fixed (from CHANGELOG, receipt-backed)

- Fixture `statementIndex` backfill (chicago 9/0, 12:30).
- `MERGE USING` missing `SELECT` fix (chicago 9/0).
- Select-list REFUSAL-03 gap closed
  (`process_scope_column_in_select_list_is_refusal_03`, dialect 18/0).
- Escape-audit defects E1/E3/E4/E5 fixed with courts; E6 refuted (not a defect);
  E7 declared by design (RECEIPTS.md §4).
- Stale test targets `gws4`/`wasm_duckdb_op` fixed (12:45).
- Ring court: shutdown-always-drains fix; restart watermark persists crash loss
  as gap markers, never silent renumbering.

## Known limitations (disclosed, typed, ticketed)

- **COPY TO egress file-locking**: DuckDB's advisory file lock on COPY
  targets fails under wasi preview1 (`Could not set lock ... Invalid
  argument`) — witnessed deterministically across artifact lineages
  (`tests/wasm_fibo_egress_eh.rs`, red-team v20-v22 adjacent). In-memory
  execution, table readbacks, and all other courts are unaffected;
  tracked as SQLRDF-27 (reopened: lock-path defect distinct from the
  closed openat/fcntl translation). Workaround: host-side egress via
  the packet-4 result frames (rows stream to the host; no in-guest
  file lock involved).

## Licenses

MIT remains the code license. `LICENSE-P28-SIL.md` (P28 Sovereign Invariant
License) governs execution/use of the substrate. Composite licensing applies:
distribute code under MIT, execution/use governed by P28-SIL. Third-party W3C
conformance fixtures carry their own provenance and license
(`tests/fixtures/w3c/README`).

## PRE-PUBLISH CHECKLIST (operator)

1. **Publish order: `graphlaw-eyeron` → `graphlaw`.** eyeron keeps its own
   independent, upstream-vendored version (currently 0.7.8, per CHANGELOG
   versioning policy) and must be published first; `graphlaw` depends on it.
2. **cargo-deny residual decision.** The deny.toml residual advisories need an
   explicit operator decision (accept/diagnose) recorded before publish — a
   silent `cargo deny check` pass is not a decision.
3. **wasm link state at publish time.** RED as checked live 2026-10-22
   (lane-isolated build, `CARGO_TARGET_DIR=target-lane-notes`):
   `frozen-duckdb-sys/build.rs:101` panics — "WASI_SDK_PATH is required to link
   libc++ into wasm32-wasip1 (the staticlib references the C++ runtime)".
   Resolution before publish: set `WASI_SDK_PATH` and re-run the boundary build;
   if green, re-pin the artifact digest and re-run `cargo test -p
   graphlaw-wasm`. Do not publish while the link is red — the pinned artifact
   then predates the released source.
4. CONFIRMED at publish staging: the workspace version is bumped
   (26.10.21 → 26.10.22, `Cargo.toml:3`) and the CHANGELOG entry dating agrees
   (the [26.10.22] entry is dated 2026-10-22; the former Unreleased wave content
   was folded under it).
5. Re-run `bash scripts/mutation_court.sh` on the release tree if the §6.2 tpch
   hand-patch falsifier (RECEIPTS.md §9c) has not been closed.

## Verification (as-of receipt, drafting time)

- Workspace version at drafting time was `3:version = "26.10.21"`; since
  confirmed bumped to `3:version = "26.10.22"` on disk.
- wasm boundary build executed live lane-isolated during drafting
  (`CARGO_TARGET_DIR=target-lane-notes cargo build -p graphlaw-wasm --target
  wasm32-wasip1 --profile wasm`): FAIL —
  `frozen-duckdb-sys/build.rs:101: WASI_SDK_PATH is required to link libc++
  into wasm32-wasip1 (the staticlib references the C++ runtime)`; no artifact
  produced by this run. Consistent with RECEIPTS.md Successor §2 ("wasm link
  pending").
- Every performance number in these notes is receipt-cited (1.584 µs →
  `designs/cold-path-ledger.md:91-95`; 292 ns → RECEIPTS.md §8); numbers without
  receipts were omitted.
