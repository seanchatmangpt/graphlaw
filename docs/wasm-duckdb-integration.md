> **SUPERSEDED 2026-10-23**: the EH wall this design worked around is closed —
> single DuckDB-linked artifact, wasmtime 48 `wasm_exceptions`, archive law
> `-mllvm -wasm-use-legacy-eh=0` (see `docs/WASM.md`, `docs/wasm-integration.md`,
> eh-runner-decision-memo.md DECISION block, SQLRDF-21). Sections describing
> per-call in-memory connections are also superseded by the session surface
> (mode `open`/`exec-with-session`, `src/duckdb_op.rs`).

# WASM + DuckDB Integration Wave (GWS-1..12)

Standing: OPERATOR-CHARTER — documented from landed artifacts only
(smalt-repo: `docs/jira/v26.10.21/WASM-SANDBOX-SPEC.md`,
`docs/jira/v26.10.21/GRAPHLAW-SPEC.md`,
`docs/jira/v26.10.21/COLDCHAIN-CASE-STUDY.md`; graphlaw-repo:
`docs/wasm-integration.md`, `wasm/src/*.rs`). Every wave item carries a typed
status token (`PENDING-GWS-n` until its receipt lands, then replaced by the
typed verdict from that receipt). Flip state as of 2026-10-21 (WRAP-2):
GWS-5, GWS-6, GWS-9, GWS-10, GWS-11, GWS-12 have landed receipts/courts and
are flipped below; GWS-1, GWS-2, GWS-3, GWS-4, GWS-7, GWS-8 remain
`PENDING-GWS-n`.
Context: full · Milestone window: v26.10.21 (smalt) / graphlaw

## 1. What is landed vs pending

Landed (sources read on disk, 2026-10-21):

- `wasm/` crate (`graphlaw-wasm`): generated shell (`ffi.rs`, `abi_meta.rs`
  rendered by `ggen sync` from `ontologies/graphlaw-wasm.ttl`) over the
  hand-written safe core `graphlaw::abi` — gl_alloc/gl_call/gl_free
  packed-u64 ABI, 16 MiB request / 256 MiB outstanding / depth-64 limits.
- `lib/smalt/wasm/{bridge,pool,worker}.ex` (Elixir): supervised worker pool
  per `WASM-SANDBOX-SPEC.md` (16 pre-warmed workers, trap_exit respawn,
  `:pool_exhausted` typed when drained).
- Landed DuckDB Integration Core: `src/sql_ast.rs` (declarative AST compiler & SHACL pre-flight gate), `src/duckdb_eval.rs` (vectorized query evaluator with `ValueRef` inspection & Parquet view registration), and `src/rules/builtins.rs` (`builtin_eval_sql`).
- Specs: `WASM-SANDBOX-SPEC.md` (sandbox topology, 64 MiB wall, 3-gate
  court) and `GRAPHLAW-SPEC.md` (GL-1..12 guard-seam topology, ANDON wire
  face, GL-6 dual-governance parity with frozen-duckdb audit filtering).
- Case study: `COLDCHAIN-CASE-STUDY.md` (OCEL 2.0 → GraphLaw →
  frozen-duckdb → N3 admission pipeline).

Pending: the GWS-1/2/3/4/7/8 receipt set. A status token flips
`PENDING-GWS-n` → typed verdict only when a witness receipt for that lane
lands on disk; no receipt, no flip.

## 2. Wave item register

| Item | Scope | Landed artifact base | Status |
|---|---|---|---|
| GWS-1 | Integration map / lane partition | specs above | PENDING-GWS-1 |
| GWS-2 | Build outcomes (single `graphlaw-wasm` module, frozen-duckdb as normal dependency; wasm32-wasip1 link is GREEN with `WASI_SDK_PATH` set — exits 0 as of 2026-10-22, full DuckDB linked; fails typed at frozen-duckdb-sys `build.rs` without it; re-pin pending the reproducible-SDK rebuild) | `wasm/Cargo.toml`, smalt `lib/smalt/wasm/` | PENDING-GWS-2 |
| GWS-3 | ABI `duckdb-op` design | `docs/wasm-integration.md` gl_* ABI | PENDING-GWS-3 |
| GWS-4 | Data-injection design (Parquet/views into guest linear memory) | COLDCHAIN-CASE-STUDY.md §2 | PENDING-GWS-4 |
| GWS-5 | SHACL in-wasm court | `shapes/ast_governance.ttl` (case study §4) | PASS — `tests/wasm_governance_preflight.rs` 3/3 (re-run 2026-10-21, this flip) |
| GWS-6 | N3 in-wasm court | `rules/gxp_compliance.n3` (case study §5) | PASS — `tests/wasm_n3_admission.rs` 3/3 incl. native differential (re-run 2026-10-21; receipt: GWS-12 finding 3) |
| GWS-7 | Bridge alignment (smalt_wasm_bridge ↔ gl_* ABI) | `lib/smalt/wasm/bridge.ex` | PENDING-GWS-7 |
| GWS-8 | Perf differential (guarded vs raw, and wasm vs native) | GRAPHLAW-SPEC GL-9 methodology | PENDING-GWS-8 |
| GWS-9 | Security fuzz | GL-10 skeptic methodology | PASS — receipt `receipts/gws-9-wasm-boundary-fuzz.md`; 312 hostile calls / 0 panics, `tests/wasm_boundary_fuzz.rs` 2/2 (re-run 2026-10-21) |
| GWS-10 | E2E (wire → pool → wasm → refusal envelope) | WASM-SANDBOX-SPEC 3-gate court | PASS — receipt `receipts/gws-10-e2e-coldchain-wasm.md`; `tests/wasm_coldchain_e2e.rs` 3/3 (re-run 2026-10-21); duckdb phases 3-5 typed PENDING-GWS-3/4 (host stub, honestly carried) |
| GWS-11 | Memory wall (64 MiB hard limit) | WASM-SANDBOX-SPEC gate 2 | PASS — receipt smalt `receipts/gws11-wasm-memory-wall.md`; 7/7 with `GRAPHLAW_WASM=priv/graphlaw.wasm` (re-run 2026-10-21); enforced-boundary map in receipt |
| GWS-12 | Determinism cross-engine (wasm vs native same-input) | wasm-integration.md compile profile | PASS — receipt smalt `receipts/gws12-cross-engine-determinism.md`; 3-leg partition-identical (native / wasm / COLD-1), `tests/gws12_coldchain_cross_engine.rs` 2/2 (re-run 2026-10-21) |

