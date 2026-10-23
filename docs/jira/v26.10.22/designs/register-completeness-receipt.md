# Register-completeness receipt (lane M12, 2026-10-22)

Defect class A2: the `docs/refusals.md` trigger column is narrower than the
implemented gates. Witnessed gaps named in the dispatch: ViewScope-in-HAVING
(inside the `VIEW_SCOPE_IN_PREDICATE` arm's behavior) and
`SCOPE_IN_PRED_AGG_ARG` (aggregate argument in a predicate position).

## Court

`tests/sql_rdf_conformance.rs::register_completeness_every_refusal_arm_is_enumerated`

- Enumerates `refusal_arm::*` constants MECHANICALLY by parsing
  `src/sql_ast/refusals.rs` via `include_str!` (no regex dep, no hand list).
- Every parsed arm must be either in `REGISTERED_ARMS` (with doc needles that
  must literally appear in `docs/refusals.md`, included via `include_str!`) or
  in `PINNED_GAPS`. A new enum arm fails the court until classified — that is
  the forcing function.
- Registered arms may not silently de-register: a needle disappearing from the
  register fails the court.

## Non-vacuity (witnessed live, not simulated)

During this session a concurrent lane appended
`refusal_arm::MERGE_ON_NO_TARGET_COLUMN` (ScopeIntegrity arm 7,
`src/sql_ast/refusals.rs:71`). The court went RED on the next run with:

```
UNREGISTERED ARM 'MERGE_ON_NO_TARGET_COLUMN': implemented in src/sql_ast/refusals.rs
but absent from both REGISTERED_ARMS and PINNED_GAPS ...
```

It is now pinned in `PINNED_GAPS` (the register flip for it is receipt-gated
and outside this lane's lease — docs/refusals.md is not in the M12 file
lease). This is the designed fail-until-registered behavior, demonstrated on
a real new arm, not a mutation shim.

## Enumeration count (as of this receipt)

53 arm constants parsed: 21 classified REGISTERED_ARMS (needle-verified
against the register text), 32 in PINNED_GAPS (defect-class-A2 drift,
appended-only). The pinned gap set includes `SCOPE_IN_PRED_AGG_ARG` and the
dual-form/laundering family (DUAL_LEFT_OPERAND, DUAL_RIGHT_OPERAND,
DUAL_OPERATOR_FORMS, DUAL_FROM, DUAL_SELECT, DUAL_WHERE, DUAL_HAVING,
UNKNOWN_STATEMENT_PREDICATE, the LIMIT/OFFSET and INDEX families,
DIRECTION_LEX/IRI, SHACL_DEPTH, LEGACY_MESSAGE, ALIASED_AGG_ARG,
CANONICALIZATION, DUP_CLAUSE_PREDICATE, SCOPE_CARDINALITY, SCOPE_WRONG_NODE,
EXPRESSION_WRAPPER, ALIAS_IN_AGG_ARG, SQL_ALIAS_IN_AGG_ARG,
SCOPE_ON_TABLE_REF, MERGE_ON_NO_TARGET_COLUMN).

## Verification

```
CARGO_TARGET_DIR=target-m12 cargo test --test sql_rdf_conformance
test result: ok. 8 passed; 0 failed; 0 ignored
```

Court count: 8 tests in the file (7 pre-existing + this court).

## Closing the pinned gaps

Each gap closes by (1) adding a row/trigger text to `docs/refusals.md` that
names the arm's behavior, (2) moving the arm from `PINNED_GAPS` to
`REGISTERED_ARMS` with its needle(s). The court asserts
`!REFUSALS_MD.contains("aggregate argument in a predicate")` as the live A2
witness; once the register names that behavior, that assertion must flip
together with the arm's reclassification, so the court stays an honest
ratchet.
