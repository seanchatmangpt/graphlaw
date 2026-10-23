# ARCHITECTURE

GraphLaw is a law-admission kernel: RDF/SHACL graphs admit through one
executable law, project to SQL/DuckDB, sign receipts, and run the same
artifact on host and wasm. This doc maps the system for contributors; `lib.rs`
opens the API for users. Every claim below is anchored to verified code.

## 1. The admission law flow

The system admits one thing one way. Turtle enters, a sealed `CompiledQuery`
leaves, evaluation verifies the seal before any engine call.

```text
Turtle graph (sql: AST in RDF)
  |
  v
validate_ast_shacl (closed-world SHACL)      src/sql_ast.rs:2051
  |   shapes = ontologies/sql-ast.ttl
  |          + ggen/sql-dialect/statement_classes.ttl
  |   shape-depth guard first: src/shacl_guard.rs:86
  |   nonconforming -> typed [REFUSAL-nn] AstError, never a DB error
  v
compile_turtle  (SOLE admission path)        src/sql_ast.rs:2155
  |   builds CompiledQuery, then reseal()
  |   admission_seal = f(sealed fields)      src/sql_ast.rs:859-883
  v
CompiledQuery (sealed)                       src/sql_ast.rs (~sql_ast.rs:909 seal fns)
  |
  v
eval_ast_query                               src/duckdb_eval.rs:498
  |   1. verify_seal() gate BEFORE prepare/execute   src/duckdb_eval.rs:516
  |      broken seal -> typed EvaluatorError::ReceiptError, never panic
  |   2. H_query = BLAKE3(h_ast || P_params) two-node form   src/duckdb_eval.rs:366-373
  |      h_ast pinned at compile time in `receipt_state` (32-byte Copy)
  |      missing anchor -> typed NoCanonicalGraph refusal (SQLRDF-17)
  |   3. C1 consult: only `cacheable` statements consult/store
  |      keyed by H_query (src/duckdb_eval.rs:563-575 region)
  |   4. prepare via $N placeholders (never inlined); stmt reuse seam
  |   5. execute on real frozen-duckdb; project rows into RDF triples
  |   6. per-row 64-byte RingDescriptor into evidentiary ring (opt-in)
  |      src/duckdb_eval.rs:744-763
  |   7. non-cacheable mutating SQL -> flush prepared handles + epoch bump
  v
ExecutionReceipt                             src/duckdb_eval.rs:127
  |
  +--> evidence ring (SPSC, capacity 4096)   src/evidence_ring.rs:38
  |      host-only build, feature `evidentiary-ring`, default OFF
  +--> query ledger (NDJSON, BLAKE3-chained) src/query_ledger.rs:454
```

### Sole admission, stated plainly

- `compile_turtle` is the only way a `CompiledQuery` gets a valid seal. The
  constructor path that bypasses compile zero-fills the seal
  (src/sql_ast.rs:881), so any `CompiledQuery` not produced by
  `compile_turtle` fails `verify_seal()` at eval time.
- The legacy raw-SQL-string hash arm and hand-built re-canonicalization
  fallback are DELETED, not gated (src/duckdb_eval.rs:530-543). A query with
  neither the compile-time canon anchor nor canon bytes is a typed
  `NoCanonicalGraph` refusal, never a raw-SQL admission.
- Parameters are unsealed: binding different params to the same admitted
  statement is lawful (src/duckdb_eval.rs:510-515).

## 2. Module map by responsibility

32 `pub mod` declarations in src/lib.rs (lines 90-252), grouped by role.
Feature gates: `abi` (default), `qualification`, `registry`,
`pack-tools`/`smon`, `evidentiary-ring` (excluded from wasm builds, default OFF).

### Law kernel (admission, tiers, authority)

- `law` (src/law.rs) - `LawState`, `Step` (AdmitShacl, DeriveN3, ...),
  `Ceiling` enum, `Lease`/`SignedLease` authority with typed
  `LeaseReason` (src/law.rs:28,50,199,247,333,387).
- `tier` (src/tier.rs) - N3 rule tiers, `admit_bytes_tiered`
  (src/tier.rs:232), `REFUSAL_04_CAPABILITY_REFUSAL` (src/tier.rs:25).
- `shacl_guard` (src/shacl_guard.rs) - `check_shape_depth` cap on SHACL
  validation depth (src/shacl_guard.rs:86).
- `policy` (src/policy.rs) - policy admission over parsed problems:
  `admit_parsed` (src/policy.rs:219), `admit` (src/policy.rs:468).
- `plan` (src/plan.rs) - bounded plans: `MAX_PLAN_TOTAL_ATOMS`
  (src/plan.rs:30), `Action`/`Plan`/builders (src/plan.rs:47,81).
- `rules` (src/rules.rs) - builtin registry; carries the REFUSAL-04
  capability-refusal text (see docs/refusals.md REFUSAL-04 row).
