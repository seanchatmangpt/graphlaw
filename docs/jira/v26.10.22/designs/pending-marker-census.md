# Pending-Marker Census — v26.10.22 (Lane L69)

- **Filed:** 2026-10-22 (lane L69, read-only census)
- **Sweep command:** `grep -rn 'PENDING\|TBD\|PENDING-LANE\|skeleton\|NOT-MET\|UNVERIFIED\|BLOCKED' docs/jira/v26.10.22/` — exit 0, **126 hit lines** (raw capture `/tmp/l69-census-raw.txt`).
- **Scope note:** the pattern matched `BLOCKED-BY` dependency vocabulary, `skeleton` as
  design terminology, and `UNVERIFIED`/`NOT-MET` as deliberate standing vocabulary.
  Only a minority of hits are genuine open work items. Classification per the four
  classes; every LIVE-BLOCKED item names its unblock condition.

## Counts

| Class | Items | Meaning |
|---|---|---|
| RESOLVED-SINCE | 7 | Marker is stale; the awaited thing landed (evidence cited). |
| LIVE-BLOCKED | 10 | Genuinely waiting; named owner/decision + unblock condition. |
| HONEST-STATE | 8 families (~70 hits) | NOT-MET / UNVERIFIED / skeleton as deliberate verdict vocabulary — keep. |
| ABANDONED | 0 | No hit lacked an owner and a path forward. |

## (a) RESOLVED-SINCE (stale markers, landing evidence)

1. **Cpk "PENDING-LANE (lane 5 / SPC lane)"** — `DFLSS-DOSSIER.md:34,49,137`,
   `:92` ("SPC court PENDING-LANE"). Landed: `scripts/spc_court.sh` +
   `receipts/spc-2026-10-22.json` / `receipts/spc-2026-10-22-inject.json`
   (RECEIPTS.md:248-249; verdict FAIL-both-legs adjudicated at
   CHARTER-DFLSS.md:93 "Cpk 1.44 NOT-MET — NOT-CAPABLE as measured"). The
   PENDING-LANE marker is stale: the SPC lane output exists, and its verdict is
   an honest FAIL, not a wait.
2. **OCAP row "wasm boundary … PENDING-LANE"** — `DFLSS-DOSSIER.md:161`. Stale:
   SQLRDF-08 wasm boundary courts landed (RECEIPTS.md:22 boundary build +
   code-path courts PASS; INDEX.md:43 phase rows). Verdict vocabulary
   (PARTIAL/UNMET) has replaced the placeholder.
3. Law admission / receipt signing / evidentiary ring / eval-cache OCAP rows
   **"PENDING-LANE"** — `DFLSS-DOSSIER.md:157-160`. Stale as *lane-wait*
   markers: each row's court counts are already embedded in the same table
   (dialect 25/0, trust_attest, ring 6/0, cache 7/0) and RECEIPTS.md §"Current
   state" (:30) records the resolution record. Remaining work is adjudication
   text, not a missing lane.
4. **PRD skeleton superseded** — `PRD-P28-SIL.md:4` "supersedes the earlier
   skeleton". RESOLVED-SINCE by its own text: the verbatim PRD is filed.
5. **Phase 2 ring/ledger skeleton** — `SQLRDF-18.md:55` "skeleton
   IMPLEMENTED … courts green" and `designs/RECEIPTS.md:25` "LANDED
   (skeleton)". Landed: `src/evidence_ring.rs`, `src/query_ledger.rs`,
   `tests/evidentiary_ring_court.rs` green.
6. **E3 conformance rung skeleton** — `designs/RECEIPTS.md:25,77` "rung
   skeleton … LANDED (4/5 green, 1 live flip failure — see §4)". Landed with a
   live defect record at `designs/RECEIPTS.md:147-154` — the marker is no
   longer a wait; it is a logged defect with an owner (E3 lane).
7. **L27 mutation-court run-mode demo BLOCKED** — RECEIPTS.md:277-281, blocked
   on `missing object_count in impl RdfAstGraphReader` aborting the sweep at
   `tests/test_cold_chain_ocel.rs:61`. The blocker's subject has landed:
   `fn object_count` now exists in the reader trait at `src/sql_ast.rs:1150`
   (trait surface) — the abort-class blocker appears fixed. Not yet a full
   pass: the sweep re-run has not been receipted. **Unblock-verify: re-run
   `bash scripts/mutation_court.sh`; it must reach mutant testing (exit
   not-4).** Until that re-run, treat as
   RESOLVED-SINCE pending a re-run receipt, downgrade to LIVE-BLOCKED if the
   sweep still aborts.

## (b) LIVE-BLOCKED (named unblock conditions)

