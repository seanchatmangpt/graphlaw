# RECEIPTS — v26.10.26

One row per `DEFINITION-OF-DONE.md` item. Evidence is lane-reported (TDD RED →
GREEN receipts + verify/re-vendor/ladder receipts) unless marked **LIVE** —
re-executed 2026-10-26 by the serialized version lane at tree `26.10.26`
(`/Users/sac/graphlaw`). A closed item carries a real command + exit; an item
without a witnessed command + exit is `OPEN(<typed reason>)`, never silently
dropped. The coordinator owns git; everything here is uncommitted working-tree
state.

## D1 — Engine findings closed upstream (`~/purrdf/crates/n3`)

Upstream repo `/Users/sac/purrdf`; lane build roots (`target-fix-*`) are leases.
Final gates: `CARGO_TARGET_DIR=/Users/sac/purrdf/target-fix-verify cargo test -p
purrdf-n3` exit 0 (lib 24, n3_parse 28/28, n3_proofs 4/4, n3_reason 16/16 = 72/0)
after shim/restore anti-vacuity cycles on lexer.rs (sha256 `67dbaf5c…` restore,
byte-exact) and reasoner.rs (sha256 `3d2d72a5…` restore, byte-exact).

| Item | Court / command (cwd /Users/sac/purrdf) | Exit | Status | Evidence |
| --- | --- | --- | --- | --- |
| F1 formula identity | `CARGO_TARGET_DIR=target-fix-u2 cargo test -p purrdf-n3 --test n3_reason --test n3_proofs` | 1→0 | CLOSED | RED: two binder-set-distinct formulas minted one shared skolem IRI (n3_reason.rs:332 panic) + proof dedup collapsed 2 steps ("left: 1, right: 2"). Fix: `Term::canonical_form`/`Triple::canonical_form` (ast.rs:274/519) into `stable_term_hash` (reasoner.rs:4874), `salted_backward_prefix` (reasoner.rs:3732), `proof_writer` triple_key (proof_writer.rs:677); ast.rs:787 unit courts pin injection. GREEN: n3_reason 16/0, n3_proofs 4/0 (distinct genids, 2 steps) |
| F2 verb-path chains | `CARGO_TARGET_DIR=target-fix-u1syntax cargo test -p purrdf-n3 --test n3_parse` | 1→0 | CLOSED | RED: 13 passed / 14 failed — chain triple `:a :b _:pathN` absent (dangling `path1` shape) across forward/reverse/verb-list/is-of/triple-term courts. Fix threads verb-generated triples through all 5 call sites + typed refusal guard on the unreachable log:query branch. GREEN: 28/28, chains asserted with matching path-node labels |
| F3 bare-word refusal | same court | 1→0 | CLOSED | RED: bare word + bare-word datatype parsed Ok minting `example.org/base/...` IRIs. Fix at the `expand_pname` chokepoint (parser.rs) covering datatype/quantifier/reifier callers. GREEN: refusals name the word at offset Some(0); facade court `:socrates a :Man` proves the `a` keyword is spared |
| F4 unknown builtin typed | `CARGO_TARGET_DIR=target-fix-u2 cargo test -p purrdf-n3 --test n3_reason` | 1→0 | CLOSED | RED: unknown builtin reported Complete with errors [] (premise pruned before evaluation — `premise_is_definitively_false`, reasoner.rs:2665). Fix: `unknown_builtin_name` families + `SearchBudget::record_error` hooked into the pruner + match_one_premise (reasoner.rs:2679/2836); status flips Incomplete on non-empty errors. GREEN: Incomplete with `UnsupportedBuiltin{math#totallyUnknownOp}`; guard court pins log:implies rule-as-data + rdf:type stay silent |
| F5 math:equalTo | same court | 1→0 | CLOSED | RED: `0.30000000000000004` == `0.3` held (f64 equality); NaN == 5 held. Fix: exact BigInt ordering → exact decimal-lexical (`parse_exact_decimal`/`exact_decimal_ordering`) → f64 XSD semantics (`math_values_equal`; NaN equals nothing, same-sign infinities equal). GREEN: all 3 courts (incl. guards) pass; ordering predicates unchanged |
| F6 exponentiation | same court | 1→0 | CLOSED | RED: `2^10000000` minted `INF^^xsd:decimal` at exit 0. Fix: inverse arm rejects non-finite ln-ratios; forward arm gates `powf` output (`!is_finite`); shared `eval_unary_numeric_with_inverse` gate corrected. GREEN: courts pass; `(2 10)=1024` exact guard pinned |
| F7 @prefixfr mis-lex | `CARGO_TARGET_DIR=target-fix-u1syntax cargo test -p purrdf-n3 --test n3_parse` | 1→0 | CLOSED | RED: `@prefix`-suffixed word parsed Ok as a directive / `AtPrefix` token. Fix: eager `starts_with("@prefix")` split deleted from lexer.rs `read_word`, exact-token directive match. GREEN: refusal names `prefixfr`; `@prefixfr` langtag keeps its language |
| F8 N3 langtags | same court | 1→0 | CLOSED | RED: `"hi"@e--n` parsed Ok; `"EN"`/`"en"` stayed distinct identities. Fix: `validate_lang_or_lang_dir` wired into the shared `finish_literal` branch; N3 folds case like rdf12 (also improves the rdf12 refusal offset; rdf12 suite stays green). GREEN: refusal at the langtag token; tags collapse to ["en"]. Court pin cites lane U1-SYNTAX F8 |
| F9 `"1e"` refusal | same court | 1→0 | CLOSED | RED: `1e` minted `xsd:double`. Fix: ≥1-exponent-digit terminal check in lexer.rs `read_number`. GREEN: 1e/1e+/1e- refused naming "exponent"; `1e5` still parses. Revert-shim anti-vacuity: collapsing the check flipped exactly this court red (27 others passed), restore sha256-verified |
| F10 spaced path ops | same court | 1→0 | CLOSED | RED: empty path component; `PName("^")` instead of Dot. Fix: parser-level continuation on standalone `!`/`^` words in `parse_pname_or_path` (lexer whitespace break stays load-bearing). GREEN: spaced == unspaced (facts vectors equal), reverse chain oriented, composite splits, leading-operator refusal pinned. Composite court added post-RED, disclosed as receipted coverage |
| F14 README truth-sync | parser.rs source read (quantified_iri 1342–1351, parse_quantifier 1374–1441) | 0 | CLOSED | Verified (not authored) by lane U3-DOCS: README states ordering-and-duplication-only refusals + scope-escape Known-limitations entry; no scope-escape check exists in the parser; U3 made no edit — the fix predates the lane (README mtime 2026-10-25 23:27) |

