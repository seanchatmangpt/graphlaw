# Lane Leases — Annealing Fleet, 2026-10-22

One canonical checkout. Disjoint file leases per wave; coordinator owns git and
Cargo.toml. `mutation_court.sh` runs solo after all lanes return. Append-only
history: new waves are added below, prior wave tables are never rewritten.

---

## Wave 1 — 16-lane audit/verify/repair/doc (2026-10-22, ~11:xx–13:10 PDT)

Dispatch table (unchanged from the original lease):

| Lane | Agent | Lease (disjoint) | Mode |
|---|---|---|---|
| A1 | Explore | src/law.rs (read-only) | AUDIT |
| A2 | Explore | src/sql_ast.rs, docs/refusals.md (read-only) | AUDIT |
| A3 | Explore | src/attest.rs, receipts/ (read-only) | AUDIT |
| A4 | Explore | src/duckdb_eval.rs (read-only) | AUDIT |
| V1 | court-runner | none (compile-only); CARGO_TARGET_DIR=target-lane1 | VERIFY chicago_validation |
| V2 | court-runner | none; CARGO_TARGET_DIR=target-lane2 | VERIFY sql_rdf_dialect |
| V3 | court-runner | none; CARGO_TARGET_DIR=target-lane3 | VERIFY duckdb_statement_kinds |
| V4 | court-runner | none; CARGO_TARGET_DIR=target-lane4 | VERIFY trust_attest |
| R1 | sql-compiler | src/ (sleep-as-coordination sweep only) | REPAIR |
| R2 | wasm-op | wasm/ | REPAIR |
| R3 | attest-keeper | receipts/ | REPAIR |
| R4 | ontology-author | ontologies/, ggen/sql-dialect/ | REPAIR |
| D1 | milestone-doc | docs/refusals.md | DOC |
| D2 | milestone-doc | docs/jira/v26.10.22/RECEIPTS.md | DOC |
| D3 | milestone-doc | docs/jira/v26.10.22/INDEX.md | DOC |
| D4 | milestone-doc | docs/jira/v26.10.22/designs/ | DOC |
| M1 | coordinator | (solo, post-wave) scripts/mutation_court.sh | MUTATION |

Standing (re-witnessed by the lane-docs re-execution pass, RECEIPTS.md
"Verdicts" section):

- V1 chicago_validation 9/0 PASS · V2 sql_rdf_dialect 33/0 PASS (supersedes
  25/0 rows) · V3 duckdb_statement_kinds 6/0 PASS (the `.gram` environmental
  exception CLEARED) · V4 trust_attest 10/0 PASS.
- A/R/D lanes: PASS — landed doctrine, refusal registers, index sync, designs/
  (10 files). Registry pin drift surfaced as RED (`registry_artifacts` 17/5,
  pin-regen class, not a hand patch) and `gws4_ipc_injection_court` 1/2 RED
  (falsifier: rebuild wasm archive with WASI_SDK_PATH — unmet on this host).
- M1 mutation_court: not re-witnessed this session (as-of-date).

## Wave W — 20-lane successor waves (2026-10-22, ~17:0x–21:4x PDT)

Consolidated L-lane/M-lane/T-lane successor waves recorded in RECEIPTS.md
(append-only; per-lane details and receipts live there, sections
"Successor wave 2 — hardening + setup" through "Wave-6 switchover sync").

- Wave 2 (hardening + setup, lane L53): PASS — INDEX closures
  (SQLRDF-03/09 → 25/0, later superseded), hardening landings; two loud
  UNFILED markers later flipped by waves 3/4.
- Wave 3 (DX + drift + pre-writes + cross-repo, lane L100): PASS with typed
  BLOCKED entries — wasm4pm oracle leg was disk-BLOCKED (11.3 GiB free),
  later witnessed 4/0 by lane V-L17 at 66 GiB free; registry/shapes leg left
  loud UNFILED lane reports, closed by wave 4.
- Wave 4 (link-green + truth pass, lane L117): PASS — wasm link GREEN
  (`docs/WASM.md` pin currency), L106 wasm4pm oracle re-witnessed 4/0,
  L108 CHAIN finding + coordinator re-baseline verified green, L107 js
  receipt with browser gap noted, L112 js limits fix on disk, L103
  advisories 8→3, L104 ARCHITECTURE.md landed; L113–L116 remain loud UNFILED.
- Wave M (M-lane sync, lane M8): PASS — all M receipts FILED in `designs/`;
  M4 wasm-boundary receipt flipped OUT→FILED by lane L149 (size ceiling PASS
  at 45,162,113 B; fuzz + memory-wall typed BLOCKED on the EH wall;
  wasi-sdk-34 personality mismatch, staged emsdk is the only green path).
- Wave 6 (switchover sync, lane T5): PASS — S8/S12 canary
  (`designs/commit-receipt-provenance-gap.md`, lane L176), surface-six entry
  patch applied (lane P7), M-wave verdicts FILED
  (`designs/m-wave-verdicts.md`), D2 embedded-output audit appended (see
  below).
