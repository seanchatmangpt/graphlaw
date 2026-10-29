# PATCH RECEIPT — duckdb_eval TermDoc + purrdf 3.0.1 staged edits (apply-on-signal)

Lane: EXECUTION (lane-e05), 2026-10-24. Lease: `src/duckdb_eval.rs`,
`src/lib.rs` (purrdf authority rows only), `README.md:154`, `GOALS.md:14`,
`docs/PHD_THESIS_GRAPHLAW_WASM_SQLRDF.md` (purrdf sites only).

This file is the deliverable. **No tree edit was made by this lane.** The coordinator
applies PATCH A through E below as part of the single bump commit
(`purrdf = "=3.0.1"` + E04 re-vendor + lockfile regen). Single-writer law:
`src/duckdb_eval.rs` must compile before and after the transition; only the
coordinator's bump commit flips the file in the same commit that makes the new
code resolvable.

Tree state at staging (pre-existing, NOT this lane's damage, witnessed by
`cargo check --locked` with `CARGO_TARGET_DIR=target-lane-e05`): the working tree
carries an uncommitted 58-file, +55,878-line rewrite of `vendor/purrdf-sparql-eval`
whose sources already target 3.0.1 APIs (`purdfrf_core::TermBox`,
`purdfrf_iri::vocab::*`, `purdfrf_xsd::ieee`) while the pin is still
`purrdf = "=2.0.2"`. The check therefore fails with 177 error locations, 100 percent
under `vendor/purrdf-sparql-eval` (verified: zero error blocks point outside
`vendor/`). All three names the vendor imports exist in the 3.0.1 sources
(`purdfrf_iri::vocab` — /tmp/purrdf-301/crates/iri/src/lib.rs:162;
`purdfrf_xsd::ieee` — xsd/src/lib.rs:175; `purdfrf_core::TermBox` —
rdf-core/src/lib.rs:210), so the bump commit alone unlocks the vendor.

## Preconditions (coordinator's bump commit, in order)

1. `Cargo.toml`: `purrdf = "=2.0.2"` becomes `purrdf = "=3.0.1"` (coordinator-owned file).
2. Apply PATCH A through E below.
3. `Cargo.lock` regen; result must contain `name = "purrdf"` / `version = "3.0.1"`.
4. `tests/authority_manifest.rs:129` (test-lane/coordinator owned): the lockfile
   literal must flip in the same commit or `resolved_lockfile_agrees` reds:
   BEFORE `assert!(lock.contains("name = \"purrdf\"\nversion = \"2.0.2\""));`
   AFTER  `assert!(lock.contains("name = \"purrdf\"\nversion = \"3.0.1\""));`
5. `src/law.rs:947` — see "Break disclosure" below; law.rs is outside this lane's lease.

## PATCH A — src/duckdb_eval.rs

Upstream change, verified in both sources: `TermValue::Triple` fields are
`Box<Self>` in 2.0.2 (purrdf-core-2.0.2 `src/ir/term.rs:294-301`) and `TermBox` in
3.0.1 (/tmp/purrdf-301/crates/rdf-core/src/ir/term.rs:351-358), where
`pub type TermBox = NestedBox<TermValue>` = `purdfrf_lex::walk::Nested<TermValue>`
(term_walk.rs:50; `use ...walk::Nested as NestedBox` at :42).

### A-1: doc comment on `enum SerTerm`, lines 75-81

BEFORE (verbatim, src/duckdb_eval.rs:75-81):

    /// Lossless serde shadow of `purrdf::TermValue` (SQLRDF-22 G3): upstream
    /// `TermValue` offers no serde support, so this tagged enum carries every
    /// variant byte-for-byte — IRI, blank node (label + scope ordinal), literal
    /// (lexical form, datatype IRI, language tag, base direction token), and
    /// recursive quoted triples. `Serialize`/`Deserialize` are exact inverses;
    /// no string rendering, no precision loss. If purrdf lands native serde,
    /// delete this shadow in favor of it.

