# REFUSAL-AUDIT (lane 03, 2026-11-17)

Scope: docs/refusals.md (R), docs/reference/refusal-catalog.md (C), docs/refusal-playbook.md (P) vs src/ and tests/. Static grep only; no cargo/probe run.

## Verdict
No stable code is missing from the docs (REFUSAL-01..05 and ABI `details.code` set all present). No live renumbering violation; one historical renumbering disclosed. Real drift: arm coverage, stale anchors, misleading wording.

## Code inventory
- Constants: REFUSAL-01/02/03/05 at src/sql_ast/refusals.rs:9,10,11,16; REFUSAL-04 at src/tier.rs:26 (literal also src/rules/builtins.rs:89). No REFUSAL-06+ anywhere.
- `RefusalCode`: MalformedAst..Capability 1-4 (:215-221), UnboundedMutation 5 (:244), InvalidIdentifier..MalformedNode 16-21, LiteralNotInt 36, LiteralNotFloat 37.
- `refusal_arm`: 25-35, 36-56 (:60-179), scope 1-7, 100-104, 0/255, 1/2.
- ABI details.code emitters: governance.rs (PlanRefused 72, ReceiptRequired 83, ReceiptRefused 92, LeaseRefused 101, NotAdmitted 129, UnverifiedLeaseRefused 341, PolicyRefused 398, ResourceLimit 34/39, Refused 42), query_engines.rs:238 Fuse, duckdb_op.rs:134 DuckDb, :797 BatchMemberFailed.
- `RefusalKind`: 9 arms (src/dialect/refusal.rs:24-57), matches R.

## Drift table
| # | Item | Status |
|---|---|---|
| 1 | REFUSAL-01..03 | OK |
| 2 | REFUSAL-04 | WEAK TEST: only tests/sql_rdf_conformance.rs:446 (constant equality); no behavioural trigger test |
| 3 | REFUSAL-05 | OK in R and C; absent from P |
| 4 | arms 36-56 | OK |
| 5 | scalar arms 1-35 | CODE-ONLY (0 hits in R, C, P) |
| 6 | scope arms 1-4 | CODE-ONLY (5-7 documented) |
| 7 | arms 100-104 (SHACL_*) | CODE-ONLY |
| 8 | UNSUPPORTED_GENERIC, LEGACY_MESSAGE, UNIMPLEMENTED_STATEMENT, NO_SQL_TEMPLATE | CODE-ONLY |
| 9 | unregistered function IRI (UnsupportedTerm) | DISCLOSED GAP |
| 10 | egress errors | OK (disclosed, no code minted) |
| 11 | harness codes | OK; MANIFEST-REFUSED/DISK-REFUSED are design-only |
| 12 | docs-only codes | none |

## Wording problems
- Three discriminant namespaces share one notation: `LiteralNotFloat = 37` vs `JOIN_TYPE_IRI = 37`; scope arm 5 vs `UnboundedMutation = 5`.
- R:48 lists `ShaclViolation` as a RefusalCode; it appears to be an AstError constructor (unconfirmed).
- C:9 "never renumbered" contradicts R's disclosed star-arm renumber (48/49 -> 50-52 at fold time); qualify it.

## Stale anchors in R
| R cite | Actual |
|---|---|
| refusals.rs:150-165 set-op arms | SET_OP_* at :164,168,173,179 |
| sql_ast.rs:284-295 | arm used at :480 |
| sql_ast.rs:304-311 | arm used at :495 |
| sql_ast.rs:987-993 | arm used at :1680 |

## Not verified
refusal_probe re-run ("12/12 MATCH" inherited); message text byte-for-byte; receipts/ and t4-refusal-reaudit.md.

## Fixes (docs only)
Update anchors; qualify C:9; add an arm-namespace table (closes rows 5-8); add REFUSAL-05 row to P; add a behavioural REFUSAL-04 test.
