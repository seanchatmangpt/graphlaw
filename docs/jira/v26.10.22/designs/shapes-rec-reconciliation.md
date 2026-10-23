# Shapes ↔ Rec §4 Reconciliation Plan (Lane L55)

Date: 2026-10-22. Adjudicates
`docs/jira/v26.10.22/designs/shapes-rec-drift.md` (L33) into a mechanically
executable plan. Design lane — read-only on `ontologies/`, `src/`,
`docs/reference/`; this file is the sole deliverable. No shape, src, or rec
file was edited.

## Method / verification receipts

```
grep -n "graphlaw.org/sql" ontologies/shapes/*.ttl
  -> ontologies/shapes/ast_governance.ttl:6  (exit 0)
grep -rn '"<>"' src/ | head -5
  -> src/dialect.rs:618 ; src/sql_ast.rs:541 ; src/sql_split/ast.rs:53  (exit 0)
grep -rn 'NotEquals' src/ | head -8
  -> src/sql_ast.rs:521,541,562,580 ; src/sql_split/ast.rs:33,53,74,92  (exit 0)
```

Line conventions follow L33: `AST:n` = `ontologies/sql-ast.ttl` line n,
`REC:n` = `docs/reference/sql-rdf-rec.md` line n, `GOV:n` =
`ontologies/shapes/ast_governance.ttl` line n.

## 0. Adjudication principles

1. REC:234 makes `ontologies/sql-ast.ttl` the normative shape set; §4
   (REC:236-319) is its verbatim inline copy. Where §4 and sql-ast.ttl
   disagree, the shape file must not be LOOSER than §4, but it MAY be
   stricter — the alias window is repo-sanctioned strictness the rec text is
   silent on (REC never names `sql:selectList`, `sql:fromTable`,
   `sql:whereClause`).
2. Aliases stay legal (SQLRDF-01 alias window; AST:80-82, AST:110,
   AST:310-315 comments). Any reconciliation that would reject alias
   documents is out of bounds.
3. Rec text is authoritative for the normative surface (string token enum,
   clause names). Governance preflight (GOV) is a Phase-2 lane with its own
   intent (GOV:8-12), not §4 scope.

## 1. Per-row verdicts (21 rows)