| # | Item | Where | Blocking | Unblock condition |
|---|---|---|---|---|
| 1 | **v26.10.22 integration commit** — SQLRDF-16/17/18/19/20, THESIS.md:119, designs/RECEIPTS.md:41,106,168, E2-shapes-reader.md:3, DFLSS-DOSSIER.md:82-86 (E1/Phase1 chains) | tickets as cited | Coordinator git: working tree still dirty (**111 modified/untracked paths**, `git status --porcelain | wc -l`, HEAD `615dbfa`) — the integration commit has not been made | Coordinator performs the v26.10.22 integration commit; after it lands, the five tickets' chains unblock in order (SQLRDF-19 → 20; E1→A1→E2→E3) |
| 2 | **OCAP control-plan rows (lane 4)** | CHARTER-DFLSS.md:100,125; DFLSS-DOSSIER.md:62,150,157 | Lane 4 output not filed; charter checklist items unchecked | Lane-4 file transcribes the OCAP rows verbatim; charter checklist `CHARTER-DFLSS.md:125` checks |
| 3 | **Verbatim operator charter text** | CHARTER-DFLSS.md:3,11,111 | Operator holds the verbatim text; "still absent" per :111 | Operator files the charter text in docs/jira/v26.10.22/; :111 checkbox flips |
| 4 | **THESIS.md verbatim operator prose** | THESIS.md:12-19 banner; INDEX.md:41 "verbatim body pending"; designs/RECEIPTS.md:14,48-50,175 | Coordinator holds the prose ("dispatch did not carry the prose") | Coordinator/operator pastes the verbatim thesis body; PENDING banner removed per designs/RECEIPTS.md:175 |
| 5 | **SQLRDF-15 frozen-pin regeneration** | SQLRDF-15.md:73; RECEIPTS.md:35,106 | Frozen-duckdb pinned sources absent on disk | Restore the frozen-pin checkout → regenerate the tpch hand-patch retiree (never hand-patch); adjudicate sf=0/sf=0.01 for the pin |
| 6 | **L3 gws4_a/gws4_c IPC-injection courts (wasm link)** | RECEIPTS.md:267-275 (harness `tests/grs4…`— `tests/gws4_ipc_injection_court.rs:237,305`) | Compiled wasm artifact link unavailable in this tree state | Restore artifact link (cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm), rerun `cargo test --test gws4_ipc_injection_court` — green or typed diagnosis |
| 7 | **L32 TC_HOT_FULL full-corpus run** | RECEIPTS.md:283-287; gates `benches/tc_hot_01.rs:154`; CTQ rows designs/ctq-traceability.md:11,20 | 10M-iteration run never executed; only SMOKE baseline ran | `TC_HOT_FULL=1 cargo bench --bench tc_hot_01` executes 10M iterations and gates, or typed failure; also resolves P28-SIL-specs.md:34 "UNVERIFIED until the TC-HOT bench lands" (bench landed, adjudication awaits this run) |
| 8 | **P28-SIL-specs TC-HOT adjudication** | designs/P28-SIL-specs.md:34 | Same run as item 7 | Same as item 7 |
| 8b | **L27 re-run receipt** | RECEIPTS.md:277-281 | Sweep re-run not yet receipted (see RESOLVED-SINCE #7) | `bash scripts/mutation_court.sh` re-run reaching mutant testing |
| 9 | **SQLRDF-19 → SQLRDF-20 chain order** | SQLRDF-20.md:6, RECEIPTS.md:152-153 | SQLRDF-19's CI cells gate SQLRDF-20's court surface | Subsumed by item 1 (integration commit) + SQLRDF-19 cells landing first |

## (c) HONEST-STATE (deliberate verdict vocabulary — keep, not defects)

- **CHARTER-DFLSS.md** NOT-MET verdicts (:12,16,22,93,100,113-114): adjudicated
  findings, not waits (refusal latency ungated, throughput NOT-MET at claimed
  scale, Cpk NOT-MET/NOT-CAPABLE).
- **mega/tc-mega-fibo-01/COURT-SPEC.md** UNVERIFIED matrix (:39-65, :95-100,
  :138): M1..M9 status columns are the court's own status vocabulary with a
  defined movement path ("convert UNVERIFIED → WITNESSED or → REFUTED",
  :100).
- **mega/tc-mega-fibo-01/PILLARS-AUDIT.md** U1-U6 ledger (:165-174, :189-190):
  "UNVERIFIED stays UNVERIFIED" until commit+command+exit-0 — the audit's own
  law.
- **designs/stale-claims-audit.md** (:224,229,267): UNVERIFIED class is the
  audit's typed class, not a wait.
- **SQLRDF-17.md:28,45,47** and DFLSS-DOSSIER/DISSERTATION "skeleton"
  (:28 dialect sniff "Sparql skeleton", DISSERTATION.md:119 "Bare-SQL
  skeletons"): design terminology for document structure classes, not pending
  markers.
- **SQLRDF-18.md:41-44,54-56** BLOCKED-BY cells in acceptance tables: same
  integration-commit dependency as LIVE-BLOCKED item 1 (counted there, not
  duplicated).
- **RECEIPTS.md:43** (`Σ_PQC` UNSUPPORTED, BLOCKED/UNSUPPORTED summary),
  :106: typed standing vocabulary.
- **PILLARS-AUDIT.md:8,54,106,139** NOT-MET/EXISTS-NOT-MET rows: adjudicated
  audit rows with citations.

## (d) ABANDONED

None. Every marker has either landed behind it (RESOLVED-SINCE), a named
owner/decision (LIVE-BLOCKED), or is deliberate verdict vocabulary
(HONEST-STATE). Closest call: the DFLSS-DOSSIER.md:157-161 PENDING-LANE
placeholder rows — their owning lanes' outputs have landed elsewhere, so the
dossier placeholder is stale rather than abandoned; recommend the coordinator
flip those five rows to cite the landed receipts at integration-commit
doc-sync time.

## Wrap checklist verdict

- LIVE-BLOCKED items: 10 (items 1-9 with 8/8b split). Each names an unblock
  condition. Dominant blocker is a single gate: the **v26.10.22 integration
  commit** (item 1), which unblocks SQLRDF-16..20 chains; second is
  **TC_HOT_FULL run** (items 7/8); remaining are lane-file and operator-text
  filings plus two re-run/build unblocks (L3 wasm link, L27 sweep re-run).
- All grep receipts embedded above (exit 0, 126 hits, raw at
  `/tmp/l69-census-raw.txt`).
