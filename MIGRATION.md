# Migration to library-backed GraphLaw

This document migrates `graphlaw` **26.9.28 → 26.10.22** (workspace calendar
version). Each section header below carries the version pair it applies to;
a section without a version pair applies to the whole 26.9.28 → 26.10.22
move.

v26.9.28 intentionally removes the historical implementation API. Migrate by
using the authoritative surfaces re-exported by `graphlaw`.

| Removed GraphLaw implementation | Replacement |
| --- | --- |
| `parser/*`, N3 pest grammar | `graphlaw::n3` (Eyeron) |
| `reasoner/*`, `builtins/*`, `backwardchaining` | `graphlaw::n3` (Eyeron) |
| `datalog`, `aggregation`, rule indexing/fixpoint code | `graphlaw::datalog` (PurRDF) |
| `tripleindex`, encoder/decoder storage machinery | `graphlaw::rdf` (PurRDF) |
| `sparql/*`, `queryengine` | `graphlaw::sparql` (PurRDF) |
| `shacl/*` | `graphlaw::shacl` (PurRDF) |
| `shex_native`, `shexc_parser` | `graphlaw::shex` (PurRDF) |
| `owlrl/*` | `graphlaw::entailment` (PurRDF) |
| local RDF event/window plumbing | `graphlaw::events` where covered by PurRDF |
| relational / SQL engine | `graphlaw::sql_ast::compile_turtle` (sole admission; see below) |
| local WASM wrapper | build the library-backed crate for `wasm32-wasip1` directly (see below) |

`compile_turtle` compiles a Turtle document conforming to `ontologies/sql-ast.ttl`
into a `CompiledQuery`, evaluated by `graphlaw::duckdb_eval` over
`frozen-duckdb 2.0` (a normal, non-optional dependency).

The WASM module target is `wasm32-wasip1`:
`cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`.
Build prerequisite: `WASI_SDK_PATH` must point at a wasi-sdk root (`bin/clang`
and `share/cmake/wasi-sdk-p1.cmake`; CI installs wasi-sdk 34 at
`/opt/wasi-sdk` — the engine cache
`~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1` is not a wasi-sdk). With the
variable set the link is green; without it the build fails typed at
frozen-duckdb-sys `build.rs` (`WASI_SDK_PATH required`). Full runbook:
`docs/WASM.md` ("Build prerequisite"). `wasm32-unknown-unknown` is not a
supported module target — it needs a JavaScript host.

There is deliberately no compatibility implementation that can silently take
over when an upstream engine rejects an input. That prevents the same document
from acquiring different semantics depending on which internal path happened to
accept it.

GraphLaw-specific ontology, pack and query assets remain in the repository.

## Artifact digest migration: sha256 -> BLAKE3 (26.10.13 -> 26.10.14)

Artifact pins have moved from SHA-256 to BLAKE3 (b3sum format
`<64-hex>  <filename>`):

- `priv/graphlaw.wasm.blake3` (was a sha256 pin; now a b3sum pin)
- release-asset sidecars `*.blake3` produced by `b3sum` in `release.yml`
- `registry/ARTIFACTS.blake3`

Consumers and scripts must switch `shasum -a 256` / `sha256sum` to `b3sum`.
`b3sum` installs from crates.io: `cargo install b3sum --locked`. Node and
WebCrypto have no blake3, so Node-side verification uses the same b3sum binary
(CI installs it via cargo); `node:crypto#createHash("blake3")` is invalid and
must not be used. The only remaining sha256 use is an internal wasm ABI-surface
comparison in `js.yml`, which never touches the artifact pin.

## SQL-in-RDF admission and typed refusals (26.10.21 -> 26.10.22)

The sql-rdf surface has one admission entry point. Hand-assembled query
structs are not an admitted path.

- **Compile through `compile_turtle`.** Call
  `graphlaw::sql_ast::compile_turtle(&str)` with a Turtle document conforming
  to `ontologies/sql-ast.ttl`. It returns `Result<CompiledQuery, AstError>`.
  `CompiledQuery`'s fields are public, but there is no admitted way to
  construct one outside admission: a hand-built query carries no
  `receipt_state`/`canon_graph_bytes`, and eval refuses it typed
  (`EvaluatorError::NoCanonicalGraph`, `src/duckdb_eval.rs`).
- **Refusals are pure data.** `compile_turtle` never returns a stringly error.
  Failures are `AstError { code: RefusalCode, ctx: RefusalContext }` — a
  stable refusal code plus a fixed-size rendering context. Render (or match on
  the code) at the edge; do not parse error strings. The code register lives
  in `docs/refusals.md` (REFUSAL-01..04).
- **Receipts hash two nodes.** An admitted statement's receipt is
  `H_query = BLAKE3(Canon(G_AST) || P_params)` — the canonicalized AST graph
  plus the typed parameters, computed once per compile
  (`graphlaw::sql_ast::CompiledQuery::canon_graph_bytes` — a public field, set
  once at compile time). There is no
  legacy hash arm keyed on absent fields; consumers pinning old receipt
  digests must recompute.
- **`frozen-duckdb` is a normal dependency.** Version 2.0.2 from crates.io
  (manifest spec `version = "2.0.2"`, `Cargo.lock` resolves 2.0.2),
  non-optional in `Cargo.toml` — there is no duckdb feature gate to enable.
  Evaluation runs on real frozen-duckdb connections
  (`graphlaw::duckdb_eval`). This tree has no `[patch]` for frozen-duckdb;
  local and published builds both resolve 2.0.2 from crates.io.

