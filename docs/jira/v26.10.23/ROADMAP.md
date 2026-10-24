# ROADMAP

Milestone v26.10.23 — post-v26.10.22 wave: wasm gap closures + Phase-2
CanonArrow. Derived from the approved plan
(`~/.claude/plans/here-is-the-voice-golden-bengio.md`). v26.10.22 is
witnessed end-to-end on the final artifact identity `472d5557…`
(58,121,500 B, full-interception wrap shim, pinned by copy at `priv/` +
registry + npm 198/198); only its solo mutation-court verdict remains.

## Sequencing law

- `scripts/mutation_court.sh` mutates the tree in place and runs SOLO,
  serialized against all lanes; its inversions false-red concurrent
  compiles. The remaining v26.10.22 mutation-court verdict gates Wave C
  (src/ closures).
- Waves are ordered: A (docs/ledger, any quiet window) → B (tests-only,
  no src/ changes) → C (src/ hardening, after the v26.10.22 verdict
  frees src/) → D (Phase-2 CanonArrow, largest; own court set) →
  E (re-witness + docs) → F (ERRC innovation candidates, folded as
  reports land). OQ1-OQ3 block Wave D; OQ4-OQ6 are non-blocking.
- Verification per wave is a DELTA check: zero NEW gate findings
  (doc_staleness carries 1,143 pre-existing R5 anchors), R2/R3/R4 stay
  zero, chain `--check` exit 0, full battery re-run on this wave's final
  artifact identity.

## Gap inventory

### Wasm behavior gaps (binary + behavioral audits; file:line verified)

- **G1** — `receipt_v2` has no production emission site (`attest.rs:704`
  ctors only; `duckdb_eval.rs` never calls attest).
- **G2** — CanonArrow in-wasm: `src/egress/arrow.rs` is DCE'd (no
  wasm/src or abi.rs reference); `query_arrow`/`stream_arrow` exist in
  frozen-duckdb; arrow crates already in the dep tree.
- **G3** — `tests/conftest.py:85-107` hardcodes the lane root + stale
  `--whole-archive -lduckdb_wasi_posix` recipe; `tests_py/conftest.py:32-36`
  uses `/tmp/pm1-stubs/glwrap.o`; neither honors `GRAPHLAW_WASM`.
  Canonical: `tests/fixtures/wasm-shim/glaw_wrap_shim.o` +
  `docs/WASM.md:20-36` recipe.
- **G4** — `read_frame` (`graphlaw_run_eh.rs:336-350`) allocates
  unbounded from the 4-byte length prefix (host-side DoS surface).
- **G5** — constants drift: WALL_BYTES 64 MiB (host) vs guest
  MAX_REQUEST_BYTES (abi_meta generated) live in different files; they
  are different limits by design, single-sourced via
  `graphlaw::abi::MAX_REQUEST_BYTES` with a startup ordering assert.
- **G6** — `sql_rdf_compiler` + `sql_rdf_admission` benches + TC-HOT
  gates have no receipted numbers on disk.
- **G7** — `abi.rs` response builders + `spec_of` uncovered
  (SQLRDF-28, 11+ missed mutants; all `pub` already — zero src change
  needed for the court).
- **G8** — sniff layer: `route_sql_rdf` (`sniff.rs:252-279`) does not
  strip string literals (corpus_conformance root cause); skeleton
  `#`-in-IRI truncation; 600-char head window; escape-naive
  `line_terms`; bare-SQL→SPARQL misroute (SQLRDF-17 ticket).

### Doc/ledger gaps

- **D1** — SECURITY.md does not enumerate the 28 imported WASI
  capability classes.
- **D2** — WASM.md:211 blanket 256 MiB vs the wasm32 split-config
  16 MiB outstanding-bytes; `persistent-duckdb-sessions.md:40` misses
  process-local MAX_SESSIONS semantics.
- **D3** — GWS register stale PENDING tokens
  (`wasm-duckdb-integration.md:49-56`); REFUSAL-05 receipt-pending
  note; INDEX orphans (SQLRDF-26, gap ledger, ADR) + SQLRDF-23
  landed/Open drift; `designs/INDEX.md` references a moved file.
- **D4** — transcript-class witnesses on `472d5557` lack durable
  `receipts/` files (fuzz, memory-wall, exit-codes x3, dialect census,
  banking, pm4pytest).

## Waves

### Wave A — docs/ledger only

Runs during any quiet window; no src/ or tests/ dependency.

1. **D1**: SECURITY.md capability-inventory paragraph — cite the real
   count command + output (never hand-write "28").
