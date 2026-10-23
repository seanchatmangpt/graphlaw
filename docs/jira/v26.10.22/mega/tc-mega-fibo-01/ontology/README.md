# TC-MEGA-FIBO-01 — Regulatory Rule Layer (lane L38, ontology surface)

Closed-world SHACL shapes for the tear-up scenario plus a stratified N3
Datalog rule layer for SA-CCR netting-set membership and EMIR margin
eligibility. All shape files and twins are GENERATED; do not hand-edit.

## Layout

```
ontology/
  gen_shapes.py                          parametric generator (source of truth)
  shapes/
    10_jurisdiction_ccp.ttl              jurisdiction enum (US/UK/EU) + CCP membership
    20_lei_hierarchy.ttl                 LEI hierarchy (parametric, 5,000 nodes / 12 holders)
    30_isda_margin.ttl                   ISDA margin/threshold fields + jurisdiction caps
    40_floating_curves.ttl               floating-curve enum (SOFR/SONIA/EURIBOR)
    50_currency_precision.ttl            per-currency decimal precision (26 currencies)
    51_precision_anchor_41209.ttl        #41209 falsifier anchor (single deterministic shape)
    55_holder_currency_precision.ttl     per-(holder,currency) precision at full scale
  twins/
    41209_benign.ttl                     leg #41209, "1500.00"^^xsd:decimal
    41209_mutated.ttl                    leg #41209, "1500.0025"^^xsd:decimal
  rules/
    saccr_netting_set_rules.n3           STRATIFICATION LAYER 1 (positive Horn)
    emir_margin_eligibility_rules.n3     STRATIFICATION LAYER 2 (negation on layer 1 only)
```

## Generation commands + exits

```
python3 gen_shapes.py          # exit 0
```

Generator stdout (actual, from the last run):

```
file,shapes,constraints,bytes
shapes/10_jurisdiction_ccp.ttl,6,14,3103
shapes/20_lei_hierarchy.ttl,14,340,24065
shapes/30_isda_margin.ttl,4,24,2908
shapes/40_floating_curves.ttl,4,7,2086
shapes/50_currency_precision.ttl,26,104,10623
shapes/51_precision_anchor_41209.ttl,1,4,1892
shapes/55_holder_currency_precision.ttl,312,1248,86484
TOTAL,367,1741,131161
twins: benign=598 bytes, mutated=600 bytes
```

Counts are constraint COMPONENTS (each occurrence of sh:minCount, sh:maxCount,
sh:datatype, sh:pattern, sh:in, sh:node, sh:class, sh:closed, sh:minInclusive,
sh:maxInclusive, sh:qualifiedValueShape), counted from emitted bytes.

## Shape count: target vs actual

- Operator target: 1,248 constraints. STATUS: REACHED in the scale family —
  `shapes/55_holder_currency_precision.ttl` contributes exactly 1,248
  (12 holders x 26 currencies x 4 constraint components).
- Total emitted set: 367 NodeShapes carrying 1,741 constraint components
  (the 1,248 scale family plus 493 in the scenario-specific shapes). The
  1,248 figure is the operator's TARGET, not a witnessed property of any
  deployed system; total bytes across shapes/ is 131,161 (~131 KB).
- The operator's 42 MB rule-figure is likewise a TARGET, not a witnessed
  fact. Actual rules/ bytes are recorded below and are orders of magnitude
  smaller by design (rules are intensional, not data-fattened).

## Rules byte counts (actual, honest)

| File | Bytes | Clauses |
|---|---|---|
| rules/saccr_netting_set_rules.n3 | 2,862 | 5 |
| rules/emir_margin_eligibility_rules.n3 | 3,409 | 4 |
| Total | 6,271 | 9 |

## Stratification (binding addendum item 1: bounded, no cyclic negation)

- **Layer 0** — extensional scenario facts (twins/, future scenario data):
  finite ground atom base over reg: predicates.
- **Layer 1** (`saccr_netting_set_rules.n3`) — positive Horn clauses only.
  All clauses are range-restricted (every head variable appears in the body);
  heads introduce only terms already present in the body, so the forward-
  chaining closure is the finite least Herbrand model. The single recursive
  clause (R1.4, transitive co-netting) is guarded by reg:nettingSetOf and
  terminates over finite facts.
