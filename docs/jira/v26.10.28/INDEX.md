# v26.10.28 — window index

Milestone: VOC→CTQ translation and QFD House of Quality for the graphlaw
kernel surfaces (wasm execution, admission law, SQL-over-RDF, receipts,
graphlaw_ex Elixir embed), the purrdf fork-swap manufacturing receipt from
the 2026-10-28 session, and the GL28 Vision-2028 register. The coordinator
owns git.

Consolidation (2026-10-29): `GL28-BACKLOG.md` is the **single
authoritative GL28 register** — one status per GL28-n lives there;
`VISION-2028-EXECUTION.md` is a stub retained for the PRIOR-ART-FIRST lane
contract; the WORK-ORDERS.md GL28 addendum points at the register instead
of duplicating statuses. ERRC tick-2 doc pass (2026-10-29): the 0-byte
`GL28-14-MCP-PROJECTION-DESIGN.md` placeholder deleted (content lives at
register §GL28-14); `GL28-6-KILLSWITCH-DESIGN.md` and
`GL28-7-AUTHORITY-PORT-DESIGN.md` reduced to stubs — status header +
landed-file pointers + the register-cited court spec only.

## Files (complete enumeration, 16 files + provenance/)

| file | role | status |
| --- | --- | --- |
| [INDEX.md](INDEX.md) | Window index | this file |
| [VOC-CTQ-TURNSTILE.md](VOC-CTQ-TURNSTILE.md) | Stakeholder segmentation, CTQ matrix (1.1–5.3), Kano classification | COMPLETE — CTQs 1.1–5.3 defined; downstream orders cite these ids |
| [QFD-HOUSE-OF-QUALITY.md](QFD-HOUSE-OF-QUALITY.md) | House of Quality: CTQ rows × engineering characteristics, roof correlations, targets | COMPLETE as-of its 2026-10-28 write time — its per-CTQ "current standing" column predates the closing wave; WORK-ORDERS.md carries the current per-CTQ standing |
| [WORK-ORDERS.md](WORK-ORDERS.md) | Per-CTQ work orders WO-1..WO-7 with session-receipt standing; GL28-n statuses delegated to the register | 5 ALIVE + 2 PARTIAL: WO-1 (SPC Cpk 6.0979/14.4149, receipts `spc-2026-10-28T165319.json` + `…T171756.json`), WO-3 (trust_attest 12/12 rerun), WO-4 (35/35 kinds), WO-5 (digest repin 6/6 + emission-witnessed receipt surface), WO-6 (wasmex admit/tamper 2/2, WASI 28/28, fork @ 2223f894); WO-2 PARTIAL (mutation court unrun), WO-7 PARTIAL (ash_graphlaw parity court + full suite pending) |
| [RECEIPTS.md](RECEIPTS.md) | Manufacturing receipt: purrdf fork swap @ 2223f894 (38 fork-sourced packages), closing wave, GL28 wave receipts — **append-only, never patched** | COMPLETE — real command output embedded |
| [GO-NO-GO.md](GO-NO-GO.md) | Evidence table + verdict | POSTURE: **GO except WO-2** — WO-1/3/4/5/6 GO on this-session receipts; WO-2 sole NO-GO pending the coordinator's solo `bash scripts/mutation_court.sh` exit 0, which flips the whole window to GO |
| [TAG](TAG) | Milestone tag `v26.10.28` cut receipt | PUSHED — tag on `82814ac4` (HEAD at cut); release.yml run 40797877715 completed/success |
| [GL28-BACKLOG.md](GL28-BACKLOG.md) | **Single authoritative GL28 register**: per-ticket statuses, dependency DAG (folded from EXECUTION), court-specification table, wave-3 wire contracts, GL28-1/GL28-3 discoveries, GL28-14 | GL28-5 LANDED (`docs/reference/conformance_claim.md`); GL28-6/-7 DESIGN filed, impl lanes in flight; GL28-2/-4 SPEC; GL28-13 CENSUS; GL28-1 envelope spec filed; GL28-9 (= WO-2), GL28-10 (= WO-7), GL28-11, GL28-12, GL28-13 impl open; npm publish NOT executed (package exists at 26.10.14; fix = Trusted Publisher mapping + rerun A3) |
| [VISION-2028-C4.md](VISION-2028-C4.md) | 4-level C4 model + EU AI Act Art. 9–15 compliance matrix; unrealized edges carry GL28 refs | COMPLETE — facts block re-read from the tree 2026-10-28 |
| [VISION-2028-EXECUTION.md](VISION-2028-EXECUTION.md) | STUB — DAG + court specs folded into GL28-BACKLOG.md; retained for the PRIOR-ART-FIRST lane contract | STUB |
| [GL28-1-ENVELOPE-SPEC.md](GL28-1-ENVELOPE-SPEC.md) | GL28-1 OCEL 2.0 event-envelope spec for the beam4pm bridge (`BeamPM.Ocel` decode paths, telemetry→OCEL field mapping) | SPEC |
| [GL28-2-SMALT-WIRE-SPEC.md](GL28-2-SMALT-WIRE-SPEC.md) | GL28-2 Smalt wire protocol: admission envelope, commit gate, refusal taxonomy, courts, §8 discovered seam | SPEC (nothing built) |
| [GL28-4-EXPORTER-SPEC.md](GL28-4-EXPORTER-SPEC.md) | GL28-4 EU attestation exporter: attest.rs reuse, output shape, six-case court, policy-gate boundary | SPEC (design; no implementation on disk) |
| [GL28-6-KILLSWITCH-DESIGN.md](GL28-6-KILLSWITCH-DESIGN.md) | STUB — LANDED @ 4d2ffd0; status header + landed-file pointers (`graphlaw_ex/lib/graphlaw_ex/kill_switch.ex`, `test/graphlaw_ex/kill_switch_test.exs`) + retained court spec §5; design narrative deleted as historical | LANDED @ 4d2ffd0 (stub) |
| [GL28-7-AUTHORITY-PORT-DESIGN.md](GL28-7-AUTHORITY-PORT-DESIGN.md) | STUB — LANDED @ 4d2ffd0; status header + landed-file pointers (`graphlaw_ex/lib/graphlaw_ex/authority.ex`, `test/a2a_authority_ceiling_court_test.exs`) + retained court spec §4; design narrative deleted as historical | LANDED @ 4d2ffd0 (stub) |
| [GL28-13-SLIMMING-CENSUS.md](GL28-13-SLIMMING-CENSUS.md) | WASM slimming census: 74,097,211 B artifact, section budget, DuckDB-archive attribution, ranked levers | CENSUS — levers 1+2 (ICU −3 MB, MinSizeRel wasm32 −10–18 MB) restore the 64 MiB gate; implementation not started |
| GL28-14-MCP-PROJECTION-DESIGN.md | DELETED 2026-10-29 — was a 0-byte placeholder; content lives in the register at [GL28-BACKLOG.md](GL28-BACKLOG.md) §GL28-14 | see register §GL28-14 |
| `provenance/` | Fork-swap provenance keepers: `Cargo.lock.pre-swap` (130,856 B), `purrdf-fork-44f4979.tar.gz` (58,773,100 B, codeload bytes), README.md with replay recipe (codeload sha256 gate vs. `git archive` caveat) + deletion proposal | ARCHIVED pre-swap state; tarball deletion is a coordinator-owned transition (PROPOSED) |

