# SQLRDF-04 — Section 7 analytical MERGE

- **Id:** SQLRDF-04
- **Title:** Section 7 analytical MERGE
- **Status:** Open

## Conformance target

Section 7 and section 7.1 of the pasted sql-rdf Recommendation, the conformance text for v26.10.22, together with the first formula in section 1.4:

`H_query = BLAKE3(Canon(G_AST) || P_params)`

The section 7 graph is a `sql:MergeIntoStatement` against `main.regional_sales`, using a grouped `SELECT` over `staging.invoices` whose status comparison is the literal `"SETTLED"`. Section 7.1 projects that graph to a parameterized `MERGE` with `$1 = "SETTLED"`.

## Current behavior

`MergeIntoStatement` in [`src/sql_ast.rs`](../../../src/sql_ast.rs) (lines 844–848) compiles the template `MERGE INTO {t} USING (SELECT {v} AS {c}) s ON {t}.{c} = s.{c} WHEN MATCHED THEN UPDATE SET {c} = s.{c}`. The compiler returns that string with an empty parameter vector (`compile_command`, lines 771–911).

[`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs) exercises that template through `state_receipt("MergeIntoStatement")` (lines 129–160). `state_receipt` hashes the SQL text plus an ordered readback of table `t` (lines 143–151). That template is the section 7 graph’s substitute in today’s court, and it is the section 7.1 statement’s substitute as well.

## Required behavior

The section 7 sample graph compiles to the parameterized `MERGE` in section 7.1:

```sql
MERGE INTO main.regional_sales
USING (
    SELECT region_id, sum(amount) AS total_invoiced
    FROM staging.invoices
    WHERE status = $1
    GROUP BY region_id
) AS src
ON main.regional_sales.region_id = src.region_id
WHEN MATCHED THEN UPDATE SET revenue = (main.regional_sales.revenue + src.total_invoiced);
```

with `$1 = "SETTLED"`. The statement runs against real `staging.invoices` and `main.regional_sales`.

The receipt is BLAKE3 of the canonical AST graph bytes concatenated with the parameter buffer. Replay of the same graph and the same buffer matches. Changing the literal changes the receipt. The court passes only when that graph-and-buffer digest is the receipt. Hashing the SQL string alone fails the court.

## Acceptance

`cargo test --test chicago_validation`.
