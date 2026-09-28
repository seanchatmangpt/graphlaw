# GraphLaw

GraphLaw is a **library-backed semantic law-state boundary** for Rust.

## v26.9.28 semantic reset

GraphLaw no longer ships a second implementation of RDF/N3/Datalog/SPARQL/SHACL/ShEx/OWL semantics. The RoXi/Praxis-derived parser, triplestore, query engine, reasoner, built-ins, Datalog fixpoint, SHACL/ShEx validators, OWL-RL rules, DRed/RSP engine code, local WASM wrapper, and local POWL helper were removed from the production crate.

Executable standards semantics are delegated to two pinned authorities:

| Capability | Authority |
| --- | --- |
| RDF 1.2, codecs, event model | PurRDF 2.0.2 |
| SPARQL 1.1/1.2 | PurRDF 2.0.2 |
| SHACL | PurRDF 2.0.2 |
| ShEx 2.1 | PurRDF 2.0.2 |
| Datalog / stratification / semi-naive fixpoint / chase | PurRDF 2.0.2 |
| RDF / RDFS / OWL-RL entailment | PurRDF 2.0.2 |
| Notation3 parsing/reasoning/proofs | Eyeron @ d6568f657c19805b64223acf28d74234156bb837 |

GraphLaw itself owns the **composition boundary and semantic assets**: ontologies, packs, queries, and the decision about which upstream implementation has standing. It does not keep a fallback implementation.

## Law-state layer

GraphLaw's own code is the composition layer above the engines:

- `dialect` routes a document to its owning engine by content (never by trusting an extension, never by trying engines in turn) and returns a typed `Refusal` naming the engine and dialect when it cannot.
- `law::LawState` is an immutable dataset identified by `sha256:` of its RDFC-1.0 canonical form. `transition(&Step)` runs an upstream-owned step (SHACL admission, N3 derivation, RDFS/OWL-RL entailment) and returns the child state with a `Receipt` naming the authority and revision. A refused step yields no state.
- `tests/differential.rs` is a cross-engine oracle: N3 (Eyeron), Datalog and OWL-RL (PurRDF) must agree with each other and with an independent Warshall closure.
- `ASSETS.sha256` pins every shipped asset by hash and sniffed dialect; `tests/corpus_conformance.rs` routes and parses all of them (`cargo test -- --ignored` adds the large vendored vocabularies).

## Pack tooling (Rust, feature `pack-tools`)

The self-monitoring pack's former Python scripts are Rust binaries that write RDF through PurRDF only:

```sh
cargo run --features pack-tools --bin smon-transcript-to-turtle -- --transcript S.jsonl --out out.ttl
cargo run --features pack-tools --bin smon-broaden-topic -- --in-ttl in.ttl --out-ttl out.ttl
```

`tests/smon_tools.rs` checks the broadening output against the committed fixture the Python script produced.

## Rust surface

```rust
use praxis_graphlaw::{n3, rdf, sparql, shacl, shex, datalog, entailment};

let dataset = rdf::parse_dataset(
    b"<https://example.org/s> <https://example.org/p> <https://example.org/o> .",
    "application/n-triples",
    None,
)?;
assert_eq!(dataset.quad_count(), 1);

let derived = n3::reason(r#"
    @prefix : <https://example.org/> .
    :s :kind :Human .
    { ?x :kind :Human . } => { ?x :kind :Mortal . } .
"#)?;
assert!(derived.contains("Mortal"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

The complete upstream surfaces are intentionally re-exported. Consumers can use the authoritative library API directly rather than a lossy GraphLaw copy.

## Production rule

**No standards algorithm is reimplemented in GraphLaw.** If an admitted upstream library lacks a required standard feature, GraphLaw either refuses that feature or contributes the capability upstream. It does not create another local parser, reasoner, validator, query engine, or fixpoint implementation.

The pre-v26.9.28 engine remains available through Git history only. There is no `legacy` feature and no extraction workflow capable of restoring it onto `main`.

See `MIGRATION.md` for the old-to-new surface map.
