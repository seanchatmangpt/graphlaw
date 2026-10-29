# Battery Receipt — FULL battery, consolidated (L1–L4 on the shipped pin; L4 RED/INCOMPLETE; L5/L6 pending)

Lane: EXECUTION (battery receipts), lease `receipts/2026-10-24-battery-*.md` +
ledger appends. Date: 2026-10-24. Legs ran in fresh per-leg target dirs
(`target-battery-{sqlrdf,chicago,meta,finance}`), 11:15–11:35 local. This is
the G3 battery receipt for the shipped pin `58a44095…`: per-leg witnesses
landed as sibling receipts (`battery-dialect-census`, `battery-receipt-v2`,
`battery-trust-attest`, `battery-banking`); this file is the consolidated
classification of record.

## Header

- HEAD: a00770b4998a55ed4ee397e3e2a1f5a880c51db0
- Dirty disclosure: working tree carries in-flight wave edits — 267
  `git status --porcelain` entries at receipt-write time (80 M-or-D tracked + 187 untracked,
  including in-flight `receipts/CHAIN.blake3` and vendor edits). Moving-tree
  disclosure: HEAD advanced during this lane (`966cec3` at dispatch -> a
  re-read `a00770b` at receipt write, concurrent wave landing); the
  logs were captured 11:15-11:35 local on the pre-advance tree, the pins
  were re-measured at write time. Verified as-is; no file outside the lease
  touched; no git mutations.
- Pin b3sums (measured this session, `/opt/homebrew/bin/b3sum`):
  - `courts/code-partition/axioms.blake3` = `2add10ce9ff782e33b75de099cba8dc8a114ab759855f93a7be85abb18581506`
  - `priv/graphlaw.wasm` = `58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a`
    (matches pin file `priv/graphlaw.wasm.blake3` and
    `registry/ARTIFACTS.blake3:13` byte-for-byte, 58,124,612 B — the battery
    is on the shipped artifact identity; no new artifact minted)
- Witness-log digests (b3sum; logs are volatile — these digests preserve
  them):
  - `/tmp/bat-L1.log` b3sum = `7dec1d1f598a6f1861824f55927bec94e56a3f6785eb208559700bd0647c29b3` (732 lines)
  - `/tmp/bat-L2.log` b3sum = `5d5337166ad00196cb37ca555f58e116854707121e563dd503e10fb37c221029` (786 lines)
  - `/tmp/bat-L3.log` b3sum = `5f69751d4c360b8e9fa79948e2cea9017550019eaa6b12a5a4b8051f5383192d` (786 lines)
  - `/tmp/bat-L4.log` b3sum = `c2eedcb4d1f09d5c4a171bb5d02036ee9f4a7eee13c01ba6e6b5adc3bc025647` (699 lines)
- **Capture-masking disclosure (load-bearing for the whole battery)**: the
  battery driver's shell wrapper echoed success unconditionally, so every
  leg's observed shell exit code is 0 regardless of cargo's outcome. The
  captured logs at the digests above are the evidence of record; every
  verdict below rests on the logs' verbatim `test result:` lines, not on any
  exit code.
- Splice provenance: every `Running tests/…` header and `test result:` line
  below is spliced mechanically from the digested logs; nothing in the fenced
  blocks is hand-typed.

## Verdict

**RED/INCOMPLETE as a whole battery; greens classified per court below.**

- L1 sqlrdf: 8 targets, 102 passed / 0 failed — PASS.
- L2 chicago: 5 targets, 154 passed / 0 failed — PASS.
- L3 meta: 15 targets, 114 passed / 0 failed — PASS.
- L4 finance: 6 of 10 planned targets executed — 28 passed / 1 failed
  (`op_differential`), 4 compiled courts unexecuted (fail-fast cut) —
  **RED/INCOMPLETE**.
- L5/L6 legs: NOT RUN (pending; their receipts flip the remaining legs).

## Leg aggregates (mechanically recomputed from the spliced `test result:` lines)

### L1 sqlrdf (8 targets, 102/0)

| Court (target) | Tests | Result | Classification |
|---|---|---|---|
| canonical_cross_dialect | 6 | 6/0 ok | PASS |
| duckdb_statement_kinds | 8 | 8/0 ok | PASS |
| k5_hostile_sqlrdf | 8 | 8/0 ok | PASS |
| refusal_crosswalk_differential | 6 | 6/0 ok | PASS |
| refusal_details | 7 | 7/0 ok | PASS |
| sql_rdf_conformance | 8 | 8/0 ok | PASS |
| sql_rdf_dialect | 36 | 36/0 ok | PASS |
| sql_rdf_laundering_corpus | 23 | 23/0 ok | PASS |
| **L1 total** | **102** | **102 passed; 0 failed** | **8 PASS / 0 CLASSIFIED / 0 SKIP** |

