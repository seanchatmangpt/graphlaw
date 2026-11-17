# RISK-SWEEP — v26.11.17 (2026-11-17)

Lane 19. Static grep of src/ (61 files), tests/ (1639 files), wasm/ (5), scripts/ (22). Ranked by risk to law-admission invariants.

## Headline counts
| Marker | Count | Verdict |
|---|---|---|
| FIXME / todo! / XXX | 0 | clean |
| unimplemented! | 3 | in a `//!` doc fence, src/abi/lease.rs; not compiled |
| TODO | 2 | 1 doc, 1 real (tests/purrdf_capabilities.rs:1005) |
| #[ignore] | 1 | tests/corpus_conformance.rs:120 |
| std::thread::sleep | 11 | 2 files / 4 sites in src (incl. bins), 7 in tests; R2 |
| panic! outside cfg(test) | 3 | R3 |
| unreachable! outside cfg(test) | 6 | R3 |
| unwrap/expect outside cfg(test) | ~70 in 20 files | R4; upper bound (lines before first cfg(test)) |
| SKIP text | 83 hits | all loud and typed; no silent skip found |

## R1 (MEDIUM) lock-poison expects on the receipt/evidence path
src/evidence_ring.rs: 15 expects (:347,405,418,451,472,522,524,550,632,659,716,723 ...); src/duckdb_op.rs: 6 (:573,703,841,906,923,929). A poisoned mutex cascades into every later admission: fails closed but takes the ring down. Confirm deliberate and court-covered, else map to a typed error.

## R2 (MEDIUM) std::thread::sleep — CLAUDE.md: "A sleep is a defect"
- src/evidence_ring.rs:700, :798: drainer park loop (EMPTY_PARK_MS), polling on the receipt-emission path. Highest priority; replace with Condvar/channel.
- src/bin/graphlaw_run_eh.rs:79 watchdog tick (documented epoch pattern, unannotated); :278 `sleep(3600s)` Mode::Hang fixture ("SLEEP-CLASS: SCHEDULING"), acceptable.
- Tests annotated `SLEEP-CLASS: POLL-STEP`: wasm_frame_bound_court.rs:254,354; wasm_env_isolation_court.rs:103,209. NOT annotated: wasm_enclosure_court.rs:225, graphlaw_run_eh_court.rs:113, common/enclosure.rs:235.
- Action: annotate/replace the 3 untagged test sleeps and the ring park; consider a gate failing on untagged sleeps.

## R3 (LOW-MEDIUM) panics in admission-adjacent code
- src/law.rs:544, :604: `unreachable!` on a new non_exhaustive `purrdf::ReasonerLimit` variant: a dependency bump turns a limit breach into a process abort instead of a typed Deny (law path).
- src/abi/ops/governance.rs:106 `unreachable!("matched LeaseRefused above")`.
- src/registry/emit_mcp.rs:59, :101 panic on out-of-vocabulary type (emit time).
- src/duckdb_eval.rs:170 unreachable on RdfTextDirection; graphlaw_attest.rs:121,144; sa2a_portability_verify.rs:88 (CLI).

## R4 (LOW-MEDIUM) unwrap/expect in parse and admission paths
- src/receipt_chain.rs:614 `.expect("promoted=true is only ever set alongside explicit_ceiling")`: highest-value expect to convert to a typed error.
- src/egress/arrow.rs:94-108: 6x fixed-slice try_into().unwrap() (infallible by type; wire-format edge).
- src/sql_ast.rs: 7 expects, 6 "checked above" (:452,514,555,575,607,713,2508).
- src/receipt_chain/record.rs:382, duckdb_eval.rs:512,549, abi/ops/meta.rs:61-62, emit_mcp.rs:98-139, smon.rs:30,121,242, abi.rs:210,340, abi_manifest.rs:215, receipt_store.rs:484, sql_ast/ast.rs:14 (`chars.next().unwrap()` — check non-empty), wasm/src/ffi.rs:148.
- bins: graphlaw_ocel_export.rs 19 sites; dl_dualroute.rs:443 HOME expect (CLI).

## R5 (LOW) skips and ignores
- corpus_conformance.rs:120 is not run by default; the large-vocabulary corpus is not a gate unless CI passes --ignored (check CI).
- tests/purrdf_capabilities.rs:1005 TODO(P4): PURREMB nearest-neighbour leg; tracked.
- Loud SKIP at tests/common/mod.rs:479-492. Scripts: receipt_chain_sync.sh:43 TAMPER-CHECK-SKIP (empty receipts dir = vacuous pass; confirm nonzero exit — highest-risk skip), receipt_replay_court.sh:100, count_gate.sh:12,37, gen_w3c_manifest.sh:88-94, mutation_families.sh:148, wasm_unknown_unknown_smoke.sh:27,56. src/bin/sa2a_run14_verify.rs:227 emits "SKIP: unreadable fixture name" — confirm nonzero exit.

## Clean
No todo!(), FIXME, live unimplemented!(); no thread::sleep in law.rs, receipt.rs, policy.rs, hooks.rs; wasm/src/lib.rs unwraps are all inside tests.

## Follow-ups (priority)
1. Replace evidence_ring sleep-park; tag/replace 3 test sleeps.
2. Typed deny for receipt_chain.rs:614 and law.rs:544/604.
3. Verify exit semantics of receipt_chain_sync.sh:43 and sa2a_run14_verify.rs:227.
4. Confirm CI runs `--ignored` for corpus_conformance.
5. Decide and document the lock-poison policy.
Caveat: counts are line-position heuristics, not AST analysis; "infallible" verdicts rest on adjacent comments and fixed-size types.