## D2 — N3 court coverage reaches the advertised surface

| Item | Gate / command (cwd /Users/sac/graphlaw unless noted) | Exit | Status | Evidence |
| --- | --- | --- | --- | --- |
| Proof pipeline court (F13) | `CARGO_TARGET_DIR=target-gl-g4 cargo test --test n3_proof_pipeline_court`; LIVE: `CARGO_TARGET_DIR=target cargo test --test n3_proof_pipeline_court` | 0; 0 | CLOSED | Lane: 5 passed / 0 failed (roundtrip, tamper rejection, tamper-flip twin, claim-line pin, skolem distinctness). LIVE 2026-10-26: 5/0 (/tmp/version-lane-gl-courts.log). Vendor-side round-trip: vendor/purrdf-n3/tests/n3_proofs.rs drives derive → proof_to_n3 → check_proof (4/0 inside the 72/0 parity run) |
| Quantifier court (F15) | `CARGO_TARGET_DIR=target-gl-g5 cargo test --test n3_quantifier_court`; LIVE: `CARGO_TARGET_DIR=target cargo test --test n3_quantifier_court` | 0; 0 | CLOSED | Lane: 10 passed / 0 failed (forall/forsome positive, skolem golden + determinism, anti-collapse, empty-universe flip, scope flip, 6 typed misuses, DeriveN3 admission wrap) + mutation witness: `:knows`→`:noknows` premise flip turned 2 courts red (8/2), byte-exact restore sha256 `a3bf208d…`. LIVE 2026-10-26: 10/0 |
| Conformance corpus under graphlaw gates | — | — | OPEN(NOT-WITNESSED-UNDER-GRAPHLAW-GATES) | No lane executed `corpora/n3` (9 cases) or an equivalent N3 corpus under a graphlaw gate. Closest evidence: upstream `cargo test -p purrdf-conformance-kit --test n3_corpus` exit 0 in /Users/sac/purrdf — an upstream-gate execution, not a graphlaw one. Close condition: a graphlaw court/gate command executing the corpus with command + exit |
| Vendored upstream suite parity | `CARGO_TARGET_DIR=target cargo test --manifest-path vendor/purrdf-n3/Cargo.toml`; LIVE re-run same command | 0; 0 | CLOSED | Re-vendor FINAL D2 receipt: 72 passed / 0 failed — lib 24 + n3_parse 28 (of 28) + n3_proofs 4 (of 4) + n3_reason 16 + doc-tests 0 — exact upstream parity at the same commit. LIVE 2026-10-26: exit 0, 72/0 identical breakdown (/tmp/version-lane-vendor-suite.log). Corrections of record: the DoD's "(46 tests)" was a stale count — real parity is 72=72; the suite runs detached (empty `[workspace]` marker in vendor/purrdf-n3/Cargo.toml) — root `cargo test -p purrdf-n3` is cargo-refused ("requires dev-dependencies and is not a member of the workspace") |

