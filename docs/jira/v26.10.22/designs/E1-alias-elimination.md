# E1 — alias-window elimination (design record)

Condensed from the E1 spec + census lanes. Maps to SQLRDF-16 row E1.
Falsifier court `tests/sql_rdf_laundering_corpus.rs` already landed this
window (pre-elimination regression net).

## Inventory (census)
~200 alias touches: reader fallbacks (src/sql_ast.rs:863-866, 833-866 operands,
987-1016 from, 1049-1060 select, 1069-1081 where, 1096-1108 having, read_operator
950-964 + from_iri 105-124); shape branches (ontologies/sql-ast.ttl:111-158 decls,
257-265 operator IRIs, 290-383 sh:or/sh:xone groups); **hard consumer found:
`ontologies/shapes/ast_governance.ttl` is written entirely in alias vocabulary**
(HasFromTable/HasSelectList/HasWhereClause + sequence paths) and wasm governance
courts assert on its shape IRIs — rewrite `sh:path` values only, keep shape IRIs
stable. In-tree fixtures/docs: ~15 files (sql_rdf_dialect.rs 52 hits is the
riskiest — it is the only exerciser of the dual-carry refusal semantics), plus
`.claude/agents/ontology-author.md` (says aliases "stay legal") and
docs/DIALECTS.md.

## Deprecation window: ZERO commits
Grep receipt: zero external consumers of any alias predicate outside the repo's
own code/tests/fixtures/docs — all migrated in one commit. Rec text uses only
normative names (`sql:alias` at rec :375 is the §7 output-alias attribute, NOT
in the window). No ontology IRI version bump owed beyond an owl:versionInfo
note + CHANGELOG.

## Rewrite order: ONE commit, both stores
Rationale: reader-first interim fails closed (alias docs refuse at reader),
shapes-first interim FAILS OPEN (alias docs pass SHACL, reader still accepts —
the gate silently loosens). Never shapes-last.
1. sql-ast.ttl: delete alias decls, collapse sh:or/xone groups to single
   normative branches, delete operator IRI individuals + IRI enum branch.
2. ast_governance.ttl: rewrite paths to normative, IRIs stable.
3. src/sql_ast.rs: delete fallback halves (from_iri, IRI branch of
   read_operator, fromTable/selectList/whereClause/havingClause/left|rightOperand
   reads) and ALL SIX dual-carry refusal blocks (they become structurally
   unconstructible).
4. Mechanical rewrite of the ~15 fixture/doc files; laundering-corpus cases
   convert from "carries both" refusals to unknown-property/missing-clause
   refusals (needle text changes — court file rewritten, not appended).
5. ontology-author agent instructions + DIALECTS.md.

## Deletable defense code (exists only because two names exist)
Six dual-carry refusal sites; .or_else fallback chains; E2 decoy countermeasure
(:988-996 + dual read); E5 dual-key operand handling; E1/E3 mixed-projection
and duplicate-alias handling. NOT deletable: single_object_checked (guards
normative single-cardinality), from_token enum refusal, discriminator-count
check, scope-integrity refusals.

## Falsifier (SQLRDF-16 row, hardened)
Law R: after E1, one name per clause; alias predicate ⇒ SHACL minCount failure
before the reader, or REFUSAL-01 at the reader — no reader branch consults an
alias at all. Kill condition: a laundering doc (benign normative + hostile
alias) still admits, or an external consumer is produced with a grep receipt
(§3 receipt satisfies the second arm preemptively). Anti-vacuity: each corpus
refusal must flip under scripts/mutation_court.sh.

## Residual risks
ast_governance shape-IRI stability (wasm courts assert by IRI); embedded-TTL
string fixtures need manual needle re-checks; corpus semantics change = court
file rewrite per window law.