- **Layer 2** (`emir_margin_eligibility_rules.n3`) — negation-as-failure via
  `log:notIncludes` ONLY over predicates fully computed by layer 1
  (reg:SACCRRecognizedNettingSet, reg:nettingSetMember, reg:coNettedWith) or
  ground layer-0 atoms. No layer-2 head predicate is ever negated, so the
  negation-dependency graph is acyclic (standard stratified negation).
  Evaluation is one ordered fixpoint pass per layer: terminates by
  construction; no open-world leakage into resolution.

## #41209 falsifier twins (binding addendum item 2)

Both twins are static files, ready for bit-level refusal assertion.

| File | Bytes | sha256 |
|---|---|---|
| twins/41209_benign.ttl | 598 | `b860ac7196020318c3046e491ce0ce317d2b93462f14c86545f923c93fcca18d` |
| twins/41209_mutated.ttl | 600 | `1af612e0a9f39f6bfa9179595b3c28ddd815f666ceed99ef662861c0f6bfc4bd` |
| shapes/51_precision_anchor_41209.ttl | 1,892 | `6f180fe5bf94112b9e02ab6104f8064d84b4b39ffd878130800d54799bbeda1e` |

**The exact mutation diff (the ONLY byte difference between the twins)**:
in `twins/41209_benign.ttl`, byte offset **532** (0-based), the object of
`reg:Leg-41209 reg:amount` is `"1500.00"^^xsd:decimal`; in
`twins/41209_mutated.ttl` at the same offset it is `"1500.0025"^^xsd:decimal`
(+2 bytes). Identical bytes elsewhere. (Offset moved from 375 to 532 when the
WO-13 namespace-declaration header was added to the twin template; the
generator asserts `benign.replace("1500.00","1500.0025") == mutated`.)

**Deterministic, byte-positioned refusal**: the mutated value violates
exactly one constraint — `sh:pattern` at byte offset **1660** in
`shapes/51_precision_anchor_41209.ttl`
(`reg:MoneyPrecision_USD_41209_p1`, single focus node `reg:Leg-41209`,
single shape, single violating component). Verified discriminatively with
the anchor regex: `"1500.00"` matches; `"1500.0025"` does not. The benign
twin admits; the mutated twin refuses on exactly one component — the anchor
is a court that can flip.

## Validation

Validator used: **python rdflib 7.6.0** (real parser; rapper present on PATH
but rdflib is primary). Results from the last run:

```
OK turtle shapes/10_jurisdiction_ccp.ttl 66 triples
OK turtle shapes/20_lei_hierarchy.ttl 845 triples
OK turtle shapes/30_isda_margin.ttl 57 triples
OK turtle shapes/40_floating_curves.ttl 35 triples
OK turtle shapes/50_currency_precision.ttl 234 triples
OK turtle shapes/51_precision_anchor_41209.ttl 11 triples
OK turtle shapes/55_holder_currency_precision.ttl 2496 triples
OK turtle twins/41209_benign.ttl 4 triples
OK turtle twins/41209_mutated.ttl 4 triples
OK n3 rules/emir_margin_eligibility_rules.n3
OK n3 rules/saccr_netting_set_rules.n3
VALIDATE_EXIT=0
```

Intermediate failures during generation (fixed, not skipped): missing
`rdf:` prefix in 20_lei_hierarchy.ttl; rdflib N3 parser rejects formula-
subject `naf:unless` — negation was re-expressed with `log:notIncludes`
inside rule bodies (CWM idiom), which parses.

## Vocabulary grounding and RECONCILIATION-REQUIRED

Grounded in repo surfaces (ontologies/sql-ast.ttl, ggen/sql-dialect/ontology/
statements.ttl, ggen/sql-dialect/statement_classes.ttl, src/sql_ast.rs):

- Rule comparison semantics are expressed against the **section 4 operator
  token enum** from src/sql_ast.rs (`Operator::from_token`): `= != < <= > >=
  AND OR + - * / % LIKE ILIKE`. `math:lessThan` in layer 2 corresponds to the
  `<` token / `sql:LessThan` axiom IRI.
- SHACL constraint vocabulary (sh:minCount/maxCount/pattern/in/node/class/
  closed) matches the sh: surface used by ontologies/sql-ast.ttl.
- Class/enum style (closed `sh:in` enums, exactly-one cardinality) mirrors
  sql-ast.ttl shapes.

RECONCILIATION-REQUIRED (terms with no exact counterpart on a repo surface;
listed, not silently renamed):