### L2 chicago (5 targets, 154/0)

| Court (target) | Tests | Result | Classification |
|---|---|---|---|
| chicago_boundary_constraints | 4 | 4/0 ok | PASS |
| chicago_cross_authority | 71 | 71/0 ok | PASS |
| chicago_fibo | 61 | 61/0 ok | PASS |
| chicago_n3_datalog | 6 | 6/0 ok | PASS |
| chicago_validation | 12 | 12/0 ok | PASS |
| **L2 total** | **154** | **154 passed; 0 failed** | **5 PASS / 0 CLASSIFIED / 0 SKIP** |

### L3 meta (15 targets, 114/0)

| Court (target) | Tests | Result | Classification |
|---|---|---|---|
| authority_manifest | 6 | 6/0 ok | PASS |
| backend_authorities | 8 | 8/0 ok | PASS |
| capabilities_digest | 6 | 6/0 ok | PASS |
| receipt_chain_golden_test | 5 | 5/0 ok | PASS |
| receipt_chain_validator_test | 4 | 4/0 ok | PASS |
| receipt_epoch_migration_port_test | 4 | 4/0 ok | PASS |
| receipt_feedback | 7 | 7/0 ok | PASS |
| receipt_lease | 6 | 6/0 ok | PASS |
| receipt_store | 4 | 4/0 ok | PASS |
| receipt_v2_court | 6 | 6/0 ok | PASS |
| receipt_v2_emission_court | 8 | 8/0 ok | PASS |
| registry_artifacts | 22 | 22/0 ok | PASS |
| registry_dispatch | 14 | 14/0 ok | PASS |
| registry_limits | 2 | 2/0 ok | PASS |
| trust_attest | 12 | 12/0 ok | PASS |
| **L3 total** | **114** | **114 passed; 0 failed** | **15 PASS / 0 CLASSIFIED / 0 SKIP** |

### L4 finance (6 executed of 10 planned)

| Court (target) | Tests | Result | Classification |
|---|---|---|---|
| amortization_court | 1 | 1/0 ok | PASS |
| bcbs_aggregation_court | 3 | 3/0 ok | PASS |
| dvp_settlement_court | 4 | 4/0 ok | PASS |
| gws12_coldchain_cross_engine | 2 | 2/0 ok | PASS |
| gws4_ipc_injection_court | 3 | 3/0 ok | PASS |
| op_differential | 15 | 15/1 FAILED | CLASSIFIED-RED (G8-class sniff differential) |
| op_examples | - | not executed | SKIP-UNEXECUTED (fail-fast cut) |
| op_forward_compat | - | not executed | SKIP-UNEXECUTED (fail-fast cut) |
| rehypothecation_gate_court | - | not executed | SKIP-UNEXECUTED (fail-fast cut) |
| ubo_sanctions_court | - | not executed | SKIP-UNEXECUTED (fail-fast cut) |
| **L4 total** | **28** | **28 passed; 1 failed** | **5 PASS / 1 CLASSIFIED-RED / 4 SKIP-UNEXECUTED** |

## The honest census

The sql-rdf statement-kind census (`tests/sql_rdf_dialect.rs`, L1) is
**36 passed / 0 failed**. **69/0 is not witnessed anywhere in any of the four
logs**; any ledger row carrying "69/0" for this court is false-at-write-time
and must be re-derived from this receipt. Verbatim (from
`/tmp/bat-L1.log`):

