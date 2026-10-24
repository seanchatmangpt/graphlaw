# Dialects

How GraphLaw routes RDF and logic syntaxes to upstream engines, what each
dialect supports, and what is permanently out of scope. Source of truth:
`src/dialect.rs`; the wire op table: `src/registry.rs`.

## Two-tier model

GraphLaw splits syntaxes into two tiers. The tier model is landed code:
`src/tier.rs` (module doc: "Two-tier document routing (R04)") defines
`Tier::{Rdf, N3Logic}`, `Dialect::tier()`, declared-tier admission
(`DialectDecl`, `admit_declared`, `admit_bytes_tiered`), and
`validate_n3_rules` (Eyeron-validated `RuleSet` with rule counts). When a
caller supplies a `DialectDecl`, sniffing is skipped entirely and dispatch is
on the declared tier; without a declaration, sniffing behaves as before.

- **Data tier (PurRDF)** — RDF dataset dialects (Turtle, TriG, N-Triples,
  N-Quads, RDF/XML, JSON-LD, YAML-LD, TriX, HexTuples). These are parsed,
  converted, canonicalized, and queried through `graphlaw::dialect` routing
  into PurRDF.
- **Logic tier (Eyeron)** — Notation3 (`Dialect::N3`,
  `Tier::N3Logic`) is owned by Eyeron
  (`Dialect::N3.engine() == Engine::Eyeron`). N3 carries forward rules and
  quantifiers (`=>`, `@forAll`/`@forSome`); it is a separate logic surface
  (the `n3` op with its own iteration/fact/byte caps), not another way to
  spell a dataset. N3 input to RDF data operations is refused as
  `TierMismatch`, never silently down-converted. The same applies in the law
  engine: `Step::DeriveN3` (`src/law.rs`) admits its rules text through
  `tier::validate_n3_rules` and refuses a carried `RuleSet` that disagrees
  with the text's rule count as `TierMismatch`, before any derivation runs.

Routing (`sniff`) decides *which* engine owns a document; it never parses one.
The owning engine is always the final authority. A document routing cannot
place is refused, not tried against every engine in turn.

## Relational Dialect (`sql-ast.ttl`)

In addition to pure RDF dataset and N3 logic dialects, GraphLaw admits declarative relational query specifications modeled in RDF Turtle under the `http://graphlaw.org/sql#` namespace.
- **Owning Engine**: `frozen-duckdb 2.0` (`src/duckdb_eval.rs`).
- **Validation**: Enforced via SHACL pre-flight gate (`validate_ast_shacl`, invoked by `compile_turtle`). Non-conforming ASTs are rejected before reaching the execution layer with typed `AstError::ShaclViolation` refusals carrying stable `[REFUSAL-nn]` codes (`docs/refusals.md`). `sql:whereClause` is optional.
- **Statement surface**: 35 statement classes (`SQL_STATEMENTS`/
  `SQL_EXTENSIONS`, `src/sql_statement_kinds.rs`, generated from
  `ggen/sql-dialect/`); the runtime executes 33 to completion and refuses
  `ConnectStatement`/`DisconnectStatement` with typed refusals (sql-rdf Rec.
  section 5). Every statement node carries a required integer
  `sql:statementIndex` — missing or non-integer is
  `REFUSAL_01_MALFORMED_AST` (`src/sql_ast.rs`, section 4 StatementBaseShape).
- **Receipt**: `ExecutionReceipt` (`src/duckdb_eval.rs`) is identified by
  `H_query = BLAKE3(Canon(G_AST) || P_params)` — RDFC-1.0 canonical AST graph
  bytes concatenated with the typed parameter buffer (sql-rdf Rec. 1.4) —
  plus `query_iri`, parameterized `sql_statement`, `rows_emitted`,
  `execution_ms`, and materialized `result_triples`.