- `dialect` (src/dialect.rs) - `sniff`/`check_typed`/`parse_rdf_typed`/
  `admit_bytes` (src/dialect.rs:682,888,1037,1091).

### SQL surface (sql-rdf compiler + eval)

- `sql_ast` (src/sql_ast.rs) - closed-world SHACL validation
  (`validate_ast_shacl`, src/sql_ast.rs:2051), `compile_turtle` sole
  admission (src/sql_ast.rs: 2155), sealed `CompiledQuery` (seal at
  src/sql_ast.rs:859-914).
- `sql_statement_kinds` (src/sql_statement_kinds.rs) - statement-kind
  grammar bridge, 19 lines total.
- `duckdb_eval` (src/duckdb_eval.rs) - `GraphLawDuckDB::open_in_memory`
  (src/duckdb_eval.rs:409), `eval_ast_query` (src/duckdb_eval.rs:498),
  receipt hash fns `fold_params_anchored` (src/duckdb_eval.rs:352) and
  `admitted_statement_hash` (src/duckdb_eval.rs:374).

### Storage / receipts / evidence

- `receipt` (src/receipt.rs) - `record`/`require` of law-step receipts,
  plain and signed (src/receipt.rs:65,72,131,164).
- `receipt_chain` (via `pub use receipt_chain::{ReceiptRecord,
  ReceiptValidator}` src/lib.rs:107) - BLAKE3-chained receipt records.
- `receipt_store` (src/receipt_store.rs) - on-disk store:
  `put`/`put_signed`/`verify_attested`/`verify`
  (src/receipt_store.rs:358,372,390,436).
- `query_ledger` (src/query_ledger.rs) - NDJSON ledger: `open`/`push`/
  `record_batch`/`flush` (src/query_ledger.rs:475,538,550,559),
  `content_digest` (src/query_ledger.rs:441).
- `evidence_ring` (src/evidence_ring.rs) - P28-SIL §3.2 SPSC ring,
  capacity 4096 (src/evidence_ring.rs:38), drop-newest push, NDJSON ledger
  offload via `init_with_ledger*` (src/evidence_ring.rs:267,284);
  gated off in wasm builds (`#[cfg(not(target_arch = "wasm32"))]`).
- `query_cache` (src/query_cache.rs) - compiled-plan cache keyed by `h_ast`,
  epoch-invalidated on non-cacheable mutations
  (src/duckdb_eval.rs:754-760).

### Boundaries: dialects, wasm, registry, eyeron

- `abi` (src/abi.rs) - JSON request/response ABI, `call` (src/abi.rs:186),
  ABI v1 limits (src/abi.rs:45-62), `dialect_by_name` (src/abi.rs:364),
  `plan_from_json` (src/abi.rs:992), `call_json` (src/abi.rs:1152) — the
  JSON front door shared by the wasm module and host runners.
- `abi_manifest`, `registry` (src/abi_manifest.rs, src/registry/definitions.rs) -
  capability surface machine-readable: `surface_blake3`,
  `registry_blake3` (src/registry/emit_json.rs:631,644), `registry_turtle`
  (src/registry/emit_ttl.rs:304). Feature `registry`.
- `dialect` (also boundary) - media-type sniffing (`sniff`,
  src/dialect.rs:682) and typed parse refusals (`ParseRefusal`,
  src/dialect.rs:295).
- `wasm/src/lib.rs` - `graphlaw-wasm` crate: generated FFI shell
  (`gl_alloc`/`gl_call`/`gl_free`, wasm/src/lib.rs:11-18) over
  `graphlaw::abi`; compile-time asserts pin rendered `abi_meta` limits to
  `graphlaw::abi` (wasm/src/lib.rs:29-37). `ffi.rs`/`abi_meta.rs` are
  ggen-rendered from ontologies/graphlaw-wasm.ttl; never hand-edited.
- `crates/graphlaw-eyeron/` - separate workspace crate: N3/rdf reasoning
  (lexer, parser, reasoner, proof_check, w3c_rdf, rdf_compat). Consumed by
  graphlaw as the `eyeron` dependency (re-exported unstable at
  src/lib.rs:123).

## 3. Invariants that make it law-admission, not a database

- **Append-only refusal codes.** `REFUSAL-01..04` plus typed sub-codes are
  registered in docs/refusals.md with stable codes, triggers, and recovery;
  errors are typed carriers, never generic DB errors. Error enums never gain
  meaning; the register flips are receipt-gated.
- **Receipt identity = BLAKE3(canon || params).** `H_query =
  BLAKE3(h_ast || P_params)` (src/duckdb_eval.rs:366-373); `h_ast =
  BLAKE3(Canon(G_AST))` pinned in `receipt_state` at compile time. The same
  formula as `admitted_statement_hash` (src/duckdb_eval.rs:374), so a
  receipt is bit-identical to the admitted-statement hash by construction.
