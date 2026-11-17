# Court Inventory — graphlaw tests/ (2026-11-17)

Method: `grep -c '#\[test\]' tests/*.rs`; env vars from env::var greps; anti-vacuity column is a keyword heuristic (anti-vacuity/inversion/vacuous/negative control/mutant), not proof. Static only.
Totals: 131 .rs files, 982 #[test]; plus 2 Python files and helpers common/{mod,enclosure}.rs. ~30 files need feature `abi`; smon courts need `pack-tools`.

## 1. Courts by domain (#tests)
- **sql-rdf:** sql_rdf_dialect 36, laundering_corpus 23, conformance 14, k5_hostile 8, canonical_cross_dialect 6, json_operator 2, star_modifier 4, set_operations 16, columns_projection 4, composite_literal 4, pivot 3, duckdb_statement_kinds 8, sql_in_rdf_duckdb_parquet 1, purrdf_capabilities 18, purrdf_columnar_duckdb 1, parser_capability_probe 7, parse_refusal 12, sniff 14. No marker: set_operations, sql_in_rdf_duckdb_parquet, purrdf_columnar_duckdb, parser_capability_probe, parse_refusal.
- **n3:** limit_census 13, hypothesis_isolation 11, budget_cliff 10, quantifier 10, transition_isolation 9, halt_latency 6, fuse 5, proof_pipeline 5, proof_lease 4, corpus 1, boundary_fuzz 1, chicago_n3_datalog 6, dl_dualroute_vector_pin 1. No marker: limit_census, hypothesis_isolation, budget_cliff, quantifier, proof_lease, corpus, chicago_n3_datalog, dl_dualroute.
- **shacl:** shacl_boundary_cliff 8, shacl_depth_guard 6 (both marked).
- **abi:** abi_manifest 10, response_builders 10, bounds 6, canonical_abi 5, pn64 9, capabilities_digest 6, mcp_manifest 6, api_stability 2, wire_completeness 7, registry_artifacts 22, registry_dispatch 14, registry_limits 7, resource_limits 10, backend_authorities 8.
- **wasm:** wasm_abi 25, duckdb_op 9, enclosure 9, memory_wall 8, frame_bound 4, size 4, governance_preflight 4, duckdb_lifecycle_eh 4, env_isolation 3, coldchain_e2e 3, n3_admission 3, fibo_fixtures_eh 3, boundary_fuzz 2, native_differential 2, fibo_egress_eh 1, graphlaw_run_eh_court 7, gws4_ipc_injection 3, gws12_coldchain_cross_engine 2.
- **attest/receipts:** trust_attest 12, two_verifier 3, receipt_integrity 5, receipt_v2 6, v2_emission 8, feedback 7, store 4, chain_golden 5, chain_validator 4, epoch_migration_port 4, refusal_details 7, refusal_crosswalk_differential 6, verify_cli 7, ocap 3, attach_bypass 3, evidentiary_ring 12.
- **lease:** k6_hostile_lease 10, receipt_lease 6, lease_authorization_differential 4, n3_proof_lease 4 (no marker in any).
- **chicago:** cross_authority 72, fibo 62, validation 12, boundary_constraints 4, n3_datalog 6.
- **finance/domain:** bcbs 3, dvp_settlement 4, rehypothecation 3, ubo_sanctions 3, amortization 1, combinatorial_eight 1, tc_mega_antivacuity 5, cold_chain_ocel 1, ocel_export 2.
- **plan/law/determinism:** plan_admission 7, builder 6, negation 9, policy_admission 7, law_state 8, qualification_algebra 7, strata_gate_differential 3, temprun_framing 5, insa_chain_differential 5, insa_law_order 6, determinism 7, p28_determinism 4, p28_hot_path_bench 2, query_cache 9, query_ledger 9, tier 7, sieve 10, knowledge_hooks 11, smon 2/5/4.
- **conformance/corpus:** w3c_conformance 7, w3c_manifest 1, corpus_conformance 13 (1 #[ignore]), differential 1, op_differential 16, op_examples 3, op_forward_compat 5, sa2a_run14 6, readme_first_ten_minutes 3, test_parquet_and_builtin_eval 3, test_edge_sovereign_ingestion 2.
- **meta guards:** no_mock_markers 4, no_mutant_markers 1.

## 2. Env-gated / ignored / loud-skip
`common::host_or_skip` prints `// SKIP(typed) [test]: ...` and returns None; the test then passes without asserting, so a pass does not prove the court ran. SKIP text appears in 17+ files (bcbs, dvp, evidentiary_ring, run_eh, gws4, n3_boundary_fuzz, rehypothecation, ubo_sanctions, w3c_manifest, wasm_boundary_fuzz/duckdb_lifecycle_eh/env_isolation/fibo_*/frame_bound/native_differential/size).
| Gate | Effect |
|---|---|
| `GRAPHLAW_WASM`, `CARGO`, `CARGO_TARGET_DIR` | all wasm courts; missing artifact -> typed SKIP |
| `GRAPHLAW_REQUIRE_WASM_PIN` | registry_artifacts: pin mandatory, no skip |
| `GRAPHLAW_RUN_EH` | run_eh_court, wasm_enclosure, wasm_env_isolation |
| `GRAPHLAW_DIAG_NO_EH`, `GWS10_DEBUG` | diagnostics |
| `GRAPHLAW_ATTACH_ROOTS` | attach_bypass, gws4 |
| `COLDCHAIN_CSV_DIR`, `DUCKDB_BIN` (/opt/homebrew default) | gws12; host-specific |
| `INSA_LAW_PACK*`, `REFUSAL_VOCAB_PACK`, `HOME` | insa_law_order, refusal_crosswalk; machine-local defaults |
| `UPDATE_ASSET_MANIFEST`, `--ignored` | corpus_conformance; the ignored test is not run by default |
| `CONFORMANCE_REPORT` | report output |

## 3. Anti-vacuity gaps (heuristic candidates)
1. Skip-capable courts without a counted-SKIP guard: wasm_abi, boundary_fuzz, frame_bound, fibo_egress_eh, run_eh_court, gws4, gws12.
2. Lease family has no marker.
3. Receipt/attest family: trust_attest, two_verifier, receipt_v2, v2_emission, chain_*, store, refusal_details.
4. Large unmarked courts: chicago_fibo 62, n3_limit_census, n3_budget_cliff, n3_quantifier, registry_dispatch, op_differential, set_operations.
5. Only tc_mega_antivacuity is a named anti-vacuity court (FIBO #41209 twins).
6. Machine-local defaults ($HOME, /opt/homebrew) in insa_law_order, refusal_crosswalk, gws12 need an override or loud skip.

Caveat: plan_negation has no marker; whether SKIP-text files all reach host_or_skip, and what env-defaulted courts do when the pack is missing (panic vs skip), was not verified.