2. **D2**: WASM.md:212 "256 MiB" is wrong in BOTH directions vs code;
   the truth is 16 MiB request + 16 MiB outstanding, uniform across
   targets (`abi.rs:48/:73`, `abi_meta.rs:29`, equality asserted
   `wasm/src/lib.rs:32-33`); sessions doc gains "process-local and
   crash-only".
3. **D3**: RECEIPTS.md is append-only with an established line-end
   `# historical` marker convention (lines 618/858/898) that the
   doc_staleness R2 gate already exempts — five stale ranges (927-929,
   962-968, 1240-1241, 1448-1452, 1463-1467) fix via pure line-end
   appends + one appended reconciliation section; INDEX/WASM/sessions/
   integration/refusals correctable in place, each recorded via a
   RECEIPTS pointer row. GWS flips governed by the register's own law
   (flip only on a landed receipt): GWS-2/GWS-3 flip now; GWS-1/4/7/8
   stay PENDING with dated re-check tokens.
4. **D4**: witness-class upgrades via RE-RUN per court on the final
   artifact, one receipts/ file per class embedding `472d5557` (fuzz,
   memory-wall, exit-codes, banking, pm4pytest, receipt_v2 +
   trust_attest); dialect census only if a rerun reproduces 69/0, else
   file the honest count; one consolidated RECEIPTS pointer row upgrades
   terminal-receipt section-2 classes TRANSCRIPT → DURABLE. Falsifier:
   any re-run count contradicting a transcript figure → divergence
   append, never laundered.
5. Verification: doc_staleness gate (R2/R4 stay zero) + coherence
   re-review.

### Wave B — tests-only closures

No src/ changes needed.

6. **G7**: `tests/abi_response_builders_court.rs` — NO visibility
   widening (`spec_of`/`term_json` fully reachable via public
   `call_json`; widening would expand the released API surface guarded
   by `api_stability.rs`). ~10 tests: three pub builders with
   exact-message + whole-details-map equality; `spec_of` both refusal
   arms; `str_field`; `term_json` all four arms via a real SPARQL
   SELECT; the json_depth pair (65 refused observed == 65 / exactly 64
   admitted) — that twin kills the currently-LIVE `check_limit >` mutant
   the sweep has in place right now. Every refusal paired with a
   positive control. Acceptance: 10-min hand-mutant pre-flight fails the
   new court on both named survivors, then mutation rerun moves all
   src/abi.rs survivors to caught.
7. **G3 conftest** (corrected file/line map): the /tmp shim is
   `tests/conftest.py:35`; the target-lane-pm2 hardcode + stale
   whole-archive recipe + wrong SDK stack is
   `tests_py/conftest.py:85-107`. Fixes: SHIM_OBJ const →
   `tests/fixtures/wasm-shim/glaw_wrap_shim.o`; GRAPHLAW_WASM-honoring
   `wasm_artifact()` with loud typed failure; WASI_SDK_PATH env or
   frozen-duckdb cache (isdir typed failure); recipe ONLY under
   `CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS` (cargo ignores plain RUSTFLAGS
   for a target when the target-scoped var is set — the current
   plain-RUSTFLAGS recipe is silently droppable). Keep pm1/pm2 lane
   defaults; assert shim .o existence, never recompile. Acceptance:
   pytest lanes green with GRAPHLAW_WASM set AND unset.

### Wave C — src/ closures (after the v26.10.22 court verdict frees src/)

8. **G4**: bound `read_frame` in BOTH hosts (`graphlaw_run_eh.rs:336-350`
   AND `graphlaw_run.rs:456-472` — identical wire protocol, same attack
   surface) with `FRAME_CAP = graphlaw::abi::MAX_REQUEST_BYTES`; new
   `RunError::FrameOversize` → exit 92. Anti-vacuity: exactly-at-cap
   frame succeeds; `0xFFFFFFFF` prefix exits 92 naming length+cap before
   any body read; zero-len+EOF exits 0.
9. **G5**: hosts re-export `graphlaw::abi::MAX_REQUEST_BYTES` (the
   single-source hook exists: `wasm/src/abi_meta.rs:20` asserted ==
   `abi::MAX` at `wasm/src/lib.rs:32`); startup assert
   MAX_REQUEST_BYTES <= WALL_BYTES (different limits BY DESIGN — do not
   force-equal); pin all three in the abi_manifest court limits table.
