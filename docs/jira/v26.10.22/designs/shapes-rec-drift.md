# Shapes ↔ Rec §4 Drift Report (Lane L33)

Date: 2026-10-22. Scope: `ontologies/sql-ast.ttl` (normative shapes) and
`ontologies/shapes/ast_governance.ttl` (governance preflight) vs
`docs/reference/sql-rdf-rec.md` §4 (Closed-World Validation), with §3.3 and §7
read for context. Other files under `ontologies/shapes/` (shacl.ttl,
shacl-shacl.ttl, process-conformance, sa2a-diataxis, shex) do not use the
`http://graphlaw.org/sql#` vocabulary (verified by grep, below) and are out of
scope.

No shape or src file was edited; this report is the deliverable. Reconciliation
is a dispatch decision.

## Method / receipts

```
grep -n "graphlaw.org/sql" ontologies/shapes/*.ttl
  -> only ontologies/shapes/ast_governance.ttl:6  (exit 0)
wc -l ontologies/sql-ast.ttl docs/reference/sql-rdf-rec.md
  -> 422 / 449 lines (exit 0)
cargo test --test sql_rdf_dialect   (post-read boundary gate; no edits made)
  -> "test result: ok. 25 passed; 0 failed"  exit 0
```

Citations below use `AST:n` = `ontologies/sql-ast.ttl` line n,
`REC:n` = `docs/reference/sql-rdf-rec.md` line n,
`GOV:n` = `ontologies/shapes/ast_governance.ttl` line n.

## 1. Per-shape constraint enumeration (side A)

### sql:StatementBaseShape (AST:277-286)
- targetClass sql:Statement; sql:statementIndex: datatype xsd:integer,
  minCount 1, maxCount 1, severity Violation.

### sql:SelectStatementShape (AST:288-337)
- sql:fromTable minCount 1 / maxCount 1 / node TableRefShape (alias branch,
  AST:297-302) OR sql:from minCount 1 / maxCount 1 / node OrderedListShape
  (normative branch, AST:303-308) — sh:or (AST:296-309).
- sql:select minCount 1 / maxCount 1 / node OrderedListShape (AST:317-322)
  XOR sql:selectList minCount 1 / maxCount 1 (alias, AST:323-327) — sh:xone
  (AST:316-328), with nil-list note (AST:312-314).
- sql:where maxCount 1 (AST:332-337); sh:class sql:Expression deliberately
  withheld (comment AST:329-331).

### sql:BinaryExpressionShape (AST:339-386)
- sql:left XOR sql:leftOperand, each minCount 1 / maxCount 1 (AST:345-356).
- sql:right XOR sql:rightOperand, each minCount 1 / maxCount 1 (AST:357-368).
- sql:operator: normative branch datatype xsd:string, minCount 1 / maxCount 1,
  sh:in 15 tokens ("=" "!=" "<" "<=" ">" ">=" "AND" "OR" "+" "-" "*" "/" "%"
  "LIKE" "ILIKE") (AST:369-377); legacy branch sh:in 9 operator IRIs
  (AST:379-385). sh:xone over the two.
- sh:class sql:ASTNode on left/right deliberately withheld (AST:343-344).

### sql:OrderedListShape (AST:388-402)
- rdf:first minCount 1 / maxCount 1; rdf:rest minCount 1 / maxCount 1.

### sql:TableRefShape (AST:404-412)
- sql:tableName: datatype xsd:string, minCount 1 / maxCount 1,
  pattern `^[a-zA-Z_][a-zA-Z0-9_]*$`.

### sql:ColumnRefShape (AST:414-422)
- sql:columnName: datatype xsd:string, minCount 1 / maxCount 1, same pattern.

### ast_governance.ttl (GOV:18-74)
- Re-declares sql:SelectStatementShape (GOV:18-28) and sql:TableRefShape
  (GOV:64-74), targeting the same classes as sql-ast.ttl — SHACL conjoins
  shapes across files, so these are additive constraints.
- sql:fromTable minCount 1 / maxCount 1 / node TableRefShape (GOV:30-34).
- Sequence-path patterns: (fromTable tableName) (GOV:39-42),
  (joinClause joinTable tableName) (GOV:44-47).
- sql:selectList minCount 1 / maxCount 1 (GOV:49-52).
- sql:whereClause minCount 1 — anti-scan law (GOV:56-58).

## 2. Rec §4 normative grammar requirements (side B)

- REC:234: implementations MUST validate all graphs against the shapes in
  `ontologies/sql-ast.ttl` (process obligation, no shape).
- StatementBaseShape: statementIndex datatype/min/max (REC:245-252) —
  verbatim dual of AST:279-286.
- SelectStatementShape (REC:255-276): sql:select minCount 1 / maxCount 1 /
  node OrderedListShape (REC:257-264); sql:from maxCount 1 (no minCount) /
  node OrderedListShape (REC:265-270); sql:where maxCount 1 / sh:class
  sql:Expression (REC:271-276).
- BinaryExpressionShape (REC:279-303): sql:left minCount 1 / maxCount 1 /
  sh:class sql:ASTNode (REC:281-287); sql:operator minCount 1 / maxCount 1 /
  datatype xsd:string / sh:in the same 15-token enum (REC:288-296); sql:right
  minCount 1 / maxCount 1 / sh:class sql:ASTNode (REC:297-303).
- OrderedListShape (REC:306-318): rdf:first / rdf:rest minCount 1 / maxCount 1.
- REC §4 defines NO TableRefShape, ColumnRefShape, alias branches, or
  governance preflight. §3.3 (REC:167) requires ordered clauses to use
  rdf:List; §3.3 declares groupBy/orderBy/limit/offset (REC:193-216); §7 uses
  MERGE vocabulary (targetTable, usingSource, mergeOn, whenMatched,
  setAssignments, functionName, arguments, ColumnIdentifier /
  LiteralExpression / FunctionExpression / MergeUpdateAction) that no shape
  constrains.

