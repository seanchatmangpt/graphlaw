# Battery Receipt — banking leg greens (L4 finance: amortization / bcbs / dvp gating leg / gws12 / gws4)

Lane: EXECUTION (battery receipts), lease `receipts/2026-10-24-battery-*.md` +
ledger appends. Date: 2026-10-24. Leg target dir:
`/Users/sac/graphlaw/target-battery-finance` (build dir preserved at receipt
write time; the leg's test-target binaries carry mtimes 11:20). All runs:
`CARGO_TARGET_DIR=…/target-battery-finance cargo test --locked --test <target> …`
(fresh lane build; `Finished \`test\` profile [unoptimized + debuginfo]
target(s) in 3m 32s`, verbatim at `/tmp/bat-L4.log:593`).

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
  below is spliced mechanically from the digested `/tmp/bat-L4.log`; nothing
  in the fenced blocks is hand-typed.

## Scope and the RED tail disclosure

This receipt claims ONLY the five green courts below (13 passed / 0 failed).
The L4 leg as launched also carried `tests/op_differential.rs`, which came
back **RED (15 passed / 1 failed)**, and its fail-fast cut left the four
alphabetically-later compiled courts unexecuted (`op_examples`,
`op_forward_compat`, `rehypothecation_gate_court`, `ubo_sanctions_court`).
Those are NOT claimed here; they are classified in
`receipts/2026-10-24-battery-full.md` (RED/INCOMPLETE classification, verbatim
panic block, capture-masking disclosure).

The `>60 seconds` lines in the blocks below are the harness's slow-test
notices under concurrent fan-out load, not sleeps and not coordination.

## Aggregate

| Court (target) | Tests | Result | Classification |
|---|---|---|---|
| amortization_court | 1 | 1/0 ok | PASS |
| bcbs_aggregation_court | 3 | 3/0 ok | PASS |
| dvp_settlement_court (G3 gating leg) | 4 | 4/0 ok | PASS |
| gws12_coldchain_cross_engine | 2 | 2/0 ok | PASS |
| gws4_ipc_injection_court | 3 | 3/0 ok | PASS |
| **Total (greens)** | **13** | **13 passed; 0 failed** | 5 PASS · 0 CLASSIFIED · 0 SKIP |

## Per-court witnesses (mechanically spliced from `/tmp/bat-L4.log`)

### 1. dvp_settlement_court (4/0) — the G3 gating leg

```
     Running tests/dvp_settlement_court.rs (target-battery-finance/debug/deps/dvp_settlement_court-4a59202d805e86ad)

running 4 tests
test two_pair_asymmetric_string_decimal_insert_pairs_by_index ... ok
test dvp_01_committed_atomic_settlement has been running for over 60 seconds
test dvp_02_aborted_atomic_settlement_zero_net_change has been running for over 60 seconds
test dvp_03_one_legged_trace_rejected_by_conformance_check has been running for over 60 seconds
test dvp_03_one_legged_trace_rejected_by_conformance_check ... ok
test dvp_02_aborted_atomic_settlement_zero_net_change ... ok
test dvp_01_committed_atomic_settlement ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 230.66s
```

`dvp_01_committed_atomic_settlement`, `dvp_02_aborted_atomic_settlement_zero_net_change`,
`dvp_03_one_legged_trace_rejected_by_conformance_check` — atomic settlement
executed on a real engine with readbacks (artifact-instantiating). G3's
condition "dvp court must stay 4/0"
(`docs/jira/v26.10.23/GO-NO-GO-v26.10.22.md` §A G3) holds on this leg:
4 passed; 0 failed. The historical RED (battery9: dvp 2 passed; 2 failed) is
closed for this court by this witness on HEAD `a00770b` + pin
`58a44095…`.

### 2. bcbs_aggregation_court (3/0)

```
     Running tests/bcbs_aggregation_court.rs (target-battery-finance/debug/deps/bcbs_aggregation_court-ee5eabaf47ceca7b)

running 3 tests
test tc_bcbs_01_valid_fold_binds_child_receipts has been running for over 60 seconds
test tc_bcbs_02_forged_child_refused_before_aggregation has been running for over 60 seconds
test tc_bcbs_03_unverified_fold_is_detectable has been running for over 60 seconds
test tc_bcbs_01_valid_fold_binds_child_receipts ... ok
test tc_bcbs_02_forged_child_refused_before_aggregation ... ok
test tc_bcbs_03_unverified_fold_is_detectable ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 333.78s
```

Valid fold binds child receipts; forged child refused before aggregation
(non-vacuity, typed); unverified fold detectable.

### 3. amortization_court (1/0)

```
     Running tests/amortization_court.rs (target-battery-finance/debug/deps/amortization_court-9af03c9eaa22ff2e)

running 1 test
test amortization_court_flip ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 57.46s
```

### 4. gws12_coldchain_cross_engine (2/0) — cross-engine partition identity

```
     Running tests/gws12_coldchain_cross_engine.rs (target-battery-finance/debug/deps/gws12_coldchain_cross_engine-d961a732e5951516)

running 2 tests
test gws12_partition_identical_across_all_three_legs has been running for over 60 seconds
test gws12_tampered_aggregate_flips_partition_identically_on_both_engines has been running for over 60 seconds
test gws12_tampered_aggregate_flips_partition_identically_on_both_engines ... ok
test gws12_partition_identical_across_all_three_legs ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 69.36s
```

Partition identical across all three legs; tampered aggregate flips
identically on both engines (non-vacuity, cross-engine agreement on the
failure).

### 5. gws4_ipc_injection_court (3/0)

```
     Running tests/gws4_ipc_injection_court.rs (target-battery-finance/debug/deps/gws4_ipc_injection_court-ea069d185434945c)

running 3 tests
test gws4_b_chunk_streaming_design_meets_16mib_bound_and_merge_equals_whole ... ok
test gws4_a_corpus_injected_as_linear_memory_buffer_loads_into_duckdb_eval has been running for over 60 seconds
test gws4_c_fail_closed_when_corpus_exceeds_bounds has been running for over 60 seconds
test gws4_c_fail_closed_when_corpus_exceeds_bounds ... ok
test gws4_a_corpus_injected_as_linear_memory_buffer_loads_into_duckdb_eval ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 70.91s
```

`gws4_a_corpus_injected_as_linear_memory_buffer_loads_into_duckdb_eval` —
real frozen-duckdb load (artifact-instantiating);
`gws4_c_fail_closed_when_corpus_exceeds_bounds` — typed fail-closed bound.

## Honesty line: machine-courted vs prose-declared

- All 13 green tests execute real settlement/fold/injection flows against the
  engine with readbacks; zero mocks.
- The banking green (5/5 courts) does NOT make the L4 leg green: the leg is
  RED/INCOMPLETE via op_differential + the fail-fast cut — classification
  and falsifiers live in `receipts/2026-10-24-battery-full.md`.

## Falsifiers

1. A rerun of `cargo test --locked --test dvp_settlement_court` printing
   anything but `4 passed; 0 failed` on this HEAD + pin reopens the G3
   banking leg.
2. A forged child that aggregation accepts must flip
   `tc_bcbs_02_forged_child_refused_before_aggregation` red; if it still
   passes, the court is vacuous.
3. Any court above returning a non-`ok` `test result:` line falsifies the
   13/0 aggregate.
4. The op_differential RED standing is discharged only by a green rerun
   receipt (see battery-full falsifier (a)); a banking rerun that stays
   green does not flip it.

Closing b3sum of this receipt is reported in the lane's final report (a file
cannot embed its own digest); verify with
`b3sum /Users/sac/graphlaw/receipts/2026-10-24-battery-banking.md`.