| # | Constraint | Verdict | Instruction for the executing lane | Grounding |
|---|---|---|---|---|
| 1 | statementIndex xsd:integer min1/max1 Violation | ACCEPT-DRIFT | None — no drift; AST is a verbatim dual of REC. | AST:279-286; REC:245-252 |
| 2 | sql:select min1/max1 + node OrderedListShape | ACCEPT-DRIFT | None — normative branch identical to REC; the widening is the sh:xone alias branch adjudicated at row 3. | AST:310-328; REC:257-264 |
| 3 | selectList alias acceptance (xone with sql:select) | RECONCILE-REC | Documented intent; rec authoritative for the normative surface. Executing lane verifies the existing comments (AST:290-291, 310-315). No shape change. | AST:310-328; REC:257-264 (selectList absent) |
| 4 | sql:from maxCount 1 + node OrderedListShape | ACCEPT-DRIFT | None — normative branch identical; widening adjudicated at row 6. | AST:303-308; REC:265-270 |
| 5 | sql:from minCount 1 (source mandatory) | ACCEPT-DRIFT | None — shape stricter than rec (rec declares maxCount only, REC:265-270); fail-closed posture, and REC:234 makes sql-ast.ttl the operative law. Comment in place (AST:292-295). | AST:292-308; REC:265-270 |
| 6 | fromTable alias branch (exactly one TableRef) | RECONCILE-REC | Alias-window extension; comments present (AST:290-291). No shape change. | AST:296-302; rec silent |
| 7 | sql:where maxCount 1 | ACCEPT-DRIFT | None — identical both sides. | AST:332-337; REC:271-276 |
| 8 | sql:where sh:class sql:Expression | RECONCILE-REC | Shapes deliberately withhold per the asserted-closure rationale (data documents lack rdfs:subClassOf axioms). Executing lane verifies the comment convention (AST:329-331). DO NOT add sh:class — it would reject normative documents. | AST:329-337; REC:274 |
| 9 | left/right min1/max1 | ACCEPT-DRIFT | None — identical both sides. | AST:345-368; REC:281-303 |
| 10 | left/right sh:class sql:ASTNode | RECONCILE-REC | Same asserted-closure withholding as row 8; comment present (AST:343-344). No shape change. | AST:341-368; REC:286, 302 |
| 11 | operator 15-token string enum | ACCEPT-DRIFT | None — token lists identical (AST:375 = REC:293). | AST:369-377; REC:288-296 |
| 12 | legacy operator IRI branch (9 IRIs, sh:xone) | RECONCILE-REC | Alias window. Optional: one comment line above AST:379 naming AST:379-385 as the alias-window form of REC:288-296. | AST:378-385; rec silent |
| 13 | OrderedListShape first/rest min1/max1 | ACCEPT-DRIFT | None — identical. | AST:388-402; REC:306-318 |
| 14 | TableRefShape tableName pattern + min1/max1 | ACCEPT-DRIFT | None — rec §4 defines no table shape, so no MUST is unmet; shapes are anti-injection defense-in-depth. | AST:404-412; GOV:64-74 |
| 15 | ColumnRefShape columnName pattern + min1/max1 | ACCEPT-DRIFT | None — same rationale as row 14. | AST:414-422 |
| 16 | §3.3 rdf:List MUST for groupBy/orderBy | RECONCILE-SHAPE | Add to sql:SelectStatementShape: property shapes on `sh:path sql:groupBy` and `sh:path sql:orderBy`, each `sh:maxCount 1 ; sh:node sql:OrderedListShape`, severity Violation, NO minCount (both clauses optional per §3.3). Mirrors the sql:from normative branch (AST:303-308). Closes the only real §3.3 closed-world gap. | AST:148-151, 158-161 (rdfs:range only, open-world); REC:167, 193-206 |
| 17 | limit/offset range drift | ACCEPT-DRIFT | None required — ontology stricter, not looser (see §3.C). Optional follow-up: `sh:datatype xsd:nonNegativeInteger ; sh:minInclusive 0` shapes. Do NOT relax AST:166/171. | AST:163-171; REC:208-216 |
| 18 | §7 MERGE vocabulary unshaped | ACCEPT-DRIFT | None — §4 (REC:232-319) is the exhaustive normative shape set; §7 is a compilation example, not a shape mandate. No MUST unmet. | REC:232-319, 364-423 |
| 19 | Anti-scan whereClause minCount 1 (GOV) | RECONCILE-SHAPE | Superseded by inter-shape finding A — apply the §2.A disjunction fix. | GOV:56-58 |
| 20 | Join table-name pattern via sequence path | ACCEPT-DRIFT | None — governance-preflight lane intent (GOV:11, GOV:36-38), not rec §4 scope; harmless under conjoin once finding A is fixed. | GOV:44-47 |
| 21 | REC:234 process MUST | ACCEPT-DRIFT | None — enforced by the src compiler + courts; boundary gate `cargo test --test sql_rdf_dialect` = "25 passed; 0 failed", exit 0 (L33 receipt). | REC:234 |

## 2. Inter-shape findings

### A. Conjoined-shape contradiction — RECONCILE-SHAPE (ontologies/shapes/ast_governance.ttl)

GOV:18-28 re-declares `sql:SelectStatementShape` targeting
`sql:SelectStatement`; SHACL conjoins it with sql-ast.ttl's shape of the same
IRI. GOV forces legacy aliases at minCount 1 (GOV:30-34 fromTable,
GOV:49-52 selectList, GOV:56-58 whereClause), so a purely normative document
(sql:select + sql:from + sql:where, no aliases) passes sql-ast.ttl and FAILS
governance. This is the one true contradiction in the L33 report.

**Primary fix — per-clause disjunction (preserves anti-scan scope).** Rewrite
each legacy-only governance PropertyShape as a sh:or of (alias form,
normative form), keeping `sh:targetClass sql:SelectStatement` so the
anti-scan law continues to govern every SELECT:

- `sql:HasFromTable` (GOV:30-34) →
  `sh:or ( [ sh:path sql:fromTable ; sh:minCount 1 ; sh:maxCount 1 ; sh:node sql:TableRefShape ]
           [ sh:path sql:from ; sh:minCount 1 ; sh:maxCount 1 ; sh:node sql:OrderedListShape ] )`
- `sql:HasSelectList` (GOV:49-52) →
  `sh:or ( [ sh:path sql:selectList ; sh:minCount 1 ; sh:maxCount 1 ]
           [ sh:path sql:select ; sh:minCount 1 ; sh:maxCount 1 ; sh:node sql:OrderedListShape ] )`
- `sql:HasWhereClause` (GOV:56-58) →
  `sh:or ( [ sh:path sql:whereClause ; sh:minCount 1 ]
           [ sh:path sql:where ; sh:minCount 1 ] )`

Keep `sql:FromTableNamePattern` (GOV:39-42) as-is; if normative-side pattern
coverage is wanted later, add a companion shape that walks the sql:from
rdf:List spine (sh:node chaining), explicitly out of scope for this fix.