## Op-15 `duckdb` is host-only in the wasm artifact (26.10.22)

The `duckdb` ABI op is now compiled only for non-wasm targets: the dispatch arm
is `#[cfg(not(target_arch = "wasm32"))]`-gated (`src/abi.rs:334-335`), and
`duckdb_op`/`duckdb_eval`/SQL builtins are documented host-only
(`src/lib.rs:246-257`). The wasm module still admits and compiles SQL-in-RDF
through `compile_turtle` unchanged; what is absent inside wasm is the
frozen-duckdb execution step.

> **SUPERSEDED (2026-10-23, SQLRDF-21 EH closure)**: the section below
> described the 26.10.22 mid-wave state, in which the `duckdb` op was
> host-only. It is superseded by "EH closure and in-module `duckdb`
> promotion" further down: the cfg gates are gone and the op runs inside the
> module. Retained for migration-path provenance; do not act on it.

Callers invoking op-15 (`"op": "duckdb"`) through the wasm artifact
(`priv/graphlaw.wasm` / `graphlaw-run` wasmi runner) must move the execution
call to the **host dispatch** (native `graphlaw::abi::call` with the
`duckdb` op, or the `graphlaw-verify`/host binaries). If you call the wasm
artifact anyway, the typed refusal you will see is:

```json
{ "kind": "Unsupported", "message": "unknown op `duckdb`" }
```

(produced by the fallthrough arm at `src/abi.rs:336`:
`other => Err(bad(format!("unknown op \`{other}\`")))` with
`RefusalKind::Unsupported`, `src/abi.rs:174-180`). This is a typed refusal,
not a crash or trap. Do not string-match the message at the edge — match on
`kind == "Unsupported"` and route execution to the host. The in-wasm SQL
execution path is expected to return with the EH-wall emsdk rebuild; until
that lands, host dispatch is the only execution route for the `duckdb` op.

## MERGE `ON` must reference a target column (26.10.22)

A MERGE statement whose `ON` condition references no target-qualified column
is now refused. Constant-only `ON` conditions (e.g. `ON $1 = $2`) match every
target row, making `WHEN MATCHED` fan-out unbounded, so admission rejects them
typed as `REFUSAL-03` arm 7 (`MERGE_ON_NO_TARGET_COLUMN`,
`src/sql_ast.rs:987-993`; register entry `docs/refusals.md`, REFUSAL-03 row).
The refusal surfaces as `sql_ast::AstError::ShaclViolation` with
`[REFUSAL-03]` in the message; appended arms carry
`code: RefusalCode::ScopeIntegrity` with `shape_id` =
`refusal_arm::MERGE_ON_NO_TARGET_COLUMN` (= 7).

Migration: qualify the `ON` condition to at least one target column, e.g.
`ON t.id = s.id` instead of `ON $1 = $2`. See the REFUSAL-03 recovery text in
`docs/refusals.md`.

## EH closure and in-module `duckdb` promotion (26.10.22, SQLRDF-21)

Reference: `docs/jira/v26.10.22/SQLRDF-21.md`. The exception-handling wall is
closed: the archive rebuild carries `-mllvm -wasm-use-legacy-eh=0` (new-EH
`try_table`/`throw` artifact), hosted by wasmtime 48 with `wasm_exceptions`;
wasmi 2.0.0 cannot host it (no exceptions-proposal support). The `duckdb` op
(op-15) is promoted in-module: the host-only cfg gates are gone and the op
authorities include frozen-duckdb on wasm. Raw SQL text is still refused —
SQLRDF-17 stands.

What consumers must change:

1. **wasmi-hosted callers had to move to `graphlaw-run-eh`.** The linked
   artifact uses new EH; wasmi cannot host it. If you drove the module
   through the wasmi-based `graphlaw-run` runner: that binary (and its
   `runner` cargo feature) was retired 2026-10-24
   (`docs/jira/v26.10.23/ADR-wasmi-retirement.md`); the
   wasmtime-48-based `graphlaw-run-eh` binary (cargo feature `runner-eh`) or
   a wasmtime 48 host with `wasm_exceptions` enabled is the replacement —
   it is a wire-superset with the same exit-code contract.
2. **Archive rebuilds must carry the flag.** Any rebuild of the wasm artifact
   from the frozen-duckdb archive must pass `-mllvm
   -wasm-use-legacy-eh=0`; a legacy-EH rebuild produces an artifact no
   available runtime hosts (the MIXED wall).
3. **The 7.1 MB pin class is legacy.** Artifact pins in the ~7.1 MB class
   (legacy-EH builds) are superseded; pin the new-EH artifact digest from
   `priv/graphlaw.wasm.blake3` / `registry/ARTIFACTS.blake3`. The exact new
   digest is pending the integration receipt — do not carry a remembered
   value forward.
4. **Supersession.** The section "Op-15 `duckdb` is host-only in the wasm
   artifact (26.10.22)" above no longer applies to the current artifact: the
   `unknown op \`duckdb\`` wasm refusal is retired. The MERGE `ON`
   target-column refusal (REFUSAL-03 arm 7) still applies, and MERGE is now
   classified mutating (execution bumps the epoch counter). WHERE-less
   UPDATE/DELETE refuse `REFUSAL-05` and require BOTH where-predicates.
