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
validate_ast_shacl (closed-world SHACL)      src/sql_ast.rs:1298
  |   shapes = ontologies/sql-ast.ttl
  |          + ggen/sql-dialect/statement_classes.ttl
  |   shape-depth guard first: src/shacl_guard.rs:84
  |   nonconforming -> typed [REFUSAL-nn] AstError, never a DB error
  v
compile_turtle  (SOLE admission path)        src/sql_ast.rs:1404
  |   builds CompiledQuery, then reseal()
  |   admission_seal = f(sealed fields)      src/sql_ast/ast.rs:518-522
  v
CompiledQuery (sealed)                       src/sql_ast/ast.rs (~src/sql_ast/ast.rs:518 seal fns)
  |
  v
eval_ast_query                               src/duckdb_eval.rs:1035
  |   1. verify_seal() gate BEFORE prepare/execute   src/duckdb_eval.rs:1161
  |      broken seal -> typed EvaluatorError::ReceiptError, never panic
  |   2. H_query = BLAKE3(h_ast || P_params) two-node form   src/duckdb_eval.rs:642
  |      h_ast pinned at compile time in `receipt_state` (32-byte Copy)
  |      missing anchor -> typed NoCanonicalGraph refusal (SQLRDF-17)
  |   3. C1 consult: only `cacheable` statements consult/store
  |      keyed by H_query (src/duckdb_eval.rs:1161 region)
  |   4. prepare via $N placeholders (never inlined); stmt reuse seam
  |   5. execute on real frozen-duckdb; project rows into RDF triples
  |   6. per-row 64-byte RingDescriptor into evidentiary ring (opt-in)
  |      src/duckdb_eval.rs:1161
  |   7. non-cacheable mutating SQL -> flush prepared handles + epoch bump
  v
ExecutionReceipt                             src/duckdb_eval.rs:127
  |
  +--> evidence ring (SPSC, capacity 4096)   src/evidence_ring.rs:38
  |      host-only build, feature `evidentiary-ring`, default OFF
  +--> query ledger (NDJSON, BLAKE3-chained) src/query_ledger/storage.rs:42
```

### Sole admission, stated plainly

- `compile_turtle` is the only way a `CompiledQuery` gets a valid seal. The
  constructor path that bypasses compile zero-fills the seal
  (src/sql_ast/ast.rs:511), so any `CompiledQuery` not produced by
  `compile_turtle` fails `verify_seal()` at eval time.
- The legacy raw-SQL-string hash arm and hand-built re-canonicalization
  fallback are DELETED, not gated (src/duckdb_eval.rs:1161). A query with
  neither the compile-time canon anchor nor canon bytes is a typed
  `NoCanonicalGraph` refusal, never a raw-SQL admission.
- Parameters are unsealed: binding different params to the same admitted
  statement is lawful (src/duckdb_eval.rs:1161).

## 2. Module map by responsibility

32 `pub mod` declarations in src/lib.rs (lines 90-252), grouped by role.
Feature gates: `abi` (default), `qualification`, `registry`,
`pack-tools`/`smon`, `evidentiary-ring` (excluded from wasm builds, default OFF).

### Law kernel (admission, tiers, authority)

- `law` (src/law.rs) - `LawState`, `Step` (AdmitShacl, DeriveN3, ...),
  `Ceiling` enum, `Lease`/`SignedLease` authority with typed
  `LeaseReason` (src/law.rs:28,50,199,247,333,387).
- `tier` (src/tier.rs) - N3 rule tiers, `admit_bytes_tiered`
  (src/tier.rs:200), `REFUSAL_04_CAPABILITY_REFUSAL` (src/tier.rs:25).
- `shacl_guard` (src/shacl_guard.rs) - `check_shape_depth` cap on SHACL
  validation depth (src/shacl_guard.rs:84).
- `policy` (src/policy.rs) - policy admission over parsed problems:
  `admit_parsed` (src/policy.rs:219), `admit` (src/policy.rs:468).
- `plan` (src/plan.rs) - bounded plans: `MAX_PLAN_TOTAL_ATOMS`
  (src/plan.rs:30), `Action`/`Plan`/builders (src/plan.rs:47,81).
- `rules` (src/rules.rs) - builtin registry; carries the REFUSAL-04
  capability-refusal text (see docs/refusals.md REFUSAL-04 row).
- `dialect` (src/dialect.rs + src/dialect/) - `sniff`
  (src/dialect/sniff.rs:128)/`check_typed`/`parse_rdf_typed`/
  `admit_bytes` (src/dialect.rs:368,516,569).

### SQL surface (sql-rdf compiler + eval)

- `sql_ast` (src/sql_ast.rs + src/sql_ast/) - closed-world SHACL validation
  (`validate_ast_shacl`, src/sql_ast.rs:1298), `compile_turtle` sole
  admission (src/sql_ast.rs:1404), sealed `CompiledQuery` (seal at
  src/sql_ast/ast.rs:518-564).
- `sql_statement_kinds` (src/sql_statement_kinds.rs) - statement-kind
  grammar bridge, 19 lines total.
- `duckdb_eval` (src/duckdb_eval.rs) - `GraphLawDuckDB::open_in_memory`
  (src/duckdb_eval.rs:946), `eval_ast_query` (src/duckdb_eval.rs:1035),
  receipt hash fns `fold_params_anchored` (src/duckdb_eval.rs:642) and
  `admitted_statement_hash` (src/duckdb_eval.rs:379).

### Storage / receipts / evidence

- `receipt` (src/receipt.rs) - `record`/`require` of law-step receipts,
  plain and signed (src/receipt.rs:65,72,131,164).
- `receipt_chain` (via `pub use receipt_chain::{ReceiptRecord,
  ReceiptValidator}` src/lib.rs:107) - BLAKE3-chained receipt records.
- `receipt_store` (src/receipt_store.rs) - on-disk store:
  `put`/`put_signed`/`verify_attested`/`verify`
  (src/receipt_store.rs:357,372,390,436).
- `query_ledger` (src/query_ledger.rs, re-exporting src/query_ledger/) -
  NDJSON ledger: `open`/`push`/`record_batch`/`flush`
  (src/query_ledger/storage.rs:42,103,115,124), `content_digest`
  (src/query_ledger/batch.rs:418).
- `evidence_ring` (src/evidence_ring.rs) - P28-SIL §3.2 SPSC ring,
  capacity 4096 (src/evidence_ring.rs:38), drop-newest push, NDJSON ledger
  offload via `init_with_ledger*` (src/evidence_ring.rs:269,284); gated
  `#[cfg(all(feature = "evidentiary-ring", not(target_arch = "wasm32")))]`
  (src/lib.rs:250) — feature opt-in AND host-only.
