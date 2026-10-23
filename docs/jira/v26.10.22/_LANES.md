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