1. `reg:` namespace (`http://graphlaw.org/ns/mega/fibo/reg#`) — the entire
   scenario vocabulary (RegulatedParty, LegalEntityIdentifier, MarginAgreement,
   MoneyAmount_*, jurisdiction, marginThreshold, floatingCurve, ...) has no
   counterpart in sql-ast.ttl or the dialect pack. Deliberate: it is the
   regulatory layer's own vocabulary; mapping to FIBO/LEGAL terms is open.
2. `reg:amount` lexical-precision constraint — SHACL `sh:pattern` over an
   xsd:decimal lexical form has no analogue in sql-ast.ttl (which constrains
   SQL AST structure, not value precision). Whether the compiler should
   enforce decimal scale natively (deterministic byte-positioned refusal at
   the SQL literal layer) is an open compiler question.
3. `log:notIncludes` negation idiom — no repo-surface counterpart; the
   sql-rdf compiler currently has no N3 rule-evaluation lane. These rules
   are intensional specification, not yet compiled artifacts.
4. `math:lessThan` (swap/math) vs `sql:LessThan` axiom IRI vs `<` token —
   three forms of the same operator across N3 and sql_ast.rs; pick one
   canonical form before any compiler admission of these rules.
5. `naf:` prefix was removed during validation (rdflib rejects formula
   subjects with plain predicates); stratification comments reference
   log:notIncludes only.

## Reconciliation decisions (WO-13, lane on /Users/sac/graphlaw, 2026-10-22)

The mega ontology is a SELF-CONTAINED scenario ontology: its terms are
scenario-local, not repo-global. Per-term decisions (implemented in
`gen_shapes.py` + the `rules/*.n3` headers, regenerated, validated):

| # | Term | Decision | Implementation |
|---|---|---|---|
| 1 | `reg:` namespace | **(b) first-class scenario-local namespace** — no mapping to sql:/sh:, no rename; the vocabulary has no repo counterpart by design (regulatory rule layer). | Declared in every generated shapes/ header (NAMESPACE DECLARATION block in `gen_shapes.py` HEADER) and in both rule files; this section is the vocabulary note. |
| 2 | `reg:amount` `sh:pattern` lexical-precision constraint | **(b) scenario-local**; kept as SHACL over the xsd:decimal lexical form. Whether the compiler should enforce decimal scale natively (byte-positioned refusal at the SQL literal layer) remains an open compiler question, tracked here — not silently mapped. | Unchanged shape semantics; documented decision only. |
| 3 | `log:notIncludes` negation idiom | **(b) scenario-local** (standard CWM/swap vocabulary, stable IRIs); these rules are intensional specification — no N3 rule-evaluation lane exists in the compiler yet. | Documented in both rule-file headers. |
| 4 | `math:lessThan` vs `sql:LessThan` vs `<` | **(a) mapped**: canonical form is the section-4 token `<` / the `sql:LessThan` axiom IRI; `math:lessThan` is the N3 spelling of exactly that operator. No rename in N3 — the correspondence is documented, and the token form is designated for any future compiler admission lane. | Mapping note in `rules/emir_margin_eligibility_rules.n3` header. |
| 5 | `naf:` prefix | Already removed (rdflib rejects formula subjects with plain predicates); confirmed absent — no `naf:` token in any ontology file. | Verified by grep (below). |

### Verification receipts (WO-13, actual output)

```
python3 gen_shapes.py                  -> GEN_EXIT=0 (stdout table above; twins benign=598, mutated=600)
rdflib parse 7 shapes + 2 twins (turtle), 2 rules (n3)
  -> all OK; triple counts unchanged: 66/845/57/35/234/11/2496, twins 4+4; VALIDATE_EXIT=0
cmp -l twins/41209_benign.ttl twins/41209_mutated.ttl
  -> divergence begins at the single literal (1-based byte 533 = 0-based 532);
     generator assert benign.replace("1500.00","1500.0025")==mutated holds:
     exactly one insertion site, +2 bytes total
shasum -a 256 (see table above)
anchor regex: "1500.00" matches; "1500.0025" does NOT; sh:pattern byte offset now 1660
grep -rn "naf:" shapes/ rules/ twins/   -> 0 matches (naf: survives only as prose in this README)
git status --porcelain ontologies/ ggen/ -> only pre-existing modifications by
  other lanes; this lane wrote ONLY under docs/jira/v26.10.22/mega/tc-mega-fibo-01/ontology/
```

Byte-count and hash drift vs the pre-WO-13 figures is fully explained by the
namespace-declaration header lines added to the generated files (+548 bytes
per shape file header, +149 per twin) — no shape semantics changed; shape and
constraint counts are identical (367 / 1,741 / 1,248).
