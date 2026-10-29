# v26.10.26 — Definition of Done (purrdf-n3 finish)

The milestone closes when every item below carries a real command + exit in
`docs/jira/v26.10.26/RECEIPTS.md`. "Fixed or receipted" means: a code fix with a
flip-capable court, or a typed known-limitation entry naming the trigger, blast
radius, and the court that would catch regression. Silence is not an outcome.

## D1 — Engine findings closed upstream (`~/purrdf/crates/n3`)

Each audit-confirmed engine defect is fixed or receipted, TDD-style (failing
court first, fix second, flip witnessed):

- [ ] F1 Formula identity substrate: `Debug` no longer the identity for
      skolem IRIs / backward-rule salting / proof dedup keys — two distinct
      formulas (differing existentials) mint distinct identities
      (`vendor/purrdf-n3/src/ast.rs:225`, `reasoner.rs` stable_term_hash,
      `proof_writer.rs` triple_key).
- [ ] F2 Verb-path generated triples are not dropped (`parser.rs:993`).
- [ ] F3 Bare-word prefixed names do not fabricate `example.org` IRIs —
      typed refusal instead (`parser.rs:1238`).
- [ ] F4 Unknown builtins refuse typed (`UnsupportedBuiltin` reachable),
      never silently Complete (`reasoner.rs:943`).
- [ ] F5 `math:equalTo`/`notEqualTo` correct on decimals and NaN
      (`reasoner.rs:6199`).
- [ ] F6 `math:exponentiation` inverse/edge behavior fixed
      (`reasoner.rs:5647`).
- [ ] F7 `@prefixfr:` no longer mis-lexes as a directive (`lexer.rs:316`).
- [ ] F8 N3-profile langtags validated (strict, matching the RDF 1.2
      profile's existing validator) instead of verbatim acceptance
      (`lexer.rs:334` / parser profile) — court pin updated.
- [ ] F9 `"1e"` refuses as malformed numeric on the N3 profile
      (`lexer.rs:273` + `number_literal`).
- [ ] F10 Spaced N3 path operators (`p1 : :p2` etc.) accepted per spec
      (`lexer.rs:298`).
- [ ] F14 upstream README claims match implemented behavior.

## D2 — N3 court coverage reaches the advertised surface

- [ ] Proof pipeline court: derive → `proof_to_n3` → `check_proof` positive
      courts exist and pass (closes audit F13).
- [ ] Quantifier court: `@forAll`/`@forSome` positive + negative courts
      exist and pass (closes audit F15).
- [ ] Conformance corpus: the upstream `corpora/n3` (9 cases) or an
      equivalent N3 corpus executes under graphlaw gates.
- [ ] The vendored crate's upstream test suite (46 tests) executes inside
      graphlaw gates (tests restored into `vendor/purrdf-n3/tests/` — the
      manifest already declares them) — closes the vendored-test-parity gap.

## D3 — Graphlaw-side drift closed

- [ ] The three limit identities in `src/law.rs` receipts report their true
      sources (not `n3_iterations/4000` for all three) (audit F11).
- [ ] Registry emission pins the real firing cap (200k, not 50M)
      (`src/registry/emit_json.rs:169`) (audit F12).
- [ ] README/DIALECTS claims match implemented behavior (audit F14/F15 doc
      side).
- [ ] `registry/ARTIFACTS.blake3` wasm row re-pinned to the freshly built
      module bytes (60,880,445 B class) — no longer only checked under
      `GRAPHLAW_REQUIRE_WASM_PIN`.

## D4 — Full ladder green at close

- [ ] `cargo test` full host suite: 0 failures.
- [ ] Wasm boundary: module rebuilds; `wasm_abi` + `chicago_cross_authority`
      + `wasm_n3_admission` + `wasm_size_court` green at the new bytes.
- [ ] `bash scripts/mutation_court.sh --force` — the 204-mutant sweep
      executes (time-boxed; partial results receipted with counts).
- [ ] `cargo bench --bench statutory_boundary_50ms` — ALL GATES PASS.
- [ ] Grep gates: zero live `eyeron`/`graphlaw-eyeron` outside vendor
      history/archives/receipts/negative assertions.

## D5 — v26.10.26 release mechanics

- [ ] Workspace `[package] version = "26.10.26"`.
- [ ] `docs/jira/v26.10.26/` window: DEFINITION-OF-DONE.md (this file),
      RECEIPTS.md with per-item commands + exits, INDEX.md.
- [ ] CHANGELOG entry under Unreleased/26.10.26.
- [ ] RECEIPTS-INDEX.md line for the milestone receipt.
- [ ] js/npm version sync flow run (gen-types + sync-version).
