# sql_rdf_conformance re-run receipt (lane L167, staleness-gate R4 remediation)

- Date: 2026-10-22 (local) / 2026-10-23T02:40:18Z
- Locus: /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`, working tree dirty (lane-isolated build `CARGO_TARGET_DIR=target-lane167`).
- Backs: `docs/jira/v26.10.22/designs/RECEIPTS.md:149` (legacy line records the 2026-10-22 12:58 PDT 4/1 flip-fixture failure).
- Command: `CARGO_TARGET_DIR=target-lane167 cargo test --test sql_rdf_conformance`

```
     Running tests/sql_rdf_conformance.rs (target-lane167/debug/deps/sql_rdf_conformance-94d63ea7e72bc3e9)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

Standing: PASS 7/0. The legacy 4/1 flip-fixture failure was repaired earlier the
same day (RECEIPTS.md §3 records 5/0 superseding it); the court file has since
grown to 7/0. The legacy failure line stays visible — this receipt records
current standing alongside it, not a laundering pass.
