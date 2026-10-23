# wasm4pm graphlaw vector oracle re-run receipt (lane V-L17, staleness-gate R4 remediation)

- Date: 2026-10-22 (local)
- Subject: /Users/sac/wasm4pm (cross-repo, read-only test execution), graphlaw side /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`.
- Backs: `docs/jira/v26.10.22/RECEIPTS.md:594` (§L106 cross-repo wasm4pm oracle 4/0 block).
- Command: `cd /Users/sac/wasm4pm && cargo test -p wasm4pm-cognition --test graphlaw_vector_oracle`

```
test graphlaw_rev_ancestry_court ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

Standing: PASS 4/0 — matches the legacy 4/0 claim in RECEIPTS.md §L106. All
four courts (`exclude_list_refuses_named_breeds`, `graphlaw_rev_court_sabotage_twin`,
`graphlaw_rev_ancestry_court`, `graphlaw_vector_oracle_roundtrip`) re-witnessed.