**Rejected alternatives:**

- *Separate target class* (`sql:GovernedSelectStatement
  rdfs:subClassOf sql:SelectStatement`, retarget GOV at it): shrinks the
  anti-scan scope — normative documents typed only `sql:SelectStatement`
  would escape the anti-scan law, inverting the governance intent
  (GOV:8-12, GOV:25-28). Viable only if the product decision is that
  preflight applies solely to cold-chain governed documents; that is a
  dispatch decision, not this plan's default.
- *Documented profile split* (never load both files together): loses the
  conjoin-time safety property; conjoining is the mechanism by which the
  preflight binds every SELECT, and a loader-order contract is unenforceable
  in SHACL.

**Gate for the fix (executing lane, Chicago — real graphs, real validator,
no mocks):** after the GOV rewrite, all three must hold: (a) a normative-only
document (select+from+where, no aliases) passes BOTH files under conjoin;
(b) an alias document still passes; (c) a WHERE-less SELECT in either
spelling is refused — and the court flips when the where/whereClause pair is
inverted in a mutated document (anti-vacuity).

### B. Not-equals spelling drift — ACCEPT-DRIFT (compiler convergence witnessed)

sql:NotEquals label is `"<>"` (AST:258); the normative enum token is `"!="`
(AST:375, REC:293). Compiler receipt: both spellings canonicalize to ONE
operator variant — `src/sql_ast.rs:562` maps the IRI
`http://graphlaw.org/sql#NotEquals` → `Op::NotEquals`; `src/sql_ast.rs:580`
maps `"!="` → `Op::NotEquals`; `src/sql_ast.rs:541` emits `"<>"` as the SQL
surface; the same convergence appears in `src/sql_split/ast.rs:53/74/92` and
`src/dialect.rs:618`. The `"<>"` label therefore records the COMPILED SQL
SURFACE spelling, not a competing validation token; changing it would diverge
the label from emitted SQL. Executing lane MAY add one comment line at
AST:258: `# label = compiled SQL surface; normative admission token is "!="
(rec section 4)`.

### C. limit/offset range drift — ACCEPT-DRIFT (ontology stricter than rec)

Rec declares `xsd:integer` (REC:211, REC:216); the ontology declares
`xsd:nonNegativeInteger` (AST:166, 171). xsd:nonNegativeInteger is an XML
Schema derivation of xsd:integer, so the ontology range is a semantic
refinement (LIMIT/OFFSET are non-negative), and no SHACL shape constrains
limit/offset today (L33 row 17) — nothing fails at admission. Do not relax
AST:166/171. Optional follow-up: `sh:datatype xsd:nonNegativeInteger ;
sh:minInclusive 0` property shapes on limit/offset.

## 3. Verdict census

Over the 21 rows: **ACCEPT-DRIFT 14** (rows 1, 2, 4, 5, 7, 9, 11, 13, 14,
15, 17, 18, 20, 21). **RECONCILE-REC 5** (rows 3, 6, 8, 10, 12).
**RECONCILE-SHAPE 2** (rows 16, 19). 14 + 5 + 2 = 21.

Final authoritative count: **ACCEPT-DRIFT 14 / RECONCILE-REC 5 /
RECONCILE-SHAPE 2**, plus inter-shape finding A = RECONCILE-SHAPE and finding
B = ACCEPT-DRIFT (finding C is row 17, counted above).

## 4. Executing-lane work order

1. (row 16) Add groupBy/orderBy property shapes to `sql:SelectStatementShape`
   in `ontologies/sql-ast.ttl` per the row-16 instruction. After the edit run
   `cargo test --test sql_rdf_dialect`; expect "25 passed; 0 failed". A
   failure there is disclosure, not a hand-fix.
2. (row 19 / finding A) Rewrite GOV:30-34, GOV:49-52, GOV:56-58 as the §2.A
   sh:or disjunctions, keeping `sh:targetClass sql:SelectStatement`. Add the
   three-court gate from §2.A.
3. (rows 3, 6, 8, 10, 12) Comment verification only — confirm the
   alias-window and asserted-closure comments (AST:290-291, 310-315,
   329-331, 341-344) are present and accurate; add the optional one-liners at
   AST:258 (§2.B) and above AST:379 (row 12).
4. (rows 17 / §2.C, optional) limit/offset shapes with
   `sh:minInclusive 0`.
5. After each shapes edit, rerun `cargo test --test sql_rdf_dialect` and
   report actual output. Do not hand-edit generated projections; do not
   commit.
