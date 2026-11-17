# WASM-BOUNDARY standing (lane 09)

Standing: UNVERIFIED (static read at dd6a80e, 2026-11-17; no court or build run; b3sum not installed).

## Pin state
- registry/ARTIFACTS.blake3 wasm row: `c92caeb6d56792587f2939769678e02b77c868f2919ef8566060e02f3c6c1b9c`, 62909537 B. Same digest in priv/graphlaw.wasm.blake3, js/npm/src/generated/artifact-meta.ts, SECURITY.md, README, CHANGELOG, MIGRATION, docs/WASM.md, graphlaw_ex MANIFEST.json. priv/graphlaw.wasm on disk is 62909537 B. Digest not recomputed; run `b3sum priv/graphlaw.wasm`.
- 10 superseded pins retained as comments; latest c38b1fdb (2026-10-29 GL28-13 relink).
- Reproducibility only same-machine/same-dir (2026-09-30); second-machine rebuild receipt (GWS-2) pending.
- Stale text: docs/WASM.md ~L125-130 calls the re-pin "coordinator-hatch" though the row landed.

## Gap register (table refreshed 2026-11-09): 17 rows — 9 CLOSED, 3 PARTIAL, 4 OPEN, 1 PENDING
| ID | Status | Owed |
|---|---|---|
| G2 | OPEN | no `egress_arrow` op; needs Twin B, pinned-view court, FIBO |
| D3 | OPEN | locus archived; PENDING-GWS-1/4/7/8 open at archive |
| D4 | OPEN | fuzz, memory-wall, exit-code classes not re-run on final artifact |
| W08 | OPEN | no native-vs-wasm differential receipt on pinned artifact |
| G6 | PARTIAL | no bench verdict vs request budgets |
| G7 | PARTIAL | 204-mutant solo verdict not cited; latest is 28-mutant sweep (19/9/0) |
| G8 | PARTIAL | post-fix differential not re-witnessed (last run 15/1 failed at op_differential.rs:181) |
| UU1 | PENDING | full-battery re-run |
Flip log ends 2026-10-26; register cites HEAD 095ce07, not re-diffed against dd6a80e.

## REFUSAL-04
Builtin SQL eval given a non-document-backed reader: a capability refusal, not a boundary refusal; SQL-in-RDF still admits in-module via compile_turtle. Sites: src/tier.rs:26 (typed const), src/rules/builtins.rs:89 (literal), src/sql_ast/refusals.rs:220, docs/refusals.md:79. Residual: call site uses a literal; only tests/sql_rdf_conformance.rs:446 pins the constant. src/bin/graphlaw_ocel_export.rs:278 reuses the label for a missing sqlite3 CLI (different trigger).

## Env-gated courts
- tests/graphlaw_run_eh_court.rs (7 tests): binary from GRAPHLAW_RUN_EH else target/debug/graphlaw-run-eh (absent here, so it would loud-SKIP: panic, exit 101, not a witness); artifact GRAPHLAW_WASM, stripped from child and passed via --wasm. 7/0 witness dates 2026-10-24, predates current pin.
- `pending_artifact_pin_capabilities_frame_exit97_uninitialized_element` accepts exit 0 or 97: passes on any pin; stale "pending" label.
- Others: wasm_frame_bound (4), enclosure, boundary_fuzz, env_isolation, size, native_differential (zero comparisons if wasm32-wasip1 absent), op_differential, op_forward_compat, wasm_abi, wasm_fibo_*_eh, wasm_duckdb_lifecycle_eh, registry_artifacts, dvp_settlement, rehypothecation_gate, gws4.
- No receipt after 2026-10-29 shows these green on pin c92caeb6.

## Runner and ADR
runner-exit-codes.md matches RunError::code (0, 2, 84, 86, 90-99; re-diffed 2026-11-09). Ladder: --wasm > GRAPHLAW_WASM > target/wasm32-wasip1/wasm/graphlaw_wasm.wasm. ADR-wasmi-retirement Implemented (2026-10-24); its See Also points to archived v26.10.23 ROADMAP; D1a/D1b PURE-artifact ticket open, no standing evidence.

## Next actions
1. Build graphlaw-run-eh (`--features runner-eh`); run run_eh_court, frame_bound, enclosure, registry_artifacts on priv/graphlaw.wasm; receipt output.
2. b3sum vs the pin surfaces. 3. Receipt W08, re-witness G8. 4. Decide G2/D3/D4. 5. Refresh register header and flip log; fix WASM.md wording; tighten the pending-pin court.
