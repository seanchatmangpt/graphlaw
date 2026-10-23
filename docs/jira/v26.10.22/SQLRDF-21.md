# SQLRDF-21 — Sovereign in-WASM DuckDB execution (EH wall closed)

- **Id:** SQLRDF-21
- **Title:** Sovereign in-WASM DuckDB execution (EH wall closed)
- **Status:** Open (courts landing this wave)
- **Base:** HEAD `dab6a73` (chore(snapshot): fleet wave safety snapshot —
  2026-10-22 annealing waves W/Z/AA/AB/K)
- **Design:** [designs/fibo-pm4py-court-matrix.md](designs/fibo-pm4py-court-matrix.md)

## Why this ticket exists

SQLRDF-08 was withdrawn because `graphlaw-wasm` is one unit and links
frozen-duckdb — the link is witnessed since frozen-duckdb 2.0.2 emits
`static=duckdb` for wasm32-wasip1. The exception-handling wall is closed by
the dual-artifact policy: the EH MIXED artifact was hostable by no available
runtime (node 26 V8 rejected the MIXED legacy+new artifact; wasmtime 48
needed `legacy_exceptions`), resolved by the A1' new-EH rebuild recipe
([designs/a1prime-trytable-recipe.md](designs/a1prime-trytable-recipe.md))
and the ENGINE-tier host decision
([designs/wasmi-eh-research.md](designs/wasmi-eh-research.md): wasmi 2.0.0
is a hard NO — no exceptions-proposal support; wasmtime 48 with
`wasm_exceptions` is YES, new EH `try_table`/`throw` default-on). The
remaining work is not admission — it is execution: the `duckdb` ABI op must
dispatch and run a full SQL lifecycle inside the module under an
EH-capable host.

## Conformance target

Section 5 of [`docs/reference/sql-rdf-rec.md`](../../reference/sql-rdf-rec.md)
(admission through `compile_turtle` inside the module) plus the refusal
register conventions of
[`docs/refusals.md`](../../refusals.md) — `REFUSAL-05`
(`UnboundedMutationRefusal`) is append-only and already registered
(`docs/refusals.md:80`).

## Current behavior (verified on disk, 2026-10-23)

- **In-module admission works** — `sql` documents admit through
  `compile_turtle` inside the wasm module (SQLRDF-08 withdrawal note). The
  graph-backed receipted path is
  `compile_turtle` → `canon_graph_bytes` → `admitted_statement_hash` over
  `BLAKE3(Canon(G_AST) || P_params)`
  (`src/duckdb_eval.rs:170-182`).
- **The `duckdb` ABI op is host-side today** — `src/duckdb_op.rs`
  (`dispatch`, `open_connection` at `src/duckdb_op.rs:74-82`, raw-SQL exec
  arm at `src/duckdb_op.rs:198-219`) runs on the host; the wasm module
  exposes admission, not a full in-module SQL lifecycle. F2 (SQLRDF-17)
  additionally shows per-call in-memory connections
  (`src/duckdb_op.rs:74-82`) mean catalog mutations are not replayable.
- **WHERE-less UPDATE/DELETE compile-refusal exists** — `REFUSAL-05`
  raises `sql_ast::AstError` (`src/sql_ast/refusals.rs`) before any engine
  contact (`docs/refusals.md:80`).
- **EH host landscape** — wasmi has no EH support; wasmtime 48 supports
  new EH (`wasm_exceptions` default-on behind crate feature `gc`)
  (`designs/v2030-zero-copy-protocol.md:122-125`,
  `designs/dual-artifact-policy-design.md:15`).

## Required behavior

1. **In-module dispatch, no cfg gate.** The `duckdb` op dispatches inside
   `wasm/src/lib.rs` with no `#[cfg(target_arch = "wasm32")]` strip, no
   `duckdb` feature, no `default-features = false` on
   [`wasm/Cargo.toml`](../../../wasm/Cargo.toml) — DuckDB is not a feature
   (repo convention).
