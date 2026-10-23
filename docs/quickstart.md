# GraphLaw quickstart

Admit a plan over an RDF state and get a content-addressed receipt chain. Needs Rust 1.96+.
The runnable source is `examples/quickstart_plan.rs`.

## The core

Build the plan with typed builders (no hand-written N-Triples), then admit it:

```rust
let is = |v| Triple::iri("urn:d:door", "urn:p:is", v);
let plan = Plan::builder()
    .action("open").requires(is("urn:v:closed")?).adds(is("urn:v:open")?).deletes(is("urn:v:closed")?)
    .goal(is("urn:v:open")?).build()?;
let admitted = plan.admit(&start)?;   // replays every action; refuses on the first unmet precondition
```

`Triple::iri` validates IRIs (refusing `<`, `>`, spaces, control characters) and `Triple::literal`
escapes quotes, backslashes and newlines, so a literal can never inject a second triple. The
struct-literal form (`Action { name, pre, pre_not, add, del }` — add
`..Default::default()` to leave `pre_not` empty; `Plan` likewise has `goal_not`) still
works and yields the same plan digest.
Admission replays the actions in order: every `pre` triple must hold, `del` is removed, then `add`
is added, and the `goal` must hold at the end. Each action yields one receipt.

## Run it

```sh
cargo run --example quickstart_plan
```

Expected output: four deterministic receipt-state hashes — three
`plan-action -> blake3:<64-hex>` lines followed by a matching
`final state: blake3:<64-hex>` line, where the final-state hash equals the
last plan-action hash. These are runtime receipt hashes, not build artifacts,
so they are not rows in `registry/ARTIFACTS.blake3`; they are reproducible
byte-exact by running the example (verified 2026-10-22).

## Tabular & Edge Compaction Quickstart (frozen-duckdb 2.0)

Compile a declarative SQL AST expressed in RDF Turtle, validate it against the SHACL pre-flight gate, and execute vectorized pushdown over embedded columnar buffers:

```sh
cargo run --example sql_in_rdf_duckdb
```

Expected output:
```text
=== 1. Initializing Embedded frozen-duckdb Substrate ===
  -> Ingested seed telemetry into 'telemetry_events' table.

=== 2. Parsing Declarative SQL AST Turtle Subgraph ===
=== 3. Admitting the AST graph through compile_turtle ===
  -> Admitted Turtle document (canon bytes pinned at compile).
  -> Validated AST conformance (identifiers safe from injection).
  -> Serialized SQL : SELECT batch_id, SUM(cost) AS total_cost FROM telemetry_events t WHERE (latency_ms > $1) GROUP BY batch_id LIMIT 10
  -> Bound Params   : [Integer(400)]

=== 4. Executing via frozen-duckdb & Producing ExecutionReceipt ===
  -> Rows Emitted    : 2
  -> Execution Time  : 0.000 ms
  -> Query Hash (BLAKE3): [fe, 54, 67, 48, 9a, a6, 64, 48]

=== 5. Materialized RDF Statements (Grounding Tabular State) ===
  <http://example.org/data/row_1> <http://example.org/vocab/batch_id> Literal { lexical_form: "batch-beta", datatype: "http://www.w3.org/2001/XMLSchema#string", language: None, direction: None }
  <http://example.org/data/row_1> <http://example.org/vocab/total_cost> Literal { lexical_form: "310", datatype: "http://www.w3.org/2001/XMLSchema#integer", language: None, direction: None }
  <http://example.org/data/row_2> <http://example.org/vocab/batch_id> Literal { lexical_form: "batch-alpha", datatype: "http://www.w3.org/2001/XMLSchema#string", language: None, direction: None }
  <http://example.org/data/row_2> <http://example.org/vocab/total_cost> Literal { lexical_form: "225", datatype: "http://www.w3.org/2001/XMLSchema#integer", language: None, direction: None }
```

`Execution Time` and the `Query Hash (BLAKE3)` bytes vary per machine and run; every
other line is deterministic. If section 3 prints a `REFUSED-01..04` code instead of
the admit line, the AST graph failed the fail-closed pre-flight gate — see
`docs/refusals.md`.

## A refused plan

```sh
cargo run --example refused_plan
```

```text
REFUSED at index 1, action `paint`
  unmet: <urn:d:door> <urn:p:has> <urn:v:key> .
```

No state is produced on refusal. See `docs/refusals.md`.

## Receipt feedback

```sh
cargo run --example receipt_feedback
```

```text
before record: receipt required: no recorded receipt for step `derive:rdfs`
after record:  admitted by `admit:require-receipt`
```

## Policy admission (feature `abi`)

```sh
cargo run --example policy_admission --features abi
```

```text
ADMITTED reachable=["g", "s0"] goals=["g"] entries=[("s0", "flip")]
```

`abi` is a default feature (Cargo.toml), so the plain `cargo run --example
policy_admission` above already admits. The hint path
(`policy_admission needs: ... --features abi`) only fires under
`--no-default-features`.

## See also

- `docs/refusals.md`, `CHANGELOG.md`, `SECURITY.md`
- External sibling (`../ggen-marketplace`): [ABI crate how-to](../../ggen-marketplace/docs/how-to/represent-a-rust-abi-crate.md), [rust-wasi-wasmex-pack](../../ggen-marketplace/packs/rust-wasi-wasmex-pack/README.md)