- **Connections**: default `GraphLawDuckDB::open_in_memory` runs unsigned
  extensions off; `allowing_unsigned_extensions` is the opt-in boundary. The
  de-skipped §6.2 extension court (`tests/duckdb_statement_kinds.rs`,
  `section6_2_certified_checks_run_through_compiled_ast_graphs`) proves the
  five certified extensions (`core_functions`, `icu`, `json`, `parquet`,
  `tpch`) answer through compiled AST graphs on a real unsigned-extensions
  connection, no skip branch.
- **Mixing discipline**: legacy aliases (`sql:selectList`, `sql:fromTable`,
  `sql:whereClause`) stay legal alongside `sql:select`/`sql:from`/`sql:where`,
  but dual-carrying either pair is REFUSAL-01 — exactly one projection list,
  one data source clause, one filter clause (`src/sql_ast.rs`; never silently
  ignored).
- **Output**: Vectorized execution directly over Parquet partitions or in-memory tables, emitting typed RDF statements accompanied by a BLAKE3 query execution receipt.

### Projection surface (SQLRDF-25)

Two SQLRDF-25 additions extend the projection/expression surface. Vocabulary
is pinned in `ontologies/sql-ast.ttl`; extraction lives in `src/sql_ast.rs`
(star/JSON/composite extractors) and `src/sql_ast/ast.rs` (operators,
`SqlExpr::validate_composite`); the render sites are
`src/sql_ast/serializer.rs` and `src/sql_ast.rs`.

**Star modifiers** (`sql:Star` / `sql:Columns`) — projection-list-only: a
`sql:Star` or `sql:Columns` node routed outside the projection list (e.g.
into `sql:where`) is REFUSAL-01 (`src/sql_ast.rs`,
`extract_select_item`). Modifiers:

- `sql:exclude` — rdf:List of column nodes to drop from `*`
  (renders `SELECT * EXCLUDE (col, ...)`). An empty `sql:exclude ()` is a
  malformed-member REFUSAL-01.
- `sql:replace` — rdf:List of `[ sql:expression <expr> ;
  sql:aliasName "col" ]` members re-binding star columns; exclude/replace
  order is preserved on render.
- `sql:columnsPattern` — a regex string literal on a `sql:Columns` node
  (renders `COLUMNS('^pattern')`).

Exclude identifiers are strictly validated (injection fence: a column name
carrying SQL-breakage characters is REFUSAL-01).

```turtle
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:S a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( ex:star ) .

ex:star a sql:Star ;
    sql:exclude ( [ sql:columnName "cost" ] ) ;
    sql:replace ( [ sql:expression [ sql:literalValue "500"^^xsd:integer ] ;
                    sql:aliasName "cost" ] ) .
```

```turtle
ex:cols a sql:Columns ;
    sql:columnsPattern "^(batch|region)"^^xsd:string .
```

**JSON operators** — `sql:JsonExtract` (`->`) and `sql:JsonExtractText`
(`->>`) are carried as binary expressions:
`sql:operator sql:JsonExtract ; sql:leftOperand <expr> ;
sql:rightOperand <path literal>` (`src/sql_ast/ast.rs` `Operator::JsonExtract`
/ `JsonExtractText`; operator tokens are settled in `src/sql_ast/ast.rs`,
read through `read_operator` in `src/sql_ast.rs`).

```turtle
[ sql:aliasName "extracted_json" ; sql:expression [
    sql:operator sql:JsonExtract ;
    sql:leftOperand [ sql:columnName "payload" ] ;
    sql:rightOperand [ sql:literalValue "$.k" ] ] ]
```

**Composite literals and accessors** — discriminated by `rdf:type`
(`composite_kind`, `src/sql_ast.rs`): `sql:StructLiteral` (fields),
`sql:ListLiteral` (items), `sql:IndexAccess` (`base[index]` — shares the
`->` render site with `JsonExtract`), `sql:SliceAccess`
(`base[start:end]`).

- `sql:StructLiteral` carries `sql:structFields` (rdf:List of
  `[ sql:fieldKey "k" ; sql:fieldValue <expr> ]`); field keys must satisfy
  strict identifier validation — a malformed member is
  `[REFUSAL-01 arm 49 COMPOSITE_MEMBER]`, nesting beyond the cap is
  `[REFUSAL-01 arm 48 COMPOSITE_DEPTH]` (cap `MAX_COMPOSITE_DEPTH = 64`,
  `src/sql_ast/ast.rs`; the cap bounds recursion, so a hostile deep graph
  cannot overflow the stack).
