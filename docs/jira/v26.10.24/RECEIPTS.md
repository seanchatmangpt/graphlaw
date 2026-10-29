# RECEIPTS — v26.10.24

Window ledger, v26.10.22 convention: one row per landed receipt — the
receipt file, its verdict with the embedded-output line it stands on, and
the as-of time. A receipt with no embedded output is not a receipt; every
row below names the verbatim line the verdict is transcribed from. The
tree was LIVE while this ledger was opened (concurrent lanes landing); the
as-of column is per-receipt, not a window-wide read.

## Landed (2026-10-24)

| Receipt | Gate / subject | Verdict (embedded-output line) | As-of (2026-10-24) |
|---|---|---|---|
| `receipts/2026-10-24-r1-rungs.md` | R-1 gate rungs: host-contract + pin-gate inversion on pin `58a44095…` | `LEG_B_EXIT=84` with `got 9872f447…` == independent b3sum; `LEG_C_READMIT_EXIT=0` | 17:2xZ |
| `receipts/2026-10-24-r2-deny-decision.md` | R-2 residual-advisory operator decision | 3 advisories ACCEPTED, dated ignores `deny.toml:10-20`, expiry 2026-11-23 | 10:19 |
| `receipts/2026-10-24-spc-full.md` | SPC FULL-mode court, tc_hot_01 clone_path | FAIL — Cpk 0.6111 < 1.33 (honest-FAIL receipt, verdict stands as OD-4 evidence) | 10:23 |
| `receipts/2026-10-24-spc-solo-rerun.md` | SPC solo falsifier rerun (idle host) | FAIL — `Cpk 0.632 < 1.33`, exit 1 bare-captured; signature changed vs fleet-load run | 03:31 |
| `receipts/2026-10-24-fuel-calibration.md` | FUEL_PER_REQUEST 10M → 100B calibration ladder | Landed value witnessed at `src/bin/graphlaw_run_eh.rs:45` (`100_000_000_000`); rungs kept only on real EH-court witnesses | 03:28 |
| `receipts/2026-10-24-wasmi-retirement-deletion.md` | wasmi absent-from-graph witness | `cargo tree` inversion probes: wasmi/wasmi_wasi absent from the dependency graph post-`6da1d33` | 17:21Z |
| `receipts/2026-10-24-wasmi-retirement-step2-green-witness.md` | pytest over `graphlaw-run-eh` (consumer flip) | `tests_py` PASSED 19/0 ×2 with `GRAPHLAW_WASM=priv/graphlaw.wasm`; zero non-eh runner refs | 10:31 |
| `receipts/2026-10-24-dialect-sql-rdf.md` | SQL-in-RDF dialect surface | All 13 courts exit 0; `sql_rdf_dialect 36 passed / 0 failed` (157/157 corpus figure recorded in-receipt) | 10:52 |
| `receipts/2026-10-24-dialect-turtle.md` | TURTLE parse/canonicalize/serialize | `test result: ok. 8 passed; 0 failed` (107/108 corpus figure recorded in-receipt) | 11:00 |
| `receipts/2026-10-24-dialect-n3.md` | N3 surface | `test result: ok. 21 passed; 0 failed`, exit 0 (30/30 corpus figure recorded in-receipt) | 10:47 |
| `receipts/2026-10-24-dialect-shacl.md` | SHACL surface | `36 passed; 0 failed` + `23 passed; 0 failed`, EXIT 0 (142/144 corpus figure recorded in-receipt) | 10:58 |
| `receipts/2026-10-24-dialect-shex.md` | ShEx surface | Honest RED rows recorded in-receipt (`2 passed; 1 failed`, `3 passed; 2 failed`) — perimeters ledgered, not hand-waved | 11:01 |
| `receipts/2026-10-24-dialect-datalog.md` | DATALOG surface | `6 passed; 0 failed` + `3 passed; 0 failed`; one honest-RED leg (`2 passed; 1 failed`) ledgered | 11:07 |

## Owed (named, not yet landed — filing order per X13: battery receipts first)

| Expected receipt | Gate | Blocking observation (as-of 2026-10-24 13:5x PDT) |
|---|---|---|
| `receipts/2026-10-24-battery-dialect-census.md` | dialect census re-run on the pinned artifact | Witnessed in `/tmp/bat-L1.log` (8 courts green) — durable file owed |
| `receipts/2026-10-24-battery-receipt-v2.md` | receipt_v2 e2e + emission courts | Witnessed in `/tmp/bat-L3.log`: `receipt_v2_court 6 passed / 0 failed`, `receipt_v2_emission_court 8 passed / 0 failed` — durable file owed |
| `receipts/2026-10-24-battery-trust-attest.md` | trust_attest re-run | Witnessed 12/0 in the battery legs — durable file owed |
| `receipts/2026-10-24-battery-banking.md` | dvp_settlement_court | `4 passed; 0 failed` (witness exists at `receipts/2026-10-23-banking-4of4-witness.md` on historical `f3570f15`) — pin-identity re-run owed |
| `receipts/2026-10-24-battery-full.md` | consolidated FULL battery pointer row | Blocked on the REMAINING half: `op_differential` sniff red (bat-L4) + 5 unexecuted L4 courts + L5/L6 re-run (compile-aborted RED 13:25-13:33 on the in-flight vendor surgery) |

## Receipt fields

- **Identity**: window ledger opened by the docs+registers lane over
  `/Users/sac/graphlaw` HEAD `a00770b` (live tree — disclosed), 2026-10-24.
- **Authority**: write lease = this file; no cargo, no git transitions.
- **Consequence**: one new file; 13 landed rows + 5 owed rows.
- **Replay**: `ls receipts/2026-10-24-*.md`; per-row verbatim lines re-read
  from the named receipt files; battery-leg lines re-read from
  `/tmp/bat-L*.log` (transient evidence — the owed receipts retire them).
- **Falsifier**: any verdict line above contradicting its receipt file on
  re-read → correction row appended; an owed receipt landing → its Owed row
  moves to Landed with its verbatim line.
- **Standing**: ledger as of 2026-10-24 13:5x PDT, HEAD `a00770b`; append
  only after this opening.
