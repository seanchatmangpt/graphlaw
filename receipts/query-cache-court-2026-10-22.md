# query_cache_court re-run receipt (lane L167, staleness-gate R4 remediation)

- Date: 2026-10-22 (local) / 2026-10-23T02:40:18Z
- Subject: /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`, working tree dirty (lane-isolated build `CARGO_TARGET_DIR=target-lane167`).
- Backs: `docs/jira/v26.10.22/RECEIPTS.md:173`.
- Command: `CARGO_TARGET_DIR=target-lane167 cargo test --test query_cache_court`

```
     Running tests/query_cache_court.rs (target-lane167/debug/deps/query_cache_court-b1560fd873e1b1bc)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

Standing: PASS 8/0. Divergence vs legacy 7/0 is court growth since the entry
was written — recorded loudly, not hidden.