2. **Full-lifecycle court under wasmtime 48.** A court runs
   CREATE TABLE → INSERT → SELECT → UPDATE inside the module and reads back
   real table state, hosted by wasmtime 48 with `wasm_exceptions` enabled
   (new-EH artifact per the A1' recipe).
3. **WHERE-less mutation refusal, byte-identical DB.** A WHERE-less
   UPDATE/DELETE AST document refuses `REFUSAL-05` before engine contact;
   the court asserts the in-module database state is byte-identical
   pre/post refusal (no partial effect).
4. **MERGE classified mutating with epoch bump.** MERGE is classified as a
   mutating statement class and its execution bumps the epoch counter, with
   a court witnessing the bump.
5. **Artifact re-pin with witnessed receipt.** The relinked artifact digest
   is re-pinned (`priv/graphlaw.wasm.blake3`,
   `registry/ARTIFACTS.blake3`) with a receipt embedding real command
   output (digest + build command + exit code) in
   [RECEIPTS.md](RECEIPTS.md).
6. **No raw-SQL admission.** SQLRDF-17 stands — the in-module op accepts a
   Turtle AST document via `compile_turtle`, never bare SQL text.

## Acceptance

- (a) `duckdb` op dispatches in-module, no cfg gate (inspect
  `wasm/src/lib.rs` + `wasm/Cargo.toml`).
- (b) full-lifecycle court CREATE/INSERT/SELECT/UPDATE green under
  wasmtime 48 with `wasm_exceptions`.
- (c) WHERE-less UPDATE/DELETE refused `REFUSAL-05` pre-engine with DB
  byte-identical pre/post.
- (d) MERGE classified mutating with epoch-bump court.
- (e) artifact re-pinned with witnessed receipt (embedded output, not a
  bare claim).
- (f) pm4pytest FIBO matrix green
  ([designs/fibo-pm4py-court-matrix.md](designs/fibo-pm4py-court-matrix.md)).

## Constraints

- **DuckDB is not a feature** — frozen-duckdb stays a normal dependency in
  every target, including wasm; never gate the SQL surface out.
- **Refusal codes are append-only** (`docs/refusals.md`) — never renumber.
- **No raw-SQL admission** — SQLRDF-17 stands; the legacy `(None, None)`
  hash arm (`src/duckdb_eval.rs:183-188`) is the fingerprint of the hole.

## Already-implemented boundary

The wasm link (frozen-duckdb 2.0.2, `static=duckdb`), the EH host
decision, the A1' new-EH recipe, the `REFUSAL-05` register row, and the
graph-backed receipted hash path are landed and receipted. This ticket
does not re-ask for any of them; it asks for the in-module dispatch and
the six acceptance courts above.

## Implementation notes (session mode, landed 2026-10-23)

The host-side `duckdb` op gained a named-session surface
(`src/duckdb_op.rs`), landing the DDL→DML→SELECT lifecycle for acceptance
item (b) on the host rung:

- `mode: "open"` registers a named persistent in-memory connection
  (`open_session`, `src/duckdb_op.rs:522-535`); duplicate names and the
  16-session cap (`MAX_SESSIONS`, `src/duckdb_op.rs:79`) are typed
  refusals.
- `exec`/`stream` accept an optional `"session"` field and run against
  that connection instead of a per-call fresh one (`exec` session branch,
  `src/duckdb_op.rs:318-331`); absent field keeps the back-compat
  fresh-connection behavior (`no_session_stays_fresh_connection`).
- `mode: "close"` drops a session idempotently (`close`,
  `src/duckdb_op.rs:540-556`); the registry is process-local crash-only
  (`sessions()`, `src/duckdb_op.rs:83-85`).

Witnessed courts (2026-10-23, real runs this session):

- `cargo test --lib duckdb_op` → `13 passed; 0 failed` — includes the 5
  session unit courts in `src/duckdb_op.rs`
  (`session_lifecycle_create_insert_select` at
  `src/duckdb_op.rs:679` asserts the exact readback through open → CREATE →
  INSERT → SELECT on one named session; plus back-compat
  (`no_session_stays_fresh_connection`, :749), unknown-session
  (:789), duplicate-open (:808), close-frees (:833) refusal courts).
- `cargo test --test wasm_duckdb_op` → `9 passed; 0 failed`, including
  `abi_session_lifecycle_create_insert_select_readback` at
  `tests/wasm_duckdb_op.rs:531`.
- `cargo test --lib sql_ast` → `25 passed; 0 failed`;
  `cargo test --test chicago_validation` → `12 passed; 0 failed`;
  `cargo test --test sql_rdf_dialect` → `36 passed; 0 failed`.

The previously-reported compile blockers (`unsafe { env::remove_var }`,
missing `Serialize` impl) no longer exist in `src/duckdb_op.rs`; the
module compiles and greens clean. Wire shapes are documented for
consumers in `docs/DIALECTS.md` (DuckDB op session mode section).

Still pending on this ticket: acceptance courts A/B (in-module dispatch
under wasmtime 48) and the artifact re-pin (item (e)) — the session mode
above is host-side and does not close (a)/(b)-in-module/(e).
