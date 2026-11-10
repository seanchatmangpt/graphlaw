# Chicago 80/20 N3 + Datalog Compliance Court

## Purpose

This court is the bounded 80/20 smoke test (introduced in v26.10.1; re-read against
`tests/chicago_n3_datalog.rs` on 2026-11-09, 6 `#[test]` functions) for GraphLaw's two rule-language
authorities. It deliberately tests the small set of invariants that catch a
large fraction of semantic regressions without claiming exhaustive conformance.

GraphLaw does not implement either semantics locally:

- purrdf::n3 owns N3 parsing/reasoning.
- PurRDF owns Datalog compilation and least-fixpoint evaluation.

The court therefore verifies GraphLaw's exact admitted authority path rather
than introducing a second reasoner.

## Canonical anchors

N3 cases are distilled from the W3C N3 language and semantics:

- https://w3c.github.io/N3/spec/
- https://w3c.github.io/N3/spec/semantics.html

Datalog recursion uses the standard transitive-closure shape documented by
Soufflé:

- https://souffle-lang.github.io/tutorial

The source examples are reduced to GraphLaw's RDF-triple Datalog surface. The
semantic invariant is preserved: direct edges seed reachability and recursive
reachability computes the least fixpoint, including cycles.

## 80/20 court

The court requires all of the following:

1. N3 implication fires when its premise holds.
2. Chained N3 implications reach closure.
3. Reordering equivalent N3 statements does not change the RDFC-canonical result.
4. Datalog recursive transitive closure matches the exact expected least fixpoint.
5. Cycles derive self-reachability where the canonical closure requires it.
6. Reordering Datalog facts and rules does not change the sorted result.
7. N3 and Datalog agree on the same recursive reachability problem.
8. Malformed N3 and malformed Datalog requests fail closed.

## Standing ceiling

PASS means:

Representative canonical N3/Datalog semantics exercised through the pinned
upstream authorities are intact on the exact tested GraphLaw subject.

PASS does not mean:

- complete W3C N3 conformance;
- complete Soufflé language compatibility;
- complete OpenRuleBench coverage;
- support for every N3 builtin or Datalog extension;
- a performance superiority claim.

Those belong to larger corpus and benchmark courts, none of which are part of this court.

## Execution

Run the court alone with:

    cargo test --all-features --test chicago_n3_datalog

As of 2026-11-09 `.github/workflows/` has no step dedicated to this court by name; the
full-suite jobs in `ci.yml` (`cargo test --locked --all-targets --features
pack-tools,abi,evidentiary-ring`) compile and execute it as part of the whole suite.

The 6 tests map to the court list above: implication/chaining (1-2) in
`w3c_n3_implication_and_chaining_fire`, order invariance (3) in
`n3_semantics_are_invariant_to_statement_order`, exact least fixpoint including the c/b
cycle (4-5) in `souffle_transitive_closure_example_has_exact_least_fixpoint`, Datalog order
invariance (6) in `datalog_fixpoint_is_invariant_to_fact_and_rule_order`, N3/Datalog
agreement (7) in `n3_and_datalog_agree_on_the_shared_recursive_closure`, fail-closed (8) in
`malformed_n3_and_malformed_datalog_fail_closed`.