- Lane D2 receipt audit (late PDT): PASS as audit — re-executed gates on the
  final tree: sql_rdf_dialect 33/0, chicago_validation 9/0, trust_attest
  10/0, sql_rdf_conformance 7/0, duckdb_statement_kinds 6/0,
  query_ledger_court 9/0, query_cache_court 8/0, ring court 10/0 featured,
  wasm pin `332ea218…` byte-exact, SPC receipts present. RED on current
  tree: `registry_artifacts` 17/5 (pin drift), `gws4_ipc_injection_court`
  1/2. CHAIN `--check` exits 1 (TAMPER-DETECTED on
  `receipts/sa2a/factory-b-v26.9.29-executable-gates.md`; chain head
  `2c316b02…` → `e4a7597b…`). Prose-only counts marked as-of-date, not
  re-witnessed.

## Wave Z — receipt/verification wave (2026-10-22, late PDT)

Z-wave lanes ran verification/receipt consolidation against the same tree
(the D2 audit section above is the Z-wave landing surface in RECEIPTS.md).
Per-lane standing is recorded in RECEIPTS.md, not duplicated here.

---

## In flight (as of this rewrite, lane Z16)

- Laundering-corpus case e4 (`tests/sql_rdf_laundering_corpus.rs`)
  adjudication; conformance anti-vacuity repair alongside it;
  `iri_object_count` trait sweep on external `RdfAstGraphReader` impls
  (same surface as the mutation-court abort).
- `gws4_ipc_injection_court` 1/2 — falsifier unmet on this host (wasm
  archive rebuild with WASI_SDK_PATH); typed BLOCKED, not a silent skip.
- L113–L116 lane reports: loud UNFILED.

## Coordinator decisions parked (not lane work)

- **Cargo.toml batch** (coordinator-only manifest): tc_hot_02 bench restore,
  duplicate example targets, workspace-member profile hoist, version bump.
- **CHAIN re-baseline** — `receipt_chain_sync.sh --check` TAMPER-DETECTED on
  `receipts/sa2a/factory-b-v26.9.29-executable-gates.md` (adjudicated
  NO-OUTPUT / "authored, not executed" by L31-wave3); coordinator decides
  re-baseline vs revert.
- **C15 operator gate** — pending coordinator admission.

---

## Standing rules (unchanged)

- No lane edits Cargo.toml. Lane build roots (`CARGO_TARGET_DIR=target-lane*`,
  `target-lane<N>`) are leases — deleted at integration.
- No `git stash` in the shared checkout; snapshot via
  `git show HEAD:path > /tmp/...`.
- Compile-freeze SLA ~10 min; minimal unblock fixes disclosed in both lanes'
  reports.
- Conflict rules: A2/D1 share docs/refusals.md — A2 read-only, D1 writer.
- Archive law (appended 2026-10-23, EH-closure wave): any future `libduckdb.a`
  rebuild MUST carry `-mllvm -wasm-use-legacy-eh=0` (with `-fwasm-exceptions`);
  wasmtime 48 is the only hosting runtime for the New-EH artifact; wasmi
  courts stay pinned to the legacy artifact class.

## Coordinator discipline (appended 2026-10-23, lane M16)

Witnessed defect class (2026-10-22/23 waves): the coordinator killed lane Z22
mid-edit, leaving the shared compile broken for ~10 minutes until the SLA
resolved it; four lanes separately reported false-green exits from pipe tails.

1. **No TaskStop on a lane holding an uncommitted shared edit.** Before killing
   a lane with in-flight edits to shared `src/` (or any shared surface), either
   (a) message the lane to revert its edit and confirm the revert landed, or
   (b) wait out the compile-freeze SLA resolution. A mid-edit kill converts one
   lane's in-flight state into a whole-fleet compile freeze — the SLA is a
   repair budget, not a license.
2. **Post-kill ground truth is the coordinator's first act.** Immediately after
   any lane kill: run `cargo check` on the shared tree and `git diff` the
   killed lanes' lease paths. Do not redispatch or re-lease until the tree is
   witnessed compiling and the killed lanes' partial edits are inventoried
   (revert, adopt, or disclose per the compile-freeze SLA).
3. **PIPESTATUS law: exit codes from pipes are not receipts.** `cmd1 | cmd2 |
   tail` reports the LAST command's exit status — a lane reading `$?` after a
   pipe has reported PASS off a `tail` that always exits 0 (witnessed 4x this
   wave). Under zsh use the `pipefail` option or check the pipe-status array
   (`$pipestatus`; bash: `PIPESTATUS`); the receipt must embed the exit code of
   the gate command itself, not of a formatter/filter downstream of it.

---

## Waves K/M/N/P + operator directives (appended 2026-10-23, lane P3/DOC)

- **Wave K — adversarial null-hypothesis run** (2026-10-22/23): per-lane
  verdicts recorded in RECEIPTS.md ("Wave K — adversarial null-hypothesis run",
  line 900).
