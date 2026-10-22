# Phase 1 — generated admission gate + canon-bytes wire (design record)

Condensed from the Phase-1 spec lane. Extends E2: E2 generates the READER;
Phase 1 generates the GATE (shapes → compiled SHACL matcher) and moves the
wire to canonical bytes. Third consumer of the same embedded shapes bytes.

## Current admission surface (all runtime text parses)
payload Turtle ×2 (validate_ast_shacl + DatasetReader::from_turtle), shapes
TTL ×2 (parse_shapes + shacl_guard's fresh copy) — every compile_turtle call
re-parses both from scratch.

## Generated gate (`ggen/sql-gate/`, mirrors sql-reader/)
- One `fn admit_<ShapeId>(&Reader, node) -> Result<(), GateRefusal>` per
  sh:NodeShape, folded into a ClassId dispatch. Constraint→arm mapping:
  minCount/maxCount → object_count match (the require_index pattern,
  generalized); sh:datatype → datatype-literal check; sh:in/sh:class → const
  IRI table match; sh:node → nested admit_* call (shape nesting = call
  graph); path sequences → generated walks.
- Kernel-only references (E2 §4 split verbatim); depth guard becomes a BUILD
  fact: `SHAPES_MAX_NESTING_DEPTH` const asserted ≤ MAX_SHACL_VALIDATION_DEPTH
  — the runtime guard's reason (unbounded attacker shapes text) disappears on
  the generated path.
- Serialized shape constants: `SHAPE_GRAPH_NQUADS: &[u8]` (canonical N-Quads
  via the receipt.rs serializer) + build-time shadow verification.

## Wire boundary (the honest "strip runtime text parsers")
NOT "remove purrdf" (load-bearing beyond admission: re-exports, receipt
serializers). It means: no release admission entry point parses text.
Post-Phase-1 wire = canonical byte form: `compile_from_bytes(&[u8])` is the
release path (zero parse_* calls — enforced by a no-parse-reachability
grep-court, mutation-tested); `compile_turtle` becomes dev/test-only behind a
`runtime-turtle-admission` feature. **Hardest deliverable / acceptance gate:**
canon bytes must deserialize into the reader index WITHOUT a text parse
(binary index, or N-Quads behind the dev cfg during transition).

## C2-adjacent flag (adjudicate, don't absorb)
The wire flip changes what duckdb_op.rs accepts — a C2-surface decision.
Register against SQLRDF-17/C2 with its own falsifier ("census passes while a
byte-form payload that fails the generated gate still executes"); sequence
AFTER SQLRDF-17 closes; declare both accepted forms on the wire contract
during transition, one census row per form.

## Phasing (courts are the constraint — all 7 call compile_turtle(&str))
P1a E2 declaration sweep + reader gen (no admission change). P1b generated
gate in SHADOW mode (runs alongside validate_ast_shacl, asserts agreement,
never refuses — divergence fails the build). P1c flip, feature-gated:
compile_from_bytes release entry; courts untouched under dev cfg; new
no-parse-reachability court. P1d wire-format adjudication with C2/SQLRDF-17.
Falsifier: (a) shape property deletion changes neither generated gate nor
reader and flips no court; (b) release-cfg compile_from_bytes reaches a text
parser; (c) overhead asserted without a canon-once baseline receipt.
