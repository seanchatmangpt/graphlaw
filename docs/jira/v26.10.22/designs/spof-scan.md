# SPOF scan — consolidated record (8 lanes, v26.10.22)

All 8 scans landed. Cross-cutting findings ranked by severity; per-scan
details in the lane reports (SPOF-1 admission, SPOF-2 evidentiary, SPOF-3
build/CI, SPOF-4 state, SPOF-5 operational, SPOF-6 legal, SPOF-7 deps,
SPOF-8 receipt identity).

## Tier 1 — witnessed or live

1. **Legacy (None,None,None) hash arm = 5-lane convergence SPOF.**
   duckdb_op exec raw-SQL admission (SQLRDF-17) + rules/builtins path:
   non-canonical bytes enter the receipt identity (CAP-10), raw SQL bypasses
   admission (CAP-4), receipts minted for non-executed DDL (F2), circumvention
   channel live (CAP-7), cursor/identity conflation for same-text-same-params
   (SPOF-8). Priority 1 post-integration.
2. **mutating_sql keyword-prefix classifier** (SPOF-4, NEW): gates C1 epoch
   bump + prepared-handle flush by leading keyword — CTE-wrapped DML, CALL,
   PRAGMA, EXPORT bypass invalidation. Silent staleness, not refusal.
3. **Disk exhaustion**: witnessed 3× this session killing ALL lanes at once;
   no CI disk guard (grep zero hits), ~35 target-* dirs on one volume.
4. **Mutex poisoning = process-wide permanent cache brick**: QueryCache is
   Arc-shared across connections; .expect on every lock; no into_inner
   recovery; one panic under the lock kills all later queries.
5. **Receipt-surface pin escape**: pins exactly duckdb_eval.rs +
   sql_ast.rs; duckdb_op.rs and evidence_ring.rs are receipt-relevant but
   unpinned (CAP-7 + SPOF-2 converge).
6. **64-bit receipt-IRI truncation** (SPOF-8): birthday collision at
   ~4.3B receipts (~172× above 25M) — the real 25M-ledger identity bound,
   on the external identifier, not the 256-bit digest.
7. **cursor_id process-scoped** (SPOF-8): restart/second process reproduces
   byte-identical cursor ids; registry last-writer-wins; unbounded registry.
8. **Hardcoded /Users/sac paths in tests/bins** (SPOF-3): insa_law_order.rs
   panics bare (no guard) under CI --all-targets; ~8 more sites.

## Tier 2 — designed-in, latent until a trigger

9. **Single purrdf =2.0.2 canonicalizer** (SPOF-4/7): every receipt and
   LawState id inherits one RDFC-1.0 impl; upstream at 3.0.1 — the pin is a
   fork-point, not a stability point; vendor-fork precedent exists
   (READY_UNSUBMITTED). No second independent canonicalizer for cross-check.
10. **No panic/OOM containment at the sole admission entry** (SPOF-1): no
    catch_unwind, no worker isolation; one hostile doc kills the native
    process. RDFC-1.0 canonicalization runs unbounded (exponential
    blank-node case); depth guard covers shapes only — data-graph recursion
    unprotected.
11. **Single licensor signing key** (SPOF-8): ed25519-dalek exact-pin,
    Σ_PQC UNMET, no rotation/threshold; compromise = forge-all, loss =
    sign-none.
12. **Single global ring, no tenant isolation** (SPOF-2): RingDescriptor has
    no tenant field; cross-tenant evidence commingling undetectable by
    construction.
13. **License text itself unpinned** (SPOF-6 L5): LICENSE-P28-SIL.md has no
    digest in ASSETS/ARTIFACTS; §8.2 is reflexive (this audit workflow is
    itself spoliation-held, no audit carve-out).
14. **CI single-combo** (SPOF-3): --all-features only at release; runner
    bin zero coverage (SQLRDF-19); no-disk-guard.

## Tier 3 — doc-comment guards, not invariants
15. C1 epoch restart-reset benign only while in-process (durable sidecar
    would collide); guard is a deferred comment.
16. raw_connection caller MUST-bump is protocol-enforced only (no debug
    assert/dirty flag).
17. stmt_cache mirror over-reports past 256 LRU evictions (telemetry only,
    disclosed).
