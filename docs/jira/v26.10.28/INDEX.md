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
| [WORK-ORDERS.md](WORK-ORDERS.md) | Per-CTQ work orders with session-receipt standing | 5 ALIVE + 1 PARTIAL: WO-1 closed (SPC Cpk 6.0979, receipt `receipts/spc-2026-10-28T165319.json`); WO-3 closed (trust_attest 12/12 rerun); WO-4 closed (35/35 kinds); WO-5 closed (digest repin 6/6 + emission-witnessed receipt surface); WO-6 closed (wasmex admit/tamper 2/2, WASI 28/28, fork @ 2223f894); WO-2 sole open gate (mutation court) |
| [RECEIPTS.md](RECEIPTS.md) | Manufacturing receipt: purrdf fork swap @ 2223f894 (38 fork-sourced packages) | COMPLETE — real command output embedded |
| [GO-NO-GO.md](GO-NO-GO.md) | Evidence table + verdict | POSTURE: **GO except WO-2** (refreshed at write time from the on-disk file) — WO-1/3/4/5/6 GO on this-session receipts; WO-2 sole NO-GO pending the coordinator's solo `bash scripts/mutation_court.sh` exit 0, which flips the whole window to GO |
| [TAG](TAG) | Milestone tag `v26.10.28` | PUSHED — tag on `82814ac4` (HEAD); release.yml run 40797877715 completed/success; npm/Hex dry-runs in flight |

Standing at final index pass (2026-10-28, re-read from WORK-ORDERS.md,
GO-NO-GO.md, and `receipts/` on disk): tag `v26.10.28` is **PUSHED** at
`82814ac4f398274e64aeb47d2448d04f89cfefcd`; release.yml run 40797877715
completed/success (auto-publishing). npm/Hex dry-runs in flight.
GO-NO-GO posture: **GO except WO-2** — WO-1/3/4/5/6 GO on this-session
receipts (SPC Cpk 6.0979 with `receipts/spc-2026-10-28T165319.json`;
trust_attest 12/12 rerun; 35/35 statement kinds; digest repin 6/6;
wasmex admit/tamper 2/2, WASI 28/28, fork @ 2223f894). Sole open gate:
**mutation court** (`bash scripts/mutation_court.sh` — no 2026-10-28
receipt in `receipts/`, re-checked at write time); its coordinator-solo
exit 0 flips WO-2 and the whole window to GO. Details and commands in
WORK-ORDERS.md and GO-NO-GO.md.
