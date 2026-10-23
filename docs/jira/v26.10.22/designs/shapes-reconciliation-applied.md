# Shapes ↔ Rec §4 Reconciliation — Applied (Lane L71)

Date: 2026-10-22. Executes `shapes-rec-reconciliation.md` (L55). Files touched:
`ontologies/sql-ast.ttl`, `ontologies/shapes/ast_governance.ttl` — both
hand-authored canonical graphs (no generator headers; no generated projections
exist for these shapes, nothing regenerated).

## Court receipts

```
CARGO_TARGET_DIR=target-lane71 cargo test --test sql_rdf_dialect
  -> test result: ok. 25 passed; 0 failed; 0 ignored; finished in 0.21s   (exit 0)
CARGO_TARGET_DIR=target-lane71 cargo test --test w3c_conformance
  -> test result: ok. 8 passed; 0 failed; 0 ignored; finished in 0.07s    (exit 0)
```

A transient `graphlaw-verify` compile error appeared mid-run and cleared on
rerun without any change on my side (shared-checkout in-flight edit by another
lane; compile-freeze SLA, no intervention needed).

## Rows applied

- **Row 16 (RECONCILE-SHAPE)** — `sql:SelectStatementShape` gains two property
  shapes on `sh:path sql:groupBy` / `sh:path sql:orderBy`: `sh:maxCount 1`,
  `sh:node sql:OrderedListShape`, severity Violation, no minCount.
- **Row 19 / finding A (RECONCILE-SHAPE)** — GOV `sql:HasFromTable`,
  `sql:HasSelectList`, `sql:HasWhereClause` rewritten as sh:or disjunctions of
  (legacy alias form, normative form); `sh:targetClass sql:SelectStatement`
  kept on `sql:SelectStatementShape`. Spec deviation from L55's literal text,
  required by SHACL: L55's shapes were typed `sh:PropertyShape` without a
  direct `sh:path`, which SHACL forbids (pyshacl ShapeLoadError as receipt).
  They are `sh:NodeShape`s referenced via `sh:node` (addressable IRIs kept;
  FromTableNamePattern/JoinTableNamePattern untouched per L55).
- **Row 12 optional** — comment above the legacy operator-IRI branch naming it
  as the alias-window form of the rec §4 token enum.
- **Finding B optional (§2.B)** — comment at `sql:NotEquals`: `label = compiled
  SQL surface; normative admission token is "!=" (rec section 4)`.

## Rows NOT applied (with reason)

- **limit/offset `sh:minInclusive 0` (row 17 / §2.C optional)** — not
  trivially safe: `sh:datatype xsd:nonNegativeInteger` on a numeric literal
  whose data-side spelling is `xsd:integer` (the rec range) would reject
  conforming documents; no shape constrains limit/offset today, so the
  follow-up buys no closed-world coverage. Not worth an admission-regression
  risk for zero coverage gain.
- All 14 ACCEPT-DRIFT and 5 RECONCILE-REC rows — no shape changes, per L55.
- REJECTED-BY-COURT: none. Zero admission regressions; no row reverted.

## L55 §2.A gate (Chicago: real graphs, real pyshacl validator, exit code)

Harness: `/tmp/l71_gov_gate.py` — conjoined sql-ast.ttl + ast_governance.ttl
shapes, real documents built per case, `validate(..., advanced=True)`.

| Case | HEAD baseline | New (L71) |
|---|---|---|
| (a) normative-only doc (select+from+where, no aliases) | **False** (finding A reproduced) | **True** |
| (b) alias doc (fromTable+selectList+whereClause) | True | True |
| (c) WHERE-less SELECT, either spelling | False | False |
| anti-vacuity: WHERE-less refused in BOTH spellings (gate flips only when fix removes (a)'s failure) | — | confirmed |

Gate exit 0. Note: an earlier gate run reported baseline normative = True;
root cause was a harness path bug (relative GOV path loaded the NEW file for
the "baseline"); corrected and reproduced — the baseline failure is real.

## Diff stat (vs HEAD)

```
 ontologies/shapes/ast_governance.ttl | 59 ++++++++++++++++++++++----------
 ontologies/sql-ast.ttl               | 21 ++++++++++++++-
 2 files changed, 63 insertions(+), 17 deletions(-)
```

## Positive no-drift closure (lane N5, 2026-10-22)

Post-landing verification with zero further edits. All three shape-coupled
courts re-run on this checkout (`CARGO_TARGET_DIR=target-lane-n5`):

| Court | Command | Result | Exit |
|---|---|---|---|
| sql-rdf dialect | `cargo test --test sql_rdf_dialect` | 25 passed; 0 failed; 0 ignored | 0 |
| corpus conformance | `cargo test --test corpus_conformance` | 12 passed; 0 failed; 1 ignored | 0 |
| W3C conformance | `cargo test --test w3c_conformance` | 8 passed; 0 failed; 0 ignored | 0 |

Verdict: no drift between `ontologies/sql-ast.ttl` + `ontologies/shapes/`
and section 4 of `docs/reference/sql-rdf-rec.md`. No shape edits required;
closure is positive (verified, not assumed).
