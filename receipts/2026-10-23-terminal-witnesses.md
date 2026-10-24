# Terminal Witness Check — EH Courts + Lifecycle (2026-10-23)

## Observed state (as witnessed 2026-10-23T16:33:10-0700)

Neither background witness run produced a final verdict. Both processes
have ended (no live `graphlaw_run_eh_court` / lifecycle processes at check
time). Pending is pending — nothing here is claimed ALIVE.

## Witness 1: /tmp/ehcourts2.log (graphlaw_run_eh_court, 7 tests)

sha256: `59e59069221894407285eafb54ab6f42a533418519c1d8d51da4b47eb216d949`
(129 lines, last write 16:24 local)

Log tail (real, embedded):

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.89s
     Running tests/graphlaw_run_eh_court.rs (target-coord/debug/deps/graphlaw_run_eh_court-790acd83960e4b44)

running 7 tests
test exit0_clean_empty_stdin ...
```

**No `test result:` line exists.** The log ends mid-test-name on the first
test, `exit0_clean_empty_stdin`. No verdict, no failure output, exit status
of the cargo process not captured.

This matches the KNOWN COURT GATE DEFECT: the skip path in
`tests/graphlaw_run_eh_court.rs:47-56` (`fn skip(binary_missing: bool) -> !`)
calls `std::process::exit(0)` from inside a test. That kills the entire test
harness with exit code 0 before any result is reported — the court's own
skip is indistinguishable from a pass, and the log ends exactly after
`test exit0_clean_empty_stdin ...` with no result line. The court cannot
exit 1 on its own skip; per exit discipline ("a gate that cannot exit 1 is
the failure"), this is a gate defect, flagged for the typed-non-zero-skip
fix (return a typed SKIP outcome and let the harness report it, never
`process::exit(0)` mid-harness).

## Witness 2: /tmp/lifecycle.log (wasm_duckdb_lifecycle_eh)

sha256: `a669b850b7d095d38ef2fc7f634ded2b3c8e1f6369b4aeefaa2aba089dd947de`
(27 lines, last write 16:24 local)

Log tail (real, embedded):

```
error: no test target named `wasm_duckdb_lifecycle_eh` in `graphlaw-wasm` package
help: available test in `graphlaw` package:
    wasm_duckdb_lifecycle_eh
```

**Invocation defect, not a test verdict.** The run targeted `-p graphlaw-wasm`,
but the test lives in the root `graphlaw` package. Cargo refused the target;
no test executed (exit non-zero from cargo, masked by the outer `echo`).
Re-run required with the correct package:

```
cargo test -p graphlaw --test wasm_duckdb_lifecycle_eh
```

## Suite re-witness summary (as-of dispatcher report, this date)

85 PASS / 8 real / 15 env-blocked, with punch-list lane attribution carried
in the lane receipts under `receipts/2026-10-23-*` (shim-artifact-rewitness,
sqlrdf27-verdict, wave-close). Numbers cited as-of-report, not re-derived
in this lane; terminal witnesses above were the open item and remain OPEN.

## Status

- EH courts (7 tests): PENDING — aborted by skip-exit-0 defect (known gate defect, flagged)
- lifecycle witness: PENDING — wrong-package invocation, needs re-run
- Punch list: memory-wall + lifecycle witnesses on canonical; SPC Cpk FAIL
  under load needs solo-machine rerun (carried from shim-artifact-rewitness)
