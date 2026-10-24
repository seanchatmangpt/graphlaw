# RECEIPTS-INDEX

renamed from RECEIPTS.md 2026-10-24; per-version ledgers live under
`docs/jira/<version>/`.

Pointer index for durable witness receipts under `receipts/`. Newest last.
Machine receipts (JSON) are append-only; rows point to their witnesses.

| date | receipt | subject | verdict | machine receipt (sha256) |
|---|---|---|---|---|
| 2026-10-24 | `receipts/2026-10-24-spc-solo-rerun.md` | SPC gate solo rerun (falsifier for 2026-10-23 fleet-load FAIL) — `CARGO_TARGET_DIR=target-lane-spc2 bash scripts/spc_court.sh`, exit 1 | FAIL — Cpk 0.632 < 1.33 (p99 167ns in spec; prior tail confirmed load-induced, sd 16282.667→64.24ns) | `receipts/spc-2026-10-24T032511.json` `b346c2ae873243cd8dd66025c9828b7c33f642b0761cb635fe84176b8061d650` |