- `sql:ListLiteral` carries `sql:listItems` (rdf:List of expressions).
- `sql:IndexAccess` carries `sql:indexBase` + `sql:indexIndex`.
- `sql:SliceAccess` carries `sql:sliceStart` (+ optional `sql:sliceEnd`).

```turtle
[ sql:aliasName "packed" ; sql:expression [
    a sql:StructLiteral ;
    sql:structFields ( [ sql:fieldKey "kind" ;
                         sql:fieldValue [ sql:literalValue "batch" ] ]
                       [ sql:fieldKey "n" ;
                         sql:fieldValue [ sql:literalValue "3"^^xsd:integer ] ] ) ] ]
```

```turtle
[ sql:aliasName "second_reading" ; sql:expression [
    a sql:IndexAccess ;
    sql:indexBase [ a sql:ListLiteral ;
        sql:listItems ( [ sql:literalValue "10"^^xsd:integer ]
                        [ sql:literalValue "20"^^xsd:integer ]
                        [ sql:literalValue "30"^^xsd:integer ] ) ] ;
    sql:indexIndex [ sql:literalValue "2"^^xsd:integer ] ] ]
```

Other malformed-composite and malformed-star refusals are the REFUSAL-01
family (see docs/refusals.md); arms 48-52 are settled codes (48
COMPOSITE_DEPTH, 49 COMPOSITE_MEMBER, 50 STAR_MODIFIER_POSITION,
51 COLUMNS_LAMBDA, 52 COLUMNS_PATTERN — `src/sql_ast/refusals.rs:119-144`).

Courts: `tests/composite_literal_court.rs` (Row 4: real frozen-duckdb
readbacks, depth-overflow and malformed-key refusals),
`tests/json_operator_court.rs` (Row 5), and the star-modifier lib tests in
`src/sql_ast.rs` (`exclude_compiles_to_star_minus_columns`,
`replace_compiles_and_keeps_exclude_order`,
`exclude_identifiers_are_strictly_validated`).

### DuckDB op session mode (persistent connections)

The `duckdb` ABI op (`src/duckdb_op.rs`) carries a named-session surface:
a `mode: "open"` call registers a persistent in-memory connection that
later `exec`/`stream` calls reuse, so a CREATE in one call is visible to a
later INSERT/SELECT on the same session.

Wire shapes:

- Open: `{"op":"duckdb","mode":"open","session":"<name>"}` →
  `{"ok":true,"session":"<name>"}`. Same GAP-2 pragmas as every connection
  (memory_limit 32 MiB, threads=1); callers cannot override them from the
  wire.
- Exec/stream: the optional `"session"` field selects the named session
  connection; without it, `exec` opens a fresh in-memory connection per
  call (per-call ephemeral DDL/DML — back-compat, unchanged).
- Close: `{"op":"duckdb","mode":"close","cursor"?,"session"?}` drops a
  cursor and/or a session; unknown handles are idempotent-ok so a
  crash-recovery path cannot wedge on close.

Typed refusals (never a trap):

- Unknown or expired session on `exec` with `"session"`:
  "unknown or expired session `<name>`" (`src/duckdb_op.rs:323`).
- Duplicate open: "session `<name>` already exists (close it before
  reopening)" (`src/duckdb_op.rs:530`).
- Over-cap open: 16 live sessions max (`MAX_SESSIONS`,
  `src/duckdb_op.rs:79`); the 17th open is a `duckdb_sessions`
  ResourceLimit refusal (`src/duckdb_op.rs:533`).

Registry semantics: the session registry is process-local and crash-only
(`sessions()`, `src/duckdb_op.rs:83-85`) — it dies with the process, like
cursor state. Closing a session frees its connection; a subsequent open of
the same name succeeds.

## Document sequences (`"document":true` exec)

