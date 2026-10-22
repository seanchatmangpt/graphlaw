# GraphLaw

GraphLaw is a library-backed semantic law-state boundary for Rust: immutable,
content-addressed RDF states, admitted transitions with receipts, and typed refusals —
over three pinned upstream engines, without reimplementing any standards algorithm.

## Install

```sh
cargo add graphlaw
```

MSRV 1.96. Feature flags: `abi` (JSON ABI, registry, verification binaries),
`pack-tools` (self-monitoring pack tooling).

## First 10 minutes

Every command below was executed against this exact tree; expected outputs are real.

```sh
cargo run --example quickstart_plan
# plan-action -> blake3:8a0d0202…, c66ea90e…, 3be6530b… (one per action)
# final state: blake3:3be6530b…  (last receipt child == final state id)

cargo run --example refused_plan
# REFUSED at index 1, action `paint` + the unmet triple

cargo run --example receipt_feedback
# before record: receipt required: no recorded receipt for step `derive:rdfs`
# after record:  admitted by `admit:require-receipt`

cargo run --example sql_in_rdf_duckdb
# compiles declarative Turtle AST -> runs vectorized pushdown in embedded DuckDB
# emitted 2 rows in 3.806ms, sealed by query hash blake3:8901beb8…

cargo test --test differential
# N3 (Eyeron), Datalog and OWL-RL (PurRDF) must agree on a transitive closure

cargo test --test wasm_abi          # builds the WASI module and drives all 14 ops

cargo test --test sql_rdf_dialect   # SQL-in-RDF compile + admission courts
cargo test --test chicago_validation  # Chicago receipt/execution courts
cargo test --test duckdb_statement_kinds  # 35 PEG statement kinds on real frozen-duckdb
```

## Documentation

Organized Diataxis-style; the hub is [docs/index.md](docs/index.md).

| | |
|---|---|
| **Tutorial** | [Quickstart](docs/quickstart.md) |
| **How-to** | [Run the WASM module](docs/how-to/run-the-wasm-module.md) · [Signed leases](docs/how-to/authorize-with-signed-leases.md) · [Offline verification](docs/how-to/verify-without-trusting-the-operator.md) · [Knowledge hooks](docs/how-to/use-knowledge-hooks.md) · [Edge Ingestion & Compaction](docs/how-to/how-to-edge-ingestion-and-compaction.md) · [Use from JavaScript](docs/how-to/use-from-javascript.md) |
| **Reference** | [ABI reference](docs/abi-reference.md) · [Capability registry](docs/capability-registry.md) · [Refusals](docs/refusals.md) · [API stability](docs/api-stability.md) · [WASM integration](docs/wasm-integration.md) · [WASM runbook](docs/WASM.md) · [Dialects](docs/DIALECTS.md) · [Full generated reference](docs/reference-generated/reference.md) |
| **Explanation** | [Qualification algebra](docs/QUALIFICATION_ALGEBRA.md) · [Chicago courts](docs/CHICAGO_N3_DATALOG.md) ([FIBO](docs/CHICAGO_FIBO.md), [cross-authority](docs/CHICAGO_CROSS_AUTHORITY.md)) · [Receipt fields](docs/explanation/receipt-fields.md) · [Edge Sovereignty & Compaction](docs/explanation/edge-sovereignty-and-zero-trust-compaction.md) |

## Edge Sovereignty & Embedded Analytical Substrate (`frozen-duckdb`)

GraphLaw unconditionally embeds the **`frozen-duckdb 2.0`** analytical engine, unlocking local analytical ingestion and **zero-trust semantic compaction (1,000:1 to 10,000:1)** at the extreme edge:

* **High-Throughput Local Ingestion**: Sensor streams and OCEL v2 event traces are appended directly into local columnar Arrow/Parquet buffers with microsecond latencies.
* **Declarative SQL-in-RDF Pushdowns**: Structured analytical reductions are authored as W3C RDF ASTs (`sql-ast.ttl`), validated by a fail-closed SHACL pre-flight gate, and pushed down into DuckDB's vectorized execution engine. 35 statement classes are
  generated from the DuckDB PEG grammar (`ggen/sql-dialect/`), with expression-level
  compilation for SELECT and the command classes, typed `REFUSAL-01..04` refusals, and
  BLAKE3 receipts binding the canonical AST graph + parameters.
