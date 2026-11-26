# v26.10.28 — window index

Milestone: VOC→CTQ translation and QFD House of Quality for the graphlaw
kernel surfaces (wasm execution, admission law, SQL-over-RDF, receipts,
graphlaw_ex Elixir embed), the purrdf fork-swap manufacturing receipt from
the 2026-10-28 session, and the GL28 Vision-2028 register. The coordinator
owns git.

Consolidation (2026-11-09 docs-triage, standing re-read from `git log`,
`receipts/` and the tree): `GL28-BACKLOG.md` is the **single authoritative
GL28 register** — one status per GL28-n lives there. The files
`VISION-2028-EXECUTION.md` (PRIOR-ART-FIRST lane contract),
`GL28-6-KILLSWITCH-DESIGN.md` and `GL28-7-AUTHORITY-PORT-DESIGN.md` (court
specs), `GL28-2-IMPLEMENTATION.md` (lane table, now with real status) and
`docs/jira/v26.10.26/PARKED.md` (still-open parked items) were folded into the
register and are removed from the window; the earlier 0-byte
`GL28-14-MCP-PROJECTION-DESIGN.md` placeholder is likewise gone (content at
register §GL28-14).

## Files (complete enumeration, 13 files + provenance/ with 2 files)

| file | role | status |
| --- | --- | --- |
| [INDEX.md](INDEX.md) | Window index | this file |
| [VOC-CTQ-TURNSTILE.md](VOC-CTQ-TURNSTILE.md) | Stakeholder segmentation, CTQ matrix (1.1–5.3), Kano classification | COMPLETE — CTQs 1.1–5.3 defined; carries no standing (pointer added 2026-11-09) |
| [QFD-HOUSE-OF-QUALITY.md](QFD-HOUSE-OF-QUALITY.md) | House of Quality: CTQ rows × engineering characteristics, roof correlations, targets | COMPLETE — "current standing" column refreshed 2026-11-09 (11 of 12 CTQs ALIVE, CTQ 1.2 PARTIAL) |
| [WORK-ORDERS.md](WORK-ORDERS.md) | Per-CTQ work orders WO-1..WO-7 with session-receipt standing; GL28-n statuses delegated to the register | **6 ALIVE + 1 PARTIAL**: WO-1,2,3,4,5,6 ALIVE (WO-2 flipped on `receipts/2026-10-29-mutation-court-v261028.md`, 66fc0e2: 19 caught / 9 unviable / 0 missed); WO-7 PARTIAL (ash_graphlaw parity court + full suite not witnessed) |
| [RECEIPTS.md](RECEIPTS.md) | Manufacturing receipt: purrdf fork swap @ 2223f894, closing wave, GL28 wave receipts, close-out, GL28-1/GL28-13 receipts, docs-triage receipt — **append-only, never patched** (cited by `.github/workflows/ci.yml` and `js.yml`) | COMPLETE — real command output embedded |
| [GO-NO-GO.md](GO-NO-GO.md) | Evidence table + verdict | POSTURE: **GO** — all six work orders met (c070392, f1321ca) |
| [TAG](TAG) | Milestone tag `v26.10.28` cut receipt (append-only note 2026-11-09) | PUSHED — annotated tag `1e72b4c` peeling to `82814ac`; GitHub release `v26.10.28` Latest; release.yml run 40797877715 success |
| [GL28-BACKLOG.md](GL28-BACKLOG.md) | **Single authoritative GL28 register**: statuses, PRIOR-ART-FIRST lane contract, dependency DAG, court-specification table (incl. GL28-6/-7 court specs), GL28-2 lane table + census, GL28-1/-3 discoveries, GL28-14, parked items | GL28-1 ALIVE (cd69757); GL28-5 LANDED; GL28-6/-7 LANDED (4d2ffd0); GL28-13 CLOSED (95cd3df); GL28-14 landed (14df7b9); GL28-9 CLOSED (= WO-2); GL28-2 PARTIAL; GL28-4 SPEC; GL28-3, GL28-8 open; GL28-10 PARTIAL (= WO-7); GL28-11, GL28-12 open; npm publish NOT executed |
| [VISION-2028-C4.md](VISION-2028-C4.md) | 4-level C4 model + EU AI Act Art. 9–15 compliance matrix; unrealized edges carry GL28 refs | COMPLETE — facts block re-read 2026-11-09 (wasm 62,909,537 B post-relink) |
| [GL28-1-ENVELOPE-SPEC.md](GL28-1-ENVELOPE-SPEC.md) | GL28-1 OCEL 2.0 event-envelope spec for the beam4pm bridge | ALIVE (design authority for the landed bridge) |
| [GL28-2-SMALT-WIRE-SPEC.md](GL28-2-SMALT-WIRE-SPEC.md) | GL28-2 Smalt wire protocol: envelope, commit gate, refusal taxonomy, courts, §8 seam | PARTIAL (smalt gate landed a33622d1; kernel receipt mint open) |
| [GL28-4-EXPORTER-SPEC.md](GL28-4-EXPORTER-SPEC.md) | GL28-4 EU attestation exporter: attest.rs reuse, output shape, six-case court, policy-gate boundary | SPEC (no implementation on disk) |
| [GL28-13-SLIMMING-CENSUS.md](GL28-13-SLIMMING-CENSUS.md) | WASM slimming census of the 74,097,211 B artifact (historical body) | CLOSED @ 95cd3df — relinked 62,909,537 B |
| `provenance/` | `Cargo.lock.pre-swap` (130,856 B) + README.md with the replay recipe for `purrdf-fork-44f4979.tar.gz` (codeload sha256 gate vs. `git archive` caveat) | tarball is NOT kept (untracked, removed in be4e2a5); replay-only |

## Standing at this index pass (2026-11-09, re-read from git/gh/receipts on disk)

- Tag `v26.10.28`: annotated tag `1e72b4c` -> commit
  `82814ac4f398274e64aeb47d2448d04f89cfefcd`; GitHub release `v26.10.28` is Latest.
- GO-NO-GO posture: **GO**. The mutation court receipt
  `receipts/2026-10-29-mutation-court-v261028.md` exists (28 enumerated: 19
  caught, 9 unviable, 0 missed).
- SPC: two receipts, `receipts/spc-2026-10-28T165319.json` (Cpk 6.0979) and
  `receipts/spc-2026-10-28T171756.json` (Cpk 14.4149), gate min 1.33.
- Artifact: `priv/graphlaw.wasm` 62,909,537 B (blake3 `c92caeb6…`) after the
  GL28-13 relink; size ceiling is the 128 MB policy.
- npm: **publish NOT executed** — no publish receipt exists; the package already
  exists at 26.10.14 (`receipts/2026-10-24-npm-attempt-ledger.md`); fix is the
  Trusted Publisher mapping + rerun A3. Hex decision open (GL28-12).
- GL28 register standing: the GL28-BACKLOG.md row above; the register is
  authoritative over any status restated elsewhere in this window.

Details in [WORK-ORDERS.md](WORK-ORDERS.md), [GO-NO-GO.md](GO-NO-GO.md) and
[GL28-BACKLOG.md](GL28-BACKLOG.md).