AFTER (replace lines 80-81; lines 75-79 unchanged):

    /// Lossless serde shadow of `purrdf::TermValue` (SQLRDF-22 G3): upstream
    /// `TermValue` offers no serde support, so this tagged enum carries every
    /// variant byte-for-byte — IRI, blank node (label + scope ordinal), literal
    /// (lexical form, datatype IRI, language tag, base direction token), and
    /// recursive quoted triples. `Serialize`/`Deserialize` are exact inverses;
    /// no string rendering, no precision loss. The shadow is permanent: purrdf
    /// 3.0.1 removed serde from ALL public types by upstream design (zero
    /// `serde` occurrences in purrdf-rdf-core 3.0.1), so the
    /// delete-if-native-serde clause is unreachable. Do not re-propose it.

### A-2: `impl From<TermValue> for SerTerm`, lines 118-122

BEFORE (verbatim):

    TermValue::Triple { s, p, o } => SerTerm::Triple {
        s: Box::new((*s).into()),
        p: Box::new((*p).into()),
        o: Box::new((*o).into()),
    },

AFTER:

    TermValue::Triple { s, p, o } => SerTerm::Triple {
        s: Box::new(s.into_inner().into()),
        p: Box::new(p.into_inner().into()),
        o: Box::new(o.into_inner().into()),
    },

Reason: the arm binds `s, p, o` by value as `TermBox`. `Nested` derefs to `T` but
has no deref-move (only `Box` has the compiler special case), so `(*s).into()` is
E0507 under 3.0.1. `TermBox::into_inner(self) -> TermValue` (walk.rs:236-238)
consumes the box lawfully; the inner `.into()` is the same-impl component
conversion as before. The by-value match itself is why upstream keeps the drop on
the box (term_walk.rs:13-16) — the arm shape is unchanged.

### A-3: `impl From<SerTerm> for TermValue`, lines 150-154

BEFORE (verbatim):

    SerTerm::Triple { s, p, o } => TermValue::Triple {
        s: Box::new((*s).into()),
        p: Box::new((*p).into()),
        o: Box::new((*o).into()),
    },

AFTER:

    SerTerm::Triple { s, p, o } => TermValue::Triple {
        s: purrdf::TermBox::new(TermValue::from(*s)),
        p: purrdf::TermBox::new(TermValue::from(*p)),
        o: purrdf::TermBox::new(TermValue::from(*o)),
    },

Reason: the target field type is `TermBox`; `*s` on `Box<SerTerm>` keeps its
legal deref-move; `purdfrf::TermBox` is reachable at the purrdf root
(rdf-core/src/lib.rs:210 re-exports `TermBox`; facade lib.rs:133 does
`pub use purrdf_rdf::*`). Alternative with identical semantics:
`Nested::new` spelled `purdfrf::TermBox::new` is what is used here.

## PATCH B — src/lib.rs, the 7 purrdf authority rows

Scope guard: the eyeron row (lines 226-230, revision string
`"0.7.7 (eyereasoner/eyeron@d6568f657c19805b64223acf28d74234156bb837, vendored)"`)
and the frozen-duckdb row (lines 231-235, also `revision: "2.0.2"`) are NOT
touched. Because the frozen-duckdb row shares the literal `revision: "2.0.2",`,
each edit must be scoped by its full 5-line row block (unique via the
`capability:` line). In all 7 rows only the revision value changes:

| row line (revision) | capability | authority |
|---|---|---|
| 194 | "RDF 1.2 / codecs / storage IR" | "purrdf" |
| 199 | "SPARQL 1.1/1.2" | "purrdf::sparql" |
| 204 | "SHACL" | "purrdf::shapes" |
| 209 | "ShEx 2.1" | "purrdf::shex" |
| 214 | "Datalog" | "purrdf::datalog" |
| 219 | "RDF/RDFS/OWL-RL entailment" | "purrdf::entail" |
| 224 | "Knowledge hooks (kh: orchestration over SPARQL)" | "purrdf::sparql" |