10. **G8**: sniff fixes — string-literal stripping in `route_sql_rdf`
    (match `skeleton.rs:33-56`), skeleton IRI-aware `#` (`:28-31` vs the
    IRI-aware stripper), escape-aware `line_terms`; bare-SQL refusal
    class per SQLRDF-17 (courts: corpus_conformance + new sniff twins).
10b. **W08 differential** is NOT pending-implementation —
     `tests/common/mod.rs:20-22` already hosts it via wasmtime; the gap
     is only RUN+RECEIPT against the pinned artifact (release-gating
     receipt, cheap).
10c. WASM.md:206 outstanding-cap "256 MiB" → 16 MiB (release-gating doc
     fix); MAX_SESSIONS process-local sentence (follow-up).

### Wave D — Phase-2 CanonArrow (largest; own court set)

11. `src/egress/collector.rs` (NEW): hand-rolled deterministic
    uncompressed Arrow IPC writer (NO arrow-rs re-import — code-size
    guard), 64-byte aligned, zero-filled validity bitmaps, streaming
    BLAKE3 per batch, `FLAG_STRICT_PROJECTION_ORDER` always, schema
    verbatim from `duckdb_eval.rs:1305-1311` column_names (already
    projection order).
12. `duckdb_eval.rs`: thread the collector through the EXISTING
    `:1320-1490` ValueRef walk (single pass — REQ-01); branch to
    `query_arrow` at `:1303`; receipt_v2 seal after the `:1574` rebuild;
    ExecutionReceipt extension via the lossless-default pattern
    (`:305-395`).
13. ABI: new `egress_arrow` op (`abi.rs:319-327`, feature-gated) OR
    response fields on duckdb exec (OQ-3); `wasm/src/abi_meta.rs` is
    ggen-rendered — edit the ontology/template, re-render; code-size
    guard = `cargo tree` diff + artifact hash before/after.
14. Courts (`tests/canonarrow_egress_court.rs`): Twin A (v1 must NOT see
    columnar bytes AND v2 must), Twin B (independent payload-region
    recompute), tamper refusals, empty arm, v1 goldens untouched, FIBO
    battery under v2, in-wasm pinned-view court. CHAIN quiesced-regen +
    registry/pin re-pin LAST (artifacts are copy-only — pins from the
    real rebuild).

### Wave E — re-witness + docs

15. REQ-01 (<=1.5 ms) / REQ-02 (<=1.2 ms in-wasm BLAKE3) benches
    receipted (resolves G6 if OQ6 defers it here).
16. SECURITY/WASM/dialect docs updated for the new surface; ggen sync
    untouched (no vocabulary change).

### Wave F — ERRC innovation catalog

Ranked by ALIVE-parts x gain / cost; the remaining 15 ERRC lenses land
as notifications and fold with the same rank discipline.

- **C31 qualified refusals** (4 ALIVE parts, ~1 day, courts-only):
  qualification algebra attacks the refusal probe — every
  `details.code` reaches Qualified only after a witnessed mutation
  attack; standing digest registered in ARTIFACTS.blake3. Highest rank.
- **C29 receipt chain as cache-coherence law**: chain_hash replaces the
  wall-clock epoch; torn-tail detection becomes a crash-consistency
  guarantee.
- **C30 chain-notarized doctrine edits**: generated-artifact edits
  append frames; hand-patched projections refuse typed.
- **C28 ring-overflow SPC**: drop-gap sequence as the control chart —
  fleet-wave saturation gets a leading indicator.
- **C32 standing-addressed registry pins**: C03 realized at home; needs
  the pending chain-serialization decision.
- **RAISE receipt_v2 → product**: `graphlaw-attest verify/seal/chain`
  CLI + self-contained replay JSON (auditor needs no graphlaw
  dependency); killer demo = one flipped Arrow byte → `ColumnarMismatch`
  naming which witness broke. Merkle tree over 64-byte payload chunks →
  `prove-row`: row-level inclusion proof in <1 ms for discovery
  (selective disclosure; no raw export can replicate). Needs Phase 2 —
  no real .arrows evidence exists to seal (constructor only today).
- **CREATE court-kit-pack**: marketplace pack porting the enforcement
  method (refusal register → typed enum generator, probe pattern,
  receipt chain port, vacuity twin-court template) — consumers
  installing it for the data model wake up with self-installing
  anti-vacuity enforcement (shadow twins, refusal-live KAT gate,
  mutation twins) as a side effect.