```
     Running tests/sql_rdf_dialect.rs (target-battery-sqlrdf/debug/deps/sql_rdf_dialect-71209326aa00c39e)

running 36 tests
test duckdb_courts::aggregate_in_group_by_is_refusal_03 ... ok
test duckdb_courts::property_stuffing_node_is_refusal_01 ... ok
test duckdb_courts::aggregate_hidden_under_alias_indirection_in_where_is_refusal_03 ... ok
test d5_operator_dual_carry_string_and_iri_is_refusal_01 ... ok
test disallowed_operator_token_is_refusal_01 ... ok
test duckdb_courts::nested_aggregate_is_refusal_03 ... ok
test duckdb_courts::admitted_ast_executes_on_frozen_duckdb ... ok
test duckdb_courts::group_having_order_by_select_executes_with_table_readback ... ok
test duckdb_courts::aggregate_in_where_is_refusal_03 ... ok
test duckdb_join_condition_courts::join_condition_aggregate_is_refusal_03 ... ok
test l163_msg_aggregate_in_where_is_verbatim ... ok
test l163_msg_no_canonical_graph_is_verbatim ... ok
test l163_msg_literal_not_float_is_verbatim ... ok
test l163_msg_connect_statement_is_verbatim ... ok
test duckdb_courts::section7_merge_compiles_to_parameterized_sql_with_params ... ok
test l163_msg_admission_seal_mismatch_is_verbatim ... ok
test d4_direction_iri_object_is_refusal_01_not_silent_asc ... ok
test l163_msg_missing_statement_index_is_verbatim ... ok
test sniff_routes_sql_select_and_leaves_plain_turtle ... ok
test sniff_sql_term_without_select_is_ambiguous ... ok
test l163_msg_literal_not_int_is_verbatim_with_truncated_ident ... ok
test section4_string_operator_tokens_compile ... ok
test l163_msg_join_type_iri_is_verbatim ... ok
test duckdb_courts::process_scope_column_in_select_list_is_refusal_03 ... ok
test duckdb_courts::connect_disconnect_carry_refusal_02 ... ok
test shape_violation_refuses_before_duckdb ... ok
test missing_statement_index_is_refusal_01_before_duckdb ... ok
test duckdb_courts::schema_qualified_from_list_compiles_and_executes ... ok
test query_hash_determinism_court::parameter_change_moves_the_digest ... ok
test query_hash_determinism_court::semantic_literal_change_moves_the_digest ... ok
test d1_empty_select_list_is_refusal_01_not_select_star ... ok
test refusal05_where_less_update_is_verbatim_and_pre_engine ... ok
test sql_rdf_joins_the_rdf_dataset_path ... ok
test refusal05_where_less_delete_is_verbatim_and_pre_engine ... ok
test query_hash_determinism_court::receipt_equals_recomputed_canon_plus_params_hash ... ok
test query_hash_determinism_court::five_cosmetic_variants_hash_identically_through_public_path ... ok

test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.27s
```

Artifact-instantiating oks inside the census (validity precondition):
`duckdb_courts::admitted_ast_executes_on_frozen_duckdb`,
`duckdb_courts::group_having_order_by_select_executes_with_table_readback`.

## L4 RED: op_differential 15/1 (G8-class sniff divergence, fresh witness)

```
     Running tests/op_differential.rs (target-battery-finance/debug/deps/op_differential-d6460b00ca85c495)

running 16 tests
test a_control_parse_and_canonical_succeed_on_the_base_fixture has been running for over 60 seconds
test b_control_a_changed_graph_changes_the_id has been running for over 60 seconds
test canonical_nquads_is_identical_whatever_dialect_the_data_arrived_in has been running for over 60 seconds
test control_closure_fixture_is_non_trivial has been running for over 60 seconds
test datalog_n3_and_owl_rl_agree_with_warshall_on_seeded_graphs has been running for over 60 seconds
test entail_closure_is_a_fixpoint has been running for over 60 seconds
test entail_rdfs_added_equals_the_quad_delta_against_canonical has been running for over 60 seconds
test hooks_rerun_on_its_own_output_fires_nothing_and_keeps_the_id has been running for over 60 seconds
test law_shacl_only_first_state_is_the_parse_id has been running for over 60 seconds
test law_shacl_refusal_is_typed_and_identical_on_both_runtimes has been running for over 60 seconds
test parse_canonical_convert_ids_agree_across_all_nine_rdf_dialects has been running for over 60 seconds
test sniff_dialect_agrees_with_parse_dialect_for_every_serialization has been running for over 60 seconds
test sniff_hint_agrees_with_parse_hint has been running for over 60 seconds
test sniff_success_implies_parse_success_for_converted_turtle has been running for over 60 seconds
test sparql_construct_quads_equal_canonical_quads_of_the_same_data has been running for over 60 seconds
test sparql_select_row_count_matches_canonical_quads has been running for over 60 seconds
test law_shacl_refusal_is_typed_and_identical_on_both_runtimes ... ok
test control_closure_fixture_is_non_trivial ... ok
test b_control_a_changed_graph_changes_the_id ... ok
test sniff_hint_agrees_with_parse_hint ... ok
test sparql_construct_quads_equal_canonical_quads_of_the_same_data ... ok
test a_control_parse_and_canonical_succeed_on_the_base_fixture ... ok
test sniff_success_implies_parse_success_for_converted_turtle ... ok
test sparql_select_row_count_matches_canonical_quads ... ok
test law_shacl_only_first_state_is_the_parse_id ... ok
test sniff_dialect_agrees_with_parse_dialect_for_every_serialization ... FAILED
test canonical_nquads_is_identical_whatever_dialect_the_data_arrived_in ... ok
test parse_canonical_convert_ids_agree_across_all_nine_rdf_dialects ... ok
test entail_closure_is_a_fixpoint ... ok
test datalog_n3_and_owl_rl_agree_with_warshall_on_seeded_graphs ... ok
test entail_rdfs_added_equals_the_quad_delta_against_canonical ... ok
test hooks_rerun_on_its_own_output_fires_nothing_and_keeps_the_id ... ok

failures:

---- sniff_dialect_agrees_with_parse_dialect_for_every_serialization stdout ----

thread 'sniff_dialect_agrees_with_parse_dialect_for_every_serialization' (10055750) panicked at tests/op_differential.rs:181:9:
assertion `left == right` failed: sniff native == wasm for ntriples
  left: Object {"dialect": String("NTriples"), "engine": String("PurRdf"), "ok": Bool(true)}
 right: Object {"dialect": String("Turtle"), "engine": String("PurRdf"), "ok": Bool(true)}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    sniff_dialect_agrees_with_parse_dialect_for_every_serialization

test result: FAILED. 15 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 72.91s
```

