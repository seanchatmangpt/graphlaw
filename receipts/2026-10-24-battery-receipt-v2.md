# Battery Receipt — receipt-v2 fold family (L3 meta leg: v1-fold pin + v2 emission + chain/store/lease/feedback)

Lane: EXECUTION (battery receipts), lease `receipts/2026-10-24-battery-*.md` +
ledger appends. Date: 2026-10-24. Leg target dir:
`/Users/sac/graphlaw/target-battery-meta` (build dir preserved at receipt
write time). All runs:
`CARGO_TARGET_DIR=…/target-battery-meta cargo test --locked --test <target> …`
(fresh lane build; `Finished \`test\` profile [unoptimized + debuginfo]
target(s) in 3m 18s`, verbatim at `/tmp/bat-L3.log:582`).

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
| receipt_chain_golden_test | 5 | 5/0 ok | PASS |
| receipt_chain_validator_test | 4 | 4/0 ok | PASS |
| receipt_epoch_migration_port_test | 4 | 4/0 ok | PASS |
| receipt_feedback | 7 | 7/0 ok | PASS |
| receipt_lease | 6 | 6/0 ok | PASS |
| receipt_store | 4 | 4/0 ok | PASS |
| receipt_v2_court | 6 | 6/0 ok | PASS |
| receipt_v2_emission_court | 8 | 8/0 ok | PASS |
| **Family total** | **44** | **44 passed; 0 failed** | **8 PASS / 0 CLASSIFIED / 0 SKIP** |

L3 as a whole leg carries 15 targets / 114 passed / 0 failed; the remaining
authority/registry courts (authority_manifest 6, backend_authorities 8,
capabilities_digest 6, registry_artifacts 22, registry_dispatch 14,
registry_limits 2) are witnessed in
`receipts/2026-10-24-battery-full.md`.

## Per-court witnesses (mechanically spliced from `/tmp/bat-L3.log`)

### 1. receipt_v2_court (6/0) — the v1 fold is pinned; v2 is pinned and differs

```
     Running tests/receipt_v2_court.rs (target-battery-meta/debug/deps/receipt_v2_court-34a3222c9ce10ad5)

running 6 tests
test v1_fold_is_pinned_and_stable ... ok
test v1_fold_matches_settled_b3sum_pin ... ok
test v2_differs_from_v1_on_pinned_inputs_and_is_pinned ... ok
test canon_arrow_empty_v1_matches_independent_blake3 ... ok
test v2_prefix_is_exact_adr_literal ... ok
test v2_differs_from_v1_under_permutations ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`v1_fold_matches_settled_b3sum_pin` — the v1 fold digest matches its settled
b3sum pin; `v2_prefix_is_exact_adr_literal` — the v2 prefix is the ADR
literal; `v2_differs_from_v1_on_pinned_inputs_and_is_pinned` +
`v2_differs_from_v1_under_permutations` — v2 is distinguishable from v1 and
permutation-sensitive on pinned inputs; `canon_arrow_empty_v1_matches_independent_blake3`
— the empty-arm fold matches an independent BLAKE3 computation
(artifact-instantiating, validity precondition).

### 2. receipt_v2_emission_court (8/0) — v2 rides the wire and self-verifies

```
     Running tests/receipt_v2_emission_court.rs (target-battery-meta/debug/deps/receipt_v2_emission_court-1987b1956e57d07b)

running 8 tests
test h_triples_empty_arm_is_pinned ... ok
test sentinel_digest_is_pinned_and_domain_separated ... ok
test v1_wire_shape_is_byte_preserved ... ok
test phase3_keys_ride_the_wire_only_when_present ... ok
test sentinel_arms_on_non_tabular_results ... ok
test v2_present_on_tabular_results_and_self_verifies ... ok
test divergent_graph_override_reseals_v2 ... ok
test v2_is_deterministic_across_independent_engines ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

`v2_present_on_tabular_results_and_self_verifies` — emission is real and
self-verifying (artifact-instantiating); `v2_is_deterministic_across_independent_engines`
— cross-engine determinism; `divergent_graph_override_reseals_v2` — reseal
on graph divergence; `v1_wire_shape_is_byte_preserved` — v1 compatibility
byte-for-byte; `sentinel_digest_is_pinned_and_domain_separated`.

### 3. receipt_chain_golden_test (5/0) — the serialized chain recomputes

```
     Running tests/receipt_chain_golden_test.rs (target-battery-meta/debug/deps/receipt_chain_golden_test-f30cade3aac01958)

running 5 tests
test receipt_record_version_is_frozen ... ok
test fixture_pin_detects_one_sided_edits ... ok
test v1_record_folds_to_the_base_hash ... ok
test recompute_chain_hash_survives_a_real_json_round_trip ... ok
test committed_tcps_head_recomputes_to_its_stored_chain_hash ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

`committed_tcps_head_recomputes_to_its_stored_chain_hash` — the committed
chain head recomputes from the real store (artifact-instantiating);
`fixture_pin_detects_one_sided_edits` — one-sided edit detection
(non-vacuity).

### 4. receipt_chain_validator_test (4/0) — tamper detection is live

```
     Running tests/receipt_chain_validator_test.rs (target-battery-meta/debug/deps/receipt_chain_validator_test-ffddcb579fb1eb5e)