- **Wave M — mistake-proofing guards landed**:
  - M00 archive: superseded docs moved to `docs/archive/`.
  - M01 anchor gate + M19: doc-staleness findings 31 → 0.
  - M02 witness gate: +178 unreceipted-claim findings surfaced.
  - M03 count gate: +29 stale-count findings surfaced.
  - M04 oracle gate; M06 64-byte receipt tamper court; M07/N7 real readbacks;
    M08 in flight (not landed).
  - M09/N1 machine-shaped receipts; M10 receipt vector pin; M11 exit-capture.
  - M12 register completeness; M13 js regen 198 passed / 0 failed.
  - M14 flag court; M15 corpus depth 17; M16 kill discipline (see coordinator
    discipline above); M17 bins; M18 fleet_cleanup reclaimed 68.69 GiB.
- **Wave PM — pm4pytest migration**: PM1 cut; PM2 e2e WORKS; PM3 15/0
  black-box; PM4–PM8 in flight.
- **Operator directives honored**: safety snapshot dab6a73 pushed; docs
  archived to `docs/archive/`; pm4pytest adopted as external oracle.

## Wave EH — DFLSS-GL EH-closure (~30 lanes, 2026-10-23)

Dispatch table (coordinator-dispatched lanes; per-lane receipts in
RECEIPTS.md "DFLSS-GL EH-closure wave", W1–W14 rows):

| Lane | Purpose | Mode |
|---|---|---|
| EH-01 | gate removal, src/lib.rs + src/abi.rs (W4) | REPAIR |
| EH-02 | registry/capabilities ungate (W4) | REPAIR |
| EH-03 | op-15 gate lift (W4) | REPAIR |
| EH-04 | MERGE classification court (W5, 12/0 + flip) | REPAIR |
| EH-05 | REFUSAL-05 compiler arm (W11, PENDING) | REPAIR |
| EH-06 | ggen WHERE shapes (W6) | REPAIR |
| EH-07 | ast_governance mirror (W6) | REPAIR |
| EH-08 | wasmtime host bin (W12, PENDING) | REPAIR |
| EH-09 | lifecycle EH court (W10, PENDING) | REPAIR |
| EH-10 | wired-op court flip (W7) | REPAIR |
| EH-11 | gws4 court | REPAIR |
| EH-12 | registry courts (W7, 7/0 + 14/0) | VERIFY |
| EH-13 | refusals register, REFUSAL-05 row (W7) | DOC |
| EH-14 | SQLRDF-21 + FIBO docs (W8) | DOC |
| EH-15 | WASM docs (W8) | DOC |
| EH-16 | ARCH/DIALECTS docs (W8) | DOC |
| EH-17 | pm4pytest survey (W13, PENDING) | RESEARCH |
| EH-18 | COPY/ATTACH audit | AUDIT |
| EH-19 | FIBO fixtures | REPAIR |
| EH-20 | abi_manifest note | DOC |
| EH-21 | size court | VERIFY |
| EH-22 | RECEIPTS append | DOC |
| EH-23 | registry courts 2 | VERIFY |
| EH-24 | dialect court fixtures | REPAIR |
| EH-25 | host-side court | REPAIR |
| EH-26 | pm4pytest driver (PENDING) | REPAIR |
| EH-27 | CI update | LANDED |
| EH-28 | pin-receipt draft | DOC |
| EH-29 | residual-claims sweep | AUDIT |
| EH-30 | xone shape repair (OPEN) | REPAIR |
| EH-31 | session mode (OPEN: duckdb_op persistent sessions) | REPAIR |
| EH-32 | REFUSAL-05 court fixtures (W11, PENDING) | REPAIR |

Standing: W1–W8 GREEN (RECEIPTS.md rows); CI landed; size court landed;
OPEN/PENDING rows: EH-05/32 (REFUSAL-05 arm + fixtures), EH-08 (wasmtime
host bin), EH-09 (lifecycle EH court), EH-13 open pending receipt, EH-17/26
(pm4pytest), EH-30 (xone shape repair), EH-31 (session mode). W14 mutation
+ SPC BLOCKED on clean tree/commit.

### Status refresh — 2026-10-23 (append-only; table above not rewritten)

GREEN since table was written (receipts in RECEIPTS.md unless noted):

- EH-20: abi_manifest note landed (2/0 court).
- WASM boundary: wasm contract 10/0 + wasm32 build check clean.
- EH-21: size court 4/0.
- Registry ascii fix landed.
- EH-32/05: REFUSAL-05 pin courts 36/0; conformance arm 8/0.
- Session ABI court 9/9; host-side court (EH-25) 8/8.
- EH-24: dialect fixtures 36/0 + chicago 12/0.
- Memory-wall port 8/8.
- graphlaw-run pre-flight exit 96 witnessed live.
- Docs: how-to, index, CHANGELOG/MIGRATION, OD-1 fill (EH-15/16/22/28).
- Pin-receipt draft witnessed: built digest 259cf5b0…, Tag×1.
- pack-smoke PASS after THESIS.md drop; npm re-band landed.

V-verification lanes all GREEN except items noted below.

Still OPEN:

- duckdb-open trap (`uninitialized element`) under diagnosis.
- EH-10: wired-op court flip still silent — no flip witnessed.
- Common-Host, fuzz, and abi ports not yet landed.
- Test census (compile-abort vs failure split) not yet landed.
