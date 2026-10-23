# TC-MEGA-FIBO-01 — Court Specification (executable-deferred)

Operation Global Tear-Up: multi-clearinghouse portfolio compression/tear-up,
100,000 derivative legs, 3 jurisdictions (US/UK/EU), 5,000 LEI nodes.

**Status**: SPECIFICATION ONLY. Every Rust artifact named here is a BINDING for
a later implementation lane, not a deliverable of this document. This lane
wrote zero Rust (see §6). Implementation binds to existing runners with zero
design freedom: the phase order, predicates, statuses, and exit codes below
are normative; the implementation lane supplies only wiring.

---

## 0. Machine disclosure (MANDATORY, printed by the court binary before phase 1)

The court must print this block to stdout and fail with `MEGA-EXIT-ENV` if any
field cannot be determined. Implementation binds to `std::env::consts` and
`std::thread::available_parallelism` — no external crates.

```
arch        = aarch64            (from uname -m / std::env::consts::ARCH)
os          = macOS              (std::env::consts::OS)
cores       = 16                 (std::thread::available_parallelism)
rustc       = 1.96.0 (ac68faa20 2026-05-25)   (build-time env)
benchmark_mode = release         (assert: court binary must be built with
                                    --release; debug builds are a typed
                                    refusal, never a pass)
```

Witnessed machine at spec time (`uname -m; uname -s; sysctl -n hw.ncpu;
rustc --version`): `arm64`, `Darwin`, `16`, `rustc 1.96.0 (ac68faa20
2026-05-25)`.

---

## 1. CTQ register — every operator number, three-state, no rounding

Each CTQ is a CEILING unless marked FLOOR. Status is one of WITNESSED /
UNVERIFIED / REFUTED and is re-read from receipts at use time. A court run
MUST re-print this table with statuses resolved against on-disk receipts;
shipping the statuses below verbatim without re-reading is spec drift, not
compliance. No rounding up of any measured number into a passing ceiling.

| CTQ | Value | Ceiling/Floor | Status | Evidence |
|-----|-------|---------------|--------|----------|
| M1 | 0.082 µs statement admission | CEILING | UNVERIFIED | note M1 |
| M2 | 1,248 SHACL constraints | FLOOR | WITNESSED (count class) | note M2 |
| M3 | 12M+ rows/sec read throughput | FLOOR | UNVERIFIED | note M3 |
| M4 | 18.4 ns SPSC ring push | CEILING | UNVERIFIED | note M4 |
| M5 | p99 214 ns hot path | CEILING | UNVERIFIED | note M5 |
| M6 | Cpk ≥ 1.33 on receipt fold | FLOOR | REFUTED (measured −0.05, run 3) | note M6 |
| M7 | 42.18 MB wasm residency | CEILING | UNVERIFIED | note M7 |
| M8 | 2.1 ns typed refusal | CEILING | UNVERIFIED | note M8 |
| M9 | ≤450 µs end-to-end (100k legs) | CEILING | UNVERIFIED | note M9 |

### CTQ evidence notes

- **M1**: No receipt on disk carries this number; it appears in no court
  assert. Implementation must witness it (per-admission fold timing at
  100k-leg scale, batch-timed per the `pce-baseline.md` hygiene note) or
  print the measured value and keep it UNVERIFIED.
- **M2**: `gen_shapes.py:4-6` targets the 1,248-constraint scale and
  re-counts emitted shapes from bytes at generation time. The COUNT is
  witnessed by generator output; ADMISSION of all 1,248 at once is
  UNVERIFIED until phase 1 runs.
- **M3**: `designs/ctq-traceability.md` CTQ 4: "PARTIAL — UNCOURTED at
  scale … 500k smoke; throughput printed never gated vs 12M; 25M full
  behind TC_HOT_FULL=1 unrun". Prior PCE claims REFUTED at the measured
  operating point (`designs/pce-baseline.md`: PCE 70.36%, substrate
  overhead 42.12% vs charter "<2.8%"). TC_HOT_FULL-scale throughput never
  witnessed.
- **M4**: No receipt on disk; `evidentiary_ring_court` asserts structural
  properties, not a timed 18.4 ns bound.
- **M5**: `designs/cpk-correction.md`: best witnessed p99 on the fold path
  is 375 ns (μ=278 ns run); 214 ns appears in no receipt. The 184-216 ns
  row in `designs/hot-path-baseline.md` is disclosed as
  contention-sensitive, not a receipt.
