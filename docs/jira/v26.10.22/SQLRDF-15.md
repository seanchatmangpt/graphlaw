# SQLRDF-15 — §6.2 tpch setup divergence: retire the sf=0.01 hand-patch into the generator + register

- **Id:** SQLRDF-15
- **Title:** §6.2 tpch setup divergence — retire the sf=0.01 hand-patch into the generator + register
- **Status:** Open

> **This is a follow-up ticket, not part of the v26.10.22 acceptance set.** The
> v26.10.22 acceptance is SQLRDF-01..SQLRDF-12 and their courts. This ticket
> registers a suspected-witness divergence and a retire-the-hand-patch plan for
> a later window.

## Conformance target

The sql-rdf Recommendation (`docs/reference/sql-rdf-rec.md`), §6.2 — the tpch
extension court runs against a real tpch-populated database with a certified
`SELECT count(*) FROM nation` result of 25 rows.

## Current behavior

The generated extension tuple for tpch carries a disclosed hand-patch in the
generated file `ggen/sql-dialect/sql_extensions.rs:22-26`:

```rust
// LANE L3 hand-patch of generated data (disclosed): witnessed on the real
// frozen engine — `dbgen(sf = 0)` populates 0 rows; rec 6.2's certified
// result of 25 nation rows is produced by `dbgen(sf = 0.01)` (witnessed
// 2026-10-22). Regenerate via extract_statements.py to overwrite.
("tpch", r#"CALL dbgen(sf = 0.01)"#, r#"SELECT count(*) FROM nation"#, 25),
```

The generator still emits the un-patched tuple: `EXTENSIONS["tpch"]` in
`ggen/sql-dialect/extract_statements.py:31` is
`("tpch": ("CALL dbgen(sf = 0)", "SELECT count(*) FROM nation", 25)`.
So the generated file and the generator disagree; the hand-patch is a
regeneration-away-from-a-regression, i.e. an unfixed divergence.

## The witness is suspect

The hand-patch's justification is a witnessed run where `dbgen(sf = 0)`
yielded 0 nation rows on the frozen engine. The upstream snapshot at
`/Users/sac/rel4r/duckdb-src/extension/tpch/dbgen/dbgen.cpp` contradicts it:

- `GetDBGenTableRowCount` (dbgen.cpp:905-910): `if (table_index < NATION)`
  multiplies `base * scale_factor`; NATION (and REGION) fall through to plain
  `base` — **never scale-factor-multiplied**.
- dbgen.cpp:1259-1271: `if (flt_scale < MIN_SCALE)` with `int_scale =
  (int)(1000 * flt_scale)` scales only the `PART..REGION` range — for sf=0
  that range becomes floor(0)=0, bumped to `base < 1 → 1` — **NATION is
  outside the scaled range**.

So on upstream semantics `dbgen(sf = 0)` should still populate all 25 nation
rows, and the 0-row witness is inconsistent with the pin's own source. Because
the frozen pin's build sources are absent (`wasm-build/` directory is gone:
`ls /Users/sac/frozen-duckdb/target/wasm-build` → "No such file or directory",
verified 2026-10-22), the frozen pin's behavior is **UNKNOWN for the pin** —
the witness cannot be replayed against the pin's own source, so the divergence
cannot be adjudicated today.

## Required behavior

Retire the hand-patch, restore generator/generated agreement, and register the
divergence:

1. **Adjudicate by re-open run.** Restore the frozen-duckdb wasm-build
   checkout (or an equivalent build of the same pin) and re-run
   `CALL dbgen(sf = 0)` + `SELECT count(*) FROM payline`... (nation count) on
   it. If it witnesses 25 rows at sf=0, `EXTENSIONS["tpch"]` in
   `extract_statements.py:31` stays `sf = 0` and the hand-patch is deleted
   outright. If it witnesses 0 rows at sf=0, move the patch into the
   generator: `EXTENSIONS["tpch"] = ("CALL dbgen(sf = 0.01)",
   "SELECT count(*) FROM nation", 25)`.
2. **Regenerate.** Run `extract_statements.py` + `ggen sync` so the generated
   `sql_extensions.rs` no longer needs any hand-edit. (Currently BLOCKED: the
   script asserts `EXT_DIR=/Users/sac/frozen-duckdb/target/wasm-build/duckdb/extension`,
   which does not exist — restore the checkout first.)
3. **Delete the hand-patch** — the disclosed block at
   `ggen/sql-dialect/sql_extensions.rs:22-26`. After retirement, the
   generated file carries no hand-edit.
4. **Register KD-style** in the frozen-duckdb `KNOWN_DIVERGENCES.md`
   convention (surface, divergence, evidence receipt, close condition), with
   the re-open condition: a frozen-pin rebuild that witnesses 25 nation rows
   at sf=0 closes the row and permits the sf=0 tuple.

## Falsifier

With the tuple regenerated (step 2) and the hand-patch deleted (step 3), the
§6.2 tpch court passes with **no hand-edit in the generated file**. A court
that passes only because of an edit inside the generated file is FAIL.

## Acceptance

- Real run: restore the frozen-duckdb wasm-build checkout, run the tpch setup
  court (`cargo test --test sql_rdf_dialect` tpch court, or the §6.2 court in
  `tests/chicago_validation.rs`) on the regenerated tree.
- The generated `sql_extensions.rs` contains `dbgen(sf = 0)` or
  `dbgen(sf = 0.01)` sourced from `extract_statements.py` — not a
  post-generation patch. Diff the generated file against generator output:
  byte-identical modulo nothing (no hand-edit).
- `cargo check` passes.