SQLRDF-23 adds a whole-document exec path to the `duckdb` ABI op. The wire
request carries `"document":true` plus the Turtle AST document:

```json
{"op":"duckdb","mode":"exec","document":true,"query_ast":"<iri>",
 "ast":{"turtle":"<Turtle AST document>"},
 "subject_prefix":"...","vocab_prefix":"..."}
```

Routing (`src/duckdb_op.rs:372-374`): the flag diverts to
`exec_document_on` (`src/duckdb_op.rs:463`) before the single-statement
compile path; the legacy path is untouched. The document compiles via
`compile_turtle_document` and evaluates member-by-member on the target
connection — a named session or a fresh connection, same routing as the
single-statement path.

Laws on the document path:

- **SQLRDF-17, document form**: wire `params` are refused — a non-empty
  wire param list is a typed refusal because each sequence member's
  literals live in the Turtle AST document itself
  (`src/duckdb_op.rs:474-477`). The AST document is the admission record.
- **G6 ATTACH policy**: every member passes `enforce_attach_policy`
  before the engine sees it (`src/duckdb_op.rs:479-481`), applied
  per member, not per document.
- **`ast.text` stays refused** (`src/duckdb_op.rs:357-360`) — graph-backed
  statements only.

Success answers a BatchReceipt wire shape:

- `query_iri` — caller-supplied envelope IRI.
- `doc_hash` — 64-hex BLAKE3 document hash, domain-separated, pinned by
  `compile_turtle_document` (sequence-of-1 differs from bare form by
  design; bare identity form is bit-identical to the pre-feature path).
- `members` — one `ExecutionReceipt` per executed member, spine order.
- `rows_emitted` — per-member row counts, same indexing.
- `failed_at` — `null` on full success.

A mid-sequence member failure is the typed `BatchMemberFailed` envelope,
never a generic DB error (`src/duckdb_op.rs:507-524`): details carry
`code: "BatchMemberFailed"`, `member` (0-based index), `message`, and
`partial_receipt` — a `BatchReceipt` (`src/duckdb_eval.rs:408-421`) whose
`members[0..failed_at]` executed and committed (documented best-effort
sequential autocommit, `src/duckdb_eval.rs:401-405`); `members[i..]` did
not execute. The error type is `EvaluatorError::BatchMemberFailed`
(`src/duckdb_eval.rs:209-212`), folded by `eval_ast_document`
(`src/duckdb_eval.rs:1052`).

Courts (lib tests, `src/duckdb_op.rs:1091+`): three-member CREATE→INSERT→
SELECT with real table readback; failing member yields the typed
`BatchMemberFailed` envelope with partial receipt; legacy exec without the
flag is unchanged. Witnessed 2026-10-23 (earlier lanes, not re-run this
session — suite runs DISK-BLOCKED): duckdb_op lib 16 passed / 0 failed
(3 document courts), duckdb_eval lib 18/0, sql_rdf_dialect 32/0.

## Support matrix

The 14 dialects of `Dialect` (`src/dialect.rs`), and what the current surface
supports:

| Dialect    | Engine | sniff | check | parse/convert | canonicalize |
|------------|--------|-------|-------|---------------|--------------|
| Turtle     | PurRDF | yes   | yes   | yes           | yes          |
| TriG       | PurRDF | yes (needs `trig` hint vs ShExC) | yes | yes | yes |
| N-Triples  | PurRDF | yes   | yes   | yes           | yes          |
| N-Quads    | PurRDF | yes   | yes   | yes           | yes          |
| RDF/XML    | PurRDF | yes   | yes   | yes           | yes          |
| JSON-LD    | PurRDF | yes   | yes   | yes           | yes          |
| YAML-LD    | PurRDF | no (hint only) | yes | yes | yes |
| TriX       | PurRDF | yes (`<TriX`) | yes | yes | yes |
| HexTuples  | PurRDF | yes (`["`) | yes | yes | yes |
| N3         | Eyeron | yes (`=>`, `@forAll`, `@forSome`) | yes | no (logic tier: `n3` op only) | no |
| ShExC      | PurRDF | yes (needs `shex`/`shexc` hint vs TriG) | yes | no (validation only) | no |
| ShExJ      | PurRDF | yes (`"type":"Schema"`) | yes | no (validation only) | no |
| Sparql     | PurRDF | yes (SELECT/CONSTRUCT/ASK/… keywords) | yes | no (query/update text) | no |
| SqlRdf     | frozen-duckdb | yes (`sql:SelectStatement`, or a `sql` hint) | yes | yes (dataset parse: `media_type` is `Some("text/turtle")`, so `parse` returns quads + canonical id; shape check runs first) | yes |

