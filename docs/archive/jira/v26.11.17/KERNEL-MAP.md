# KERNEL-MAP — src/ module map (2026-11-17)

Scope: `src/**/*.rs`, 32,549 LOC incl. bins. Source of truth: src/lib.rs. Static; LOC from `wc -l`. Feature gates: `abi` gates abi_manifest, qualification, registry; `pack-tools` gates smon; `evidentiary-ring` gates evidence_ring (native-only, off by default).

## 1. Admission kernel
| Module | LOC | Purpose | Entry points |
|---|---|---|---|
| law.rs | 1278 | content-addressed LawState (BLAKE3 of RDFC-1.0), receipted transitions, leases, clocks, bounded N3 | LawState, Step, Receipt, Ceiling, Lease/SignedLease, Clock, reason_n3_bounded |
| tier.rs | 278 | routes document to RDF, N3 rules, or SQL-in-RDF | Tier, DialectDecl, admit_declared, admit_bytes_tiered |
| plan.rs | 643 | independent plan admission by replay | Plan/PlanBuilder, Action |
| policy.rs | 474 | STRONG-CYCLIC FOND policy check (ferroplan) | admit_parsed, PolicyRefused |
| hooks.rs | 466 | knowledge hooks (kh:) | Hook, HookPack, Firing, Verdict |
| shacl_guard.rs | 378 | bounded SHACL nesting/limit guard | check_shape_depth, check_shape_limits |
| qualification.rs | 199 | Chicago qualification algebra (evidence only) | evaluate, compose_courts |
| proof_lease.rs | 141 | binds check_proof to source bytes via lease digest | verify_leased_proof |

## 2. Receipts and attestation
attest.rs (809, Ed25519, TrustedKeys, sign/verify_receipt), receipt.rs (256), receipt_store.rs (497, `<dir>/<subject>/<blake3>.json`), receipt_chain.rs + frame/record/store (2362 total, verbatim port, wire shapes frozen).

## 3. Query execution, evidence, egress
duckdb_eval.rs (3314, ExecutionReceipt), duckdb_op.rs (1760, ABI op `duckdb`), sql_ast.rs + ast/refusals/serializer (7590 total, largest), sql_statement_kinds.rs (19), query_cache.rs (174), rules/builtins.rs (139, builtin_eval_sql), evidence_ring.rs (931, SPSC ring, rtrb 4096), query_ledger* (1052), egress/{arrow 277, collector 348} (CanonArrow).

## 4. Surface, registry, tooling
dialect.rs + refusal + sniff (590/282/468); abi.rs + lease + ops/* (JSON ABI mirroring wasm); abi_manifest.rs (260); registry.rs + definitions/emit_json/emit_mcp/emit_ttl (2395); smon.rs (541); lib.rs (267, re-export modules around PurRDF); 11 bins, ~3,058 LOC (dl_dualroute, graphlaw_attest, graphlaw_ocel_export, graphlaw_registry, graphlaw_run_eh, graphlaw_verify, n3_derive, sa2a_*, smon_*).

## 5. Dependencies (crate:: grep; confirm fragments)
- law <-> attest, receipt, plan, hooks, shacl_guard, receipt_store are mutually importing: the main coupling hotspot (legal in Rust).
- attest uses egress, law, proof_lease; proof_lease uses attest (odd edge).
- policy and qualification are standalone leaves.
- evidence_ring -> query_ledger, receipt_store; rules::builtins -> duckdb_eval, sql_ast; abi -> dialect, duckdb_op, law, registry.
Layering bottom-up: dialect/sql_ast; law + tier/plan/hooks/shacl_guard/receipt/attest/proof_lease; receipt_store/receipt_chain; evidence_ring/query_ledger; abi/registry/bins.

## 6. Doc coverage gaps (md files mentioning module name; crude)
proof_lease 1, shacl_guard 1 (only a bench report), receipt_store 3, smon 3, abi_manifest 4, query_cache 4, evidence_ring 6, query_ledger 7 (one design doc), receipt_chain 8, qualification 10, tier 14. Bare `pub mod` with no `///` in lib.rs: abi, abi_manifest, attest, dialect, hooks, law, plan, policy, qualification, query_ledger, receipt, receipt_chain, receipt_store, registry, shacl_guard, tier.

## 7. Caveats
Edge list from grep (some `crate::n3` fragments); entry-point lists truncated at 14; sql_ast/registry/abi submodules not read in depth.