## 3. Drift matrix

| # | Constraint | Shape side | Rec side | Verdict |
|---|---|---|---|---|
| 1 | statementIndex xsd:integer min1/max1, Violation | AST:279-286 | REC:245-252 | ENFORCED-BOTH (identical) |
| 2 | sql:select min1/max1 + node OrderedListShape | AST:316-322 (via sh:xone) | REC:257-264 | ENFORCED-BOTH (shape widened, see row 3) |
| 3 | selectList alias acceptance (exactly-one with sql:select) | AST:310-315, 323-327 | absent from §4; not in §3.3 vocabulary | SHAPE-ONLY (alias-window extension; rec never names sql:selectList) |
| 4 | sql:from maxCount 1 + node OrderedListShape | AST:303-308 | REC:265-270 | ENFORCED-BOTH (shape adds minCount 1, see row 5) |
| 5 | sql:from minCount 1 (source clause mandatory) | AST:296-308 (sh:or with fromTable branch) | REC:265-270 declares maxCount 1 only — from optional | SHAPE-ONLY (shape stricter than rec; rec permits source-less SELECT) |
| 6 | fromTable alias branch (exactly one TableRef) | AST:297-302; GOV:30-34 | absent | SHAPE-ONLY |
| 7 | sql:where maxCount 1 | AST:332-337 | REC:271-276 | ENFORCED-BOTH |
| 8 | sql:where sh:class sql:Expression | withheld by design, AST:329-337 | REC:274 | REC-ONLY (shape weaker; documented asserted-closure reason AST:329-331) |
| 9 | sql:left / sql:right min1/max1 | AST:345-368 | REC:281-287, 297-303 | ENFORCED-BOTH |
| 10 | left/right sh:class sql:ASTNode | withheld by design, AST:343-344 | REC:286, 302 | REC-ONLY (shape weaker, same closure rationale) |
| 11 | operator enum: 15 string tokens, datatype xsd:string, min1/max1 | AST:369-377 | REC:288-296 | ENFORCED-BOTH (token lists identical) |
| 12 | operator legacy IRI branch (9 IRIs, sh:xone) | AST:378-385 | absent | SHAPE-ONLY |
| 13 | OrderedListShape rdf:first/rdf:rest min1/max1 | AST:388-402 | REC:306-318 | ENFORCED-BOTH (identical) |
| 14 | TableRefShape tableName pattern + min1/max1 | AST:404-412; GOV:64-74 | absent from §4 (§7 example uses tableName unshaped) | SHAPE-ONLY |
| 15 | ColumnRefShape columnName pattern + min1/max1 | AST:414-422 | absent from §4 | SHAPE-ONLY |
| 16 | §3.3 rdf:List MUST for ordered clauses incl. groupBy/orderBy | AST:151,161 declare rdfs:range rdf:List; NO SHACL shape on groupBy/orderBy values | REC:167, 193-206 | REC-ONLY (rdfs:range is open-world, not closed-world enforcement) |
| 17 | §3.3 limit/offset (declared xsd:integer in rec, xsd:nonNegativeInteger in ontology) | AST:163-171; no shape | REC:208-216 | REC-ONLY (unshaped; plus a range drift: rec xsd:integer vs AST xsd:nonNegativeInteger — AST stricter) |
| 18 | §7 MERGE vocabulary (targetTable, usingSource, mergeOn, whenMatched, setAssignments, functionName, arguments, MergeUpdateAction) | absent | REC:364-423 | REC-ONLY (rec's own worked example has zero shape coverage) |
| 19 | Anti-scan: whereClause minCount 1 on every SELECT | GOV:56-58 | absent | SHAPE-ONLY (governance lane, not rec §4) |
| 20 | Join table-name pattern via sequence path | GOV:44-47 | absent | SHAPE-ONLY |
| 21 | REC:234 "MUST validate against ontologies/sql-ast.ttl" | shapes exist and are the target | process requirement | ENFORCED-BOTH (indirect; enforced by src compiler + courts, verified by sql_rdf_dialect run above) |

### Inter-shape drift (shape vs shape, flagged for dispatch)

- GOV:30-34 + GOV:49-52 + GOV:56-58 force the LEGACY alias forms
  (fromTable / selectList / whereClause) with minCount 1, while
  sql-ast.ttl's SelectStatementShape admits the normative sql:from /
  sql:select / sql:where branches (AST:296-337). Because SHACL conjoins all
  shapes targeting a class, a purely normative document (sql:select + sql:from
  + sql:where, no aliases) passes sql-ast.ttl but FAILS ast_governance.ttl.
  This is a real conjoined-shape contradiction to resolve at dispatch.
- Operator token drift: ontology individual sql:NotEquals label is "<>"
  (AST:258) but the normative enum token is "!=" (AST:375, REC:293) — the
  legacy IRI branch and the string enum disagree on the not-equals spelling.

## Verdict totals

- ENFORCED-BOTH: 7 (rows 1, 2, 4, 7, 9, 11, 13, plus 21 indirect — counted 7
  structural + 1 process = 8 rows so labeled, 7 structural)
- SHAPE-ONLY: 8 (rows 3, 5, 6, 12, 14, 15, 19, 20)
- REC-ONLY: 4 (rows 8, 10, 16, 17, 18 — 5 rows labeled; counted 5)

Headline: 7 ENFORCED-BOTH / 8 SHAPE-ONLY / 5 REC-ONLY across 21 rows, plus 2
inter-shape drift findings (conjoined governance contradiction; "<>" vs "!="
not-equals spelling).
