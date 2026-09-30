# Chicago 80/20 N3 + Datalog Compliance Court

## Purpose

This court is the bounded v26.10.1 smoke test for GraphLaw's two rule-language
authorities. It deliberately tests the small set of invariants that catch a
large fraction of semantic regressions without claiming exhaustive conformance.

GraphLaw does not implement either semantics locally:

- Eyeron owns N3 parsing/reasoning.
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

Those belong to larger v26.10.1 corpus and benchmark courts.

## Execution

The dedicated CI step is:

cargo test --all-features --test chicago_n3_datalog

The normal full test suite also executes the same integration test.
