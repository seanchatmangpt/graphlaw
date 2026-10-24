# DIALECT VERIFICATION lane — SHACL surface witnessed state (2026-10-24)

## Header

- Repo: `/Users/sac/graphlaw` (canonical checkout, one-tree fan-out)
- HEAD: `6da1d33a93f61068fa9b033bec353db78d335460` (branch `main`)
- Dirty disclosure: working tree carries 468 dirty paths (pre-existing
  release-closeout edits, present at lane start; verified as-is per dispatch).
  Lane-relevant inputs audited for provenance:
  - `tests/sql_rdf_conformance.rs` — MODIFIED in working tree (owned by
    another lane; its PASS below is on the modified file, disclosed not fixed).
  - `ontologies/sql-ast.ttl`, `ontologies/shapes/ast_governance.ttl`,
    `tests/wasm_governance_preflight.rs`, `priv/graphlaw.wasm`,
    `tests/op_differential.rs`, `tests/shacl_depth_guard_court.rs` — all
    git-CLEAN at HEAD.
- Pins (b3sum, `/opt/homebrew/bin/b3sum`):
  - `ontologies/sql-ast.ttl` — `91f58dd684fee8978687001b1140a67fb31b7f2c5a1e23b6ee7645ec34fa6d69`
    (sha256 at mutation-court time: `e35acceceaaa93424c891aee5b7a7678dd4c49d03f361adcca49676aca6e5982`)
  - `ggen/sql-dialect/statement_classes.ttl` — `464ef4e718b8ba41a42bd108f2b82c481f9b84fce91fb83d77d20e0b657531d3`
  - `ggen/sql-dialect/ontology/statements.ttl` — `1bd67ce601974e64b9cde3ff20170ddc80cae20d67f1e19371b5c00f0b2b2ee8`
  - `priv/graphlaw.wasm` — `58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a`
  - pin file `courts/code-partition/axioms.blake3` — `2add10ce9ff782e33b75de099cba8dc8a114ab759855f93a7be85abb18581506`
- Lane env: `CARGO_TARGET_DIR=/Users/sac/graphlaw/target-lane-dial-shacl`,
  `GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm` on every cargo run.
- Concurrency disclosure: 3 sibling DIALECT lanes (`target-lane-dial-sqlrdf`,
  `-turtle`, `-n3`) compiled concurrently throughout; wasm courts show
  >60 s per-test notices from CPU contention (220.17 s / 204.00 s totals).

## Discovery: SHACL in two roles

(a) **Shapes admission** — every SQL AST graph is SHACL-validated against
`ontologies/sql-ast.ttl` (21 `sh:NodeShape`, closure expressed via
minCount/maxCount cardinality + `sh:in` operator enum + rdf:List shapes per
rec section 4 — the file carries zero `sh:closed true`; the rec's own embedded
shapes use the same cardinality/enum style) BEFORE admission;
REFUSAL-01 `MalformedAstRefusal` is the violation carrier. Alias window:
`sql:selectList` / `sql:fromTable` / `sql:whereClause` remain legal
(`ontologies/sql-ast.ttl` lines 388-426, 111, 115, 144).
(b) **Standalone `shacl` registry op** — `src/abi.rs:325` ->
`src/abi/ops/governance.rs:462` (`Step::AdmitShacl`), two registry definition
rows (`src/registry/definitions.rs:463,793`), executed on host AND inside wasm.

## Courts — commands and verbatim results

All runs sequential; nothing re-run to green.

1. `CARGO_TARGET_DIR=... GRAPHLAW_WASM=... cargo test --test shacl_depth_guard_court`
   - `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s` — EXIT 0
2. `cargo test --test sql_rdf_dialect`
   - `test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.35s` — EXIT 0
3. `cargo test --test sql_rdf_laundering_corpus`
   - `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s` — EXIT 0
4. `cargo test --test sql_rdf_conformance` (file working-tree-modified, disclosed)
   - `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s` — EXIT 0
   - count note: file carries 10 literal `#[test]` tokens, 8 are real
     attributes; 2 are comment prose (lines 3, 415). 8 = exact.
5. `cargo test --test tc_mega_antivacuity`
   - `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` — EXIT 0