- **M6**: `designs/cpk-correction.md`: μ=278.0 ns, σ=202.1 ns,
  Cpk = −0.05, P(X>USL=250ns)=42.9%, n=1M samples, release mode. Charter
  Cpk 1.48/1.44 does not reproduce. The court must print
  `REFUTED (measured -0.05, run 3)`. A quiet-machine re-run does not flip
  the status by itself; only a receipt postdating the correction record
  per that record's own close condition does.
- **M7**: No receipt on disk carries this number; `wasm_size_court` gates
  a different bound.
- **M8**: `designs/ctq-traceability.md` CTQ 1: refusal leg is "timed but
  UNGATED (no 100ns assert)"; the 0.10 µs figure appears nowhere in code,
  and 2.1 ns appears in no receipt. Heap `String` per refusal is current;
  the Static zero-alloc fast path has not landed.
- **M9**: No receipt on disk at 100k-leg scale; phase 4 is the acceptance
  event.

### Status census at spec time

WITNESSED: 1 (M2 count class only; full-corpus admission UNVERIFIED)
UNVERIFIED: 7 (M1, M3, M4, M5, M7, M8, M9)
REFUTED: 1 (M6)

**No CTQ enters this suite WITNESSED on the passing side.** The suite's
purpose is to convert UNVERIFIED → WITNESSED or → REFUTED with embedded
output, and to keep M6 loudly REFUTED until its close condition fires.

---

## 2. Phase structure

### Phase 1 — Admission

**Input**: full corpus — 100,000 derivative legs, 5,000 LEI nodes, 3
jurisdictions, 1,248-shape regulatory layer (`ontology/shapes/`).

**Machinery (implementation binding, no design freedom)**:
`src/sql_ast.rs` `compile_turtle` path for corpus compile; `src/law.rs`
admission; shapes via the existing SHACL admission path.

**Gate predicates (ALL must hold)**:
- P1.1 All 100,000 legs admit or refuse TYPED. Zero untyped error strings in
  stdout/stderr. Predicate: scan court stdout for `error`/`panic`/`Result::Err`
  renderings that lack a `RefusalCode` prefix; any hit → fail phase.
- P1.2 Admission wall time printed, batch-timed (one wall-clock pair around
  the admission loop, not per-op `Instant::now`), feeding CTQ M1. No gate on
  M1 in this phase — M1 is reported, not enforced, until WITNESSED twice.
- P1.3 The 1,248-shape corpus admits in full (count re-derived from admitted
  shape IRIs, not from the generator's claim).
- P1.1 is the binding predicate for CTQ M8's correctness half (typed-ness).

### Phase 2 — Vectorized execution

**Machinery (binding)**: `src/duckdb_eval.rs` prepared-statement path
(prepare ONCE, typed binds, re-execute — the pce-baseline protocol); parquet
mount legs from `benches/tc_hot_02.rs`.

**Gate predicates**:
- P2.1 Real frozen-duckdb in-memory connection
  (`GraphLawDuckDB::open_in_memory`), real 100k-leg table readbacks.
  Chicago: no mocks.
- P2.2 Throughput computed from wall time and row counts, printed against the
  M3 floor of 12M rows/sec. Status stays UNVERIFIED unless TC_HOT_FULL-scale
  corpus (25M) is in play; the 100k corpus gates P2.3, not M3.
- P2.3 End-to-end eval of the compression/tear-up query family over the 100k
  legs completes with correct row counts asserted on real readbacks
  (deterministic corpus → exact expected counts, computed by
  `generators/gen_corpus.py` and hardcoded as constants in the court).
- P2.4 Hot-path p99 printed against M5 (reported, not gated — see census
  note on M5).

### Phase 3 — Ring/ledger offload

**Machinery (binding)**: `src/evidence_ring.rs` SPSC ring, real consumer
thread (Chicago: real collaborators, no stub consumer).

- P3.1 Every admitted statement folds into the ring; consumer drains all
  100,000 entries with byte-identity per entry.
- P3.2 Ring push cost printed against M4 (reported, not gated until
  WITNESSED twice).
- P3.3 Ledger offload: drained entries hash-chain into the receipt chain
  (`src/receipt_chain.rs`); chain verifies end-to-end.

### Phase 4 — Anti-vacuity falsifier (GATES EXIT 0 — not skippable)

**Binding**: the malformed twin pair. Leg #41209 exists in exactly two
variants, byte-identical except one field:

- benign: `"1500.00"^^xsd:decimal`
- mutated: `"1500.0025"^^xsd:decimal`

**Gate predicates (ALL must hold)**:
- P4.1 Benign #41209 admits.
- P4.2 Mutated #41209 emits a **typed RefusalCode** (stable-coded per
  `docs/refusals.md`), not a string.
