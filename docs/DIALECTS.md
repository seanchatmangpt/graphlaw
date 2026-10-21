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

## Support matrix

The 13 dialects of `Dialect` (`src/dialect.rs`), and what the current surface
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

Notes, grounded in the code:

- "yes" for canonicalize means the dialect denotes an RDF dataset
  (`Dialect::media_type()` is `Some`) and can be canonicalized to RDFC-1.0
  N-Quads via the `canonical` op / `LawState::from_dataset`. The canonical id
  is the BLAKE3 of the RDFC-1.0 canonical N-Quads.
- `convert` serializes between RDF **dataset** dialects only
  (`to` = target dialect name). ShExC/ShExJ/SPARQL/N3 have no serializer.
- Ambiguities that content alone cannot resolve — TriG vs ShExC
  (brace-delimited) — return `RefusalKind::Ambiguous` demanding a hint rather
  than guessing (`sniff`, `src/dialect.rs`).

## Refusal kinds

Six `RefusalKind` variants (`src/dialect.rs`, `#[non_exhaustive]` — always
match with a wildcard):

| Kind | Meaning |
|------|---------|
| `NotSemanticContent` | Empty input, an HTML page, or an HTTP error body. |
| `Ambiguous` | Routing cannot tell two dialects apart without a hint. |
| `EngineRejected` | The owning engine rejected the document. |
| `Unsupported` | Valid, but not usable in this operation. |
| `ResourceLimit` | A documented `abi::MAX_*` cap was exceeded. |
| `TierMismatch` | The document belongs to the other semantic tier (R04): N3 logic to an RDF parser, or N3 sniffed undeclared. See `src/tier.rs`. |

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
