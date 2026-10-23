# frozen-duckdb pin receipt (lane L167, staleness-gate R4 remediation)

- Date: 2026-10-22 (local) / 2026-10-23T02:26:41Z
- Subject: /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`, working tree dirty (139 modified files per `git diff --stat`).
- Purpose: durable pointer backing `docs/jira/v26.10.22/RECEIPTS.md:116`.
- Result: **DIVERGENCE WITNESSED, loudly.** The legacy line records frozen-duckdb
  2.0.0 / checksum `f9eae23d…`; the live tree has moved.

## Command + output (real)

```
$ grep -n -A5 'name = "frozen-duckdb"' Cargo.lock
1197:name = "frozen-duckdb"
1198-version = "2.0.2"
1199-source = "registry+https://github.com/rust-lang/crates.io-index"
1200-checksum = "7b2cd958aa5fdbea2b239001e8c1c397c624c5b49a90729c3e85dfe753ea5692"
1201-dependencies = [
$ grep -n 'frozen-duckdb' Cargo.toml
35:frozen-duckdb = { version = "2.0.2" }
```

## Reading

- Live pin: frozen-duckdb **2.0.2**, checksum `7b2cd958aa5fdbea…5692`, Cargo.lock:1197-1200.
- Legacy claim (RECEIPTS.md:116): 2.0.0, checksum `f9eae23d…66399`, Cargo.lock:1182-1186 —
  as-of-date historical record, superseded by the 2.0.2 pin. Recorded here so the
  pointer is content-backed, not a laundering pass.
