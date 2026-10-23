# query_ledger_court re-run receipt (lane L167, staleness-gate R4 remediation)

- Date: 2026-10-22 (local) / 2026-10-23T02:40:18Z
- Subject: /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`, working tree dirty (lane-isolated build `CARGO_TARGET_DIR=target-lane167`).
- Backs: `docs/jira/v26.10.22/RECEIPTS.md:165`.
- Command: `CARGO_TARGET_DIR=target-lane167 cargo test --test query_ledger_court`

```
     Running tests/query_ledger_court.rs (target-lane167/debug/deps/query_ledger_court-e85b40237fbec00b)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

Standing: PASS 9/0 — matches the legacy 9/0 claim.
