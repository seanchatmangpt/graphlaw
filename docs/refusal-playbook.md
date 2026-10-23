# Refusal Playbook (sql-rdf surface)

User-facing troubleshooting for the refusals the sql-rdf kernel actually emits.
Every message below was captured by triggering the refusal FOR REAL through
`compile_turtle` / `eval_ast_query` / `builtin_eval_sql` / `canonical_ast_graph_bytes`
on real frozen-duckdb connections — not invented. Probe source:
`examples/refusal_probe.rs` (see [Receipts](#receipts)).

Companion register: [refusals.md](refusals.md) (stable codes, append-only).
Conformance text: `docs/reference/sql-rdf-rec.md` (§4 statement shapes, §1.3
scope boundary, §1.4 receipt law, §5 refusal taxonomy).

## Troubleshooting table (symptom → refusal → cause → fix)

Verbatim messages from the 2026-10-22 probe run (exit 0, 13/13 MATCH). Where a
message embeds document-specific IRIs, `<IRI>` / `<col>` / `<n>` marks the
user-supplied part.

| # | Message you see (verbatim, probe #) | Code | Cause | Fix |
|---|---|---|---|---|
| 1 | `SHACL AST structural violation: [REFUSAL-01] MalformedAstRefusal:statement http://example.org/queries#Q is missing required sql:statementIndex (section 4 StatementBaseShape)` (P1) | REFUSAL-01 | The SelectStatement node lacks exactly-one non-negative `xsd:integer` `sql:statementIndex` (§4 StatementBaseShape). | Add `sql:statementIndex 1 ;` to the statement node named in the message. |
| 2 | `SHACL AST structural violation: [REFUSAL-01] MalformedAstRefusal:xsd:integer literal 'not-a-number' on _:gts_00000000000000000000000005_0 does not parse as an integer` (P2) | REFUSAL-01 (`LITERAL_NOT_INT`, RefusalCode = 36) | `sql:literalValue` typed `xsd:integer` whose lexical form is not a valid i64. Never silently coerced to 0. | Fix the lexical form to a valid integer, or retype the literal `xsd:string`. |
| 3 | `SHACL AST structural violation: [REFUSAL-01] MalformedAstRefusal:sql:joinType IRI 'http://graphlaw.org/sql#FullJoin' is not sql:InnerJoin or sql:LeftJoin; an unknown join-type IRI refuses typed, it never silently downgrades to Inner` (P3) | REFUSAL-01 (`JOIN_TYPE_IRI`, refusal arm 37) | `sql:joinType` IRI outside the {InnerJoin, LeftJoin} enum. A *missing* `sql:joinType` defaults to Inner (pinned, not a refusal). | Use `sql:InnerJoin` or `sql:LeftJoin`. |
| 4 | `compile refusal: http://graphlaw.org/sql#ConnectStatement [REFUSAL-02] UnimplementedStatementRefusal: not implemented by this DuckDB build` (P4) | REFUSAL-02 | `ConnectStatement` (and identically `DisconnectStatement`) are recognized statement kinds with no evaluation path in this build. | Rewrite the query with admitted kinds (SelectStatement, MergeIntoStatement). |
| 5 | `SHACL AST structural violation: [REFUSAL-03] ScopeIntegrityRefusal:aggregate in a WHERE/JOIN/GROUP BY position is refused (section 1.3/5); re-scope the expression` (P5) | REFUSAL-03 | An aggregate (SUM/COUNT/...) in WHERE, JOIN, or GROUP BY violates the §1.3 process/view boundary. Evasion shapes (alias-indirection, nested aggregates) refuse the same way. | Move the aggregate into SELECT or HAVING as the shapes permit. |
| 6 | `SHACL AST structural violation: [REFUSAL-03] ScopeIntegrityRefusal:column 'latency_ms' is sql:ProcessScope but sits in the projection list (section 1.3: a selected column crosses into the consumer's view)` (P6) | REFUSAL-03 | A column carries `sql:scope sql:ProcessScope` but appears in `sql:selectList`. | Drop the `sql:scope sql:ProcessScope` marker from projected columns (ViewScope or unscoped is lawful in projection). |
| 7 | `AST resolution failure: REFUSAL-04 CapabilityRefusal: reader is not document-backed — builtin eval admits only graph-backed queries compiled via compile_turtle (SQLRDF-17)` (P7) | REFUSAL-04 | `builtin_eval_sql` received a reader whose `query_document_turtle()` is `None`. | Supply the admitted Turtle document and compile via `compile_turtle` before builtin eval. |
| 8 | `no canonical graph bytes — hand-built CompiledQuery must use compile_turtle (query http://example.org/handbuilt)` (P8) | NO-CANONICAL-GRAPH | A `CompiledQuery` carrying neither `receipt_state` nor `canon_graph_bytes` reached `eval_ast_query`. Reachable from outside the crate via the public `SelectStatement::compile()`, which does not pin the canon anchor. | Compile with `compile_turtle`, which pins the §1.4 receipt anchor at compile time. |
| 9 | `Receipt generation error: admission seal mismatch on query <IRI>: CompiledQuery is not in its admitted compile state; re-compile via compile_turtle (sole-admission path)` (P9) | ReceiptError (admission seal) | A sealed field (`sql`, `receipt_state`, `canon_graph_bytes`, ...) was mutated after `compile_turtle`. Eval refuses before any engine call — never a generic DB error. | Re-compile from the Turtle source. Bind different parameters via `compiled.params` (unsealed, lawful); to change the query, change the graph. |
| 10 | `Receipt generation error: AST graph parse refused: RdfDiagnostic { severity: Error, code: "native-codec-parse", message: "SPARQL lex error at byte 19: empty language tag", ... location: ... line: Some(1), column: Some(20) ... }` (P10) | ReceiptError (canonicalization) | The Turtle handed to `canonical_ast_graph_bytes` (or the compile-time canonicalization) is malformed. The embedded `RdfDiagnostic` carries line/column. | Fix the Turtle at the reported line/column. See finding F-1 for the message-form defect. |
| 11 | `Unsupported operator or function IRI: http://example.org/functions#MEDIAN` (P11) | (none — unclassified gap) | Function/operator IRI outside the enum. Carries **no** `[REFUSAL-nn]` marker today (pinned known gap in `docs/refusals.md`). | Same recovery class as REFUSAL-01: use an operator from the §4 enum or a function from `http://graphlaw.org/functions#`. |
| 12 | `SHACL AST structural violation: Node <<http://x>>: [REFUSAL-01] MalformedAstRefusal: a SelectStatement MUST contain exactly one projection list (sql:select, or legacy alias sql:selectList — never zero, never both; an empty rdf:nil list is ne (+1 more violations)` (P12) | REFUSAL-01 | Direct `validate_ast_shacl` callers get the summary form: first violation text (truncated at the 192-byte ident buffer) plus a count of the rest. | Fix violations one at a time, re-running validate after each fix; see finding F-8. |

## Grade: does each message say WHAT was wrong / WHERE / WHAT TO DO?

| # | WHAT | WHERE | WHAT TO DO | Grade |
|---|---|---|---|---|
| 1 | yes | yes (statement node IRI) | partial (shape section only, no remedy syntax) | PASS-partial |
| 2 | yes (lexical + datatype) | yes (lexical + blank-node id) | no remedy hint | PASS-partial |
| 3 | yes | yes (offending IRI) | yes (names both legal values) | PASS |
| 4 | yes (kind IRI) | yes | no explicit remedy | PASS-partial |
| 5 | yes | **no** (no node/expression identity) | yes ("re-scope the expression") | PASS-partial |
| 6 | yes | yes (column name) | yes (§1.3 reason inline) | PASS |
| 7 | yes | n/a (capability, not data) | yes (names compile_turtle) | PASS |
| 8 | yes | yes (query IRI) | yes (names compile_turtle) | PASS |
| 9 | yes | yes (query IRI) | yes (names re-compile) | PASS |
| 10 | partial (Debug-form struct dump) | yes (line/column inside the dump) | no plain-language remedy | FAIL (see F-1) |
| 11 | partial ("operator or function" ambiguous) | yes (the IRI) | no | FAIL (see F-5) |
| 12 | yes | yes | partial (truncated first violation) | PASS-partial (see F-8) |

## Findings for a src lane (Display strings needing improvement)

Display strings live in `src/duckdb_eval.rs` and, post SQLRDF-18 switchover,
in `src/sql_ast/refusals.rs` (the `AstError` Display table; anchor correction
2026-10-22, lane V-L11 — previously cited as `src/sql_ast.rs`) — not edited
here per lane ownership; documented as-is above.

- **F-1 (#10)**: `EvaluatorError::ReceiptError` from
  `canonical_ast_graph_bytes` embeds the purrdf `RdfDiagnostic` **Debug**
  format (`RdfDiagnostic { severity: ..., gts_quad_index: None ... }`). A user
  must wade through struct noise to find `line: Some(1), column: Some(20)`.
  Emit Display, or extract `message` + line/column into a plain string.
- **F-2 (#5)**: the aggregate-in-WHERE REFUSAL-03 message names no node or
  expression identity — in a multi-statement document the user cannot locate
  the offending aggregate. Other REFUSAL-03 arms prove `RefusalContext.ident`
  can carry the node/column; populate it here.
- **F-3 (#4)**: REFUSAL-02 says what is unimplemented but not what IS
  admitted — append "admitted kinds: SelectStatement, MergeIntoStatement".
- **F-4 (#2)**: LITERAL_NOT_INT / LITERAL_NOT_FLOAT carry no remedy clause;
  add "fix the lexical form or retype as xsd:string / xsd:double".
- **F-5 (#11)**: unclassified unknown function IRI (`UnsupportedTerm`): no
  `[REFUSAL-nn]` marker (pinned gap in `docs/refusals.md`), and the prefix
  "Unsupported operator or function IRI" does not say which of the two it was.
- **F-8 (#12)**: multi-violation summary truncates the first violation at the
  192-byte ident buffer mid-word ("is ne") and hides the remaining N.
  Acceptable for `compile_turtle` (first violation suffices to act on), but a
  verbose mode or non-truncating iterator would remove the validate→fix loop
  cost for direct `validate_ast_shacl` callers.
- **F-9 (#2, #5)**: blank-node ids in messages (`_:gts_0000...05_0`) are
  stable but meaningless to users; predicate/source-line context would help.

(Graded PASS, no change needed: #3, #6, #7, #8, #9.)

## Receipts

Real probe run: real frozen-duckdb connections, real refusals, no mocks, no
sleeps. Probe source: `examples/refusal_probe.rs` (renamed from
`l85_refusal_probe.rs` post-receipt, lane L96 —
disclosed: examples/ was outside the stated file-ownership list; it is the
probe vehicle the task asked for).

```
$ cargo run --example refusal_probe
=== REFUSAL-01 missing statementIndex ===
SHACL AST structural violation: [REFUSAL-01] MalformedAstRefusal:statement http://example.org/queries#Q is missing required sql:statementIndex (section 4 StatementBaseShape)
[exit-class MATCH]
...
=== ReceiptError canonicalization refused (malformed Turtle) ===
Receipt generation error: AST graph parse refused: RdfDiagnostic { severity: Error, code: "native-codec-parse", message: "SPARQL lex error at byte 19: empty language tag", ... }
[exit-class MATCH]
...
ALL REFUSAL PROBES MATCHED
EXIT=0
```

13/13 probes MATCH, exit code 0. Every message in the table above is copied
from this run. Reproduce with:

```
cargo run --example refusal_probe   # exit 0 iff every probe fires its expected refusal class
```

Environment note: the first attempt with a fresh lane target dir
(`CARGO_TARGET_DIR=target-lane85`) failed to build the dependency
`frozen-duckdb-sys` (build-script compile error, pre-existing state of the
`/Users/sac/frozen-duckdb` working tree, not this repo). The probe was run
against the shared `target/` build cache instead; disclosed per the
compile-freeze disclosure rule.
