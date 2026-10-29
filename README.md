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

cargo test --test differential
# N3 (purrdf::n3), Datalog and OWL-RL (PurRDF) must agree on a transitive closure

cargo test --test wasm_abi          # builds the WASI module and drives the hashed 14-op table (`duckdb` is dispatched by the module and omitted from that table until a digest bump)
                                    #   prerequisite: the canonical relink recipe — WASI_SDK_PATH at the unpacked frozen-duckdb cache SDK (~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1; its legacy-EH sysroot matches libduckdb.a; the published wasi-sdk-34 sysroot fails the link) plus the in-tree wrap shim compiled from tests/fixtures/wasm-shim/. Alternatively point GRAPHLAW_WASM at a prebuilt module. Recipe: docs/WASM.md.

cargo test --test sql_rdf_dialect   # SQL-in-RDF compile + admission courts
cargo test --test chicago_validation  # Chicago receipt/execution courts
cargo test --test duckdb_statement_kinds  # 35 PEG statement kinds on real frozen-duckdb
```

## Documentation

Organized Diataxis-style; the hub is [docs/index.md](docs/index.md).

| | |
|---|---|
| **Index** | [docs/INDEX.md](docs/INDEX.md) |
| **Tutorial** | [Quickstart](docs/quickstart.md) |
| **How-to** | [Run the WASM module](docs/how-to/run-the-wasm-module.md) · [First in-WASM SQL lifecycle](docs/how-to/first-in-wasm-sql-lifecycle.md) · [Persistent DuckDB sessions](docs/how-to/persistent-duckdb-sessions.md) · [Document sequences](docs/how-to/document-sequences.md) · [Signed leases](docs/how-to/authorize-with-signed-leases.md) · [Offline verification](docs/how-to/verify-without-trusting-the-operator.md) · [Knowledge hooks](docs/how-to/use-knowledge-hooks.md) · [Edge Ingestion & Compaction](docs/how-to/how-to-edge-ingestion-and-compaction.md) · [Use from JavaScript](docs/how-to/use-from-javascript.md) |
| **Reference** | [ABI reference](docs/abi-reference.md) · [Capability registry](docs/capability-registry.md) · [Refusals](docs/refusals.md) · [API stability](docs/api-stability.md) · [WASM integration](docs/wasm-integration.md) · [WASM runbook](docs/WASM.md) · [Dialects](docs/DIALECTS.md) · [Full generated reference](docs/reference-generated/reference.md) |
| **Explanation** | [Qualification algebra](docs/QUALIFICATION_ALGEBRA.md) · [Chicago courts](docs/CHICAGO_N3_DATALOG.md) ([FIBO](docs/CHICAGO_FIBO.md), [cross-authority](docs/CHICAGO_CROSS_AUTHORITY.md)) · [Receipt fields](docs/explanation/receipt-fields.md) · [Edge Sovereignty & Compaction](docs/explanation/edge-sovereignty-and-zero-trust-compaction.md) |

## Edge Sovereignty & Embedded Analytical Substrate (`frozen-duckdb`)

GraphLaw unconditionally embeds the **`frozen-duckdb 2.0`** analytical engine, unlocking
local analytical ingestion and zero-trust semantic compaction at the extreme edge.
Compaction ratios and latencies are workload-dependent and intentionally uncited; the
reduction claim is court-scoped: `tests/sieve_court.rs` witnesses a 1,000,000-row
in-engine GROUP BY reduction landing inside the evaluator's default 64-row result
envelope.

* **High-Throughput Local Ingestion**: Sensor streams and OCEL v2 event traces are
  appended directly into local columnar Arrow/Parquet buffers. Throughput is
  workload-dependent; no fixed latency figure is asserted.
* **Declarative SQL-in-RDF Pushdowns**: Structured analytical reductions are authored as W3C RDF ASTs (`sql-ast.ttl`), validated by a fail-closed SHACL pre-flight gate, and pushed down into DuckDB's vectorized execution engine. 35 statement classes are
  generated from the DuckDB PEG grammar (`ggen/sql-dialect/`), with expression-level
  compilation for SELECT and the command classes, typed `REFUSAL-01..04` refusals, and
  BLAKE3 receipts binding the canonical AST graph + parameters.
* **Signed Semantic Compaction**: Massive event bursts reduce to minimal,
  cryptographically verifiable RDF facts sealed with BLAKE3 execution receipts. Burst
  sizes and output footprints are workload-dependent; no fixed ratio is asserted. The
  court-witnessed surface is `tests/sieve_court.rs`: a 10^6-row source reduces
  in-engine to 64 result rows inside the default envelope, and any result set over the
  envelope refuses with a typed `ResourceLimit` (`duckdb_result_rows`).
* **WASM size**: The pinned artifact in `registry/ARTIFACTS.blake3` is **60,895,924 bytes**
  (blake3 `0858a760…a5ddd2f1`, byte-identical to `priv/graphlaw.wasm` on this tree) — the
  shipped DuckDB-linked new-EH artifact, verified live on all four pin surfaces
  (`priv/graphlaw.wasm.blake3`, `registry/ARTIFACTS.blake3` wasm row,
  `js/npm/src/generated/artifact-meta.ts`, live `b3sum`; four-surface re-verification
  2026-10-26, X2 lane re-pin). Historical pins (prior artifact
  generations, each superseded and retained here only for provenance): the 26.10.26 L2-WASM
  re-mint `430da572…` (60,893,336 B), the 26.10.23 four-surface settle pin `58a44095…d2280a`
  (58,124,612 B; final-tree-audit check 2, `docs/jira/v26.10.23/final-tree-audit.md`), the
  EH-closure settle pin `fa155830…` (57,781,384 B), the pre-closure pin `991b2bee…`
  (46,138,360 B), and the legacy pre-DuckDB pin `f149433c…` (7,104,147 B). The ~61 MB
  size class vs the 46 MB pre-closure pin is the wave's own statically-registered
  core_functions/icu extensions + statement-sequence/REFUSAL-05 code under LTO
  (libc swap effect measured at −106 B). The size court ceiling is the 64 MiB sandbox wall
  (`tests/wasm_size_court.rs`). A SQL-in-RDF document parsed as `sql` returns the same
  dataset id as the same bytes parsed as Turtle. `REFUSAL-04` is the non-document-backed
  builtin reader, not a missing engine.
* **Disconnected Fail-Closed Operations**: Missing filter bounds or excursion breaches trigger immediate local actuator lockouts and quarantined states without requiring an upstream cloud coordinator.

See [docs/how-to/how-to-edge-ingestion-and-compaction.md](docs/how-to/how-to-edge-ingestion-and-compaction.md) and [docs/explanation/edge-sovereignty-and-zero-trust-compaction.md](docs/explanation/edge-sovereignty-and-zero-trust-compaction.md).

## JavaScript / TypeScript

The WASM module is also wrapped as the npm package
`@seanchatmangpt/graphlaw`, so the same law-state surface can be driven from
Node or the browser without touching Rust. See
[docs/how-to/use-from-javascript.md](docs/how-to/use-from-javascript.md).

> **Publish state** (verified 2026-10-24): the package IS published — latest
> on npm is `26.10.14` (published pre-OIDC). Version `26.10.22` is packed and
> CI-validated (`publish-npm` run 40084544193: validate-and-pack GREEN;
> tag `v26.10.22` pushed at `29c6d98`), but its publish is typed-blocked on
> the one-time npm Trusted Publisher configuration (E404, machine-impossible
> by design). Until that lands, `npm i @seanchatmangpt/graphlaw` installs
> `26.10.14`. Status locus: `docs/jira/v26.10.23/PUBLISH-STATUS.md`.

```sh
npm i @seanchatmangpt/graphlaw  # published: 26.10.14 (26.10.22 publishes after the Trusted Publisher act)
```

```js
import { createClient } from '@seanchatmangpt/graphlaw'
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
HexTuples) route to PurRDF; Notation3 routes to purrdf::n3 as a separate logic
tier (`n3` op, never silently down-converted to a dataset). Ambiguous content
(TriG vs ShExC) is refused with `Ambiguous` demanding a hint rather than
guessed. The parse surface is pinned by the vendored W3C RDF 1.1 syntax
conformance court (`cargo test --test w3c_conformance`). The N3 logic tier's
parse and proof surface is pinned by the purrdf fork's own N3 suite (parse,
proofs, reason, and the forgery court under `crates/n3/tests/`), executed in
fork CI at the rev pinned below. Full matrix:
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
results. Host runner: `graphlaw-run-eh` (wasmtime 48, `wasm_exceptions`
enabled) is the sole packet-4 stdio runner — it hosts the DuckDB-linked
artifact; sessions, document sequences, and the SQL surface run there. The
wasmi-based `graphlaw-run` binary was retired 2026-10-24
(`docs/jira/v26.10.23/ADR-wasmi-retirement.md`; the EH runner is a
wire-superset with the same exit-code contract). Build and embed details:
[docs/WASM.md](docs/WASM.md).