Per row, BEFORE (5-line block, shown for row 1; the other six are identical in
shape with their own capability/authority values):

    BackendAuthority {
        capability: "RDF 1.2 / codecs / storage IR",
        authority: "purrdf",
        revision: "2.0.2",
    },

AFTER: same block with `revision: "3.0.1",`.

## PATCH C — README.md line 154

BEFORE (verbatim):

    RDF 1.2, codecs, SPARQL 1.1/1.2, SHACL, ShEx 2.1, Datalog/stratified fixpoint, RDF/RDFS/OWL-RL entailment | PurRDF 2.0.2 (`purrdf = "=2.0.2"`)

AFTER:

    RDF 1.2, codecs, SPARQL 1.1/1.2, SHACL, ShEx 2.1, Datalog/stratified fixpoint, RDF/RDFS/OWL-RL entailment | PurRDF 3.0.1 (`purrdf = "=3.0.1"`)

## PATCH D — GOALS.md line 14

BEFORE (verbatim):

    - PurRDF 2.0.2: RDF 1.2, SPARQL 1.1/1.2, SHACL, ShEx 2.1, Datalog, RDF/RDFS/OWL-RL entailment, RDF events.

AFTER:

    - PurRDF 3.0.1: RDF 1.2, SPARQL 1.1/1.2, SHACL, ShEx 2.1, Datalog, RDF/RDFS/OWL-RL entailment, RDF events.

## PATCH E — docs/PHD_THESIS_GRAPHLAW_WASM_SQLRDF.md, purrdf version sites

Each edit is the substring `PurRDF 2.0.2` (or the line-10 backticked form) becoming
the 3.0.1 form. Eyeron and frozen-duckdb version strings on the same lines are NOT
touched. Sites (11 total; verified absent elsewhere: lines carrying a bare
"PurRDF" with no version — 22, 880 — and `purrdf::` API paths — 37, 124, 167,
172, 437, 698 — are out of scope, as are the `vendor/purrdf-sparql-eval` name
sites 568, 1343-1355, 1514, which survive the bump since the vendor directory
name is unchanged):

| line | BEFORE substring | AFTER substring |
|---|---|---|
| 10 | ``1. `PurRDF = 2.0.2` `` | ``1. `PurRDF = 3.0.1` `` |
| 88 | `PurRDF 2.0.2` | `PurRDF 3.0.1` |
| 135 | `PurRDF 2.0.2` | `PurRDF 3.0.1` |
| 184 | `PurRDF 2.0.2` | `PurRDF 3.0.1` |
| 198 | `PurRDF 2.0.2` | `PurRDF 3.0.1` |
| 201 | `PurRDF 2.0.2` | `PurRDF 3.0.1` |
| 242 | `PurRDF 2.0.2` | `PurRDF 3.0.1` |
| 249 | `PurRDF 2.0.2` | `PurRDF 3.0.1` |
| 250 | `PurRDF 2.0.2` | `PurRDF 3.0.1` |
| 451 | `PurRDF 2.0.2` | `PurRDF 3.0.1` |
| 565 | `PurRDF 2.0.2` | `PurRDF 3.0.1` |

The replacement is mechanical: on each listed line, the first `PurRDF 2.0.2`
occurrence becomes `PurRDF 3.0.1`. No line carries more than one occurrence.

## Break disclosure (outside this lane's lease): src/law.rs:947

2.0.2 `ValidationResult.message: Option<String>`
(purrdf-shapes-2.0.2 src/report.rs:102) became 3.0.1
`ValidationResult.messages: Vec<Literal>`
(/tmp/purrdf-301/crates/shapes/src/report.rs:321; Literal at shapes/src/term.rs:111
with `value() -> &str` at :172).