- P4.3 **Zero state mutation**: full state hash before the mutated-admission
  attempt is bit-identical to the hash after. Predicate: BLAKE3 over the
  canonical admission state (all 100k legs' admitted-statement hashes in
  corpus order) computed before and after; equality asserted. Refused
  admissions are non-mutating.
- P4.4 **SUITE-LEVEL FAIL**: the court exits NON-ZERO if ANY of:
  - (a) the mutated twin is ADMITTED (precision check vacuous);
  - (b) an untyped error string is emitted instead of a typed RefusalCode;
  - (c) phase 4 is not executed (e.g. short-circuit by an earlier failure)
    — skipping the falsifier is itself a failure mode, never a pass.
- P4.5 The court must also demonstrate non-vacuity of its own flip: a
  mutation-court-style check that the precision shape would admit the
  mutated literal if the precision constraint were removed (single-shape
  corpus), proving the refusal is caused by the constraint, not by accident.
  Bind to `scripts/mutation_court.sh` protocol.

Phase 4 cannot be waived by any earlier phase's success. It gates exit 0.

---

## 3. Exit-code table

| Code | Meaning | Set by |
|------|---------|--------|
| 0 | All phases pass INCLUDING phase 4 | suite driver |
| 1 | Phase 1 admission failure (untyped error, shape count mismatch) | phase 1 |
| 2 | Phase 2 execution failure (readback mismatch, wall-time print failure) | phase 2 |
| 3 | Phase 3 ring/ledger failure (drain mismatch, chain verify fail) | phase 3 |
| 4 | Anti-vacuity failure: mutated twin admitted | phase 4 (P4.4a) |
| 5 | Anti-vacuity failure: untyped error instead of typed RefusalCode | phase 4 (P4.4b) |
| 6 | Anti-vacuity failure: falsifier not executed | suite driver (P4.4c) |
| 7 | Environment failure: machine-disclosure block incomplete or debug build | §0 |

Codes 4/5/6 are the anti-vacuity exits: **exit 0 is impossible without a
passing phase 4.** A run that prints only phase 1-3 passes and exits 0 has
malfunctioned, not passed — the driver must treat a missing phase-4 record as
code 6.

---

## 4. Falsifier binding detail (leg #41209 twin pair)

`generators/gen_corpus.py` MUST emit both variants of leg #41209 into two
corpus files (identical bytes except the notional literal on the one leg):

```
benign :  ... <reg:notional> "1500.00"^^xsd:decimal ...
mutated:  ... <reg:notional> "1500.0025"^^xsd:decimal ...
```

The court runs the mutated corpus against the same admission state (all other
100k legs identical). Assertions, in order:

1. benign corpus: #41209 present in admitted set;
2. mutated corpus: #41209 carries a `RefusalCode` from `docs/refusals.md`
   with a precision/currency-minor-unit trigger;
3. state hash (BLAKE3 over admitted-statement hash sequence) identical
   before/after the mutated attempt — refusal leaves zero residue;
4. single-shape corpus (precision shape only, mutated leg): ADMITS when the
   constraint is removed → proves the constraint causes the refusal
   (anti-vacuity of the falsifier itself).

## 5. Implementation-bindings index

| Phase | Binding | Path |
|-------|---------|------|
| 1 | Turtle compile + law admission | `src/sql_ast.rs` (`compile_turtle`), `src/law.rs` |
| 1 | Refusal typing | `src/dialect.rs:152-200` `Refusal` |
| 2 | Prepared eval protocol | `src/duckdb_eval.rs` prepared path |
| 2 | Throughput bench legs | `benches/tc_hot_02.rs`; full scale env `TC_HOT_FULL=1` |
| 2 | Corpus generator (100k legs, twins) | `mega/tc-mega-fibo-01/generators/gen_corpus.py` |
| 1 | 1,248-shape layer generator | `mega/tc-mega-fibo-01/ontology/gen_shapes.py` |
| 3 | SPSC evidence ring | `src/evidence_ring.rs` |
| 3 | Receipt chain | `src/receipt_chain.rs` |
| 3 | Ring structural court | `tests/evidentiary_ring_court.rs` |
| 4 | Anti-vacuity protocol | `scripts/mutation_court.sh` |
| — | SPC gate protocol (M6 close condition) | `scripts/spc_court.sh` |
| — | Typed refusal register | `docs/refusals.md` |
| — | CTQ provenance | see See Also below |