6. `cargo test --test refusal_details`
   - `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` — EXIT 0
7. `cargo test --test law_state`
   - `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s` — EXIT 0
8. `cargo test --test k6_hostile_lease`
   - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s` — EXIT 0
9. `cargo test --test wasm_governance_preflight` (engine court, real wasm module)
   - `test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 220.17s`
   - verbatim panic: `expected sql:HasWhereClause (sql:SelectStatementShape) / MinCount violation, got [("<http://graphlaw.org/sql#SelectStatementShape>", "<http://www.w3.org/ns/shacl#NodeConstraintComponent>")]`
   - instrumentation note: this run's EXIT echo is invalid (zsh has no bash
     `PIPESTATUS`; the echo printed `tail`'s 0). True exit derived from
     cargo's `error: test failed, to rerun pass --test wasm_governance_preflight`
     line: cargo exit 101. Runs 10-12 use corrected file-based capture.
10. `cargo test --test op_differential` (engine court)
   - `test result: FAILED. 15 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 204.00s` — EXIT 101 (witnessed)
11. `cargo test --test registry_artifacts` (engine court)
   - `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.89s` — EXIT 0 (witnessed)

## shape_mutation_court verdict (anti-vacuity over sql-ast.ttl)

- `bash scripts/shape_mutation_court.sh all` (17:53:41Z baseline sha256 `e35accec…`):
  **EXIT 5 — refused to judge.** Verbatim: `SHAPE-COURT: mutation mutate_m1
  did not apply — refusing to judge`. Root cause (witnessed): M1's regex
  requires `"LIKE" "ILIKE"` immediately before `)`, but the operator enum
  (`ontologies/sql-ast.ttl:490`) now ends `... "LIKE" "ILIKE" "->" "->>" )` —
  JSON-path tokens were added and the script mutation went stale. M2 was never
  attempted. Restore verified byte-exact (post-run digest = baseline).
- `bash scripts/shape_mutation_court.sh M2` (window 17:55:40Z–17:55:51Z):
  **EXIT 0 — PASS, 1 named court flip witnessed.** Verbatim:
  `SHAPE-COURT: mutate_m2 FLIP: fromless_select_refuses_via_shacl_shape_channel`
  / `SHAPE-COURT: PASS — 1 named court flip(s) witnessed`. Restore verified
  byte-exact (digest `e35accec…` = baseline). Shared-tree mutation window was
  11 s; sibling lanes compiling in that window would have baked the mutation
  in — disclosed for correlation.
- Honest verdict line: the gate is NOT vacuous (M2 flips a named court), but
  the M1 operator-enum shape mutation is UNJUDGED on this tree — its script
  mutation no longer applies. The script itself documents M1-class as E2
  coverage with a code-side backstop (`sql_ast.rs` `Operator::from_str`);
  operator-refusal behavior is courted at code layer
  (`disallowed_operator_token_is_refusal_01`, `d5_operator_dual_carry_string_and_iri_is_refusal_01` — both ok).

## AGGREGATE

- 11 cargo courts, 144 tests: **142 passed, 2 failed**, 0 ignored.
- Shape mutation court: M2 flip witnessed (gate live); M1 unjudged (stale
  mutation), refuse-to-judge fail-closed verified; both restores byte-exact.
- Shape admission surface (role a): GREEN on host, GREEN on laundering corpus,
  GREEN in registry op differential (SHACL refusal typed and IDENTICAL on both
  runtimes), GREEN on depth guard (typed `ResourceLimit`-class refusal naming
  `shacl_validation_depth`, no stack overflow at 1 MiB stack, cap boundary
  exact, cycles not over-refused).
- Standalone `shacl` op (role b): GREEN host/wasm parity
  (`law_shacl_refusal_is_typed_and_identical_on_both_runtimes` ok), GREEN
  registry-shape conformance (`i_committed_ttl_conforms_to_shapes`,
  `j2_model_shapes_reject_malformed_model_rows`, schema negative controls ok).
- Alias acceptance: `sql_rdf_dialect` 36/36 with alias-window courts —
  consistent with `docs/reference/sql-rdf-rec.md` section 4 shapes.

## Classification of non-greens

Typed SHACL refusals are the PRODUCT; these are the two test FAILURES, both
pre-existing at HEAD (all inputs git-clean; lane introduced nothing):

1. `wasm_governance_preflight::governance_preflight_refuses_whereless_scan_naming_select_statement_shape`
   — SHACL-adjacent report-attribution divergence, NOT a missed refusal: the
   wasm engine REFUSED the where-less scan (non-empty violation list naming
   `SelectStatementShape`) but attributed it to the `sh:node` edge
   (`NodeConstraintComponent`) instead of descending to
   `(sql:HasWhereClause, MinCountConstraintComponent)` as the court asserts.
   The supplied shapes graph (`ontologies/shapes/ast_governance.ttl`, git-clean,
   lines 75-85) binds `HasWhereClause` via `sh:node` and its own comment
   (lines 49-51) documents "sh:node folds nested results into
   NodeConstraintComponent" — the prebuilt wasm validator (`priv/graphlaw.wasm`,
   built 2026-10-24 01:08) reports the folded form; the court expects the
   descended form. Host-side equivalent shape courts pass. Product-vs-defect
   split: the refusal fired (fail-closed held); the typed-report granularity
   inside wasm is the divergence. Owner: wasm/engine lane, not this lane; NOT
   fixed (outside lease).
2. `op_differential::sniff_dialect_agrees_with_parse_dialect_for_every_serialization`
   — NOT a SHACL failure: dialect-sniff native/wasm divergence (native
   `{dialect: NTriples}` vs wasm `{dialect: Turtle}` for ntriples input).
   Both SHACL tests in the same court passed. Disclosed for the sniff/turtle
   sibling lane.

## Honesty line (courted vs prose)

- COURTED: closed-world admission of the 35 statement classes is witnessed at
  three rungs — inventory + shape-name presence + gates-refuse
  (`sql_rdf_conformance` `section3_2_inventory_is_35_classes…`,
  `section4_shape_names_present_and_gates_refuse`), shape-channel refusals on
  real malformed graphs (36 + 23 laundering/shape courts), and wasm/host parity
  of the typed refusal. Depth guard, registry-shape conformance, and the M2
  shape mutation flip are witnessed, not prose.
- PROSE / NOT courted on this run: (i) M1 operator-enum shape mutation is
  unjudged (stale script mutation; code-side backstop courted instead);
  (ii) per-kind execution coverage of all 35 kinds lives in
  `duckdb_statement_kinds` (SQL/PEG surface — sibling lane's rung, not run
  here); (iii) wasm SHACL report component-attribution granularity is courted
  and currently DIVERGENT (finding 1) — until that lands, "wasm SHACL reports
  name the violated property constraint" is prose, not witnessed;
  (iv) mutation anti-vacuity exists for `sql-ast.ttl` only — no mutation court
  exercises `ontologies/shapes/ast_governance.ttl` or `shacl-shacl.ttl`;
  (v) `chicago_validation` contains zero SHACL references (checked: not a
  SHACL court on this tree) and `parse_refusal_court` is parse-rung, not
  shape-rung.

## Falsifiers

- Re-run `bash scripts/shape_mutation_court.sh M2`: must flip
  `fromless_select_refuses_via_shacl_shape_channel`; a clean pass = gate dead.
- Fix `scripts/shape_mutation_court.sh` M1 regex for the `->`/`->>`-extended
  enum and re-run `all`: an M1 application that flips nothing would falsify
  the "code-side backstop courts the operator enum" claim.
- Host/wasm differential: run `wasm_governance_preflight` against a wasm
  artifact rebuilt from current HEAD — if attribution still reports
  `NodeConstraintComponent`, the divergence is in the current validator, not
  artifact staleness.
- `git show HEAD:ontologies/sql-ast.ttl | b3sum` vs the pin above: mismatch
  falsifies the as-is claim.

## Lane hygiene

Files written: this receipt only. No source, ontology, script, or git
mutations. `ontologies/sql-ast.ttl` verified byte-identical after both
mutation-court runs. Lane build root `target-lane-dial-shacl` left for
coordinator deletion at integration.

Closing b3sum (bytes above this line): afce6979a3263c2946638f49eb17b705412ecf5878f511b6b1b6f490da863482