* **Signed Semantic Compaction**: Massive event bursts reduce to minimal, cryptographically verifiable RDF facts sealed with BLAKE3 execution receipts. A 500,000-event telemetry burst compresses to ~1 KB of signed triples before syncing across intermittent/DDIL networks.
* **Under the 25 MiB WASM Ceiling**: The core `graphlaw.wasm` artifact is **~6.3 MiB** (6,654,496 bytes, pinned by `registry/ARTIFACTS.blake3`), enforced by a 25 MiB court ceiling (`tests/wasm_size_court.rs`) — comfortably inside tight browser/WASM runtime memory arenas without dynamic swapping or memory fragmentation. (The DuckDB engine executes host-side; the wasm boundary refuses SQL by typed `REFUSAL-04` capability refusal, not by embedding the engine.)
* **Disconnected Fail-Closed Operations**: Missing filter bounds or excursion breaches trigger immediate local actuator lockouts and quarantined states without requiring an upstream cloud coordinator.

See [docs/how-to/how-to-edge-ingestion-and-compaction.md](docs/how-to/how-to-edge-ingestion-and-compaction.md) and [docs/explanation/edge-sovereignty-and-zero-trust-compaction.md](docs/explanation/edge-sovereignty-and-zero-trust-compaction.md).

## JavaScript / TypeScript

The WASM module is also wrapped as the npm package `graphlaw`, so the same law-state
surface can be driven from Node or the browser without touching Rust. See
[docs/how-to/use-from-javascript.md](docs/how-to/use-from-javascript.md).

> **Not yet published**: the npm package is built and tested locally but has not
> been published to the npm registry yet (publish fires on `v*` tags; none has
> been pushed for the JS package). `npm i graphlaw` will fail until the operator
> pushes the publish tag.

```sh
npm i graphlaw  # NOT YET PUBLISHED
```

```js
import { createClient } from 'graphlaw'
```

## The law-state loop

`dialect::sniff` routes a document to its owning engine by content. `LawState` is an
immutable dataset identified by the blake3 of its RDFC-1.0 canonical form.
`LawState::transition(&Step)` runs an upstream-owned step (SHACL admission, N3
derivation, RDFS/OWL-RL entailment) and returns the child state with a `Receipt` naming
the authority; a refused step yields no state. `receipt::record` writes a receipt into
the state, changing its id — so verification loops can require prior receipts
(`Step::RequireReceipt`). `Plan::admit` replays a candidate plan action-by-action into a
chain of content-addressed states; the first violated precondition refuses with
`PlanRefused { index, action, missing, violated_absent }`.

## Dialects and tiers

`dialect::sniff` recognizes 14 dialects under a two-tier model: dataset
syntaxes (Turtle, TriG, N-Triples, N-Quads, RDF/XML, JSON-LD, YAML-LD, TriX,
HexTuples) route to PurRDF; Notation3 routes to Eyeron as a separate logic
tier (`n3` op, never silently down-converted to a dataset). Ambiguous content
(TriG vs ShExC) is refused with `Ambiguous` demanding a hint rather than
guessed. The parse surface is pinned by the vendored W3C RDF 1.1 syntax
conformance court (`cargo test --test w3c_conformance`). Full matrix:
[docs/DIALECTS.md](docs/DIALECTS.md).

## WebAssembly module

The `graphlaw-wasm` member builds one artifact for `wasm32-wasip1` (the lib
facade also compiles clean under `wasm32-unknown-unknown`, WASI-import-free).
The artifact is pinned by BLAKE3 digest and byte size
(`registry/ARTIFACTS.blake3` `wasm` row + the `priv/graphlaw.wasm.blake3`
sidecar; checked by `tests/registry_artifacts.rs` and
`scripts/wasm_audit.sh`). The wire surface is pinned by
`contracts/abi-manifest.json`, byte-compared against live constants by
`tests/abi_manifest_court`. A differential court
(`tests/wasm_native_differential.rs`) replays canonical inputs natively
and through the module in a real wasm runtime, asserting byte-identical
results. Build and embed details: [docs/WASM.md](docs/WASM.md).

