# design-sniff-fixes

Design spec for the sniff-layer fixes (G8 root-cause class behind the
`corpus_conformance` failures). One file changes: `src/dialect/sniff.rs`, plus a
new court file `tests/sniff_court.rs`. No manifest edits; no generated artifacts.

## Current behavior (file:line)

All pointers verified against the working tree this session (2026-10-23).

- `src/dialect/sniff.rs:28-32` — `skeleton`'s `#` arm treats every `#` as a
  comment start, including `#` inside `<...>` IRIs (where it is a fragment
  separator) and inside string literals. A document whose first statement IRI
  contains `#` is truncated at the fragment: the statement tail is blanked and
  downstream classification misfires.
- `src/dialect/sniff.rs:245-319` — `route_sql_rdf`'s comment stripper has an
  `in_iri` guard (`sniff.rs:264-272`) but no literal guard: a `sql:` token inside
  a `"..."` literal is tokenized as syntax, so quoted text like
  `ex:s <p> "a sql:SelectStatement" .` misroutes to `Dialect::SqlRdf`, and
  `ex:s <p> "predicate sql:foo" .` fires the `Ambiguous` refusal (`sniff.rs:309-317`).
- `src/dialect/sniff.rs:101-122` — `line_terms` closes a literal with a bare
  `tail.find('"')` (`sniff.rs:111`); an escaped quote (`\"`) terminates the scan
  early, so `<s> <p> "escaped \" quote" .` counts 4 "terms" instead of 3,
  `line_based` fails, and the document falls through to `Turtle` instead of
  classifying `NTriples`.
- `src/dialect/sniff.rs:187-205` — the first-keyword set routes any document
  whose first keyword is `SELECT`/`INSERT`/... to `Dialect::Sparql`. Bare SQL
  text (`SELECT a, b FROM t`) admits as `Sparql` and then fails SPARQL parse
  with a generic error instead of a typed refusal.
- `src/dialect/sniff.rs:140-144` — the 600-char head window truncates the head
  mid-token; routing tokens landing past offset 600 are invisible.

Already-implemented boundary (do not re-file): the stripper's IRI-aware `#`
handling (`sniff.rs:264-272`), `skeleton`'s quote/triple/escape state machine
(`sniff.rs:33-56`), and the `sql`/`trig`/`shex`/`sql` hint short-circuits
(`sniff.rs:156-161`) all exist and are correct. Fixes 1 and 2 are ports of
existing machinery to the sibling function, not new mechanisms.

## Required behavior

1. `route_sql_rdf` strips string literals before tokenizing: port
   `skeleton`'s quote/triple-quote/backslash-escape state machine
   (`sniff.rs:33-56`) into the stripper loop, replacing literal spans with `""`
   placeholders, keeping `#`-as-comment suppression inside literals (a `#` in a
   literal is data, not a comment).
2. `skeleton` gains IRI-aware `#`: track `in_iri` across `<`/`>` exactly as the
   stripper does (`sniff.rs:264-272`); inside an IRI, `#` is pushed as data, and
   an IRI is closed at newline.
3. `line_terms` closes literals escape-aware: on `\`, skip two chars before
   resuming the `find('"')` scan; `\"` never closes the literal.
4. Bare-SQL refusal: leading SQL keyword (`SELECT`/`INSERT`/`UPDATE`/
   `DELETE FROM`/`CREATE TABLE`) plus a clause keyword (`FROM`/`INTO`/`SET`/
   `VALUES`) in the head, with none of the SPARQL markers present (`WHERE`,
   `FILTER`, graph-pattern `{`, `a` type token, `sql:` vocabulary) → typed
   refusal `RefusalKind::BareSql`. SPARQL text with any marker still routes
   `Sparql`. The check runs BEFORE the keyword set at `sniff.rs:187-205`.
5. Head window: widen to a **budget of 8 KiB** charged after
   comment-stripping, literal-blanking and whitespace-collapse, and document
   the bound here as law: routing never sees more than 8 KiB of skeleton per
   document; documents whose routing-relevant tokens land beyond that budget
   require an explicit `hint`. The budget is a named constant
   (`HEAD_BUDGET: usize = 8192`), asserted by court, not a magic literal.

## Refusal arms (append-only, per docs/refusals.md discipline)

| Arm | Code | Trigger | Court file |
|---|---|---|---|
| SNIFF-01 | `RefusalKind::BareSql` (new variant) | bare SQL keyword+clause shape, no SPARQL markers | `tests/sniff_court.rs` |
| SNIFF-02 | none — negative arm for fix 1: `sql:` inside a literal classifies `Turtle`, no refusal | `tests/sniff_court.rs` |
| SNIFF-03 | none — fixes 2 and 3 are reclassifications (to `NTriples`), no new arm | `tests/sniff_court.rs` |

