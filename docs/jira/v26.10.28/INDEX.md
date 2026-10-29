# v26.10.28 — window index

Milestone: VOC→CTQ translation and QFD House of Quality for the graphlaw
kernel surfaces (wasm execution, admission law, SQL-over-RDF, receipts,
graphlaw_ex Elixir embed), plus honest-standing work orders and the
purrdf fork-swap receipt from the 2026-10-28 session. The coordinator
owns git.

| file | role | status |
| --- | --- | --- |
| [INDEX.md](INDEX.md) | Window index | this file |
| [VOC-CTQ-TURNSTILE.md](VOC-CTQ-TURNSTILE.md) | Stakeholder segmentation, CTQ matrix (1.1–5.3), Kano classification | COMPLETE — CTQs 1.1–5.3 defined; downstream orders cite these ids |
| [QFD-HOUSE-OF-QUALITY.md](QFD-HOUSE-OF-QUALITY.md) | House of Quality: CTQ rows × engineering characteristics, roof correlations, targets, current standing | COMPLETE — standing graded from 2026-10-28 session evidence only |
| [WORK-ORDERS.md](WORK-ORDERS.md) | Per-CTQ work orders with session-receipt standing | 6 orders: WO-1 ALIVE closed (SPC court exit 0, Cpk 6.0979 ≥ 1.33, latency tail embedded in `receipts/spc-2026-10-28T165319.json`); WO-4 GO (35/35 statement kinds); WO-2/3/5/6 PARTIAL with named pending gates |
| [RECEIPTS.md](RECEIPTS.md) | Manufacturing receipt: purrdf fork swap @ 2223f894 (38 fork-sourced packages) | COMPLETE — real command output embedded |
| [GO-NO-GO.md](GO-NO-GO.md) | Evidence table + verdict | NO-GO as written — §A rows predate the 16:53 SPC landing (WO-1 still shown pending there); refresh in flight by the GO-NO-GO lane. On-disk truth: 4 of 6 orders carry named pending gates (WO-2/3/5/6); WO-1 and WO-4 met. Rows flip receipt-gated only |
| [TAG](TAG) | Milestone tag `v26.10.28` | PENDING — coordinator tag; not yet cut |

Standing at final index pass (2026-10-28, re-read from WORK-ORDERS.md,
GO-NO-GO.md, and `receipts/` on disk): purrdf fork swap @ 2223f894 ALIVE
(cargo check 0 errors; chicago_validation 8/8; proof_pipeline 12/12;
conformance 12/12; census 13/13). Wasm build ALIVE (exit 0, 5m22s, blake3
`348c97fd…`). SPC court LANDED: `bash scripts/spc_court.sh` exit 0, Cpk
6.0979 ≥ 1.33, p99 125 ns, tail_fraction 0.0 — receipt
`receipts/spc-2026-10-28T165319.json` (re-read at write time), latency tail
embedded — so WO-1 is closed, not pending. Still PENDING this window:
**mutation court** (`bash scripts/mutation_court.sh` — no 2026-10-28
receipt in `receipts/`, the one open admission gate), `trust_attest`,
full `cargo test`, full `mix test`, WO-5 emission census + traceability
sweep. Details and commands in WORK-ORDERS.md and GO-NO-GO.md.