## D3 — Graphlaw-side drift closed

| Item | Gate / command | Exit | Status | Evidence |
| --- | --- | --- | --- | --- |
| F11 limit identities report true sources | `CARGO_TARGET_DIR=target-gl-g1 cargo test --test registry_limits --test n3_fuse_court`; LIVE: `CARGO_TARGET_DIR=target cargo test --test registry_limits` | 0; 0 | CLOSED | SHIM-COURT non-vacuity: collapsing the disjunct back to always-50M flipped the MatchSteps court red with receipt `{"limit":"n3_match_steps","max":50000000,"observed":400000}` (exit 1), restore byte-exact (law.rs sha256 `dfa0f52b…`). GREEN: receipt `{"limit":"n3_match_steps","max":200000,"observed":400000}`; registry_limits 7/7 + n3_fuse_court 5/5; resource_limits 10/10. Term-bytes / derived-facts / iterations identity courts ok (an inversion shim flipped the TermSize court red). LIVE registry_limits 7/0 at 26.10.26 |
| F12 registry emission pins 200k | `CARGO_TARGET_DIR=target-gl-g2 cargo test --test registry_limits --test capabilities_digest --test registry_artifacts` + `graphlaw-registry --write` + `--check` | 0 | CLOSED | RED: registry pins the run-wide 50M figure under the match-steps name (left: 50000000, right: 200000); the engine's real per-search cap is the 200k library default (`DEFAULT_MAX_MATCH_STEPS`). Fix: `pub const N3_MAX_MATCH_STEPS: usize = 200_000` (src/law.rs:470) wired into `reason_n3_bounded`, emitted at src/registry/emit_json.rs:170; EXPECTED court tables re-paired. GREEN: registry_limits + capabilities_digest 6/0 + registry_artifacts 22/0; json:299 + ttl `gac:limitValue 200000`; `--check` all ok; court ties `N3_MAX_MATCH_STEPS == ReasonerOptions::default().max_match_steps == 200_000`. LIVE: capabilities_digest 6/0 + registry_artifacts 22/0 + registry_limits 7/0 at 26.10.26 (post truth-sync, below) |
| README/DIALECTS claims match behavior | LIVE filter run: `CARGO_TARGET_DIR=target cargo test --test docs_drift_courts purrdf_n3_authority_row` | 0 | CLOSED | G3-DOCS: README +4 lines (Dialects and tiers), DIALECTS +13-line Courts bullet, thesis Ch 15.1 +11 lines — all citing the dated 2026-10-26 parity receipt; vendor README corrected upstream-side (see F14). LIVE 2026-10-26: `purrdf_n3_authority_row_is_current` ok (1 passed / 0 failed). Note: the docs_drift_courts binary carries an unrelated red pin court — see the D4 full-suite row |
| ARTIFACTS.blake3 wasm row re-pinned | L2 ladder: `bash /tmp/l2-repin-wasm.sh` + `GRAPHLAW_REQUIRE_WASM_PIN=1 GRAPHLAW_WASM=target/wasm32-wasip1/wasm/graphlaw_wasm.wasm CARGO_TARGET_DIR=target cargo test --test registry_artifacts` | 0 | CLOSED | Re-pin: row `wasm 430da5728bca0d19… 60893336 graphlaw_wasm.wasm`, superseded `58a44095…`/58124612 preserved as historical comment; non-vacuity ALIVE: registry_artifacts 22/22 including the artifacts_pin_file assertion against the fresh bytes (panics "wasm pin is stale" on drift). Re-verified on disk 2026-10-26 (row intact at ARTIFACTS.blake3:14; survives registry `--write`, which preserves foreign rows) |

## D4 — Full ladder green at close