error: test failed, to rerun pass `--test op_differential`

Reading of the failure (the assertion text is court output above):
`sniff_dialect_agrees_with_parse_dialect_for_every_serialization` panicked at
`tests/op_differential.rs:181:9` — native runtime sniffs the same bytes as
`{"dialect": "NTriples", "engine": "PurRdf", "ok": true}` while the wasm
runtime sniffs them as `{"dialect": "Turtle", …}`: a native-vs-wasm sniff
differential on one serialization, on the shipped pin. This is the G8-class
sniff root-cause family documented in
`docs/jira/v26.10.23/design-sniff-fixes.md` (sniff-layer fixes; the file's
current behavior section describes the sibling literal/IRI defects) — this
panic is a FRESH WITNESS of the differential class for that design doc: the
sniffer is not runtime-deterministic on identical input today.

Classification: **CLASSIFIED-RED** — a real differential, typed assertion,
no crash-loop, no silent pass. Left red per lane rules; not fixed by this
lane. Fix path: `docs/jira/v26.10.23/design-sniff-fixes.md` (sniff-layer),
then the rerun falsifier (a) below.

## The fail-fast cut: 4 named unexecuted courts

The L4 leg compiled exactly **10 test targets** (build-dir evidence:
`target-battery-finance/debug/deps/`, test binaries all mtime 11:20;
`Finished … in 3m 32s` before the first `Running`). Cargo executed them in
alphabetical order and stopped at the first failure: `op_differential` is the
alphabetical cut. **Compiled in-leg and never executed (4, named):**

1. `tests/op_examples.rs`
2. `tests/op_forward_compat.rs`
3. `tests/rehypothecation_gate_court.rs` (also witnessed by the leg's own
   compile-warning line, `/tmp/bat-L4.log:592`)
4. `tests/ubo_sanctions_court.rs`

Count reconciliation: the dispatch recorded "5 fail-fast-unexecuted courts";
the mechanical count from the log + build dir is **4 unexecuted**, and
op_differential itself RED makes **5 courts without a green verdict** in L4.
Both numbers are recorded so the ledger does not silently drift between them.

These four are typed SKIP-UNEXECUTED (fail-fast, cargo's own typed
`error: test failed, to rerun pass `--test op_differential``), not passes and
not failures. Their discharge is a rerun receipt.

## frame_bound currency note

The v5 closeout's G1 row cites `wasm_frame_bound_court` **8/0**. The court on
disk today (`tests/wasm_frame_bound_court.rs`) carries **4** `#[test]`
functions (`eh_zero_len_frame_clean_exit0`, `eh_at_cap_frame_admitted_and_answered`,
`eh_oversize_0xffffffff_exit92_before_body`, `eh_over_cap_by_one_exit92`) —
the 8/0 figure is historical-at-write-time (pre wasmi-retirement split,
commit `1e8ec32`), not witnessed by this battery. Cite 4/0-current basis for
the frame-bound court until a fresh receipt witnesses it on this HEAD.