## 3. Integration build order

GWS-1's map receipt has not landed (`PENDING-GWS-1`); the order below is the
provisional build order derived from the landed specs and must be confirmed
or amended by the GWS-1 receipt before it is treated as authoritative.

1. Host the `graphlaw-wasm` module under wasmtime
   (`--max-memory 67108864`) behind `Smalt.Wasm.Worker` (port driver,
   packet-4 stdio, 5 s execution timeout, fail-closed).
2. Compile the WASM side. (Historical name, 2026-10-22 note: an earlier
   draft referred to `graphlaw_duckdb.wasm` — a historical name, retired;
   see the correction below.) Historically there is no separate
   `graphlaw_duckdb.wasm` (historical name) target:
   the unit is ONE `graphlaw-wasm` module, and frozen-duckdb is a normal
   dependency of the `graphlaw` crate it compiles (no feature gate) — the
   module dispatches duckdb itself. Build prerequisite: `WASI_SDK_PATH` must
   point at a wasi-sdk root (`bin/clang`, `share/cmake/wasi-sdk-p1.cmake`;
   CI installs wasi-sdk 34 at `/opt/wasi-sdk`). The engine cache
   `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1` is not a wasi-sdk.
   frozen-duckdb 2.0.2 emits `static=duckdb`, and the link then succeeds
   (witnessed exit 0, 2026-10-22 — full DuckDB in the module, exports
   `gl_alloc`/`gl_call`/`gl_free`; lane byte count in `docs/WASM.md`). Without
   `WASI_SDK_PATH`, the build fails typed at frozen-duckdb-sys `build.rs`
   (`WASI_SDK_PATH required`) — no `-lduckdb` linker scatter. The
   reproducible-SDK rebuild receipt is still GWS-2's deliverable; until it
   lands the on-disk `priv/graphlaw.wasm` pin (`332ea218…`) remains the last
   linked artifact and re-pin is pending (never claim re-pin done without the
   real artifact).
3. Extend the gl_* JSON ABI with a `duckdb-op` request kind (GWS-3's
   deliverable): request names the registered query artifact and its
   parameters; the guest runs frozen-duckdb pushdown and returns rows or a
   typed refusal under the same packed-u64 response contract
   (`(out_ptr << 32) | out_len`).
4. Data injection (GWS-4): mount pre-partitioned Parquet as zero-copy
   views; the request carries parameters only — bulk data never crosses the
   boundary through gl_alloc, keeping the 16 MiB request cap meaningful.
5. In-wasm courts for SHACL (GWS-5) and N3 (GWS-6) on the guest side,
   mirroring the case-study pipeline sections.
6. Bridge alignment (GWS-7): `Smalt.Wasm.Bridge` focus-node packing emits
   requests the duckdb-op surface accepts; refusal mapping stays
   wire-exact (422 + `x-ms-error-code` HTTP; AMQP `rejected` +
   `smalt:business-rule-failed`).
7. Qualification courts: perf differential (GWS-8), security fuzz (GWS-9),
   E2E (GWS-10), memory wall (GWS-11), determinism cross-engine (GWS-12).

## 4. Case-study cross-reference

`/Users/sac/smalt/docs/jira/v26.10.21/COLDCHAIN-CASE-STUDY.md` is the
domain worked example for this wave: the OCEL 2.0 → GraphLaw →
frozen-duckdb → N3 pipeline the GWS lanes prove inside the sandbox. Mapping
to this register:

| Case-study section | Wave item |
|---|---|
| §2 Parquet partitioned views | GWS-4 (data injection) |
| §3 SQL AST in RDF | GWS-3 (duckdb-op ABI) |
| §4 SHACL pre-execution gate | GWS-5 |
| §5 N3 deductive admission | GWS-6 |
| §6 execution trace (78% I/O reduction, 1.842 ms) | GWS-8 (perf differential) |
| §7 fail-closed GxP properties | GWS-9 (fuzz), GWS-10 (E2E) |

Standing of that case study is OPERATOR-CHARTER reference design — its
numbers (78% I/O reduction, 1.842 ms) are design targets, not measured
results for this wave; measured numbers require the GWS-8 receipt.

## 5. Receipt convention

Each GWS-n lands a witness receipt (graphlaw
`receipts/<gws-n>-*.md` or smalt
`docs/jira/v26.10.21/receipts/gws<n>-*.md`) with real command output and a
standing line. This document's status tokens then flip per the standard
no-receipt-no-flip rule. This doc is NOT in the generated set — it is
updated by hand, token by token, only when the corresponding receipt exists
on disk.
