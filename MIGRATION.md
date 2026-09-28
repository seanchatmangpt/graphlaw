# Migration to library-backed GraphLaw

v26.9.28 intentionally removes the historical implementation API. Migrate by
using the authoritative surfaces re-exported by `praxis_graphlaw`.

| Removed GraphLaw implementation | Replacement |
| --- | --- |
| `parser/*`, N3 pest grammar | `praxis_graphlaw::n3` (Eyeron) |
| `reasoner/*`, `builtins/*`, `backwardchaining` | `praxis_graphlaw::n3` (Eyeron) |
| `datalog`, `aggregation`, rule indexing/fixpoint code | `praxis_graphlaw::datalog` (PurRDF) |
| `tripleindex`, encoder/decoder storage machinery | `praxis_graphlaw::rdf` (PurRDF) |
| `sparql/*`, `queryengine` | `praxis_graphlaw::sparql` (PurRDF) |
| `shacl/*` | `praxis_graphlaw::shacl` (PurRDF) |
| `shex_native`, `shexc_parser` | `praxis_graphlaw::shex` (PurRDF) |
| `owlrl/*` | `praxis_graphlaw::entailment` (PurRDF) |
| local RDF event/window plumbing | `praxis_graphlaw::events` where covered by PurRDF |
| local WASM wrapper | build the library-backed crate for `wasm32-unknown-unknown` directly |

There is deliberately no compatibility implementation that can silently take
over when an upstream engine rejects an input. That prevents the same document
from acquiring different semantics depending on which internal path happened to
accept it.

GraphLaw-specific ontology, pack and query assets remain in the repository.