- `query_cache` (src/query_cache.rs) - compiled-plan cache keyed by `h_ast`,
  epoch-invalidated on non-cacheable mutations
  (src/duckdb_eval.rs:1161).

### Boundaries: dialects, wasm, registry, eyeron

- `abi` (src/abi.rs) - JSON request/response ABI, `call` (src/abi.rs:186),
  ABI v1 limits (src/abi.rs:45-62), `dialect_by_name` (src/abi.rs:238),
  `plan_from_json` (src/abi/ops/governance.rs:353), `call_json`
  (src/abi.rs:339) — the
  JSON front door shared by the wasm module and host runners.
- `abi_manifest`, `registry` (src/abi_manifest.rs, src/registry/definitions.rs) -
  capability surface machine-readable: `surface_blake3`,
  `registry_blake3` (src/registry/emit_json.rs:631,644), `registry_turtle`
  (src/registry/emit_ttl.rs:304). Feature `registry`.
- `dialect` (also boundary) - media-type sniffing (`sniff`,
  src/dialect/sniff.rs:128) and typed parse refusals (`ParseRefusal`,
  src/dialect.rs:150).
- `wasm/src/lib.rs` - `graphlaw-wasm` crate: generated FFI shell
  (`gl_alloc`/`gl_call`/`gl_free`, wasm/src/lib.rs:27) over
  `graphlaw::abi`; compile-time asserts pin rendered `abi_meta` limits to
  `graphlaw::abi` (wasm/src/lib.rs:31). `ffi.rs`/`abi_meta.rs` are
  ggen-rendered from ontologies/graphlaw-wasm.ttl; never hand-edited.
- `crates/graphlaw-eyeron/` - separate workspace crate: N3/rdf reasoning
  (lexer, parser, reasoner, proof_check, w3c_rdf, rdf_compat). Consumed by
  graphlaw as the `eyeron` dependency (re-exported unstable at
  src/lib.rs:123).

## 3. Invariants that make it law-admission, not a database

- **Append-only refusal codes.** `REFUSAL-01..05` plus typed sub-codes are
  registered in docs/refusals.md with stable codes, triggers, and recovery;
  errors are typed carriers, never generic DB errors. Error enums never gain
  meaning; the register flips are receipt-gated. `REFUSAL-05`
  (`UnboundedMutationRefusal`) is compile-time: a WHERE-less UPDATE/DELETE
  never reaches the engine (src/sql_ast.rs:2457).
- **Receipt identity = BLAKE3(canon || params).** `H_query =
  BLAKE3(h_ast || P_params)` (src/duckdb_eval.rs:642); `h_ast =
  BLAKE3(Canon(G_AST))` pinned in `receipt_state` at compile time. The same
  formula as `admitted_statement_hash` (src/duckdb_eval.rs:379), so a
  receipt is bit-identical to the admitted-statement hash by construction.
- **The seal.** `CompiledQuery` carries `admission_seal` computed by
  `reseal()` (src/sql_ast/ast.rs:518); eval verifies it before prepare
  (src/duckdb_eval.rs:1161). Bypass compile -> zero-filled seal -> typed
  refusal at eval. This is the sole-admission property.
