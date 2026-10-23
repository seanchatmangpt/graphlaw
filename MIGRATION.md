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
