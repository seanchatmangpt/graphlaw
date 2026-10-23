# duckdb_statement_kinds re-run receipt (lane L167, staleness-gate R4 remediation)

- Date: 2026-10-22 (local) / 2026-10-23T02:40:18Z
- Subject: /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`, working tree dirty (lane-isolated build `CARGO_TARGET_DIR=target-lane167`).
- Backs: `docs/jira/v26.10.22/RECEIPTS.md:55`.
- Command: `CARGO_TARGET_DIR=target-lane167 cargo test --test duckdb_statement_kinds`

```
     Running tests/duckdb_statement_kinds.rs (target-lane167/debug/deps/duckdb_statement_kinds-863b762a06b271ec)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.58s
```

Standing: PASS 6/0. Divergence vs legacy (5 passed / 1 failed with the `.gram`
pin-source environmental exception) — the exception has since cleared; the
legacy line stands as an honest historical record, not laundered.
