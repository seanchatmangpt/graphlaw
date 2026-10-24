# Banking battery witness — DvP 4/4 GREEN

Date: 2026-10-23. Lane: witness lane (target-lane-wit), canonical checkout `/Users/sac/graphlaw`.

## Command and embedded real output

```
$ CARGO_TARGET_DIR=target-lane-wit cargo test --test dvp_settlement_court 2>&1 | tail -3

warning: `graphlaw` (lib) generated 4 warnings
   Compiling graphlaw v0.1.0 (/Users/sac/graphlaw)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

Verdict: `test result: ok. 4 passed; 0 failed` — matches the prior DvP receipt claim (target-lane-dvp2).

## Transcript note (compile-freeze SLA)

The first run of this lane hit a shared-lib compile abort (duplicate `CompositeKind`
at `src/sql_ast.rs:35` and `:45`, 7 E0428/E0119 errors) — a concurrent lane's
in-flight edit, not a regression. A later run in this same lane, after the owning
lane's fix landed, compiled and passed as embedded above. Recorded under the
census rule: compile-abort (exit 1, 0 tests executed) is not a failure count.

## Per-court results (4/4 GREEN)

Court binary: `tests/dvp_settlement_court.rs` (4 `#[test]` functions).

1. `two_pair_asymmetric_string_decimal_insert_pairs_by_index`
   (tests/dvp_settlement_court.rs:627) — exact-SQL regression, in the passing set.
2-4. The other three DvP settlement courts — in the passing set (4 passed, 0 failed).
   Individual names not re-listed from memory; the count and 0-failed line above is
   the receipt, not recollection.

## Pairing law

DvP pairing is assignment-by-index: `assign_pairs` consumes `document_order_assigns`
over a quote-aware scan of the statement text, pairing leg i with leg i by position
in document order — no name-based or value-based matching.

## Anti-vacuity statement

The witnessed court is a real frozen-duckdb court asserting on real table readbacks
(Chicago style). The exact-SQL regression at tests/dvp_settlement_court.rs:627 pins
the literal statement text, so any change to the pairing law's output flips the
court: removing the index-based assignment or switching to name-based pairing fails
`two_pair_asymmetric_string_decimal_insert_pairs_by_index`. The court gates on
admission + evaluation output, not on call counts or stubs.

## Related standing

- DvP: 4/4 GREEN (this receipt, witnessed on target-lane-wit).
- BCBS / Rehypo / UBO: 3/3 each (prior banking inventory witness, target-pub lane);
  not re-run per no-duplicate-work discipline — standing cited as-of prior witness.

## Integrity

The b3sum of this file cannot be embedded in itself. The digest of the final
file as written is recorded in the pointer row appended to
`docs/jira/v26.10.22/RECEIPTS.md`.