- **CREATE Arrow consumer demo** ("the $25M wire that flunks its own
  digest"): FIBO high-value query → in-wasm Arrow → Polars zero-copy →
  consumer flips one amount byte → digest + Ed25519 both fail in the
  consumer's own process, no graphlaw consulted; the tampered value
  would have silently crossed a compliance threshold — text egress
  could never catch it. Doubles as a permanent REQ-03 negative.
- **RAISE egress to a first-class ABI op**: egress has the strongest
  adoption signal (6.83x receipt, FIBO egress leg, adopted in a dozen
  courts, EH-19 open) and the weakest surface — zero registry presence,
  no op, no typed refusal codes. The repo's own law applies to its own
  registry: egress absence is currently a silent skip.
- **SURPRISE autonomy lattice** (objection-window form): RELEASE_HOLD
  reads as the archetypal human control but transmits one bit a signed
  attestation transmits better — swap the console variable for an
  Ed25519-attested release lease over (version, commit, chain_head)
  verified in release.yml's plan step; every primitive (attest,
  TrustedKeys, SignedLease) is in-tree and golden-tested. Stronger
  form: timed signed objection window — after the attest-signed release
  receipt lands, publication waits N hours; the machine proceeds unless
  the human signs a *refusal* (root-key `release-objection`
  attestation). 999 unattended releases cost nothing; the one that
  smells wrong costs exactly one signature. Silence becomes evidence.
  Pin-by-copy → derive-then-write is the best ratio (near-zero
  machinery). crates.io autonomy deliberately last (highest machinery);
  chain re-baseline automation deliberately never (quiesced-regen
  timing is the human-judgment residue that should stay human).
- **SURPRISE self-hosting (process-mining the dev loop)**: receipts/ IS
  an OCEL log (1,561 files); mining it: Edit dominates 84% (1,273 vs
  241 Write — perpetual repair mode); sql_ast.rs touched 124x,
  duckdb_op 50x, duckdb_eval 47x (the disjoint-lease invariant violated
  by design pressure); 1,510/1,515 transition receipts UNSIGNED-NO-KEY;
  waves grew 16→20→32 lanes with ~half the terminal states
  BLOCKED/UNFILED, blockers environmental not code. Interventions:
  (I-1) gate the gates — wave close requires CHAIN `--check` clean (the
  tamper-detected file still in-tree mints a typed chain-integrity
  refusal); (I-2) data-driven leases — any file >10 events/wave becomes
  coordinator-owned-serial next wave; (I-3) pre-wave
  environment-admission court (df/staging/SDK pins) replacing ~20
  recurring BLOCKED rows. Self-hosting endgame: `dev-loop.ttl` where
  the coordinator's integration commit must be an ADMITTED transition —
  receipts/ as event log, receipt_chain as fold, law.rs as gate.
  Machinery ~90% built; currently governs only the product, never the
  process.
- **ELIMINATE wasmi host retirement — safe today, no blocker**: wasmi
  2.0.0 cannot parse the EH artifact (vendored receipts: no EXCEPTIONS
  feature, open upstream since 2024-07); the only remaining wasmi code
  is `src/bin/graphlaw_run.rs` + unused dev-deps + 5 profile blocks
  (~1,500 lines); graphlaw-run-eh is a protocol superset. One PR: flip
  tests_py blackbox (3 sites) + conftest lookup, witness green, then
  delete bin/feature/deps/profiles; deletion receipt with
  `cargo tree -i wasmi` absent-from-graph witness. Preserve-as-note:
  wasmi is a pure interpreter (no-JIT host = future create, not
  preservation).
- **ELIMINATE census** (16 lanes of dead weight): vendor/
  purrdf-sparql-eval is 68,282 lines carrying a 77-line patch
  (elimination shape: upstream the patch); dead examples with zero
  consumers (gws8_shacl_native, rss_expansion_probe (blackbox_fixture CORRECTED: live consumer at tests_py/conftest.py:30 — keep));
  undeclared sa2a_portability_verify bin (no consumer); docs/jira
  process receipts = 142 MB tracked (move to release storage, keep
  INDEX+RECEIPTS); mutants.out.old 16 MB.
- **REDUCE artifact size**: Code = 54.3 MB / 98%; LTO and
  codegen-units are ALREADY saturated — remaining attacks are deps +
  post-link: (1) `wasm-opt -Oz` post-link = 10-18 MB at LOW risk
  (biggest cheap win; digest re-pin ceremony required); (2) DuckDB
  build pruning (extension subset) = 15-30 MB at medium risk, preserves
  repo law; (3) `graphlaw-minimal` SKU via feature-gating frozen-duckdb
  = 35-45 MB but requires a CHARTER AMENDMENT (DuckDB-is-not-a-feature
  law).