running 4 tests
test empty_store_load_is_ok_empty ... ok
test version_2_record_refused_with_graphlaw_error_text ... ok
test tampered_middle_record_fails_verification ... ok
test round_trip_write_load_verify_chain ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`tampered_middle_record_fails_verification` — a tampered middle record fails
(non-vacuity); `version_2_record_refused_with_graphlaw_error_text` — typed
refusal carries graphlaw error text.

### 5. receipt_epoch_migration_port_test (4/0) — v1 reader port

```
     Running tests/receipt_epoch_migration_port_test.rs (target-battery-meta/debug/deps/receipt_epoch_migration_port_test-df4769e1809f416c)

running 4 tests
test migration_receipt_new_pins_legacy_observed_ceiling_and_round_trips ... ok
test v1_receipt_verifies_identically_through_both_readers ... ok
test v1_legacy_reader_refuses_v2_wire_shape ... ok
test v1_legacy_round_trip_preserves_all_14_fields ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`v1_receipt_verifies_identically_through_both_readers`;
`v1_legacy_reader_refuses_v2_wire_shape` (typed);
`v1_legacy_round_trip_preserves_all_14_fields`.

### 6. receipt_feedback (7/0) — receipts gate re-admission

```
     Running tests/receipt_feedback.rs (target-battery-meta/debug/deps/receipt_feedback-f38234fbb37a117b)

running 7 tests
test plan_digest_is_stable_and_sensitive_to_actions_and_goal ... ok
test abi::abi_plan_receipts_carry_digest_and_index ... ok
test abi::capabilities_report_abi_version ... ok
test second_admission_refused_without_receipt_admitted_with_it ... ok
test receipt_for_a_different_step_does_not_satisfy_the_gate ... ok
test recorded_receipt_content_changes_the_state_id ... ok
test abi::abi_record_then_require_admits_and_bare_require_refuses ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

`second_admission_refused_without_receipt_admitted_with_it` — the receipt
gate flips admission both ways (non-vacuity);
`receipt_for_a_different_step_does_not_satisfy_the_gate` (subject-binding).

### 7. receipt_lease (6/0) — lease ceilings bind

```
     Running tests/receipt_lease.rs (target-battery-meta/debug/deps/receipt_lease-29ae07eaa4dc414b)

running 6 tests
test refused_lease_yields_no_state ... ok
test expired_lease_refused ... ok
test out_of_scope_step_refused ... ok
test ceiling_below_required_refused ... ok
test unleased_transition_has_no_lease_id ... ok
test valid_lease_admits_and_receipt_carries_lease_id ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

Expired/out-of-scope/below-ceiling leases refuse;
`refused_lease_yields_no_state`.

### 8. receipt_store (4/0) — the store survives reopen

```
     Running tests/receipt_store.rs (target-battery-meta/debug/deps/receipt_store-1d7451d0d74234f3)

running 4 tests
test missing_receipt_breaks_the_chain ... ok
test one_tampered_byte_is_refused ... ok
test put_then_reopen_recovers_the_same_chain ... ok
test subject_keys_cannot_escape_the_store ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

`put_then_reopen_recovers_the_same_chain` (artifact-instantiating);
`one_tampered_byte_is_refused` (non-vacuity);
`subject_keys_cannot_escape_the_store`.

## Honesty line: machine-courted vs prose-declared

- All 44 family tests are machine-courted against real stores, real folds,
  and pinned digests; zero prose-only claims in this family.
- The v2 emission leg covers tabular results and sentinel arms on non-tabular
  results in-court; the fold-law text lives in the ADR cited by
  `v2_prefix_is_exact_adr_literal` (court-anchored, not restated here).
- CHAIN.blake3 is NOT resynced by this lane (coordinator runs chain sync at
  integration); the in-flight `M receipts/CHAIN.blake3` at receipt-write time
  is disclosed in the header.

## Falsifiers

1. Any court above returning a non-`ok` `test result:` line on this HEAD with
   pin `58a44095…` falsifies the green aggregate.
2. Perturbing the v2 ADR prefix literal must flip `v2_prefix_is_exact_adr_literal`
   red; if it still passes after a prefix change, the pin is vacuous.
3. Any one-sided edit to a committed chain record must flip
   `committed_tcps_head_recomputes_to_its_stored_chain_hash` /
   `fixture_pin_detects_one_sided_edits` red.
4. A v2 emission that does not self-verify on a mutated result set must flip
   `v2_present_on_tabular_results_and_self_verifies` red.

Closing b3sum of this receipt is reported in the lane's final report (a file
cannot embed its own digest); verify with
`b3sum /Users/sac/graphlaw/receipts/2026-10-24-battery-receipt-v2.md`.
