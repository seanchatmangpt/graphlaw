# law-kernel lib courts re-run receipt (lane V-L17, staleness-gate R4 remediation)

- Date: 2026-10-22 (local)
- Subject: /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`, working tree dirty.
- Backs: `docs/jira/v26.10.22/designs/RECEIPTS.md:100` (legacy `cargo test --lib` 17/0 record, 12:58 PDT).
- Command: `cargo test --lib`

```
     Running unittests src/lib.rs (target/debug/deps/graphlaw-6db8075342a684cc)
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

Standing: PASS 34/0 — the lib suite has grown since the legacy 17/0 record; all
current law-kernel unit courts pass. The legacy 17/0 line stays visible as an
as-of-date historical record.
