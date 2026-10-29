# Battery Receipt — L1 sqlrdf leg (dialect census + refusal surface + frozen-duckdb courts)

Lane: EXECUTION (battery receipts), lease `receipts/2026-10-24-battery-*.md` +
ledger appends. Date: 2026-10-24. Leg target dir:
`/Users/sac/graphlaw/target-battery-sqlrdf` (build dir preserved at receipt
write time; the leg's test-target binaries carry mtimes 11:20). All runs:
`CARGO_TARGET_DIR=…/target-battery-sqlrdf cargo test --locked --test <target> …`
(fresh lane build; `Finished \`test\` profile [unoptimized + debuginfo]
target(s) in 3m 10s`, verbatim at `/tmp/bat-L1.log:582`).

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
    `registry/ARTIFACTS.blake3:13` byte-for-byte, 58,124,612 B)
- Witness-log digests (b3sum; logs are volatile — these digests preserve
  them; the battery driver's shell wrapper echoed success unconditionally,
  so each leg's observed shell exit code is 0 regardless of cargo's outcome —
  the captured logs are the evidence of record, not the leg exit codes):
  - `/tmp/bat-L1.log` b3sum = `7dec1d1f598a6f1861824f55927bec94e56a3f6785eb208559700bd0647c29b3` (732 lines)
  - `/tmp/bat-L2.log` b3sum = `5d5337166ad00196cb37ca555f58e116854707121e563dd503e10fb37c221029` (786 lines)
  - `/tmp/bat-L3.log` b3sum = `5f69751d4c360b8e9fa79948e2cea9017550019eaa6b12a5a4b8051f5383192d` (786 lines)
  - `/tmp/bat-L4.log` b3sum = `c2eedcb4d1f09d5c4a171bb5d02036ee9f4a7eee13c01ba6e6b5adc3bc025647` (699 lines)
- Splice provenance: every `Running tests/…` header and `test result:` line
  below is spliced mechanically from the digested logs at their digested
  state; nothing in the fenced blocks is hand-typed.

## Aggregate

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

## Per-court witnesses (mechanically spliced from `/tmp/bat-L1.log`)

### 1. sql_rdf_dialect — the census (36/0, the honest census)

The sql-rdf statement-kind census is **36 passed / 0 failed**. **69/0 is not
witnessed anywhere in this log**; any ledger row carrying "69/0" for this
court is false-at-write-time and must be re-derived from this receipt.

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

Artifact-instantiating oks inside this census (validity precondition — real
frozen-duckdb, real readbacks, not stubs):
`duckdb_courts::admitted_ast_executes_on_frozen_duckdb`,
`duckdb_courts::group_having_order_by_select_executes_with_table_readback`,
`duckdb_courts::schema_qualified_from_list_compiles_and_executes`.
Non-vacuity inside this census:
`query_hash_determinism_court::five_cosmetic_variants_hash_identically_through_public_path`
+ `parameter_change_moves_the_digest` (sensitivity proven both ways).
Typed-refusal arms: `d1_empty_select_list_is_refusal_01_not_select_star`,
`shape_violation_refuses_before_duckdb`, the `l163_msg_*` verbatim-message
family, `refusal05_where_less_{update,delete}_is_verbatim_and_pre_engine`.

### 2. canonical_cross_dialect (RDFC-1.0 + BLAKE3, 6/0)

```
     Running tests/canonical_cross_dialect.rs (target-battery-sqlrdf/debug/deps/canonical_cross_dialect-d077846a58e5a250)

running 6 tests
test n3_is_refused_unsupported_for_canonical ... ok
test nonvacuity_one_quad_perturbation_moves_id ... ok
test dataset_court_named_graph_differs_from_default_graph ... ok
test dataset_court_named_graph_id_matches_across_trig_and_nquads ... ok
test graph_court_pure_graph_id_identical_across_dialects ... ok
test blank_node_court_relabeling_and_cross_dialect_id ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Includes `nonvacuity_one_quad_perturbation_moves_id` (in-court non-vacuity)
and `n3_is_refused_unsupported_for_canonical` (typed UNSUPPORTED, not a
silent pass).

### 3. duckdb_statement_kinds (statement kinds over real frozen-duckdb, 8/0)

```
     Running tests/duckdb_statement_kinds.rs (target-battery-sqlrdf/debug/deps/duckdb_statement_kinds-ac8c8a60c033f682)

running 8 tests
test generated_kinds_match_the_committed_grammar ... ok
test copy_statement_writes_a_real_file_matching_the_table ... ok
test expression_courts_flip_under_operator_inversion ... ok
test every_statement_kind_runs ... ok
test every_source_extension_answers_or_is_absent ... ok
test every_command_class_compiles_from_expressions_and_runs_on_its_own_connection ... ok
test attach_statement_binds_a_real_file_database_readable_through_the_alias ... ok
test section6_2_certified_checks_run_through_compiled_ast_graphs ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.26s
```

Real frozen-duckdb execution courts over the statement-kind surface —
artifact-instantiating by construction (open in-memory engine, execute,
read back).

### 4. k5_hostile_sqlrdf (8/0)

```
     Running tests/k5_hostile_sqlrdf.rs (target-battery-sqlrdf/debug/deps/k5_hostile_sqlrdf-d40ca70096eb3143)