law.rs:947 reads `r.message.clone().unwrap_or_default()` and does not compile
under 3.0.1. Suggested coordinator fix (keeps the first-message semantics the
line had; Verified shapes::Literal has no Display, only `value()`):

    message: r.messages.first().map(|l| l.value().to_string()).unwrap_or_default(),

This is the one incompatibility in an otherwise clean sweep; it is why the
per-file count is 12-of-13-compatible (law.rs partially), not 13-of-13.

## Nested contract verification (purrdf 3.0.1, purrdf_lex::walk)

From /tmp/purrdf-301/crates/lex/src/walk.rs:

- `pub struct Nested<T: Dismantle>(Option<Box<T>>)` (:225).
- Construction: `Nested::new(value: T)` (:230), `impl From<T>` (:308),
  `impl From<Box<T>>` (:314).
- Consumption: `into_inner(self) -> T` (:236), `into_box(self) -> Box<T>` (:242),
  `take(&mut self) -> Option<Box<T>>` (:249).
- Access: Deref (:274), DerefMut (:282), AsRef/AsMut/Borrow (:290-306). No
  deref-move: `*nested` cannot be moved out (generic Deref returns and T is behind
  a non-Box smart pointer; the Box deref-move is a compiler special case Nested
  does not get).
- Trait delegation to the inner value: Clone (:320), PartialEq/Eq (:326-332),
  PartialOrd/Ord (:334-344), Hash (:346), Debug (:352).
- Drop (:266-272) routes through `T::dismantle` — a work-list walk, never
  recursion; `Dismantle for TermValue` at rdf-core term_walk.rs:52-72.
- `pub type TermBox = NestedBox<TermValue>` (rdf-core term_walk.rs:50).