Existing variants (`NotSemanticContent`, `Ambiguous`) are untouched; no
renumbering.

## Courts

New file `tests/sniff_court.rs`, modeled on `tests/corpus_conformance.rs`:
one `#[test]` per row, real `sniff()` on real bytes, assert on the returned
`Result<Dialect, Refusal>` (Chicago — no mocks). Every row must FLIP: applying
the inversion on the fixed tree makes the row fail; a row that passes on both
sides is vacuous and the court exits 1.

| Fix | Twin input (real bytes into `sniff`) | Expected post-fix | Inversion (mutation that must flip the row) |
|---|---|---|---|
| 1 | `ex:s <p> "predicate sql:foo" .` | `Ok(Turtle)` (pre-fix: `Err(Ambiguous)`) | delete the literal-strip arm in `route_sql_rdf`'s stripper |
| 1b | `ex:s <p> "a sql:SelectStatement" .` | `Ok(Turtle)` (pre-fix: `Ok(SqlRdf)`) | same inversion |
| 1c | `# comment mentioning sql:SelectStatement\nex:s <p> <o> .` | `Ok(Turtle)` (comment stripping still works after literal-strip port) | same inversion (guards the port did not break `in_comment`) |
| 2 | `<http://e/s#t> <http://e/p> <http://e/o> .` | `Ok(NTriples)` (pre-fix: IRI truncated at `#`, tail blanked, falls to `Turtle`) | revert `skeleton`'s `#` arm to the bare `sniff.rs:28-32` shape |
| 2b | `<http://e/s#t> <p> "lit with # hash" .` | `Ok(NTriples)` | same inversion |
| 3 | `<s> <p> "escaped \\" quote" .` (one backslash in the raw bytes) | `line_terms` = 3, `Ok(NTriples)` (pre-fix: scan ends early, falls to `Turtle`) | re-break `line_terms` to bare `find('"')` |
| 3b | full-`sniff` duplicate of row 3 | `Ok(NTriples)` | same |
| 4 | `SELECT a, b FROM t` | `Err(BareSql)` (pre-fix: `Ok(Sparql)`) | delete the BareSql arm from `sniff()` |
| 4b | `INSERT INTO t VALUES (1)` | `Err(BareSql)` | same |
| 4c | `SELECT * WHERE { ?s ?p ?o }` | `Ok(Sparql)` — negative court: the BareSql arm must not capture SPARQL | non-flip guard: with the arm deleted this row must still pass |
| 4d | `PREFIX e: <https://e/> SELECT * WHERE { ?s ?p ?o }` | `Ok(Sparql)` | same |
| 5 | doc with a `@prefix` header block long enough to push the first statement past offset 600 but within 8 KiB | routes on the pushed content (`Ok(SqlRdf)` for a `sql:SelectStatement` assertion past 600) | hard-code the budget back to 600 |
| 5b | `sniff` never reads more than the budget: doc >8 KiB of skeleton routes on the first 8 KiB | pinned behavior (may be `Ok(Turtle)`); pins the bound as law | none — law pin, not a flip |

## Flip-test discipline

- Row-level inversions live in the court as documented mutations; the operator
  applies them by hand-editing `sniff.rs`, reruns
  `cargo test --test sniff_court`, and expects the named rows to fail.
- `bash scripts/mutation_court.sh` runs solo (mutates the tree in place) and
  must be serialized against other lanes; it provides the automated sweep over
  the same inversions once the arms land in its mutation table.

## Acceptance

- `cargo test --test sniff_court` — green on the fixed tree; every flip row
  demonstrated red under its inversion (receipt embeds both runs).
- `cargo test --test corpus_conformance` — G8 rows flip from fail to pass.
- `cargo test` — full host suite green (no regressions in `sql_rdf_dialect`,
  `parse_refusal_court`, `tier_court`, which all consume `sniff`).
- `docs/refusals.md` gains the SNIFF-01 row (`BareSql`) — append-only.

## Scope and lane lease

This lane owns only `docs/jira/v26.10.23/design-sniff-fixes.md`. Implementation
(`src/dialect/sniff.rs`, `tests/sniff_court.rs`, `docs/refusals.md` append) is
dispatched to a code lane against this spec; `Cargo.toml` is coordinator-only
and no manifest change is required.

## See Also

- `docs/refusals.md` (append-only refusal register)
- `tests/corpus_conformance.rs` (G8 surfacing court)
