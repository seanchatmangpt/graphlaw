# chicago_validation re-run receipt (lane L167, staleness-gate R4 remediation)

- Date: 2026-10-22 (local) / 2026-10-23T02:40:18Z
- Subject: /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`, working tree dirty (lane-isolated build `CARGO_TARGET_DIR=target-lane167`).
- Backs: `docs/jira/v26.10.22/RECEIPTS.md:54`.
- Command: `CARGO_TARGET_DIR=target-lane167 cargo test --test chicago_validation`

```
     Running tests/chicago_validation.rs (target-lane167/debug/deps/chicago_validation-0c8bfa1a94361a5d)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.57s
```

Standing: PASS 9/0 — matches the legacy 9/0 claim.
