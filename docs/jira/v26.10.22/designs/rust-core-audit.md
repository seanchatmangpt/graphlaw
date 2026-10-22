# Rust core-team audit — consolidated record (8 lanes, post-v26.10.22)

RUST-1 ownership · RUST-2 unsafe/FFI · RUST-3 errors · RUST-4 concurrency ·
RUST-5 API design · RUST-6 performance · RUST-7 test quality · RUST-8
supply-chain. All read-only; findings are post-integration backlog, not
commit blockers. Full detail in the lane reports.

## Headline findings

1. **Arc the receipt/canon spine** (RUST-1 X1/X2/X3/X5/X6): CompiledQuery
   deep-cloned on memo paths (sql_ast.rs:2002-2006, :2051), canon bytes
   cloned per cache miss (duckdb_eval.rs:763), CacheEntry cloned under the
   Mutex — one refactor (Arc<[u8]> canon + Arc<ExecutionReceipt> cache
   values) removes five findings.
2. **RdfAstGraphReader ownership-only trait** (RUST-1 X4, API-constraining):
   every accessor returns owned Strings; ~15 property reads per node each
   cloning — dominates compile cost. Add borrowing lifetimes or intern to
   Rc<str> BEFORE more callers accrete.
3. **CompiledQuery should be sealed** (RUST-5, strongly recommended): 6 pub
   fields with 3 Options encoding a sum type (Graph-backed vs Legacy) as
   option-flags; eval already refuses the legacy arm; courts hand-build
   states the compiler would never emit. Sealed enum (or private fields +
   getters) makes the cacheable/receipt laws structural.
4. **RefusalCode needs #[non_exhaustive] NOW** (RUST-5): the enum slated to
   grow ggen-emitted arms; every appended discriminant is currently a semver
   break. Also: name P29-RFL as the ABSORPTION TARGET of
   dialect::ParseRefusal + policy::PolicyRefusalKind, not a fourth taxonomy.
5. **evidence_ring concurrency** (RUST-4): NDJSON write-fail advances
   parent anyway (:610/:623) → permanent BrokenChain + silent batch loss
   (HIGH); OnceLock double-init corrupts the shared ledger (init does I/O
   before GLOBAL.set) (HIGH); "wait-free" doc vs Mutex-guarded push (doc
   mismatch); finalize watermark under-read (Low).
6. **graphlaw_run.rs: 17 expect/unwrap on external wasm/CLI input** (RUST-3,
   HIGH in a binary): malformed .wasm panics the CLI instead of typed exit.
7. **SBOM fix introduced a latent deny.toml blocker** (RUST-8): the new
   `MIT AND LicenseRef-P28-SIL-2.1.0` expression is not in deny.toml's
   allow-list (silent today — cargo-deny not wired to CI); MulanPSL-2.0
   denial blocks any purrdf 3.x migration. frozen-duckdb caret pin
   recommends exact =2.0.0 (self-published single-owner supply).

## Witnessed-correct (keep as-is)

- `#![forbid(unsafe_code)]` in core; wasm FFI comment-enforced where
  inherent, type-enforced everywhere else (RUST-2: zero UNSOUND).
- AstError{code, ctx} migration coherent; Display exhaustive, no wildcard.
- Ring pause-vs-close race: none. consult-time epoch filter: correct (the
  pre-store check is dead weight but harmless — fix the overclaiming
  comment at duckdb_eval.rs:754-759).
- ParseRefusal is the model error type (char-boundary-safe, non_exhaustive,
  doctest).
- evidentiary_ring_court.rs: typed TornSlot matches, bounded polls, unique
  tmp dirs — exemplar. wasm_boundary_fuzz: seeded PRNG, exemplary.
- ParamBytes::encode/fold single-update: correct A03 shape (adopt in the
  eval fold path, drop hash_param_buffer from hot).

## Ranked post-integration backlog

**OPERATOR SEQUENCING OVERRIDE (2026-10-22): the ranked table below is
superseded by the operator's three-wave landing order, which correctly
places concurrency correctness (Item 6) before performance (Items 2/9) —
an integrity break in the ledger invalidates all downstream auditability
claims, and Arc-ing across the still-allocating reader would refactor the
same call sites twice.**

### Operator Wave 1 — Correctness & Integrity (Items 5, 4, 8)
- evidence_ring parent-advance-on-success + OnceLock init I/O race (RUST-4 HIGHs)
- #[non_exhaustive] on RefusalCode; dialect/policy taxonomy absorption
- deny.toml LicenseRef/MulanPSL allow + CI wiring; exact-pin frozen-duckdb

### Operator Wave 2 — Structural Boundary & Legacy Deletion (Items 1, 3, 4)
- SQLRDF-17 closure ((None,None,None) raw-SQL fallback deleted)
- Seal CompiledQuery (AdmittedGraph variant, Arc<[u8]> canon)
- Arc the receipt/canon cache values

### Operator Wave 3 — Nanosecond Collapse & Trait Evolution (Items 2, 9, 10, 11)
- hazmat single-update fold (A03/ParamBytes)
- RdfAstGraphReader borrowed/interned migration
- PCE µs-scale alloc hoists (predicate IRIs, column names, row_subject)

### Original ranked table (retained for reference)

| # | Item | Source | Effort |
|---|---|---|---|
| 1 | Close SQLRDF-17 (5-lane convergence SPOF) | CAP-4/7/10/11 + SPOF-8 | M |
| 2 | Arc the receipt/canon spine (X1-X3, X5, X6) | RUST-1 | M |
| 3 | Borrowing reader trait (X4) | RUST-1 | L→XL (API break) |
| 4 | Seal CompiledQuery | RUST-5 | M |
| 5 | RefusalCode #[non_exhaustive] + taxonomy absorption note | RUST-5 | S |
| 6 | evidence_ring: parent-advance-on-success + init mutex | RUST-4 | S-M |
| 7 | graphlaw_run typed errors; with_num try_from; ReasonerLimit From loss | RUST-3 | S |
| 8 | deny.toml: allow LicenseRef + MulanPSL, wire to CI; exact-pin frozen-duckdb | RUST-8 | S |
| 9 | hazmat ChunkState mid-state fold (40-80ns) + ParamBytes single-update (15-30ns) + bench batch timing (30-40ns measured) | RUST-6 | M |
| 10 | Corpus: typed RefusalCode asserts (kill ~17 prose needles); fix 2 wrong-reason twins; e3 deletion-flip | RUST-7 | S-M |
| 11 | PCE µs-scale alloc hoists (column_names, predicate IRIs, row_subject) | RUST-6 | M |