## Engines and authorities

Capability | Authority
---|---
RDF 1.2, codecs, SPARQL 1.1/1.2, SHACL, ShEx 2.1, Datalog/stratified fixpoint, RDF/RDFS/OWL-RL entailment | PurRDF 3.0.1 (git [`seanchatmangpt/purrdf`](https://github.com/seanchatmangpt/purrdf), rev-pinned)
Notation3 parsing/reasoning/proofs | **Notation3** — purrdf::n3 3.0.1 (purrdf crates/n3, git-sourced: [`seanchatmangpt/purrdf`](https://github.com/seanchatmangpt/purrdf), rev-pinned)
Vectorized columnar execution, Parquet scan pipelines, SIMD aggregations | DuckDB 2.0 (`frozen-duckdb = "2.0"`)

GraphLaw owns the composition boundary and semantic assets (ontologies, packs,
queries) and the decision about which upstream implementation has standing.
`graphlaw::BACKEND_AUTHORITIES` documents the map in code.

## Capability registry

The public op surface is published as a machine-readable registry
(`registry/capability-registry.json` + Turtle + `op-examples.json`), emitted by the
`graphlaw-registry` binary and carried as release assets with `.blake3` files. See
[docs/capability-registry.md](docs/capability-registry.md).

## External oracle

An out-of-tree pytest suite (the `pm4pytest/graphlaw` suite in `ggen-marketplace`)
drives this repo's real binaries and examples over subprocess — no mocks, no
in-process fakes. CI runs it as the `external-oracle` job
(`.github/workflows/ci.yml`); see [docs/how-to/verify-without-trusting-the-operator.md](docs/how-to/verify-without-trusting-the-operator.md)
for the offline-verification discipline it exercises.

## Verifying without trusting the operator

```sh
graphlaw-verify --start state.nt --plan plan.json --receipts ./receipts \
    --trusted-key <hex-public-key>
```

Replays the plan, checks the receipt store holds exactly the replayed chain, verifies
every Ed25519 attestation — offline, no sockets. `ADMITTED` (0) or
`REFUSED: <code>: <detail>` (1). Details: [docs/how-to/verify-without-trusting-the-operator.md](docs/how-to/verify-without-trusting-the-operator.md).

## License

Execution/use of the kernel is governed by the P28 Sovereign Invariant License v2.1.0, `urn:graphlaw:license:p28-sil:v2.1.0-sovereign` ([LICENSE.md](LICENSE.md)) — statutory metric classifications with the $50M Class-I safe harbor, anti-circumvention ratchet, harmonic regularization, and zero hot-path drag invariant, with its implementation-status appendix non-normative.

## Releasing

CI (`.github/workflows/release.yml`) builds and tests, tags `vX.Y.Z`, and creates the GitHub release with `graphlaw.wasm` (plus checksum) whenever a change that bumps the version in `Cargo.toml` reaches `main`. graphlaw does not publish to crates.io: its purrdf dependency is the rev-pinned [`seanchatmangpt/purrdf`](https://github.com/seanchatmangpt/purrdf) fork, so the crate cannot publish; distribution is this repository and the GitHub release assets. Building `graphlaw` for WASI works directly from the pinned sources: `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`.

## Production rule

No standards algorithm is reimplemented in GraphLaw. If an admitted upstream library
lacks a required standard feature, GraphLaw either refuses that feature or contributes
the capability upstream — it does not create another local parser, reasoner, validator,
query engine, or fixpoint implementation. GraphLaw does not build a relational execution engine; it delegates tabular pushdown and Parquet zone-map evaluation to `frozen-duckdb 2.0`, restricting GraphLaw's scope strictly to AST validation, declarative translation, and cryptographic receipt issuance. The pre-v26.9.28 engine remains available
through Git history only; see [MIGRATION.md](MIGRATION.md) for the old-to-new surface
map, [SECURITY.md](SECURITY.md) for key-custody and supported-versions policy, and
[docs/api-stability.md](docs/api-stability.md) for the stability contract
(`#[non_exhaustive]`, `#![deny(missing_docs)]`, cargo-semver-checks/deny/audit in CI).