- **The seal.** `CompiledQuery` carries `admission_seal` computed by
  `reseal()` (src/sql_ast.rs:859-883); eval verifies it before prepare
  (src/duckdb_eval.rs:516). Bypass compile -> zero-filled seal -> typed
  refusal at eval. This is the sole-admission property.
- **Reproducibility.** Canonicalization is deterministic
  (`purrdf::try_canonicalize`, re-exported src/lib.rs:130); Ed25519 signing
  is deterministic RFC 8032, no RNG (src/attest.rs:1-11), so
  `sign_bytes`/`sign_receipt`/`sign_lease` (src/attest.rs:417,571,624)
  replay byte-identically. Receipts embed real output; a receipt without
  embedded output is not a receipt.
- **Closed-world admission.** Shapes come from the pinned graphs
  (`include_str!` of ontologies/sql-ast.ttl + dialect statement classes,
  src/sql_ast.rs:2156-2159), not from the query document.
  `check_shape_depth` caps SHACL recursion before any engine call
  (src/sql_ast.rs:2053-2057).
- **Typed refusals over crashes.** Broken seal -> `ReceiptError`;
  missing canon anchor -> `NoCanonicalGraph`; shape-depth over -> `MalformedAst`
  with `SHACL_DEPTH` arm (src/sql_ast.rs:2053-2056).

## 4. The wasm unit at its true state

- `graphlaw-wasm` (wasm/Cargo.toml:2, version 26.10.21) is ONE artifact:
  the kernel (`graphlaw` with `abi` feature, wasm/Cargo.toml:19) plus
  frozen-duckdb — SQL-in-RDF admits and evaluates inside the module.
  frozen-duckdb is a non-optional dependency (root Cargo.toml:35), so the
  duckdb eval path is in the wasm build, matching the src/lib.rs claims
  (src/lib.rs:248-251).
- Generated shell: `wasm/src/ffi.rs` and `wasm/src/abi_meta.rs` are
  ggen-rendered from ontologies/graphlaw-wasm.ttl. Compile-time asserts
  (wasm/src/lib.rs:29-37) pin the rendered limits to `graphlaw::abi`, the
  single source of truth for ABI limits.
- Protocol: JSON over linear memory (`gl_alloc`/`gl_call`/`gl_free`,
  wasm/src/lib.rs:11-18), always-JSON responses.
- Digest pins: registry/ARTIFACTS.blake3 and priv/graphlaw.wasm.blake3 are
  minted at the 26.10.21 source state and must be re-pinned together at the
  next artifact rebuild (wasm/Cargo.toml:4-8).
- Host-side wasmi runner is host-only (`runner` feature, root Cargo.toml:83)
  — wasmi cannot compile for wasm32 — running the real artifact through the
  smalt sandbox packet-4 stdio protocol (root Cargo.toml:101).

## 5. System diagram

```text
                Turtle / N3 / other dialects
                          |
                    dialect sniff/check       src/dialect.rs:682,888
                          |
       +------------------+-----------------------+
       |                                          |
  sql-rdf surface                          law kernel
  validate_ast_shacl                       tier admit, LawState
  compile_turtle (seal)                    policy, plan, rules
       |                                          |
  CompiledQuery (sealed)                   receipt/require per step
       |                                          |
  eval_ast_query  <--------- ABI call() ---+--------- wasm module
  seal verify -> H_query                   |         graphlaw-wasm
  prepare -> execute (frozen-duckdb)       |         (ffi shell, ggen-rendered)
  rows -> RDF triples                      |
       |                                   |
  ExecutionReceipt                         |
       |                                   |
  evidence ring (opt-in SPSC) -> NDJSON query ledger (BLAKE3-chained)
  receipt_store / receipt_chain / attest (Ed25519, deterministic)
```

## Verification receipts (grep commands run while authoring)

```text
grep -n "^pub mod\|^mod" src/lib.rs            # 32 pub mods, lines 90-252
grep -n "pub fn compile_turtle\|pub fn eval_ast_query\|fn validate_ast_shacl" \
  src/*.rs
  -> src/sql_ast.rs:2155 / src/duckdb_eval.rs:498 / src/sql_ast.rs:2051
grep -n "seal" src/duckdb_eval.rs | head       # verify_seal gate at :516
sed -n 2155,2185p src/sql_ast.rs               # compile_turtle body incl.
                                               #   include_str! shapes
sed -n 490,600p src/duckdb_eval.rs             # seal gate, H_query, C1 consult
sed -n 740,775p src/duckdb_eval.rs             # ring push + epoch bump
sed -n 1,40p wasm/src/lib.rs                   # generated FFI, assert pins
grep -n "frozen-duckdb" Cargo.toml wasm/Cargo.toml   # non-optional dep
grep -n "fn sign" src/attest.rs                # sign_bytes/sign_receipt/sign_lease
```