Consequence for the patch: match arms destructuring
`TermValue::Triple { s, p, o }` by value keep working unchanged (that is exactly
what upstream's term_walk.rs:13-16 doc says the box is for); only the six
expressions that deref-moved the components had to change.

## Per-file compatibility matrix (purrdf 3.0.1 sources at /tmp/purrdf-301)

| file | purrdf API used | verdict | evidence |
|---|---|---|---|
| src/dialect.rs | `RdfDiagnostic` fields code/location, Display | COMPATIBLE | 3.0.1 diagnostic.rs:475-486: `#[non_exhaustive]` struct, `pub code: String` (:480), `pub location: Option<Box<RdfLocation>>` (:486); location line/col (:310/:312); Display (:684). dialect.rs:224-252 uses field access only, never variant matching, so non_exhaustive is harmless. |
| src/hooks.rs | root `SparqlEngine, SparqlRequest, SparqlResult, RdfDatasetBuilder, TermValue`; `sparql::{NativeSparqlEngine, SparqlParser}` | COMPATIBLE | root re-export survives: crates/rdf/src/lib.rs:242 exports the three SPARQL types; facade lib.rs:133 `pub use purrdf_rdf::*`; `pub mod sparql` facade lib.rs:198. SparqlRequest fields identical in both versions (2.0.2 backend.rs:128-135 vs 3.0.1 backend.rs:136-143). SparqlResult::Solutions/Graph present (3.0.1 backend.rs:151,163). Builder new/freeze/push_dataset/intern_iri: builder.rs:493/1450/1055/532, freeze error RdfDiagnostic: Debug. NativeSparqlEngine::new engine.rs:858, impl SparqlEngine :4333. |
| src/shacl_guard.rs | `parse_dataset`, `RdfDataset, TermId, TermValue` | COMPATIBLE | parse_dataset signature AND Arc return identical in both versions (2.0.2 parse.rs:137-141, 3.0.1 parse.rs:134-138, both `Result<Arc<RdfDataset>, RdfDiagnostic>`); `&ds` deref-coerces to `&RdfDataset` for check_dataset_depth. |
| src/tier.rs | `Arc<purrdf::RdfDataset>` | COMPATIBLE | parse_dataset already returned Arc in 2.0.2; no explicit Arc::new rewrap at the call sites. |
| src/abi.rs | `serialize_dataset(ds, "application/n-quads", SerializeGraph::Dataset)` | COMPATIBLE | 3.0.1 serialize.rs:119-123 keeps the exact 3-arg form `(&D, &str, SerializeGraph) -> Result<Vec<u8>, RdfDiagnostic>`; SerializeGraph exported (rdf-core lib.rs:155). |
| src/receipt.rs | same serialize_dataset | COMPATIBLE | same evidence. |
| src/plan.rs | same serialize_dataset | COMPATIBLE | same evidence. |
| src/smon.rs | `RdfLiteral`, serialize/parse_dataset, `TermId` | COMPATIBLE | RdfLiteral model.rs:29; BlankScope `pub struct BlankScope(pub u32)` term.rs:129 with `ordinal()` :137. |
| src/sql_ast.rs | parse_dataset (:1821,:2366,:3828), TermValue variant matches (:2371-2390), try_canonicalize (:3829), parse_shapes/validate_dataset via `crate::shacl` shim (:1824,:1827) | COMPATIBLE | `crate::shacl` is `pub use purrdf::shapes::*` (lib.rs:142-143); the `.map_err(|_| ...)` at :1822/:1825/:1828 discards the error value, so the 2.0.2 `Result<_, String>` to 3.0.1 `Result<_, ShapesError>` change survives untouched. try_canonicalize identical (2.0.2 canon.rs:498, 3.0.1 canon.rs:495). |
| src/law.rs | parse/serialize/canonicalize, `entail::Materialization::{Rdfs,OwlRl}` + materialize, `shapes::engine::{parse_shapes,validate_dataset}` | PARTIAL — one break | :933-935 SURVIVES: `Refusal::engine(dialect, message: impl std::fmt::Debug)` (src/dialect/refusal.rs:120) and 3.0.1 ShapesError derives Debug (shapes error.rs:44; Display also at :156). Materialization variants Rdfs/OwlRl present (entail lib.rs:295-296); materialize `<D: DatasetView>` :814; ValidationResult focus_node/result_path/source_constraint_component/severity all present (report.rs:289-312). The break is :947 (message field) — see disclosure above. |
| src/duckdb_op.rs | `TermValue::Iri/Blank` matches (:794-795) | COMPATIBLE | variants unchanged in 3.0.1 term.rs:327-359. |
| src/duckdb_eval.rs | the 6 TermDoc sites | FIXED BY PATCH A | upstream field type change Box -> TermBox; see Nested contract above. |

Also verified for the vendor unlock: `purdfrf_iri::vocab` (iri lib.rs:162),
`purdfrf_xsd::ieee` (xsd lib.rs:175), `purdfrf_core::TermBox` root re-export
(rdf-core lib.rs:210) — the three names vendor/purrdf-sparql-eval imports that
2.0.2 does not provide.

Serde claim grounding for A-1: zero `serde` occurrences in
/tmp/purrdf-301/crates/rdf-core/Cargo.toml and src/ (grep count 0), and no
serde derive on TermValue/backend types — the shadow has no native-serde
replacement to migrate to.

## Gate (honest record)

- Command: `CARGO_TARGET_DIR=/Users/sac/graphlaw/target-lane-e05 cargo check --locked`.
- Result: FAILS — 177 error locations, all under `vendor/purrdf-sparql-eval`
  (uncommitted 58-file, +55,878-line rewrite vs HEAD, targeting 3.0.1 APIs against
  a 2.0.2 registry). Zero error blocks outside vendor/. This lane made no edits;
  the failure is the pre-existing vendor-ahead-of-pin state that this receipt's
  bump preconditions resolve.
- This lane's own lease files are byte-identical to lane start; the patch above is
  the staged content, not an applied edit.
