# sql_rdf_laundering_corpus re-run receipt (lane V-L17, staleness-gate R4 remediation)

- Date: 2026-10-22 (local)
- Subject: /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`, working tree dirty.
- Backs: `docs/jira/v26.10.22/designs/RECEIPTS.md:72` (legacy 11/11 laundering-corpus record, 12:58 PDT) and `docs/jira/v26.10.22/RECEIPTS.md:137` (same legacy count).
- Command: `cargo test --test sql_rdf_laundering_corpus`

```
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
```

Standing: PASS 23/0 — the corpus has grown since the legacy 11/11 record; all
current laundering courts pass. The legacy 11/11 lines stay visible as
as-of-date historical records; this receipt records current standing alongside
them, not a laundering pass.