## Honesty line: machine-courted vs prose-declared

- 398 tests executed across the four legs (102 + 154 + 114 + 28), 397
  passing, 1 failing, 4 courts unexecuted, 2 legs not run. All verdicts rest
  on log `test result:` lines at the digests above.
- Machine-courted here: the census (36/0), the refusal/crosswalk surface, the
  receipt-v2 fold family, the Ed25519 attestation surface, the banking
  settlement/fold/injection flows, and the sniff differential FAILURE itself.
- Prose-declared residues: the L5/L6 legs (not run), the root-cause locus of
  the sniff differential (design hypothesis in design-sniff-fixes.md, not
  proven by this failure), and the frame-bound historical figures.

## §C Receipt fields (X13 shape)

- **Identity**: consolidated battery receipt over `/Users/sac/graphlaw` HEAD
  a00770b4998a55ed4ee397e3e2a1f5a880c51db0 (dirty tree disclosed), logs captured 11:15–11:35 local
  2026-10-24, four legs on fresh per-leg target dirs against the shipped
  artifact identity `58a44095…` (58,124,612 B).
- **Authority**: write lease = `receipts/2026-10-24-battery-*.md` (5 files) +
  append-only ledger sections (`docs/jira/v26.10.23/GO-NO-GO-v26.10.22.md`
  after :318; `docs/jira/v26.10.22/RECEIPTS.md` appended section;
  `RECEIPTS-INDEX.md` appended rows). No git mutations; no chain sync by
  this lane (coordinator runs it at integration).
- **Consequence**: 5 durable receipts + 3 ledger appends; zero code edits;
  zero status flips executed by this lane (the G3 SPLIT below is a
  classification performed here for the coordinator to land — the ledger
  append carries it as recorded text).
- **Replay** (re-read at use time): `b3sum /tmp/bat-L1.log /tmp/bat-L2.log
  /tmp/bat-L3.log /tmp/bat-L4.log` must reproduce the digests above;
  `grep "test result:" /tmp/bat-L<leg>.log` must reproduce the per-court
  lines; per-court reruns:
  `CARGO_TARGET_DIR=$(mktemp -d) cargo test --locked --test sql_rdf_dialect`
  (expect 36/0), `… --test dvp_settlement_court` (expect 4/0),
  `… --test op_differential` (expect 15/1 until the sniff fix lands),
  `… --test receipt_v2_court --test receipt_v2_emission_court` (expect 6/0 +
  8/0), `… --test trust_attest` (expect 12/0).
- **Falsifiers** (X13 (a)–(d)):
  - (a) A green rerun of `cargo test --locked --test op_differential`
    (sniff-parity: 16 passed; 0 failed) on this HEAD + pin flips the
    op_differential row CLASSIFIED-RED → DISCHARGED; a rerun that fails on a
    DIFFERENT test extends the classification instead.
  - (b) L5/L6 leg receipts landing with embedded `test result:` lines flip
    the remaining legs from NOT-RUN.
  - (c) Any court in the spliced tables returning a non-`ok` line on a rerun
    of this HEAD + pin `58a44095…` falsifies that row's PASS.
  - (d) A sniff fix landing WITHOUT flipping (a) green is not a fix (same
    rule as SQLRDF-27); the design doc's fix list must shrink only via its
    own court (`tests/sniff_court.rs`) + this differential going green.
- **Standing**: battery RED/INCOMPLETE as of 2026-10-24 on HEAD `a00770b`;
  G3 battery obligation: the green legs (L1–L3 + dvp gating leg) are
  witnessed; the op_differential differential + 4 unexecuted courts + L5/L6
  remain open against posture P. G2 stays REMAINING with its config
  precondition as dispatched: `abi_response_builders_court` is missing from
  the kill suite (E07 lands the fix) and was not part of any leg of this
  battery — G2's solo-window sweep must run on a tree where the E07 fix is
  committed. This lane's live read found the entry present at
  `.cargo/mutants.toml:28` in the moving working tree; recorded as an
  observation, not a flip — the gate moves only on E07's own receipt.

Closing b3sum of this receipt is reported in the lane's final report (a file
cannot embed its own digest); verify with
`b3sum /Users/sac/graphlaw/receipts/2026-10-24-battery-full.md`.
