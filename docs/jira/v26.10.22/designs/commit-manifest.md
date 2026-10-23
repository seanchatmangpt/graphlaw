# Commit Manifest — pre-commit prep, tree 615dbfa (2026-10-22, refresh #3 / L173)

Census: `git status --porcelain=v1 | wc -l` = **276** entries as of the
L177 refresh (S15 measured 276 live; the L173 snapshot recorded 270 — the
delta is the 4 paths S15 found unassigned plus sibling-lane drift; re-run
at integration). Historical: 218 at the
L146 refresh #2; the switchover landed in the working tree since:
src/sql_ast/, src/registry/, src/abi/, src/dialect/, src/query_ledger/,
src/receipt_chain/, crates/graphlaw-eyeron/src/{parser,reasoner}/ are now
WIRED dirs under facade files). Snapshot: `/tmp/l173-status-now.txt`
(270 lines at L173; superseded — see census above, re-run at integration).
8 buckets, ordered by dependency. Refresh #3 re-buckets per the broken-tree
rule: **each facade + its wired dir travel in the SAME commit** — either half
alone is a compile break.

## Switchover state (verified on disk 2026-10-22)

| Facade | Wired dir | State |
|---|---|---|
| src/sql_ast.rs (2338 ln, was ~4.3k) | src/sql_ast/{ast,refusals,serializer}.rs | wired (`mod` + `pub use`; facade retains inline test mods) |
| src/dialect.rs (583 ln) | src/dialect/{refusal,sniff}.rs | wired |
| src/query_ledger.rs (29 ln) | src/query_ledger/{batch,storage}.rs | wired |
| src/receipt_chain.rs (1033 ln) | src/receipt_chain/{frame,record,store}.rs | wired (`pub mod frame/store`; `epoch` + `validator` inline in facade) |
| src/registry.rs (130 ln) | src/registry/{definitions,emit_json,emit_ttl}.rs | wired |
| src/abi.rs (341 ln) | src/abi/{lease.rs,ops/} | wired |
| eyeron parser.rs (342 ln, was ~2.2k) | crates/graphlaw-eyeron/src/parser/{cursor,grammar}.rs | wired (lib.rs `pub mod parser;`) |
| eyeron reasoner.rs (4248 ln) | crates/graphlaw-eyeron/src/reasoner/{builtins,limits,solve_context}.rs | wired (partial split; monolith remainder is noted, not a defect) |

No `*_split/` staging trees remain anywhere in src/ or crates/ (verified:
zero matches). The L146 "commit-or-exclude" decision on the unwired
pre-writes is vacuous — superseded by the switchover. SPLIT-NOTES.md
(untracked, repo root) travels with bucket 1 as the switchover record.

## Order and buckets