Notes, grounded in the code:

- "yes" for canonicalize means the dialect denotes an RDF dataset
  (`Dialect::media_type()` is `Some`) and can be canonicalized to RDFC-1.0
  N-Quads via the `canonical` op / `LawState::from_dataset`. The canonical id
  is the BLAKE3 of the RDFC-1.0 canonical N-Quads.
- `convert` serializes between RDF **dataset** dialects only
  (`to` = target dialect name). ShExC/ShExJ/SPARQL/N3 have no serializer;
  SqlRdf does (it denotes a Turtle dataset).
- Ambiguities that content alone cannot resolve — TriG vs ShExC
  (brace-delimited) — return `RefusalKind::Ambiguous` demanding a hint rather
  than guessing (`sniff`, `src/dialect.rs`).

## Refusal kinds

Seven `RefusalKind` variants (`src/dialect.rs`, `#[non_exhaustive]` — always
match with a wildcard):

| Kind | Meaning |
|------|---------|
| `NotSemanticContent` | Empty input, an HTML page, or an HTTP error body. |
| `Ambiguous` | Routing cannot tell two dialects apart without a hint. |
| `EngineRejected` | The owning engine rejected the document. |
| `Unsupported` | Valid, but not usable in this operation. |
| `ResourceLimit` | A documented `abi::MAX_*` cap was exceeded. |
| `TierMismatch` | The document belongs to the other semantic tier (R04): N3 logic to an RDF parser, or N3 sniffed undeclared. See `src/tier.rs`. |
| `DuckDb` | The frozen-duckdb execution substrate (ABI op `duckdb`) failed or the request misused it. Never traps. The op dispatches inside the wasm unit too (frozen-duckdb links on every target; AST-compiled statements only, raw SQL text still refused — SQLRDF-17). The op staying off the hashed 14-op registry table is a digest-stability choice (`registry_blake3` moves only on an explicit digest bump; `NATIVE_ONLY_OPS`, `src/registry/definitions.rs`), not a host-only statement — absence from `capabilities.ops` is not this refusal and is not REFUSAL-04. |

## Typed parse refusals

Beyond the routing-level `Refusal`, parse failures carry a typed
`ParseRefusal` (`src/dialect.rs`): which engine, which dialect, a
machine-readable `ReasonClass`, location fields (1-based line/col, byte
offset — present only when the refusing backend supplies them, never
guessed), a snippet bounded at 64 bytes, and the diagnostic text.
`ReasonClass` variants (`#[non_exhaustive]`): `Syntax`, `UndefinedPrefix`,
`BadIri`, `FormulaInRdfLayer`, `NotSemanticContent`, `AmbiguousDialect`,
`RemoteContext`, `Limit { name, observed, max }`. Typed entry points:
`check_typed`, `parse_rdf_typed`.

A `Refusal` names the kind, the dialect and engine when known, and the
engine's diagnostic text. Wire-level refusal codes: [refusals.md](refusals.md)
and [abi-reference.md](abi-reference.md).

## Declaring dialects in packs

Packs that accept semantic input should declare the dialects they consume as
extension/media-subtype globs (the same strings `sniff` accepts as hints:
`trig`, `shex`, `shexc`, file extensions without the dot). Scope the glob to
what the pack actually routes: a pack feeding `admit_bytes` needs dataset
dialects only; a pack feeding the `n3` op declares `n3`. Do not declare
dialects the pack refuses downstream — routing refusals are the contract, and
over-declaring turns typed refusals into silent expectations.

