# Migration to library-backed GraphLaw

v26.9.28 intentionally removes the historical implementation API. Migrate by
using the authoritative surfaces re-exported by `graphlaw`.

| Removed GraphLaw implementation | Replacement |
| --- | --- |
| `parser/*`, N3 pest grammar | `graphlaw::n3` (Eyeron) |
| `reasoner/*`, `builtins/*`, `backwardchaining` | `graphlaw::n3` (Eyeron) |
| `datalog`, `aggregation`, rule indexing/fixpoint code | `graphlaw::datalog` (PurRDF) |
| `tripleindex`, encoder/decoder storage machinery | `graphlaw::rdf` (PurRDF) |
| `sparql/*`, `queryengine` | `graphlaw::sparql` (PurRDF) |
| `shacl/*` | `graphlaw::shacl` (PurRDF) |
| `shex_native`, `shexc_parser` | `graphlaw::shex` (PurRDF) |
| `owlrl/*` | `graphlaw::entailment` (PurRDF) |
| local RDF event/window plumbing | `graphlaw::events` where covered by PurRDF |
| local WASM wrapper | build the library-backed crate for `wasm32-unknown-unknown` directly |

There is deliberately no compatibility implementation that can silently take
over when an upstream engine rejects an input. That prevents the same document
from acquiring different semantics depending on which internal path happened to
accept it.

GraphLaw-specific ontology, pack and query assets remain in the repository.