| Item | Gate / command | Exit | Status | Evidence |
| --- | --- | --- | --- | --- |
| Full host suite 0 failures | `WASI_SDK_PATH=/Users/sac/wasi-sdks/wasi-sdk-34 CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS="-C target-feature=+exception-handling" CARGO_TARGET_DIR=target cargo test --no-fail-fast` | 101 | OPEN(CLASSIFIED-RED) | Ladder lane, full enumeration: 1035 passed / 22 failed / 1 ignored over 140 binaries (10 failing). Classification of record: 10 env-gap loud-skips (graphlaw-run-eh binary needs GRAPHLAW_RUN_EH or `--features runner-eh`; wasm artifact gates need GRAPHLAW_WASM or a wasm-profile build) + 12 pre-existing tree-state defects, all verified on disk by the ladder lane: docs_drift_courts::doc_pin_claims_match_on_disk_pins (priv/ vs registry half-landed re-pin — re-verified on disk this lane: priv/graphlaw.wasm is still 58,124,612 B pin `58a44095…` vs registry row `430da572…`/60,893,336 B); no_mock_markers (tests/wasm_duckdb_lifecycle_eh.rs:555 `unwrap_or_default`, on disk); receipt_integrity (log-retention receipt cites 3 battery logs with no EXIT tail); sql_rdf_conformance ×3; wasm_governance_preflight; w3c_conformance ×4 |
| Wasm boundary at new bytes | `WASI_SDK_PATH=… CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS="-C target-feature=+exception-handling" CARGO_TARGET_DIR=target cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` + boundary courts | 0 | CLOSED | Provenance-gated rebuild: attempt 1 unattributable under concurrent in-place mutation (SRC_STABLE=NO, rejected); attempt 2 SRC_STABLE=YES — adopted `430da572…`, 60,893,336 B (was 58,124,612). Boundary courts ALIVE at the adopted bytes: wasm_abi 25/25, chicago_cross_authority 71/71 (both json_depth cases), wasm_n3_admission 3/3, wasm_size_court 4/4. Residual: `target/wasm32-wasip1/` is now absent from disk — the GRAPHLAW_REQUIRE_WASM_PIN rung is not live re-runnable this session (env-gap; loud, not a silent skip) |
| Mutation sweep (time-boxed) | `CARGO_TARGET_DIR=target-mut bash scripts/mutation_court.sh --force --log /Users/sac/graphlaw/mutation-court-L3.log` | 124 | CLOSED(TIME-BOXED-PARTIAL, per the DoD clause) | Gate 1 unmutated baseline PASS (9 targets, 95 tests, 0 failed). Sweep: callsite-revert leg FLIPPED under shim (law_state exit 101, expected) + byte-exact restore (sha256-verified ×5 files); cargo-mutants inventory 204, killed by the 2h background limit at 113/204 recorded — 61 caught, 52 unviable, 0 missed, 0 timeout, 91 not run; gated-surface missed scan EMPTY (registry.rs, registry/*.rs, abi.rs capabilities/dispatch); restore integrity after SIGKILL: sha256-match ×5, git status identical, zero mutant markers, zero SHIM-COURT |
| `statutory_boundary_50ms` ALL GATES PASS | `CARGO_TARGET_DIR=target cargo bench -p graphlaw --bench statutory_boundary_50ms` (runs 2, 3 loaded; run 4 solo 2026-10-26 after target/ rebuild) | 0 | CLOSED(PASS-AS-MEASURED) | Runs 2-3 (load-40-55 box, concurrent cargo-mutants lab): Art.14 p99 7,820-8,966µs > 5,000µs — verdict of record was GATE FAILURE, diagnosis load-induced tail noise. Run 4 SOLO serialized against compile lanes: Art.12 fold p99 167ns, Art.14 validate p99 78,667ns / halt p99 81,583ns (~62x under ceiling), Art.15 p99 33,500ns, END-TO-END p99 sum 193.9µs vs 40,000µs budget — ALL GATES PASS, exit 0. The per-leg Art.14 ceiling is load-fragile under fan-out; solo/serialized measurement is the ruling rung for this gate |
| Grep gates: zero live eyeron | `grep -rn eyeron src/ tests/ examples/ wasm/` (ladder lane) + this lane's re-verify | — | OPEN(STRICT-READING-FAIL) | 15 raw hits: 8 negative assertions, 5 retirement/attribution comments, and 2 LIVE string literals in tests/differential.rs:74 (`expect("eyeron closure")`) and :171 (`"eyeron, seed {seed}, edges {es:?}"`) — re-verified on disk 2026-10-26. Strict zero-live reading FAILs on those 2; zero-live-dependency reading PASSes (no imports, crate refs, or invocations anywhere in src/ tests/ examples/ wasm/). Close condition: two-line message-string rename in tests/differential.rs (not this lane's lease) |

## D5 — v26.10.26 release mechanics (executed by this lane, LIVE)

| Item | Command | Exit | Status | Evidence |
| --- | --- | --- | --- | --- |
| Workspace `[package] version = "26.10.26"` | `perl -pi -e 's/^version = "26\.10\.22"$/version = "26.10.26"/' Cargo.toml` (pre-edit snapshot /tmp/version-lane-cargo-toml-before.toml) | 0 | CLOSED | `grep -n '^version' Cargo.toml` → line 3 `version = "26.10.26"`, exactly one occurrence of the new value. The Cargo.toml working-tree diff also carries pre-existing fix-wave changes (coordinator owns git) |
| Cargo.lock regen | `CARGO_TARGET_DIR=target cargo check` | 0 | CLOSED | First run `Finished dev profile … in 48.66s`; clean-exit re-run `CARGO_CHECK_EXIT=0` (0.24s); Cargo.lock `name = "graphlaw"` → `version = "26.10.26"` |
| js/npm sync-version | `node js/npm/scripts/sync-version.mjs` | 0 | CLOSED | `sync-version: package.json -> 26.10.26` (was 26.10.22) |
| js/npm gen-types | `node js/npm/scripts/gen-types.mjs` (run twice: post-sync, then again after the registry regen re-stamped the digest) | 0 | CLOSED | Final run stamped types.ts:2 `registry_blake3: blake3:7140cf12…`; also projects the Engine enum without `Eyeron` (fix-wave registry, now version-current) |
| CHANGELOG 26.10.26 section | Edit (this lane) | — | CLOSED | Section added between [Unreleased] and [26.10.22]; the [Unreleased] n3 bullet moved into [26.10.26] Added (it ships here); registry truth-sync recorded under Fixed |
| Window files | docs/jira/v26.10.26/{DEFINITION-OF-DONE,RECEIPTS,INDEX}.md | — | CLOSED | This file + INDEX.md written; DEFINITION-OF-DONE.md pre-existing |
| RECEIPTS-INDEX line | `b3sum docs/jira/v26.10.26/RECEIPTS.md` → index row | — | CLOSED | Row added at the top of the RECEIPTS-INDEX.md table (on-disk row order is newest-first; the stale "Newest last" header prose corrected to match practice) |

## Registry truth-sync disclosure (this lane)

The version bump alone turned two capabilities_digest courts red:
`emitted_registry_digests_recompute_and_match_live` and
`negative_mutating_an_op_name_changes_surface_and_registry_digests` (live digest
`7140cf12…` vs committed `7086e6bd…`) because `emit_json.rs:142` embeds
`env!("CARGO_PKG_VERSION")` as `graphlaw_version`. Resolution was a generator run,
never a hand-edit: `CARGO_TARGET_DIR=target cargo run --quiet --bin
graphlaw-registry -- --write` (exit 0) produced exactly a 2-line diff in each of
capability-registry.json/ttl (`graphlaw_version`, self-digest `registryBlake3`)
plus the ARTIFACTS.blake3 registry-row re-pin; the foreign wasm row
(`430da572…`/60893336) was preserved byte-identical (verified by diff against the
/tmp/version-lane-prestate/ snapshot). `--check` exit 0, all `ok`; gen-types
re-run. Post-sync battery, all green at exit 0: capabilities_digest 6/0,
n3_proof_pipeline_court 5/0, n3_quantifier_court 10/0, registry_artifacts 22/0,
registry_limits 7/0 (/tmp/version-lane-gl-courts.log).

## Residuals (typed, for the coordinator)

1. `priv/graphlaw.wasm` + `priv/graphlaw.wasm.blake3` still carry the pre-EH module
   (58,124,612 B / `58a44095…`) while registry pins `430da572…`/60,893,336 B — the
   half-landed re-pin; docs_drift_courts::doc_pin_claims_match_on_disk_pins stays
   red until priv/ is synced (not this lane's lease).
2. `target/wasm32-wasip1/` is absent from disk, so the GRAPHLAW_REQUIRE_WASM_PIN
   registry rung is not live re-runnable this session (env-gap; loud, not a skip).
3. Bench Art.14 per-leg p99 ceilings are load-fragile under fan-out: solo rerun of
   `statutory_boundary_50ms` (serialized against compile lanes) before code action.
4. tests/differential.rs:74/:171 live `eyeron` string literals (two-line rename).
5. D2 conformance corpus has no graphlaw-gate execution witness (OPEN row above).
6. Lane build roots `/Users/sac/purrdf/target-fix-*` and the L3 mutation snapshot
   dir `/Users/sac/.cache/tmp/mutation-court.Asgr5H` (permission-denied to this
   session) remain for the coordinator to delete at integration.

## W0-runeh — EH court env-gating receipt (LIVE 2026-10-26)

| Item | Gate / command | Exit | Status | Evidence |
| --- | --- | --- | --- | --- |
| `tests/graphlaw_run_eh_court` (7 loud-skip fails in the D4 ladder enumeration) is env-gated BY DESIGN — green under the EH env | Build: `CARGO_TARGET_DIR=target-arr-w0c cargo build --features runner-eh --bin graphlaw-run-eh`; Court: `GRAPHLAW_RUN_EH=$PWD/target-arr-w0c/debug/graphlaw-run-eh GRAPHLAW_WASM=$PWD/priv/graphlaw.wasm CARGO_TARGET_DIR=target-arr-w0c cargo test --test graphlaw_run_eh_court` (abs paths, tree `/Users/sac/graphlaw`) | 0; 0 | CLOSED | Build exit 0 (bin 85,572,496 B). Court exit 0: `7 passed; 0 failed; 0 ignored … finished in 250.75s` — full log `/tmp/lane-W0-runeh-court.log` (tail carries `EH_COURT_EXIT=0`). The D4 7-fail enumeration was the loud typed skip (`eh_bin` None → `skip(true)` panic, tests/graphlaw_run_eh_court.rs:47-59), not defects; no src change was needed. Disclosure: artifact resolved via `GRAPHLAW_WASM=priv/graphlaw.wasm` — 60,895,924 B, sha256 `bc8177695f079d1d…` — because `target/wasm32-wasip1/wasm/graphlaw_wasm.wasm` is absent this session (matches the D4 residual); note the priv bytes are neither the superseded `58a44095…`/58,124,612 B nor the D4-adopted `430da572…`/60,893,336 B (third-byte-state drift — not this lane's lease). Capabilities-frame probe on those exact bytes: exit 0 — the exit97 pending-artifact pin (SQLRDF-24 `uninitialized element`) has taken the court's accepted clean flip at current priv bytes (`/tmp/lane-W0-runeh-cap-probe.err`, empty stderr). Lane lease honored: docs/ only; build root `target-arr-w0c` is a lane lease for coordinator deletion at integration |
| Re-verify (re-dispatch resumes from checkpoint): standing refreshed on the CURRENT tree, which the parallel eyeron→purrdf migration wave mutated mid-verify | Same env + commands as the row above; re-run 2026-10-26 ~11:29–11:36 | 0; 0 | CLOSED | Build exit 0 (bin 85,572,736 B, 11:29). Court exit 0: `7 passed; 0 failed; 0 ignored … finished in 19.40s` — full log `/tmp/lane-W0-runeh-court2.log` (tail carries `EH_COURT_EXIT=0`). Artifact identity re-read from disk at re-run time: `priv/graphlaw.wasm` 60,895,924 B, sha256 `bc8177695f079d1d9c30b3d4b4ecd3b5620abdd56f6619e197bf8ea3d39a5408` — byte-identical to the row above. During re-verify the shared tree was red → converging 4→2→1→0 compile errors from the parallel migration wave (`src/law.rs`, `src/shacl_guard.rs`, `vendor/purrdf-n3/*` edits 11:08–11:29:03, owner lane — not this lane's lease); this lane polled the compile gate as a real observable (no sleeps, no edits to leased-out files) and ran the court only after the gate returned exit 0. exit97 pin still takes the court's accepted clean flip (capabilities probe exit 0 in court2 log). Lane lease honored: docs/ only |

## Parked: moonshot array v2 (2026-10-26)

| Moonshot array v2 | Workflow resumeFromRunId wf_fbd828d9-fad (script: session workflows/moonshot-array-v2.js, operator-saved) | - | PARKED(operator-pivot) | Park state: docs/jira/v26.10.26/PARKED.md (resume command, journal path, lane checkpoints, DoD state at park). All agents stopped cleanly; journal + /tmp lane checkpoints + the script-on-disk carry 100% of the state. |
