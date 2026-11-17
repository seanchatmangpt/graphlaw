# N3 / Rules Engine Standing (v26.11.17)

Static read (no builds). Sources: src/law.rs:450-760, proof_lease.rs, hooks.rs, rules/, tests/n3_*.rs, docs/CHICAGO_N3_DATALOG.md, docs/reference/n3-audit-100-report.md (2026-10-26).

## 1. Ownership
GraphLaw owns no reasoner: N3 parse/reasoning is vendored purrdf-n3 3.0.1; Datalog is PurRDF. `src/rules/` is only builtins.rs (DuckDB builtin returning ExecutionReceipts) + 3-line mod.rs. Entry: `law::reason_n3_bounded` (law.rs:693), via Step::DeriveN3 and ABI op `n3`.

## 2. Budget model (law.rs:456-486)
| limit | value |
|---|---|
| n3_iterations | 4,000,000 |
| n3_term_bytes | 64 KiB |
| n3_total_bytes | 256 MiB |
| derived facts | 500,000 |
| n3_match_steps | 200,000 per premise search |
| n3_total_steps | 50,000,000 |
| n3_backward_depth | 32 |
| n3_backward_solutions_per_goal | 1,024 |
Term depth cap 64 (parse-time). Dispatch is census-ordered: first `ReasonerLimit::ALL` member in `limits_reached` becomes `N3Error::Limit`; no catch-all. Incomplete with no census limit => `Refused` with the engine summary. Counter limits report observed == max; measure-at-the-door limits use sentinel max+1. Courts: n3_budget_cliff_court, n3_limit_census_court. Registry (capability-registry.json:324) and abi-manifest (:102) both read 4000000.

## 3. Fuse and halt
`{ premise } => false .` is a semantic verdict: `reason_n3_bounded` checks `result.fuse` and returns `N3Error::Fuse` (RefusalKind::Fuse; ABI details.code "Fuse"); never silent Ok(""). Courts: n3_fuse_court, n3_halt_latency_court (parse refusal < fuse < resource limit), n3_boundary_fuzz (depths 0/1/63/64/65/1000, unicode). Hooks (hooks.rs): `kh:effect "refuse"` is a pack-declared fuse -> Verdict::Refuse; MAX_ROUNDS 64, MAX_FIRINGS 10,000, quad ceiling; exceeding is refused, not truncated; BLAKE3 kh:Firing marker gives at-most-once per lineage.

## 4. Hypothesis isolation
`log:conclusion` nested reasoner; n3_hypothesis_isolation_court pins quarantine (canonical_form byte-identical reruns; contradicting hypothesis => Ok with empty derived set; negative twin proves detection). n3_transition_isolation_court (P16): refused transition leaves parent LawState byte-identical.

## 5. Proof lease
`verify_leased_proof` BLAKE3-addresses the source first; mismatch = LeaseDigestMismatch and check_proof never runs. No clocks. Courts: n3_proof_lease_court, n3_proof_pipeline_court (closes F13).

## 6. Courts
n3_boundary_fuzz, budget_cliff, corpus, fuse, halt_latency, hypothesis_isolation, limit_census, proof_lease, proof_pipeline, quantifier (closes F15/C10), transition_isolation; chicago_n3_datalog (6; PASS is not full W3C N3/Souffle/OpenRuleBench conformance; no CI step names it); parse_refusal_court (12; pins only colon-ful unknown-prefix path).

## 7. Open audit findings (n3-audit-100, 2026-10-26)
14 confirmed (12 major, 2 minor), 38 refuted (30 are verifier deaths, unadjudicated). Per the report; no closure evidence seen unless stated.
| id | location | claim |
|---|---|---|
| C1 | reasoner.rs:5691 | blank in builtin RHS vacates constraint (unsound firing) |
| C2 | reasoner.rs:5476 | math:quotient wrong xsd:integer on big ints |
| C3 | reasoner.rs:5613 | INF/NaN minted as invalid xsd:decimal |
| C4 | parser.rs:1238 | bare words fabricate example.org IRIs |
| C5 | src/law.rs:590 | three limit identities misread as n3_iterations |
| C6 | emit_json.rs:169 | registry pins a cap no code enforces |
| C7 | parser.rs:993 | path verbs drop chain triples |
| C8 | lexer.rs:334 | @-words become Lang without validation |
| C9 | vendor README:35 | promised scope-escape refusal missing |
| C10 | DIALECTS.md:285 | quantifier surface never executed; partly CLOSED by n3_quantifier_court |
| C11 | lexer.rs:316 | @prefix split mis-lexes |
| C12 | reasoner.rs:3431 | explain/checker trust boundaries diverge |
| C13 | unresolved | truncated entry |
Verdict: N3 is sound only on the court-pinned happy-path subset; route others through a human/court gate until C1-C3 fixed.

## 8. New observation
src/law.rs:458 reads `N3_MAX_ITERATIONS = 4_000_000; // SKV-REVERT-TEST-B: budget disabled` — a stray test-revert marker; default is 1,000,000 so the const is 4x. Registry and manifest agree, so no drift, but remove the comment or document the raise. Introduced in 6a6a863.
