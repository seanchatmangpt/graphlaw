# CI & SCRIPT MAP (v26.11.17, lane 05, static read only)

## 1. Workflows (.github/workflows/)
| file | triggers | jobs | scripts/ called | notes |
|---|---|---|---|---|
| ci.yml | push main, PR | native, castle-diff, wasm-court, workspace-members, semver, deny, audit, external-oracle | none | fmt/clippy/test/doc, graphlaw-registry --check, wasip1 build |
| js.yml | push, PR | js-build, js-test-node, js-test-browser, js-package-lint, js-drift | none | b3sum pin of priv/graphlaw.wasm.blake3 |
| elixir.yml | push, PR | graphlaw_ex-courts | none | mix test; GRAPHLAW_REQUIRE_ENGINE=1 |
| mutation.yml | dispatch, cron `17 5 * * 1` | mutation | scripts/mutation_court.sh --check | cargo-mutants 27.1.0 |
| release.yml | push main, dispatch | release | none | RELEASE_HOLD gates all steps |
| publish-npm.yml | tag v*, dispatch | validate, publish | none | OIDC trusted publish, env `npm` |

Env vars: CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS, BINDGEN_EXTRA_CLANG_ARGS, GRAPHLAW_WASM, GRAPHLAW_TARGET_DIR, GRAPHLAW_BROWSER_TESTS, GRAPHLAW_REQUIRE_ENGINE, CARGO_TARGET_DIR, RELEASE_HOLD, GH_TOKEN, VERSION/TARBALL.

## 2. Scripts and exit discipline
| script | purpose | exits |
|---|---|---|
| mutation_court.sh | cargo-mutants over registry/abi surface | 0 ok, 2 usage, 3 --check MISSED, 4 fail-closed |
| mutation_families.sh | fleet driver | 0, 2, 8 concurrent-court refusal |
| shape_mutation_court.sh | TC-MUT-01 on ontologies/sql-ast.ttl | 0 flipped, 3 vacuous, 4 restore failed, 5 not applied |
| spc_court.sh | SPC Cpk on receipt hot path (min 1.33) | 0/1; vacuous gate = 1 |
| partition_court.sh (+2 selfchecks) | code-partition census | 1 violation, 2 BLOCKED |
| receipt_chain_sync.sh | b3sum chain -> receipts/CHAIN.blake3 | 0, 1 tamper/missing, 2 usage |
| chain_flag_court.sh | pins receipt_chain_sync flag surface | 0 / 1 |
| receipt_replay_court.sh | replays sampled receipts | 0 / 1 |
| run_court.sh | capture law: logs to receipts/logs | passes command rc |
| exit_capture_check.sh | Exit-capture self-test | 0/1 |
| count_gate.sh | stale "N passed; 0 failed" in docs | 1 on stale |
| doc_staleness_gate.sh (+selfcheck) | prose staleness court | 0, 1, 2 |
| witness_gate.sh | docs witnesses resolve | 1, 2 |
| machine_shaped_gate.sh | forbids /Users/ literals | 1 on hit |
| wasm_audit.sh | size ceiling + header | 0, 1, 3 |
| wasm_unknown_unknown_smoke.sh | zero wasi imports | 0, 1, 3 loud skip |
| gen_w3c_manifest.sh | regenerates tests/w3c_manifest.rs | generator |
| gen_wasm4pm_graphlaw_vectors.py | vector generator (not read in detail) | n/a |
| fleet_cleanup.sh | removes stale target-* dirs | always 0 |

## 3. Caller graph
- Only edge workflow -> script: mutation.yml -> mutation_court.sh.
- No workflow caller: count_gate, witness_gate, doc_staleness_gate, machine_shaped_gate, partition_court, receipt_replay_court, wasm_audit, wasm_unknown_unknown_smoke, shape_mutation_court, fleet_cleanup. These gates are advisory in CI.

## 4. Reference integrity
- OK: docs/jira/v26.10.28/RECEIPTS.md (cited ci.yml, js.yml); commit 09bafd2 at RECEIPTS.md:592; docs/WASM.md.
- STALE: ci.yml:96 comment cites tests/authority_manifest.rs, which does not exist (nearest tests/chicago_cross_authority.rs). Comment only.

## 5. Open items
- Decide whether doc/witness/count gates join ci.yml.
- gen_wasm4pm_graphlaw_vectors.py ownership not examined.
