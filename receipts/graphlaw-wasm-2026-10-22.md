# graphlaw-wasm boundary re-run receipt (lane L167, staleness-gate R4 remediation)

- Date: 2026-10-22 (local) / 2026-10-23T02:40:18Z
- Subject: /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`, working tree dirty (lane-isolated build `CARGO_TARGET_DIR=target-lane167`).
- Backs: `docs/jira/v26.10.22/RECEIPTS.md:201` and `:322`.
- Command: `CARGO_TARGET_DIR=target-lane167 cargo test -p graphlaw-wasm`

```
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

Standing: PASS 9/0 — matches the legacy 9/0 claims at both cited lines. First
attempt at 02:3x hit a transient shared-lib privacy error (`sorted_entries`,
E0603) from another lane's in-flight edits; compile-freeze SLA disclosure — the
repair landed mid-attempt and this lane's retry is the receipted run above.