- **REDUCE lane thrash** (measured: 1,515 transition receipts,
  sql_ast.rs 124 writes, 25 lane roots ≈ 120 GiB, 95% full volume):
  (1) pre-dispatch drift gate (~zero machinery — eliminates the
  dispatch-onto-broken-tree class outright); (2) mechanical hot-file
  lease locks (4 files: sql_ast, duckdb_op, duckdb_eval, RECEIPTS);
  (3) crate splitting (sql_ast/duckdb_eval/duckdb_op → workspace
  crates) — the fix that makes 20-agent width CHEAPER per agent, not
  smaller.
- **RAISE dialect completeness** (ledger-truth correction: the
  SQLRDF-25 gap ledger itself is STALE — all five rows landed in
  88d6665; this exercise re-ranks residuals + NEW gaps): #1 set
  operations UNION/INTERSECT/EXCEPT — verified absent everywhere, yet
  `test_cold_chain_ocel` already uses raw UNION ALL SQL text (breaking
  the sealed boundary); strictly cheaper than PIVOT with bigger
  elimination. #2 GROUP BY ALL / ORDER BY ALL (tiny render arm). #3
  COLUMNS lambda residual. Then Row 3 PIVOT residuals last.
- **ELIMINATE test redundancy** (ranked consolidation): DELETE
  `wasm_duckdb_op_wired.rs` (508 LOC — test 1 strictly weaker than its
  sibling; port only legacy_pin test; 3/4 skip-prone Node leg), shacl
  pair in abi_bounds (superset exists), src-side pins re-asserted by
  receipt_v2_court's independent b3sum witness; MERGE the duckdb-op
  triple (native|wasmtime|node runtime matrix, strictest assertion per
  leg); REDUCE 4 finance courts' ~70-LOC boilerplate into common;
  CONSOLIDATE the 17 GRAPHLAW_WASM read sites into one
  `common::subject()`. Surprise: `built_module_links_the_duckdb_dispatch`
  can never catch anything its sibling misses — deletable today.
- **REDUCE API surface** (ranked top-10, all repo-wide-grep verified):
  `src/qualification.rs` is entirely test-only (~200 pub lines, zero
  non-test consumers — pub(crate)/move/delete); three dead
  `ReceiptEpochV2Builder` methods; `QueryCache::with_entries` exists
  ONLY so one test can panic inside the locked critical section
  (rewrite the test through the pub surface, delete/demote the
  accessor); `EvidenceRing` 7-method test-only introspection family;
  dialect untyped/typed duplicate wrappers; `GraphLawDuckDB`'s
  6-constructor family → one `open_in_memory_with(config)`;
  `abi_manifest` pretty/write pair test-only. Public-surface shrink
  feeds `api_stability.rs`.

## Operator open questions

- **OQ1** (Wave D blocker): H_pre/H_post witnesses do not exist on eval
  paths — (a) sentinel digests (weak), (b) state digests before/after
  (hot-path cost), (c) restrict v2 to mutation statements.
- **OQ2** (Wave D blocker): H_triples canonical source — N-Triples
  rendering (blows REQ-01 on the hot path), SerTerm canonical form, or
  an existing triples digest.
- **OQ3** (Wave D blocker): transport — base64-in-JSON (+33%, risks the
  16 MB cap) vs dedicated binary op returning side-table pointers (ABI
  version bump).
- **OQ4**: dual-receipt (v1+v2 additive CHAIN) vs v1 retirement (full
  quiesced-regen).
- **OQ5**: does a 0-row tabular result emit CANON_ARROW_EMPTY_V1 or a
  real empty-batch payload? ADR says non-tabular only.
- **OQ6**: G6 bench receipts now (Wave B/C window) or folded into Wave
  D's bench pass?

## Verification

- Wave A: doc_staleness gate (R2/R4 stay zero) + coherence re-review.
- Wave B: `cargo test --test abi_response_builders_court`; mutation
  rerun moves the family to caught; pytest suites pass with
  GRAPHLAW_WASM set to the canonical artifact AND unset (lane
  fallback).
- Wave C: sniff twins + corpus_conformance green; read_frame bound
  court (oversize prefix → typed exit, no alloc).
- Wave D: receipt_v2 end-to-end court (dual digests over a real DuckDB
  result, v1 golden untouched); wasm lifecycle + FIBO green on the new
  artifact; benches receipted against REQ budgets.
- Full battery re-run on the final artifact identity of this wave.
