# BACKLOG

Standing: UNVERIFIED

Rows appended from `docs/jira/v26.11.25/AUDIT-WAVE3.md`. Status is OPEN until a lane receipt
says otherwise; the coordinator updates it.

| Id | Item | Reason deferred | Source rows | Status |
|---|---|---|---|---|
| B-DEF-1 | Advertise hidden duckdb op in capabilities.ops | Intentional HIDDEN_OPS; changes manifest contract | R2-7 | OPEN |
| B-DEF-2 | Cargo.toml autobins, runner-eh docs.rs, wildcards, MSRV check, deny.toml expiries (lapse 2026-11-28) | Coordinator-only manifest | R6-3, R6-6..R6-9 | OPEN |
| B-DEF-3 | Lease issued_unix Option, DialectDecl, DeriveN3 types; non_exhaustive on EvaluatorError and ProofLeaseError | Breaking public API | R6-1, R6-2, R6-10 | OPEN |
| B-DEF-4 | Pool.Leases sharding | Design change | A1-6 | OPEN |
| B-DEF-5 | Kill-switch cluster propagation beyond the hook | Needs distribution design | A2-8 | OPEN |
| B-DEF-6 | ABI version negotiation on the wire | Protocol change | none | OPEN |
| B-DEF-7 | frozen-duckdb gating | Violates DuckDB-is-not-a-feature convention | R6-7 | OPEN |

- B-DEF-8 OPEN: ReceiptRecord omits DenialPolarity (receipt_meta() replays ADMITTED so non-ADMITTED records fail recompute_chain_hash). Add `denial: u64` (#[serde(default)]) + restore via DenialPolarity; update literals in tests/{determinism_court,bcbs_aggregation_court,receipt_epoch_migration_port_test,receipt_chain_*}.rs; court: non-ADMITTED receipt verifies after JSON round-trip. Deferred from v26.11.25 (public struct change).
