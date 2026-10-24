# Whitepaper: GraphLaw v26.10.22 — Sovereign In-WASM DuckDB Execution

*Status: Authoritative release specification grounded in witnessed receipts
from the v26.10.22 wave. Items pending final quiescence runs are explicitly
marked.*

---

## Abstract

GraphLaw v26.10.22 resolves the historical split in enterprise semantic
computing: the separation between governed graph semantics (RDF/OWL/SHACL) and
high-throughput relational execution. The complete DuckDB analytical engine
executes inside a sealed WebAssembly sandbox (`priv/graphlaw.wasm`, BLAKE3
`472d5557…`, 58,121,500 B), where every SQL statement is compiled from an RDF
graph, admitted through SHACL shapes before engine contact, executed by a
statically linked frozen-duckdb kernel, and sealed into a cryptographically
chained receipt. Host delegation is eliminated: the identical artifact runs
deterministically across host and wasm environments without ambient authority.

## 1. The Problem: The Semantic Graveyard

Since 2008, the Financial Industry Business Ontology (FIBO) promised
deterministic financial modeling. The barrier was architectural: legacy RDF
triple-stores fell off performance cliffs under operational write volume,
forcing institutions to treat FIBO as a static reference taxonomy while
production trading, risk, and settlement systems remained fragmented across
brittle relational databases and bespoke glue code.

This architecture suffered from two systemic flaws:

- **Governance without execution**: Admission layers that could refuse but not
  compute.
- **Execution without evidence**: Engines that computed but could not legally
  prove what they executed.

The pre-v26.10.22 codebase embodied both: a `#[cfg]` build gate compiled the
DuckDB surface out of the WASM artifact, silently falling through to
`unknown op 'duckdb'`.

## 2. Architecture & Invariants

```
[Caller] → JSON envelope (gl_alloc/gl_call/gl_free, linear memory)
→ IN-WASM: Turtle parse → SHACL admission (fail-closed)
→ H_query = BLAKE3(Canon(G_AST) ∥ P_params)   ← module-computed
→ SQL codegen → frozen-duckdb C API (in-module, direct calls)
→ ExecutionReceipt (rows_emitted, pre/post state, epoch CAS)
→ CHAIN.blake3 receipt chain (BLAKE3 fold, replayable)
```

### Core System Invariants

1. **Zero-Ambient Authority**: The admission gate computes the query identity
   digest in-module. A caller cannot forge or pre-digest a payload past the
   SHACL reticle.
2. **Typed Refusals Before Engine Contact**: REFUSAL-01 through REFUSAL-05
   (including WHERE-less mutation containment) execute at both the SHACL and
   compiler layers; anti-vacuity twin suites prove the rejection gates flip
   under mutation.
3. **One Unit of Law**: No target-architecture feature gating. The
   `graphlaw-wasm` crate typechecks and links the complete engine directly for
   `wasm32-wasip1`.
4. **End-to-End W3C Exception Handling**: Standard W3C EH (`try_table`/`Tag`
   sections) across the binary archive, validated by an opcode gate refusing
   legacy-EH objects under wasmtime 48.

## 3. Witnessed Results

### Engine Root Cause (SQLRDF-24 Trap Resolution)

The `uninitialized element` trap previously preventing guest instantiation was
traced to a musl `atexit` slot re-attachment and walker offset defect. Resolved
via replacement of `atexit.c.obj` and completion of the reactor runtime
contract (`.init_array` extension registration, explicit `atexit` no-op
overrides), verified across core dumps and symbolized static inspection.

### Qualification Battery

- **Lifecycle Courts (3/3)**: Full `CREATE → INSERT → SUM → UPDATE` lifecycle
  executed in-WASM with bit-exact readbacks.
- **System Courts**: boundary fuzzing (1,172 frames / 0 defects),
  memory-wall plateau (8/0, re-lawed per-cycle-bounded Δ ≤ 2 pages/cycle), WASM ABI
  (25/0), and exit-code verification (7/0).
- **Core Verification**: Dialect conformance (36/0), refusal family (75/0), SPC gate (latest witness FAIL Cpk 0.0008 under fleet load — solo-machine rerun owed; prior PASS 1.7573),
  npm packaging (198/198), and `CHAIN.blake3` integrity
  (VERIFIED at exit-0 `receipt_chain_sync.sh --check`, 2026-10-23; re-baseline 4 documented in `receipts/2026-10-23-chain-rebaseline-note.md`; entry count cited at the pinned release commit).

### Banking Domain Verification (FIBO / wasmtime 48)

*Witness receipts: BCBS/Rehypothecation/UBO runs receipted in-wave; DvP
witness `receipts/2026-10-23-banking-4of4-witness.md`.*

- **BCBS 239 Risk Aggregation**: 3/3 green, Petri-net conformance (fitness 1.0,
  precision 1.0 via pm4pytest).
