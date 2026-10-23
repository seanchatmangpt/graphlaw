# SPOF scan — consolidated record (8 lanes, v26.10.22)

All 8 scans landed. Full lane reports carry per-site file:line; this record
carries the cross-cutting findings ranked by severity.

## Tier 1 — witnessed or live

1. **Legacy (None,None,None) hash arm = 5-lane convergence SPOF.**
   duckdb_op exec raw-SQL admission (SQLRDF-17) + rules/builtins path:
   non-canonical bytes enter the receipt identity (CAP-10), raw SQL bypasses
   admission (CAP-4), receipts minted for non-executed DDL (F2), circumvention
   channel live (CAP-7), cursor/identity conflation (SPOF-8). DELETED by the
   legacy-fallback lane (SQLRDF-17 closure).
2. **mutating_sql keyword-prefix classifier** (SPOF-4): gates C1 epoch bump +
   prepared-handle flush by leading keyword — CTE-wrapped DML, CALL, PRAGMA,
   EXPORT bypass invalidation. Silent staleness.
3. **Mutex poisoning = process-wide cache brick** (SPOF-4): QueryCache is
   Arc-shared; .expect on every lock; no into_inner recovery.
4. **Disk exhaustion** (SPOF-5): witnessed 3× this session killing all lanes;
   no CI disk guard; tc_hot benches measure the volume they write to.
5. **64-bit receipt-IRI truncation** (SPOF-8): birthday collision at ~4.3B
   receipts — the real 25M-ledger identity bound.
6. **cursor_id process-scoped** (SPOF-8): restart/second process reproduces
   byte-identical ids; registry last-writer-wins; unbounded.
7. **No panic/OOM containment at the sole admission entry** (SPOF-1): no
   catch_unwind, no worker isolation; RDFC-1.0 canonicalization runs
   unbounded (exponential blank-node case); depth guard covers shapes only.
8. **Legal-layer root SPOF** (SPOF-6): enforcement claims terminate in a
   receipt stream witnessed by a 2-file pin, issued by a license file with
   no digest pin of its own; §8.2 is reflexive (this audit is itself
   spoliation-held, no audit carve-out).

## Tier 2 — designed-in, latent until a trigger

9. **Single purrdf =2.0.2 canonicalizer** (SPOF-7): upstream at 3.0.1; the
   exact pin is a fork-point; vendor patch READY_UNSUBMITTED; deny.toml
   denies MulanPSL-2.0 (blocks 3.x).
10. **Receipt-surface pin covers 2 files** (SPOF-2/6): duckdb_op.rs and
    evidence_ring.rs receipt-relevant but unpinned.
11. **Single licensor signing key** (SPOF-7/8): Ed25519 exact-pin, no
    rotation/threshold; Σ_PQC UNMET.
12. **Single global ring, no tenant isolation** (SPOF-2).
13. **CI single-combo; --all-features only at release; runner zero
    coverage** (SPOF-3; = SQLRDF-19).
14. **Hardcoded /Users/sac paths in tests/bins** (SPOF-3): insa_law_order
    now guarded (loud skip landed); ~7 more sites.
15. **OnceLock double-init corrupts shared ledger** (RUST-4 HIGH).
16. **NDJSON write-fail advances parent anyway** (RUST-4 HIGH).
17. **Statement identity conflation on the legacy arm** (SPOF-8 #2).

## Tier 3 — doc-comment guards, not invariants

18. C1 epoch restart-reset benign only while in-process.
19. raw_connection caller MUST-bump is protocol-only.
20. stmt_cache mirror over-reports past 256 LRU evictions.
21. "wait-free" doc vs Mutex-guarded push (evidence_ring).
22. Pre-store epoch check is dead weight (consult-time filter is the real
    guard); comment overclaims "aborted".
23. FRAME_FIELDS offset transposition undetectable for equal-length
    adjacent fields.
