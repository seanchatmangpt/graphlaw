# SQLRDF-25 close-out receipt — 2026-10-23

- **Subject**: SQLRDF-25 remaining-gap ledger
  (`docs/jira/v26.10.22/SQLRDF-25-remaining-gaps.md`, Rows 1-5) — wave
  close-out aggregating this wave's five per-row courts.
- **Kind**: close-out witness receipt (aggregation; no flip of any chained
  file; codes append-only).
- **Environment**: repo `/Users/sac/graphlaw`, branch main, 2026-10-23.
  Disk gate: `df -h /` → 48 GiB free (≥ 20 GiB required). Build isolation:
  `CARGO_TARGET_DIR=target-lane-rs` (lane lease; coordinator deletes at
  integration).

## Per-court results (real output, captured this session)

Each command run as
`CARGO_TARGET_DIR=target-lane-rs cargo test --test <court> 2>&1 | tail -3`
from `/Users/sac/graphlaw`:

| court (ledger row) | result line (verbatim tail) |
|---|---|
| `pivot_court` (Row 3) | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s` |
| `star_modifier_court` (Rows 1+2) | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s` |
| `columns_projection_court` (Row 2) | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s` |
| `json_operator_court` (Row 5) | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s` |
| `composite_literal_court` (Row 4) | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s` |

Totals: 5/5 courts exit 0, **17 tests passed, 0 failed** across the five
courts (3+4+4+2+4).

## Arms 48-52 fold statement

Per `docs/refusals.md` ("Appended star-modifier / composite discriminants
(v26.10.22, SQLRDF-25)") and `src/sql_ast/refusals.rs`: the interim
`serializer::star_refusal_arm` table is folded into the single
`refusal_arm` source at `= 48 .. = 52`; the duplicate 48/49 numbering is
reconciled — 48 `COMPOSITE_DEPTH`, 49 `COMPOSITE_MEMBER`, star arms
renumbered 50 `STAR_MODIFIER_POSITION`, 51 `COLUMNS_LAMBDA`,
52 `COLUMNS_PATTERN` — folded **before any external serialization of the
interim numbers**. Codes append-only, never renumbered.

## Shapes / generator identity

- `b3sum ontologies/sql-ast.ttl` →
  `e4ee566129071417acbf9d34ebde17e7bef3139340cbdbe5c65ac7b1a339297b`
- `b3sum ggen/sql-dialect/statement_classes.ttl` →
  `464ef4e718b8ba41a42bd108f2b82c481f9b84fce91fb83d77d20e0b657531d3`
- ggen sql-dialect pack receipt
  (`ggen/sql-dialect/.ggen-v2/receipt.json`): `graph_hash
  2ab3f9fef57b9be36496847610ad7d49549bae0cd2fa8e4a4d4b1f0903d80027`;
  its latest `statement_classes.ttl` output digest `464ef4e7…7531d3`
  matches the on-disk `b3sum` above byte-exactly — the landed shapes are
  the generator's current output.

## Open rows

- **Struct readback (Row 4)**: not failing — `composite_literal_court`
  passed 4/4 this session (see table), which covers struct/list literal and
  accessor real-readback. No open failing row remains from Rows 1-5.
- Outstanding (non-failing, out of this receipt's scope): wasm one-unit
  court (`cargo test -p graphlaw-wasm`) and shape mutation court
  (`scripts/shape_mutation_court.sh`) are the ledger's other acceptance
  rungs and are NOT witnessed here; the ledger's Status stays as the
  coordinator's flip decision, gated on those receipts.