- **Rehypothecation Limits**: 3/3 green, verified custody chain traversal.
- **UBO Sanctions Unrolling**: 3/3 green, multi-tier corporate graph traversal.
- **DvP Settlement**: 4/4 green after the INSERT serializer fix — the original
  defect (independent AST traversal swapping column-value associations) caused
  a fail-closed atomic batch abort at Member 1 of 3 with an emitted partial
  failure receipt; the fix pairs assignColumn/assignValue by index via a
  quote-aware document-order scan, pinned by an exact-SQL regression unit.

### Performance Envelope (Divan, 100 samples/leg)

| Execution Path | Measured Median | Operational Context |
| :--- | :--- | :--- |
| **C1 Query-Identity Hit** | **2.541 µs** | BLAKE3 canonical digest match + cache short-circuit; zero engine contact. |
| **Raw Engine Prepared** | **117.4 µs** | Frozen-DuckDB execution floor for pre-compiled physical plan. |
| **Cold Graph-Backed Eval** | **23.13 ms** | Turtle parsing + SHACL admission + relational codegen + plan construction (receipts/2026-10-23-engine-baseline.md; fresh-connection leg 16.42 ms). |

The ~9,100× delta (median-to-median: 23.13 ms / 2.541 µs) is a regime change: the cognitive cost of inductive proof
generation is paid once upon admission. Subsequent requests pay only a
constant-time cryptographic identity check.

### Columnar Egress Baseline

Canonical N-Triples text rendering currently measures **9.246 ms / 10k rows**
against **1.354 ms** raw DuckDB row materialization. This establishes an
empirical **6.83× serialization tax**, providing the performance mandate for
the native Arrow IPC egress implementation.

## 4. Dialect & Language Coverage

The compile surface supports 35 PEG statement kinds:

- **Transaction & Sequences**: Document sequences (`sql:StatementSequence`)
  supporting atomic and pipelined execution modes with `BatchReceipt` emission
  (`status`, `state_delta`, rollback verification).
- **Dialect Parity**: Direct support for `MERGE INTO`, `EXPLAIN`, `PRAGMA`,
  `SET`, `INSTALL`, and `LOAD`.
- **SQLRDF-25 Clamped Extensions**:
  - **Row 1 & 2**: Star projection modifiers (`EXCLUDE`, `REPLACE`) and
    `COLUMNS('regex')`.
  - **Row 3**: `PIVOT` and `UNPIVOT` statements with SHACL shape constraints
    enforcing closed input boundaries.
  - **Rows 4 & 5**: JSON extraction operators (`->`, `->>`) and composite
    literals (depth limit 64, typed refusal codes 48 and 49).

## 5. Architectural Standards & In-Flight Implementation

- **CanonArrow Egress (ADR-26.10.23-B Clamped into v26.10.22)**:
  - Memory Layout: `ArrowEgressHeader` — 128 bytes total,
    `repr(C, align(64))`, 24-byte reserved tail (repr(C) padding math per
    the in-source derivation).
  - Dual-Witness Attestation (`receipt_v2`): Defined and unit-tested (golden
    v1-preservation pin, domain separation, `CANON_ARROW_EMPTY_V1`); production
    emission wiring in `duckdb_eval` is the Phase 2 in-flight item.
  - Domain Constants: Hardened null digests (`CANON_ARROW_EMPTY_V1`) ensuring
    backward compatibility with legacy receipt verifiers.
- **Steady-State Optimization Horizon**:
  - Prepared-Statement Ring Cache (P1): Keyed by module-computed
    H_query to drop parameterized queries from 117.4 µs toward the low-µs floor.
  - Parametric Workload Amortization: Establishing the empirical latency curve
    T(c) = c·T_cold + (1−c)·T_hot across Zipfian distribution sweeps.

## 6. Scope Boundaries & Constraints

1. **Not a Distributed Lakehouse**: GraphLaw is a sovereign, fixed-cost compute
   substrate running in-process per core. It is not an elastic, multi-tenant
   cloud data warehouse.
2. **Not Zero-Parse**: The module canonicalizes and hashes every incoming
   payload. The governance boundary cannot be bypassed; what is eliminated is
   intermediate text marshalling and plan re-derivation.
3. **Parametric Latency**: The 2.541 µs floor represents repeated identical
   query evaluation. First-contact queries must pay the 23.13 ms admission fee
   to construct the formal evidentiary record.

## 7. Evidentiary Basis & Non-Vacuity

Every performance figure and invariant cited in this specification maps to an
entry in the `CHAIN.blake3` ledger (per-file verify exit-0; re-baselines
documented, not silent) (VERIFIED at exit-0 check,
2026-10-23; entry count is chain-extension-live and cited at the pinned
release commit, not inline). Under the platform's anti-vacuity doctrine, every qualification
court is paired with an inversion twin to ensure that any relaxation of
security, schema, or memory bounds forces an immediate exit code 1.