## Consumer admission matrix (ggen-engine `load_declared`)

The ggen-engine fact-input path (`crates/ggen-engine/src/graph.rs`,
`load_declared`) admits declared dialects as of v26.10.20: `turtle`,
`ntriples`, `nquads`, `trig`, `rdf-xml`/`rdfxml`, `json-ld`/`jsonld`. RDF/XML
and JSON-LD are first-class — JSON-LD admission runs the same remote-context
guard as GraphLaw (`jsonld_has_remote_context`, guard pattern copied from
`graphlaw::dialect::jsonld_remote_context`; a remote `@context` is a typed
`FD_REMOTE_CONTEXT_FORBIDDEN` refusal, offline fail-closed, never a fetch).
`n3` is disallowed on the fact path (`FD_DIALECT_FEATURE_DISALLOWED`; N3 rule
documents route through `GraphEngine::load_rules`); any other declared string
is a typed `FD_UNDECLARED_INPUT_DIALECT` refusal, and a parse failure under a
declared dialect is a typed `FD_MALFORMED_SYNTAX` refusal naming the dialect —
never a silent skip.

## Courts

Cross-dialect claims are enforced by court tests, not prose:

- `tests/canonical_cross_dialect.rs` — same content in different dialects
  yields the identical RDFC-1.0 + BLAKE3 canonical id; named graph vs default
  graph distinguished; blank-node relabeling does not move identity; N3
  refused as `Unsupported`; non-vacuous (a one-quad perturbation moves the
  id); PurRDF refusals are typed skips in a run ledger, never silent.
- `tests/corpus_conformance.rs` — every semantic asset shipped in the
  repository is routed by content and accepted by its owning authority,
  pinned by `ASSETS.blake3`.
- `tests/tier_court.rs` — the data/logic tier separation as a court: declared-
  tier dispatch (`admit_declared`/`admit_bytes_tiered`), tier-mismatch
  refusals, and unchanged undeclared sniff behavior.
- `tests/parse_refusal_court.rs` — 12 tests over the typed parse-refusal
  surface: every `ReasonClass` constructible from a real bad input, backend
  location fields present only when supplied, snippet bounded at 64 bytes,
  legacy `check` still refusing as `EngineRejected`.
- `tests/w3c_conformance.rs` + `tests/w3c_manifest.rs` — the W3C RDF 1.1
  test-suite corpus (Turtle, N-Triples, N-Quads syntax tests; 325 manifest
  entries, generated by `scripts/gen_w3c_manifest.sh`). Every result must be
  pass or a **registered** divergence in `KNOWN_DIVERGENCES` /
  `KNOWN_POSITIVE_DIVERGENCES`; an unregistered divergence fails the court and
  a stale register row (no matching fixture) fails `w3c_known_divergences_are_
  exhaustive`. Currently 9 registered divergences (8 negative: long-string
  literals accepted in N-Triples/N-Quads, malformed langtag / bad BASE / N3
  extras accepted in Turtle; 1 positive: Turtle `123.E+1` DOUBLE refused).
  Per-dialect known-diverged counts are pinned by literal assertions that
  break when an engine fix closes a divergence, forcing register removal.

## Permanent boundaries

Stated as scope, not roadmap:

- **No remote JSON-LD context loading.** JSON-LD is parsed with the contexts
  present in the document; GraphLaw never fetches contexts over the network.
  (`ReasonClass::RemoteContext` / `FD_REMOTE_CONTEXT_FORBIDDEN`.)
- **No N3-as-codegen.** N3 is a logic/derivation surface (Eyeron); GraphLaw
  does not treat N3 rules as template/generation logic.
- **DeriveN3 rules text input (open follow-up, G02).** `Step::DeriveN3`
  carries its N3 rules as `&'a str` text plus an Eyeron-validated `RuleSet`
  that is re-checked at admission (rule-count agreement, `TierMismatch`
  otherwise). No additional input ergonomic (e.g. a raw-string/binary form
  for rules containing quote-heavy syntax) has landed; the follow-up is
  open, not a landed feature.