| # | Bucket | Draft | Files (approx) |
|---|--------|-------|----------------|
| 1 | eyeron switchover (parser/reasoner facades + wired dirs + record) | /tmp/l124-commits/01-pre-writes.txt | crates/graphlaw-eyeron/src/{parser.rs,parser/,reasoner.rs,reasoner/}, SPLIT-NOTES.md |
| 2 | sql surface + seal + refusals (sql_ast + dialect switchover) | /tmp/l124-commits/02-sql-surface.txt | src/{sql_ast.rs+sql_ast/, dialect.rs+dialect/, duckdb_eval, duckdb_op, plan, hooks, law, tier, lib, rules/mod}.rs, ontologies/{sql-ast.ttl, shapes/ast_governance.ttl}, ggen/sql-dialect/, docs/refusals.md (L172 receipt-surface 6-entry patch rides HERE), docs/reference/sql-rdf-rec.md, sql courts + examples + fixtures + benches, src/bin/dl_dualroute.rs (S15 fix: vector-gen provenance — adds `graphlaw_version` = CARGO_PKG_VERSION to emitted vector provenance; dl/vector-emission surface, nearest bucket 2; diff is 2 added lines, no shared-surface coupling) |
| 3 | ring/ledger hardening + chain (query_ledger + receipt_chain switchover) | /tmp/l124-commits/03-ring-ledger.txt | src/{query_ledger.rs+query_ledger/, receipt_chain.rs+receipt_chain/, evidence_ring, query_cache, attest}.rs, src/bin/graphlaw_{attest,verify,run}.rs, receipts/*, scripts/{receipt_chain_sync, spc_court}.sh, ring/ledger/cache/verify courts |
| 4 | mega setup + partition/staleness courts | /tmp/l124-commits/04-mega-courts.txt | docs/jira/v26.10.22/mega/, courts/code-partition/, scripts/{partition_court.sh, partition_court_selfcheck_g1.py, g2, doc_staleness_gate.sh}, smon bins + tests/smon_cli_court.rs, .gitignore (S15 fix: adds `/var/` ignore for the durable tc-hot bench corpus — courts/bench infra), deny.toml (S15 fix: license allow-list additions for purrdf 3.x migration path + bzip2/MulanPSL — gate config; NOTE it does NOT carry scripts/mutation_court.sh, see EXCLUDED-HOT-PATH below) |
| 5 | wasm link-green + js/npm + CI | /tmp/l124-commits/05-wasm-linkgreen.txt | wasm/ incl. wasm/ggen.toml (relative paths as of L160 — flag RESOLVED, see scan), Cargo.toml/lock, build.rs, js/npm/, tests/common/mod.rs, tests/fixtures/l125_wasm_driver.mjs, tests/wasm_duckdb_op_wired.rs, wasm/wasi/size/memory/fuzz/op/coldchain courts, scripts/{wasm_audit.sh, gen_wasm4pm_graphlaw_vectors.py, gen_w3c_manifest.sh}, tests/w3c_manifest.rs, vendor/purrdf-sparql-eval/UPSTREAM-DRAFT.md |
| 6 | docs truth pass (top-level + tickets) | /tmp/l124-commits/06-docs-truth.txt | README/CHANGELOG/MIGRATION/SECURITY/CONTRIBUTING/CLAUDE.md, docs/*.md incl. ARCHITECTURE.md + refusal-playbook.md, LICENSE.md add + LICENSE/LICENSE-P28-SIL.md deletes, docs/jira INDEX/RECEIPTS + tickets + designs/commit-manifest.md + RELEASE-NOTES-DRAFT |
| 7 | truth-pass + EH decision records (jira designs corpus) | /tmp/l124-commits/07-eh-truth-records.txt | docs/jira/v26.10.22/designs/* EXCEPT commit-manifest.md — all wave receipts, EH memo + advisory amendment, dual-artifact design, a1prime recipe, stale-wasi sweep, reconciliation/census/receipt designs |
| 8 | registry/ABI switchover + re-pin (generated) — OPERATOR-DECISION-PENDING | /tmp/l124-commits/08-registry-repin.txt | src/{registry.rs+registry/, abi.rs+abi/, abi_manifest}.rs, src/bin/graphlaw_registry.rs, registry/{ARTIFACTS.blake3, capability-registry.json, capability-registry.ttl}, ASSETS.blake3, contracts/abi-manifest.json, tests/{registry_dispatch, authority_manifest}.rs |

Bucket 8 remains **OPERATOR-DECISION-PENDING** (dual-artifact pin design,
bucket 7): if the operator adopts dual-artifact pinning, the pin surface
changes shape and the bucket must be re-derived, not committed as-is.

## Commit-sequence consequence (switchover ↔ registry re-pin)

The switchover commit(s) must precede or coincide with the registry re-pin
bucket. In this split the registry/ABI switchover (src/registry.rs +
src/registry/, src/abi.rs + src/abi/) RIDES bucket 8 itself, so the
coincidence is structural; the law reduces to **bucket 8 stays LAST** —
no earlier bucket may carry any of those four paths. The L172 receipt-surface
6-entry patch (docs/refusals.md) rides bucket 2, per the sql-surface
assignment.

## EXCLUDED-HOT-PATH (typed exclusion, not a bucket gap)

- `scripts/mutation_court.sh` (M) — **EXCLUDED-HOT-PATH**: this is the
  hot-path lane's DIRTY file (107-line change in the working tree, in
  flight at L177). It MUST NOT ride any commit in this sequence until the
  hot-path lane lands; committing it here would attribute another lane's
  in-flight work to this sequence. Coordinator re-includes it in a
  follow-up commit once the hot-path lane lands.

## Do NOT commit (exclude, coordinator decides .gitignore follow-up)

- `.claude/hooks/__pycache__/` — compiled bytecode; gitignore candidate.
- `.claude/settings.json` — local hooks/permission state; not repo policy.
- `.claude/hooks/{disk_guard,manifest_guard}.py` — local harness hooks; commit
  only if the coordinator intends them as repo policy (flagged, ambiguous).
  Note: `manifest_guard.py` embeds `/Users/sac/graphlaw` (line 4).
- `.claude/agents/{court-runner,sql-compiler,wasm-op}.md` — modified; flagged
  ambiguous: likely intentional agent-definition refresh, but confirm intent.

## Ambiguity flags

- `src/lib.rs` — touches the whole crate; assigned to bucket 2 (surface
  wiring). If it also wires ledger modules, coordinator may move it to 3.
  RESOLVED (L177 grep, S15 check-2 residual): the lib.rs diff contains NO
  mod/pub-mod additions — `grep -n '^pub mod \|^mod '` on the working file
  shows the registry/abi/receipt_chain/query_ledger mod lines (90, 93,
  103, 105, 110) are PRE-EXISTING at HEAD, not part of this diff (diff is
  doc comments + BACKEND_AUTHORITIES revision 2.0.0→2.0.2 only). So no
  earlier bucket pre-touches the registry/ABI surface; bucket 8 remains
  the first touch. lib.rs stays in bucket 2.
- `examples/fixtures/gws8_request.json` — fixture; travels with bucket 2
  (examples). Verify the consuming test is in the same commit to keep
  per-commit `cargo test` green.
- `tests/common/mod.rs` — shared harness; assigned to 5 since wasm courts
  consume it (now includes the panic-text update); if duckdb courts also
  consume it, coordinator may prefer bucket 3. Flagged.
- RESOLVED: `vendor/purrdf-sparql-eval/UPSTREAM-DRAFT.md` references
  `vendor/upstream/purrdf-sparql-eval.patch`; verified already tracked and
  unmodified (`git ls-files vendor/ | grep patch` → hit; not in status).
- `.claude/agents/*` — see exclusion list above.
- `docs/jira/v26.10.22/RELEASE-NOTES-DRAFT-26.10.21.md` — draft, committed in
  bucket 6 with DRAFT marker retained.
- `wasm/ggen.toml` — RESOLVED at L160/L173: all paths now relative to wasm/
  (source, imports, query, template lines verified relative on disk); no
  `/Users/sac` paths remain. Commits in bucket 5 unflagged.
- Designs split (buckets 6/7): only `designs/commit-manifest.md` stays in 6;
  all other designs are decision records and wave receipts, in bucket 7.

## Machine-specific path scan (added lines only; modified-diff `^+` lines plus untracked file contents)

Re-run 2026-10-22 (L173). Code/config class:

1. `tests/duckdb_statement_kinds.rs` (added line, `sql:repositoryPath
   "/Users/sac/.duckdb/extensions"` test-data literal) — STILL PRESENT,
   still flagged (carried from L124/L146). Data value, not a code constant;
   if the court depends on that literal path existing it is an
   env-dependent test; recommend coordinator confirm or tempath-ize
   (`/tmp/...`). Rides bucket 2.
2. `wasm/ggen.toml` — CLEARED (was flagged at L124/L146 lines 17/18/26/27):
   now fully relative (`../ontologies/graphlaw-wasm.ttl`,
   `../../ggen-marketplace/packs/...`), deterministic b3sum receipt in the
   header comment.
3. `vendor/purrdf-sparql-eval/UPSTREAM-DRAFT.md:4,96` — `/Users/sac/graphlaw`
   in lane/provenance prose of an upstream submission draft; benign
   (document identity), rides bucket 5.
4. `.claude/hooks/manifest_guard.py:4` — `/Users/sac/graphlaw`; excluded
   from commit anyway.

No absolute paths in `src/` code constants on added lines. Doc/receipt class
(benign by lane brief — receipts must name their subject): `designs/*` wave
receipts name `/Users/sac/graphlaw`, `/Users/sac/wasm4pm`,
`/Users/sac/frozen-duckdb`, `/Users/sac/emsdk`, `/Users/sac/rel4r/duckdb-src`,
`/Users/sac/smalt`, `/Users/sac/.frozen-duckdb/cache/...`,
`/Users/sac/Library/Caches/ms-playwright/...`, `/Users/sac/.cache/tmp` —
subject/provenance identities in receipts, out of scope for tempath-izing.

Pre-existing absolute paths (`src/bin/dl_dualroute.rs`,
`tests/insa_law_order.rs`, etc.) are NOT part of this diff's added lines and
out of scope.

## Receipts

- `git status --porcelain=v1 | wc -l` → **276** (S15 live count, confirmed
  L177; was 218 at L146, 270 at L173; dominated by the 8 wired switchover
  dirs + new designs/receipts — re-run at integration)
- L177 assignment re-run (S15 fix receipt): programmatic assignment of all
  status paths against bucket sets → **0 unassigned** after placing
  .gitignore + deny.toml (bucket 4) and src/bin/dl_dualroute.rs (bucket 2),
  with scripts/mutation_court.sh typed-EXCLUDED-HOT-PATH; **0
  double-assigned**; draft subjects 63/63/70/54/51/61/61/70 — all ≤72
  (02 shortened twice: first pass measured 76, trimmed again). Exit 0.
  Bucket 7 draft body now carries the designs/*-except-
  commit-manifest.md form, not a partial enumeration.
- `git diff --stat | tail -1` → "142 files changed, 6562 insertions(+),
  15024 deletions(-)" (as-of L173; re-run at integration)
- Facade diff shrink (L173): sql_ast −2018/…, registry −1963, receipt_chain
  −1144, query_ledger −1004, abi −867, dialect −567, eyeron parser −1908,
  reasoner −4341 (net 996 insertions / 12816 deletions across the 8 facades)
- Census snapshot: `/tmp/l173-status-now.txt` (270 lines)
- Compile gate on the switchover tree: `cargo check` → "Finished `dev`
  profile ... in 48.28s", exit 0 (includes graphlaw-eyeron — all 8 wired
  facade/dir pairs compile as-is)
- `/tmp/l124-commits/` drafts: 8 files, refreshed in place (01 repurposed to
  eyeron switchover, 02/03/08 updated for facade+dir pairs, 07 body rewritten
  to the designs/*-except-commit-manifest.md form at L177; 04–06 unchanged
  from L146), verified below.

---

## Wave-2 integration addendum (2026-10-22/23) — delta since snapshot dab6a73

Scope: `git diff --stat dab6a73 | tail -1` → **"61 files changed, 2181
insertions(+), 256 deletions(-)"** (measured 2026-10-22, this lane). These
are the wave-2 working-tree changes grouped into the same 8-bucket order
above; bucket numbers below reference the table at L34.

### (1) Behavior — bucket 2/8 rides (sql surface + registry/ABI)

- `src/law.rs` (+15): `Step::TestCapabilityProbe(&'static str)` — doc-hidden
  test-only probe step with authority callsite coverage, `Ceiling::Observe`,
  so courts can drive `LawState::transition` with a capability absent from
  `BACKEND_AUTHORITIES` and assert typed `REFUSAL_UNKNOWN_STEP_CAPABILITY`
  through the public transition path. Rides bucket 2 (law surface).
- `src/sql_ast.rs` (+36) + `src/sql_ast/refusals.rs` (+9):
  `expr_references_target()` + MERGE scope-integrity gate (lane K5b,
  REFUSAL-03 class): an ON condition referencing no target-qualified column
  is refused typed (`ScopeIntegrity` /
  `refusal_arm::MERGE_ON_NO_TARGET_COLUMN`) before any SQL is emitted.
  Rides bucket 2.
- `src/abi.rs` (+4), `src/lib.rs` (+13): `#[cfg(not(target_arch = "wasm32"))]`
  gates on the frozen-duckdb staticlib surface (`duckdb_eval`,
  `query_cache`, `sql builtins`, ABI op `duckdb`) — host-only because
  frozen-duckdb's wasm staticlib emits the exception-handling proposal,
  rejected by wasmi 2.0.0; SQL-in-RDF admission stays on the wasm dialect
  path. lib.rs diff is cfg-gates + doc comments + BACKEND_AUTHORITIES
  2.0.0→2.0.2 (no mod wiring in the diff — pre-existing at HEAD per the
  L177 RESOLVED note above). Ride bucket 8 (registry/ABI switchover) with
  `src/abi.rs`; the lib.rs half stays bucket 2 as resolved at L177.
- Bins: `src/bin/{graphlaw_run,dl_dualroute,sa2a_portability_verify,
  sa2a_run14_verify}.rs` — typed error exits + debug-harness removal
  (+52/+17/+10/+13). Ride bucket 2 with the dl/vector-emission surface
  (dl_dualroute already assigned there at L41).

### (2) Courts (bucket 2/3 riders, `tests/`)

New files (untracked): `tests/k5_hostile_sqlrdf.rs`,
`tests/k6_hostile_lease.rs`, `tests/ocel_export_court.rs`,
`tests/dl_dualroute_vector_pin.rs`. Strengthened:
`tests/trust_attest.rs` (+72 — K2 sig-tamper guard, all 64 Ed25519
signature bytes proven load-bearing, `assert_eq!(refused, 64)`);
`tests/chicago_validation.rs` (+164 — exact-rowcount + exact-value readbacks,
group/having/order-by reads back its own rows); `tests/law_state.rs` (+49 —
end-to-end probe → typed refusal through `LawState::transition`);
`tests/gws4_ipc_injection_court.rs` (+82), `tests/sql_rdf_dialect.rs`
(+96), `tests/sql_rdf_conformance.rs` (+157), `tests/insa_law_order.rs`,
`tests/refusal_crosswalk_differential.rs`, `tests/no_mock_markers.rs`
(+335 — anti-mock gate), `tests/duckdb_statement_kinds.rs`,
`tests/gws12_coldchain_cross_engine.rs` (+7). 4 files new,
11 files strengthened, +941/−61.

### (3) Gates — bucket 4 riders (`scripts/`)

As re-read on disk 2026-10-22: **7 new** untracked gate scripts —
`scripts/{chain_flag_court,count_gate,exit_capture_check,fleet_cleanup,
machine_shaped_gate,receipt_replay_court,witness_gate}.sh` — plus **2
hardened**: `scripts/mutation_court.sh` (+117, EXCLUDED-HOT-PATH per the
L60 note — coordinator re-includes after the hot-path lane lands) and
`scripts/doc_staleness_gate.sh` (+56). This is a count correction against
the dispatch figure of "14 new + 4 hardened"; on-disk census is 16 total
`scripts/*.sh` (7 new + 2 changed in this delta; the remainder pre-exist at
dab6a73). Per-commit gates ride bucket 4 (mega setup + partition/staleness
courts); mutation_court.sh stays typed-EXCLUDED-HOT-PATH.

### (4) Docs truth pass — bucket 6/7

`docs/{THESIS,UNIFIED_DISSERTATION}.md` → `docs/archive/` (rename, zero
content delta), `docs/archive/README.md` new; truth passes on
`docs/{ARCHITECTURE,WASM,api-stability,index,wasm-integration,DIALECTS}.md`,
README/CHANGELOG/SECURITY; residual-unknowns record
(`docs/jira/v26.10.22/designs/residual-unknowns.md`); 16 new designs/*
decision records + RECEIPTS.md +239, INDEX.md +64, _LANES.md +47.
Ride buckets 6 and 7 as assigned at L43/L44.

### (5) External suite pointer — bucket 5 rider (`.github/workflows/ci.yml`, +40)

`external-oracle` job checks out `seanchatmangpt/ggen-marketplace` pinned
`ref: fc2e7f7dbda8a95d625bdb5aa3ff58aa36698534` (ggen-marketplace main @
2026-10-22; comment in-file records the pin replacing a moving `ref: main`
lease) and runs the pm4pytest/graphlaw pytest suite — **the suite itself
lives in ggen-marketplace, not this repo**; ci.yml carries only the pinned
checkout + invocation. Rides bucket 5 (CI).

### (6) Re-pins — bucket 8, LAST (generated, as-regenerated)

`ASSETS.blake3`, `registry/{ARTIFACTS.blake3, capability-registry.json,
capability-registry.ttl}`, `receipts/CHAIN.blake3` (+260), `priv/graphlaw.wasm`
(6793617 → 7098370 bytes) + `priv/graphlaw.wasm.blake3`, plus js/npm
generated projections (`js/npm/src/generated/{artifact-meta,types}.ts`) —
all regenerated, never patched; none may ride an earlier bucket (registry
LAST law, L51). These carry bucket 8's OPERATOR-DECISION-PENDING status via
the dual-artifact pin design (bucket 7 record).

### Standing decision

**C15 operator gate remains (OD-1).** The wave-2 tree does not change the
L47 OPERATOR-DECISION-PENDING status of bucket 8; the dual-artifact pin
decision is still the operator's, and the re-pins in (6) commit only after
it.

### Laws restated

- **Registry LAST**: bucket 8 commits after all others; no earlier bucket
  carries `src/{registry,abi}.rs`, `src/{registry,abi}/`, `registry/`,
  `ASSETS.blake3`, `receipts/CHAIN.blake3`, or `priv/graphlaw.wasm*`.
- **Facades-with-dirs**: each facade + its wired dir travel in the same
  commit (L12 broken-tree rule); wave-2 behavior edits inside facades
  (`sql_ast.rs`, `abi.rs`, `lib.rs`) do not change the pairing.
- **Generated-as-regen**: the (6) re-pins are projections of their
  generators; hand-patching them is a lithography violation.

### Receipt (this addendum)

- `git diff --stat dab6a73 | tail -1` → "61 files changed, 2181
  insertions(+), 256 deletions(-)" — real output, exit 0, measured this
  lane 2026-10-22.
- Untracked census: `git status --porcelain | grep -c '^??'` → **292**.
- Court file census: 4 new `tests/` files + 11 strengthened (+941/−61) from
  `git diff --stat dab6a73 -- tests/`.
- Script census: `ls scripts/*.sh` → 16 total; 7 untracked-new, 2 changed
  (git status on scripts/, real output above).