## Engines and authorities

Capability | Authority
---|---
RDF 1.2, codecs, SPARQL 1.1/1.2, SHACL, ShEx 2.1, Datalog/stratified fixpoint, RDF/RDFS/OWL-RL entailment | PurRDF 2.0.2 (`purrdf = "=2.0.2"`)
Notation3 parsing/reasoning/proofs | Eyeron 0.7.8 (vendored `crates/graphlaw-eyeron` from upstream 0.7.7 @ `eyereasoner/eyeron@d6568f65`; see its `UPSTREAM.md`)
Vectorized columnar execution, Parquet scan pipelines, SIMD aggregations | DuckDB 2.0 (`frozen-duckdb = "2.0.0-dev"`)

GraphLaw owns the composition boundary and semantic assets (ontologies, packs,
queries) and the decision about which upstream implementation has standing.
`graphlaw::BACKEND_AUTHORITIES` documents the map in code.

## Capability registry

The public op surface is published as a machine-readable registry
(`registry/capability-registry.json` + Turtle + `op-examples.json`), emitted by the
`graphlaw-registry` binary and carried as release assets with `.blake3` files. See
[docs/capability-registry.md](docs/capability-registry.md).

## Verifying without trusting the operator

```sh
graphlaw-verify --start state.nt --plan plan.json --receipts ./receipts \
    --trusted-key <hex-public-key>
```

Replays the plan, checks the receipt store holds exactly the replayed chain, verifies
every Ed25519 attestation — offline, no sockets. `ADMITTED` (0) or
`REFUSED: <code>: <detail>` (1). Details: [docs/how-to/verify-without-trusting-the-operator.md](docs/how-to/verify-without-trusting-the-operator.md).

## License

Code is MIT ([LICENSE](LICENSE)); execution/use of the kernel is governed by the P28 Sovereign Invariant License v2.1.0, `urn:graphlaw:license:p28-sil:v2.1.0-sovereign` ([LICENSE-P28-SIL.md](LICENSE-P28-SIL.md)) — statutory metric classifications with the $50M Class-I safe harbor, anti-circumvention ratchet, harmonic regularization, and zero hot-path drag invariant, with its implementation-status appendix non-normative.

## Releasing

CI (`.github/workflows/release.yml`) builds and tests, tags `vX.Y.Z`, and creates the GitHub release with `graphlaw.wasm` (plus checksum) whenever a change that bumps the version in `Cargo.toml` reaches `main`. It never publishes to crates.io: publishing is local-only — from a logged-in machine run `cargo cicd publish run` (gated on doc-claims verification and a `cargo publish --dry-run` fence), then `cargo publish -p graphlaw` (the registry credential is the machine's own cargo login, never a repository secret). Building `graphlaw` for WASI outside this workspace is a compile error by design (the clock fix lives in `vendor/`); use the release asset.

## Production rule

No standards algorithm is reimplemented in GraphLaw. If an admitted upstream library
lacks a required standard feature, GraphLaw either refuses that feature or contributes
the capability upstream — it does not create another local parser, reasoner, validator,
query engine, or fixpoint implementation. GraphLaw does not build a relational execution engine; it delegates tabular pushdown and Parquet zone-map evaluation to `frozen-duckdb 2.0`, restricting GraphLaw's scope strictly to AST validation, declarative translation, and cryptographic receipt issuance. The pre-v26.9.28 engine remains available
through Git history only; see [MIGRATION.md](MIGRATION.md) for the old-to-new surface
map, [SECURITY.md](SECURITY.md) for key-custody and supported-versions policy, and
[docs/api-stability.md](docs/api-stability.md) for the stability contract
(`#[non_exhaustive]`, `#![deny(missing_docs)]`, cargo-semver-checks/deny/audit in CI).
