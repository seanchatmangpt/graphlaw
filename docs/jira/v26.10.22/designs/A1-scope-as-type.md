# A1 — scope-as-type via entailment closure (design record; = SQLRDF-13 implementation plan)

Condensed from the A1 spec lane. Maps to SQLRDF-16 row A1; depends on E1.

## Closure (Option 2, LawState-safe)
- New helper (e.g. `src/shacl_closure.rs`): `validate_with_rdfs_closure(data, shapes,
  ontology_axioms_ttl)` — parse axioms, union data+axioms in a THROWAWAY store,
  `purrdf::entail::materialize(.., Rdfs)`, validate the closure, drop it. The closure is
  never returned or stored.
- **LawState hazard (the scoping rule):** LawState identity = blake3(RDFC-1.0 canon)
  (`src/law.rs:731-748`). Folding the closure into admitted state would (a) change every
  receipt digest, (b) grow state monotonically. Structural rule: closure lives in a local
  binding inside the match arm; `from_dataset` unreachable from the closure path (contrast
  `Step::EntailRdfs`, a deliberate state-transforming step). Canon digests bit-identical.
- Call sites: `src/sql_ast.rs::validate_ast_shacl` + `src/law.rs::Step::AdmitShacl`.
- Ontology: un-withhold `sh:class sql:Expression` (sql:where) + `sh:class sql:ASTNode`
  (operands); delete the two alias-window comment blocks.
- Falsifier court: `sql:where` pointing at a literal → `ShaclViolation` with `sh:message`
  at admission (not a deep extract error).

## §1.3 as shape data
`sh:pattern`-on-scope-IRI is position-blind (RDFS closure cannot know a node's clause
position; SHACL paths can't recurse unbounded). So: derive positional classes via SHACL
TripleRules (purrdf-shapes 2.0.2 rules engine, `sh:order`-scheduled, fixpoint-bounded) in
the throwaway dataset: `sql:ViewExpression` / `sql:ProcessExpression` derived from clause
roots (`sql:where|joinCondition|groupBy|having` → Process; `sql:select|orderBy` → View)
and propagated down operand/argument edges. Then §1.3 constraints are trivial shape data:
view-surface column shape `sh:not hasValue sql:ProcessScope`; predicate-surface column
`sh:not hasValue sql:ViewScope`. Rules run in the throwaway dataset only.

## Court migration (six REFUSAL-03 courts)
- Delete Rust checks, keep courts: aggregate-in-where, alias-indirection, ProcessScope-in-
  projection, aggregate-in-groupBy, aggregate-in-joinCondition (positional forbidden-marker
  rules).
- Property-stuffing (columnName+function): plain `sh:xone` — shape-expressible NOW, no
  closure needed.
- Nested-aggregate: shape-expressible (aggregate-under-aggregate rule); migrate last,
  optionally keep the two-line Rust arm as defense-in-depth.
- Alias-pair co-occurrence refusals (REFUSAL-01): `sh:xone` over alias pairs — separate
  cleanup, independent of closure.
- **Message-text hazard:** courts pin "REFUSAL-03"; when a check migrates to shapes the
  tag must move into `sh:message` (`"[REFUSAL-03] ..."`) — explicit acceptance criterion.

## Performance
Cache the ontology-axioms closure process-wide keyed by blake3(axioms_ttl) (it is static);
per admission, data ∪ O_cl + type-lattice propagation of asserted types (linear, not a
fixpoint). Prefer subClassOf-only regime over full Rdfs (avoids domain/range inference over
data). Bonus: reuse the parsed dataset store across validate/extract (currently parsed
twice). TripleRule fixpoint bounded by purrdf (fails closed, typed).

## Sequencing
(1) closure helper + scoping + un-withhold + falsifier; (2) positional TripleRules + §1.3
shapes; (3) court migration + tag move; (4) alias-pair shapes (independent); (5) O_cl cache.