running 8 tests
test b1_hostile_table_name_is_typed_refusal ... ok
test e2_aggregate_wrapped_on_is_typed_refusal ... ok
test b2_hostile_column_name_is_typed_refusal ... ok
test d_where_admits_everything_is_lawful_select_semantics ... ok
test a_literal_drop_table_smuggles_as_data_only ... ok
test e1_schema_suffix_collision_is_typed_refusal ... ok
test c_merge_tautological_on_is_typed_refusal ... ok
test e3_unqualified_on_column_is_typed_refusal ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
```

Hostile table/column smuggling stays typed-refused;
`a_literal_drop_table_smuggles_as_data_only` (data-only smuggling admitted
as data, never executed) and `d_where_admits_everything_is_lawful_select_semantics`
(positive control).

### 5. refusal_crosswalk_differential (6/0)

```
     Running tests/refusal_crosswalk_differential.rs (target-battery-sqlrdf/debug/deps/refusal_crosswalk_differential-d6a3407ca9a31d62)

running 6 tests
test refusal_crosswalk_pin_matches_live_sibling ... ok
test sabotage_twin_mutated_verdict_fails_court ... ok
test erlmcp_code_totality_fail_closed ... ok
test valve_class_differential_agreement ... ok
test crosswalk_projection_query_agrees ... ok
test refusal_crosswalk_pack_gates_clean_and_engines_agree ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.53s
```

Includes the sabotage twin `sabotage_twin_mutated_verdict_fails_court`
(a mutated verdict FAILS the court — the crosswalk is load-bearing) and
`erlmcp_code_totality_fail_closed`.

### 6. refusal_details (7/0)

```
     Running tests/refusal_details.rs (target-battery-sqlrdf/debug/deps/refusal_details-431ce9f216939765)

running 7 tests
test lease_refusal_carries_reason_lease_id_step ... ok
test receipt_required_carries_step ... ok
test policy_refusal_carries_kind_state_action ... ok
test plan_refusal_carries_index_action_and_unmet ... ok
test every_law_error_variant_has_a_code ... ok
test rust_variant_keeps_count_and_results_in_agreement ... ok
test shacl_refusal_carries_one_structured_entry_per_violation ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

`every_law_error_variant_has_a_code` — the typed-refusal register is total
over the error enum.

### 7. sql_rdf_conformance (8/0)

```
     Running tests/sql_rdf_conformance.rs (target-battery-sqlrdf/debug/deps/sql_rdf_conformance-14e6eda98690fab5)

running 8 tests
test report_emission_is_additive_and_clause_inventory_is_exhaustive ... ok
test section6_2_certified_extension_values_match_generated_table ... ok
test register_completeness_every_refusal_arm_is_enumerated ... ok
test section7_1_compiles_to_target_parameterized_projection ... ok
test section5_refusal_codes_in_register_and_in_src ... ok
test section3_2_inventory_is_35_classes_with_typed_engine_refusals ... ok
test section7_merge_admit_refuse_flip_on_real_duckdb ... ok
test section4_shape_names_present_and_gates_refuse ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
```

`section5_refusal_codes_in_register_and_in_src` — register exhaustiveness
courted in both directions (`docs/refusals.md` codes ↔ src arms);
`register_completeness_every_refusal_arm_is_enumerated`.

### 8. sql_rdf_laundering_corpus (23/0)

```
     Running tests/sql_rdf_laundering_corpus.rs (target-battery-sqlrdf/debug/deps/sql_rdf_laundering_corpus-60d56dd4bbfaafd2)

running 23 tests
test benign_skeleton_compiles ... ok
test aggregate_unscoped_in_where_still_refuses_03 ... ok
test aggregate_process_scope_in_having_compiles ... ok
test d3_order_by_aggregate_expression_wrapper_compiles ... ok
test d3_expression_wrapper_plus_discriminator_refuses ... ok
test e4_malformed_offset ... ok
test e3_duplicate_where_clause ... ok
test dual_scope_on_column_refuses ... ok
test e5_left_leftoperand_dual_key ... ok
test aggregate_scope_process_in_where_refuses_03 ... ok
test fromless_select_refuses_via_shacl_shape_channel ... ok
test scope_on_operator_node_refuses ... ok
test e2_hostile_from_under_benign_fromtable ... ok
test e3_unknown_where_predicate_refuses_typed ... ok
test e4_malformed_limit ... ok
test e1_mixed_projection_select_item ... ok
test view_scope_column_in_where_refuses_03 ... ok
test process_scope_column_in_select_refuses_03 ... ok
test d1_scope_on_from_table_refuses ... ok
test d1_scope_on_join_table_refuses ... ok
test tcb_aggregate_node_process_scope_in_where_refuses_03 ... ok
test tcb_process_scope_argument_in_legal_having_aggregate_refuses_03 ... ok
test deep_nested_expression_refuses_recursion_limit ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
```

Aggregate-laundering corpus: benign skeletons compile; aggregate-in-WHERE
variants still refuse 03.

## Honesty line: machine-courted vs prose-declared

- The census claim of record for this session is the log line above: **36/0**.
- Every refusal code exercised here is courted against `docs/refusals.md` in
  both directions (§7 above); no generated doc states a census percentage.
- The 102/0 aggregate is computed from the spliced `test result:` lines of
  this leg only; it does not cover L2–L4 (see sibling battery receipts).

## Falsifiers

1. Any court above returning a non-`ok` `test result:` line on this HEAD with
   pin `58a44095…` falsifies the green aggregate.
2. A rerun of `cargo test --locked --test sql_rdf_dialect` that prints
   anything other than `36 passed; 0 failed` falsifies the census row —
   including a 69/0 print, which would mean the court grew without a receipt.
3. Breaking any typed-refusal arm (e.g. accepting an empty select list) must
   flip `d1_empty_select_list_is_refusal_01_not_select_star` red; if it still
   passes, the census is vacuous.
4. Deleting a code from `docs/refusals.md` must flip
   `section5_refusal_codes_in_register_and_in_src` red (register is
   load-bearing in both directions).

Closing b3sum of this receipt is reported in the lane's final report (a file
cannot embed its own digest); verify with
`b3sum /Users/sac/graphlaw/receipts/2026-10-24-battery-dialect-census.md`.
