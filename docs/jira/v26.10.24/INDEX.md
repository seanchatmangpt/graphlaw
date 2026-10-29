# INDEX — milestone window v26.10.24

Registered 2026-10-24 (docs+registers lane). Active window. Thesis: the
single-release consolidation — v26.10.23 was **skipped** by operator
directive 2026-10-24 (no release cut under that moniker; the post-v26.10.22
wave that had accumulated in `v26.10.23/` ships under v26.10.24). Every
markdown doc in `docs/jira/v26.10.24/` is registered below; doc count
re-read from disk at registration.

## Register: window docs

- [GO-NO-GO-v26.10.24.md](GO-NO-GO-v26.10.24.md) — Skeleton (2026-10-24,
  this lane). Ground state §0 re-read at write (HEAD `a00770b`, pin
  `58a44095…` 4/4 surfaces, chain head `bed6b904…`); §A carries the
  v26.10.22 closeout rows with per-row falsifiers; §B verdict block UNFILLED
  (operator fill-in); §C receipt fields; §D see-also.
- [ROADMAP-v26.10.24.md](ROADMAP-v26.10.24.md) — Active plan (2026-10-24,
  this lane). RETIRE/CARRY disposition of the v26.10.23 roadmap plus the
  new lanes: N3/Datalog query-compiler, purrdf 3.0.1 + eyeron 0.8.18
  (MSRV 1.98), gate-first doc migration, publish interlock law.
- [RECEIPTS.md](RECEIPTS.md) — Window ledger (2026-10-24, this lane).
  One row per receipt in the v26.10.22 convention; opening rows for the 13
  landed 2026-10-24 receipts plus the owed-but-unlanded battery-receipt
  rows.

## Register: carried rows (cross-stitch; owners named)

Carried from the v26.10.22 closeout (GO-NO-GO v5 §A) and the v26.10.23
registers. Each row keeps its falsifier; full wording in
[GO-NO-GO-v26.10.24.md](GO-NO-GO-v26.10.24.md) §A.

| Row | Subject | State at registration | Owner / next act |
|---|---|---|---|
| G2-rem | Solo mutation-court verdict | Config gap FIXED `a00770b` (kill suite += `abi_response_builders_court`; examine_globs += `src/registry/*.rs`; GUARDED restore widened). Inventory **88 → 204** (definitions 15, emit_json 50, emit_ttl 51). Verdict still owed; solo window, exit ∈ {0,2}, zero gate-relevant misses | coordinator, solo |
| G3-rem | FULL-battery remainder | DISCHARGED legs: L1-L3 all green, dialect census 36/0, receipt_v2 6/0, emission 8/0, trust_attest 12/0, dvp 4/0. REMAINING: `op_differential` sniff red (bat-L4, fresh G8-class witness), 5 unexecuted L4 courts, L5/L6. L5a/L5b/L6 ran 13:25-13:33 and are RED compile-abort (`purrdf_stack` unresolved; pytest conftest build fail) on the in-flight vendor surgery — receipts still owed | coordinator |
| R-1-wrap | SQLRDF-27 wrap-flag inversion | Pin-gate inversion WITNESSED (`receipts/2026-10-24-r1-rungs.md`: admit → refuse exit 84 → admit, got==b3sum). The literal wrap-flag form (drop `--wrap=open,--wrap=openat` → tc08 EINVAL) remains owed on the .24 artifact | coordinator |
| R-2 | Deny-advisory re-review | DISCHARGED 2026-10-24 (`receipts/2026-10-24-r2-deny-decision.md`; `deny.toml:10-20` dated ignores, expiry 2026-11-23). Carried as calendar, not debt | calendar |
| OD-4 | SPC production-bar decision | Evidence memo filed (OD-4 memo); DECISION line BLANK — operator signature owed | operator |
| OD-6 | `.claude/` as repo law | ANOMALOUS-LINE finding stands; signature owed | operator |
| G2-CanonArrow | egress as first-class ABI op | `src/egress/collector.rs` landed on disk (12,197 B, 2026-10-24); **no `egress_arrow` op** in `src/abi.rs` — adoption half remains | code lane |
| G6 | Bench receipts | `sql_rdf_compiler` / `sql_rdf_admission` / TC-HOT numbers unreceipted on disk | code lane |
| D3 | GWS register stale tokens | PENDING-GWS-1/4/7/8 re-checked 2026-10-24, no owning receipts (`docs/wasm-duckdb-integration.md:49-56`) | code lane |
| D4 | Witness durability | Census class now DURABLE (dialect ×6 receipts); fuzz / memory-wall / exit-codes still transcript-class pending the L5-class re-runs | coordinator |
| W08 | Differential run + receipt | `tests/common/mod.rs:20-22` hosts the harness; run+receipt against the pinned artifact owed (the bat-L4 sniff divergence makes it release-relevant) | coordinator |
| ref-gen | reference-generated regen at cut | `docs/reference-generated/` (5 files) must be regenerated at the release cut, never patched | coordinator |

## Calendar

- **2026-11-23** — R-2 re-review: the three dated deny.toml ignores expire
  (RUSTSEC-2026-0258, RUSTSEC-2025-0134, RUSTSEC-2024-0449).
- **2026-11-23** — deny.toml twin: same expiry; one re-review discharges
  both rows.

## v26.10.23 disposition note

v26.10.23 was skipped as a release moniker by operator directive
2026-10-24. The window directory `docs/jira/v26.10.23/` stays as written
(27 docs on disk at re-read 2026-10-24, all registered by its INDEX, plus
`_LANES.md` which its INDEX counts as coordination matter rather than a
registered doc — count correction recorded in its INDEX). It has **no
RECEIPTS.md** — that finding is recorded here; no retroactive ledger is
created. Its open rows carry forward into this window's registers above.
