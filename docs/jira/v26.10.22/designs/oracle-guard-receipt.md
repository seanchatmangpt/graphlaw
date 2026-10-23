# M04 — X1 oracle/unwrap_or guard: design + receipt

Lane M04 (guard builder), lease: `tests/no_mock_markers.rs` (extension),
`docs/jira/v26.10.22/designs/oracle-guard-receipt.md`. Date: 2026-10-22.

## Defect class (X1, wave W)

Test files defining test-local decision closures/oracles the test then asserts
on (London-style: the test verifies its own model, not real collaborator
state), and `.unwrap_or(...)`/`.unwrap_or_default()` laundering a failed read
into "ran fine" inside `tests/`.

## Gate design

Extends `tests/no_mock_markers.rs` (TC-MOCK-00 build gate) with two scanners
plus one court:

1. **Oracle heuristic** — `scan_oracle(rel, text)`: a line declaring
   `fn <name>(...) -> ...` (written return type required, so `fn test_x()`
   fns are out of scope) followed within `ORACLE_WINDOW_LINES = 40` lines by
   an assert-macro line referencing `<name>(` is flagged as a test-local
   oracle. Allowlist `ORACLE_ALLOWLIST: (file, fn, justification)` sanctions
   existing helpers adjudicated today (real DuckDB/SPARQL/subprocess/envelope
   readbacks, dual-anchored golden digests, fixture builders).
2. **unwrap_or gate** — `scan_unwrap_or(rel, text)`: any `.unwrap_or(` or
   `.unwrap_or_default()` in `tests/*.rs` fails unless the line contains an
   exact snippet in `UNWRAP_ALLOWLIST: (file, line-snippet, justification)`.
   Snippet widening is an allowlist edit. `tests/no_mock_markers.rs` itself
   is skipped structurally (it carries the allowlist literals). Loud
   `.unwrap_or_else(|e| panic!(...))` and plain `.unwrap()` are out of scope —
   they already fail loud.
3. **Non-vacuity court** — `gate_flags_planted_violations_nonvacuity` plants
   a violating file in /tmp (oracle fn + assert within the window + an
   unsanctioned unwrap_or on an `Err` read), asserts both scanners flag it,
   and asserts a clean helper outside the window is not flagged. /tmp file is
   removed in the same test; no sleeps.

## Existing-use adjudication (grep of today, 2026-10-22)

- Oracle heuristic pre-existing hits, all sanctioned in ORACLE_ALLOWLIST
  (19 entries): abi_manifest_court `expected_hash_bytes` (dual-anchored:
  also asserted against pinned `GOLDEN_FRAME_HASH_HEX`), evidentiary_ring_court
  `desc`, k5_hostile_sqlrdf `hostile_t_survives` (real `query_row` readback;
  failure -> `false` FAILS the survival assert), law_state `read`/`select_count`,
  op_differential `s`, op_forward_compat `envelope_ok`, plan_negation `req`,
  receipt_feedback `plan`, receipt_lease `base`, refusal_crosswalk_differential
  `blake3_hex`, registry_dispatch `message`, strata_gate_differential
  `blake3_hex`/`hex`, verify_cli `stdout`, smon_cli_court `stderr`,
  knowledge_hooks `obligations`, chicago_boundary_constraints
  `violation_component`, wasm_abi `refusal_of`.
- unwrap_or-class pre-existing hits (~40 lines across 21 files), all
  sanctioned as serde_json optional-field probes on already-parsed envelopes,
  environment/numeric defaults on lawful-None sources, or poll readbacks whose
  default keeps a bounded poll waiting (times out loud, never passes).
  Embedded literally in UNWRAP_ALLOWLIST with per-entry justification.

## Receipt (real command output, 2026-10-22)

```
$ cargo test --test no_mock_markers
running 3 tests
test gate_flags_planted_violations_nonvacuity ... ok
test no_test_local_oracles_or_unwrap_or_laundering_in_tests ... ok
test no_banned_mock_markers_in_source ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s

$ cargo check
    Checking graphlaw v26.10.21 (/Users/sac/graphlaw)
    Finished `dev` profile [unoptimized + debug] target(s) in 1m 21s
```

## Notes for test-lane owners

- New unsanctioned `.unwrap_or(` in tests/ now fails
  `no_test_local_oracles_or_unwrap_or_laundering_in_tests` with the exact
  line; recovery: fail loud or add an exact-snippet allowlist entry with
  justification.
- New helper fn asserted on within 40 lines fails the oracle scan; recovery:
  assert on real collaborator state or allowlist (file, fn) with
  justification.