## Standing at this index pass (2026-10-29 UTC, re-read from the register
files and `receipts/` on disk)

- Tag `v26.10.28` is **PUSHED** at `82814ac4f398274e64aeb47d2448d04f89cfefcd`
  (TAG receipt); release.yml run 40797877715 completed/success.
- GO-NO-GO posture: **GO except WO-2**. Sole open gate: the mutation court
  (`bash scripts/mutation_court.sh`) — **no mutation-court receipt exists
  in `receipts/`** (re-checked at this pass); its coordinator-solo exit 0
  flips WO-2 and the whole window to GO.
- SPC: **two** independent receipts, both re-read from the JSON files on
  disk: `receipts/spc-2026-10-28T165319.json` (Cpk 6.0979, p99 125 ns) and
  `receipts/spc-2026-10-28T171756.json` (Cpk 14.4149, p99 118 ns), gate
  min 1.33, tail_fraction 0.0.
- npm: **publish NOT executed** — no publish receipt exists; the package
  `@seanchatmangpt/graphlaw` already exists on the registry at **26.10.14**
  (`receipts/2026-10-24-npm-attempt-ledger.md`), so no manual first publish
  is needed; the fix is the Trusted Publisher mapping (operator console) +
  rerun A3. Hex publish decision open (GL28-12).
- GL28 register standing: see the GL28-BACKLOG.md row above; the register
  file is authoritative over any status restated elsewhere in this window.

Details and commands in [WORK-ORDERS.md](WORK-ORDERS.md),
[GO-NO-GO.md](GO-NO-GO.md), and [GL28-BACKLOG.md](GL28-BACKLOG.md).