- **Reproducibility.** Canonicalization is deterministic
  (`purrdf::try_canonicalize`, re-exported src/lib.rs:126); Ed25519 signing
  is deterministic RFC 8032, no RNG (src/attest.rs:13), so
  `sign_bytes`/`sign_receipt`/`sign_lease` (src/attest.rs:417,571,624)
  replay byte-identically. Receipts embed real output; a receipt without
  embedded output is not a receipt.
- **Closed-world admission.** Shapes come from the pinned graphs
  (`include_str!` of ontologies/sql-ast.ttl + dialect statement classes,
  src/sql_ast.rs:1404), not from the query document.
  `check_shape_depth` caps SHACL recursion before any engine call
  (src/sql_ast.rs:1298).
- **Typed refusals over crashes.** Broken seal -> `ReceiptError`;
  missing canon anchor -> `NoCanonicalGraph`; shape-depth over -> `MalformedAst`
  with `SHACL_DEPTH` arm (src/sql_ast.rs:1298).

## 4. The wasm unit at its true state

- `graphlaw-wasm` (`wasm/Cargo.toml`, version 26.10.22) is one artifact:
  the kernel (`graphlaw` with the `abi` feature). SQL-in-RDF admission
  (`compile_turtle`) AND columnar execution stay in the module:
  `frozen-duckdb` is a normal dependency on every target (root Cargo.toml:68,
  wasm32 included) and `query_cache`, `duckdb_eval`, `duckdb_op`, and
  `rules` are ungated. The `duckdb` op (op-15) dispatches inside the module
  (src/abi.rs:186) — AST-compiled statements only; raw SQL text is a typed
  refusal (SQLRDF-17), and a WHERE-less UPDATE/DELETE is the compile-time
  `REFUSAL-05` `UnboundedMutationRefusal` before any engine contact.
  The frozen-duckdb wasm staticlib is rebuilt with `-fwasm-exceptions
  -mllvm -wasm-use-legacy-eh=0` (uniform W3C exceptions) and is hosted by
  wasmtime 48 with `Config::wasm_exceptions` (`runner-eh` feature, bin
  `graphlaw-run-eh`); wasmi 2.0.0 has no EH support and no exceptions knob,
  so it is retained only for the legacy pre-duckdb artifact courts
  (tests/wasm_abi.rs).
- Generated shell: `wasm/src/ffi.rs` and `wasm/src/abi_meta.rs` are
  ggen-rendered from ontologies/graphlaw-wasm.ttl. Compile-time asserts
  (wasm/src/lib.rs:31) pin the rendered limits to `graphlaw::abi`, the
  single source of truth for ABI limits.
- Protocol: JSON over linear memory (`gl_alloc`/`gl_call`/`gl_free`,
  wasm/src/lib.rs:27), always-JSON responses.
- Digest pins: `registry/ARTIFACTS.blake3` and `priv/graphlaw.wasm.blake3`
  are `fa1558306690493249b176ec312a4f01e97798937e05cd49f2946d61a7e6d107`
  (57,781,384 B, tag section count 1) — the DuckDB-linked new-EH artifact,
  re-pinned 2026-10-23 after the SQLRDF-24 root cause was CONFIRMED and
  FIXED (musl atexit `slot` re-attach reset + walker fn-read offsets; also
  static extension registration + atexit-family no-op overrides in
  wasm/src/lib.rs per the reactor contract; clean trap table witnessed,
  `receipts/2026-10-23-eh-closure-wave.md`; registry `graphlaw-registry
  --check` exit 0). The legacy pin `f149433c…` (7,104,147 B,
  `pin-source=26.10.22`, tag section count 0) is historical.
- Host-side runners are host-only (`runner` / `runner-eh` features, root
  Cargo.toml:99) — neither emulator compiles for wasm32. `graphlaw-run`
  (wasmi) replays the legacy pre-duckdb artifact courts through the smalt
  sandbox packet-4 stdio protocol; `graphlaw-run-eh` (wasmtime 48,
  `Config::wasm_exceptions`) hosts the current DuckDB-linked artifact.

## 5. System diagram

```text
                Turtle / N3 / other dialects
                          |
                    dialect sniff/check       src/dialect/sniff.rs:128, src/dialect.rs:368
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
  -> src/sql_ast.rs:1404 / src/duckdb_eval.rs:1035 / src/sql_ast.rs:1298
grep -n "verify_seal" src/duckdb_eval.rs | head  # verify_seal gate at :649
sed -n 1137,1167p src/sql_ast.rs               # compile_turtle body incl.
                                               #   include_str! shapes
sed -n 631,700p src/duckdb_eval.rs             # seal gate, H_query, C1 consult
sed -n 900,935p src/duckdb_eval.rs             # ring push + epoch bump
sed -n 1,40p wasm/src/lib.rs                   # generated FFI, assert pins
grep -n "frozen-duckdb" Cargo.toml wasm/Cargo.toml   # unconditional dependency (no target gating; links into the one unit on every target)
grep -n "fn sign" src/attest.rs                # sign_bytes/sign_receipt/sign_lease
```