## 6. NO-RUST-IN-SETUP

This spec lane wrote no Rust. All Rust artifacts above are owned by the
implementation lane, which binds them to the named paths with zero design
freedom over phase order, predicates, statuses, or exit codes. Deviations
require a filed correction record alongside this file, not a silent re-spec.

## See Also

`docs/jira/v26.10.22/designs/ctq-traceability.md` ·
`docs/jira/v26.10.22/designs/cpk-correction.md` ·
`docs/jira/v26.10.22/designs/pce-baseline.md` · `docs/refusals.md`

## 7. WO-08 execution receipt — two-verifier H_query court

Executed 2026-10-22, lane WO-08, `/Users/sac/graphlaw`, isolated target
`CARGO_TARGET_DIR=target-lane-wo8`.

```text
$ CARGO_TARGET_DIR=target-lane-wo8 cargo test --test two_verifier_court -- --nocapture
running 3 tests
test wo08_formula_consistency_inline_and_large_param_branches ... ok
[WO08] H_query (A == B == reference) = 9ddd051bec7f5a91addcfda7f710e941478eb36c6e535b4cd3923e22bbbc4348
[WO08] diverged: A = 9ddd051bec7f5a91addcfda7f710e941478eb36c6e535b4cd3923e22bbbc4348, B(mutated param) = 8c59c6b515eef0a933a582a9255be4eafbf7992a5c74dbb10e026877d7b07cdb
test wo08_two_verifiers_agree_on_h_query ... ok
test wo08_one_mutated_param_makes_digests_diverge ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

Court count: **3 tests, 3 passed, 0 failed, 0 skipped** (exit 0).

- **Leg 1** (`wo08_two_verifiers_agree_on_h_query`): verifier A compiles the
  §4 SELECT document via `compile_turtle`, executes on a real frozen-duckdb
  instance, extracts `H_query` from the `ExecutionReceipt`. Verifier B uses
  an INDEPENDENT `GraphLawDuckDB` instance (separate connection, statement
  and C1 caches) and recomputes `H_query` from `Canon(G_AST)` + `P_params`
  alone: `h_ast = BLAKE3(Canon(G_AST))` then `Fold(h_ast, P_params)` via
  `fold_params_anchored` with a fresh `blake3::Hasher` — A's hasher, cache,
  and receipt are never consulted. Both digests and B's own eval receipt are
  byte-identical and match the reference
  `admitted_statement_hash(Canon, P_params)`.
- **Leg 2 (falsifier)** (`wo08_one_mutated_param_makes_digests_diverge`):
  one bound parameter mutated between A and B (params are unsealed by
  design) diverges every digest path — receipt hash, fresh-hash fold —
  while still matching the reference formula under the mutated params.
  Divergence is DETECTABLE: no parameter substitution survives the
  cross-check.
- **Leg 3** (`wo08_formula_consistency_inline_and_large_param_branches`):
  `fold_params_anchored` is bit-identical to `admitted_statement_hash` on
  both param-buffer branches (inline <= 64B encoded and 200-byte param),
  pinning this court to the in-file bit-identity tests in
  `src/duckdb_eval.rs` (`receipt_state_clone_path_bit_identical_to_reference`,
  `param_bytes_inline_fold_bit_identical_to_reference`).

### Scope boundary (honest reading)

This court witnesses DETERMINISM and REPLAY agreement between two verifier
instances in one process. It does NOT witness cryptographic non-repudiation
against an adversary: no signatures, no network, no adversary in the loop.
The Σ_PQC / Ed25519 signature layer (`sign_bytes` / `sign_receipt` over the
receipt payload, `src/attest.rs`) is the SQLRDF-13 follow-up and is out of
scope here. Leg 2 proves the two-node hash is a real function of `P_params`
(mutation detectable), not that a signer cannot lie.

Deviation note: IMPLEMENTATION-PLAN WO-08 named Ed25519 signing and
`tests/crossparty_nonrepudiation.rs`; the executed court is
`tests/two_verifier_court.rs` with hash-agreement + falsifier legs only,
per the dispatch correction — the signature leg is deferred to SQLRDF-13.
