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
| local WASM wrapper | build the library-backed crate for `wasm32-wasip1` directly (`cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm`; `wasm32-unknown-unknown` is not a supported module target — it needs a JavaScript host) |

There is deliberately no compatibility implementation that can silently take
over when an upstream engine rejects an input. That prevents the same document
from acquiring different semantics depending on which internal path happened to
accept it.

GraphLaw-specific ontology, pack and query assets remain in the repository.

## Artifact digest migration: sha256 -> BLAKE3 (26.10.14)

Artifact pins have moved from SHA-256 to BLAKE3 (b3sum format
`<64-hex>  <filename>`):

- `priv/graphlaw.wasm.blake3` (was a sha256 pin; now a b3sum pin)
- release-asset sidecars `*.blake3` produced by `b3sum` in `release.yml`
- `registry/ARTIFACTS.blake3`

Consumers and scripts must switch `shasum -a 256` / `sha256sum` to `b3sum`.
`b3sum` installs from crates.io: `cargo install b3sum --locked`. Node and
WebCrypto have no blake3, so Node-side verification uses the same b3sum binary
(CI installs it via cargo); `node:crypto#createHash("blake3")` is invalid and
must not be used. The only remaining sha256 use is an internal wasm ABI-surface
comparison in `js.yml`, which never touches the artifact pin.
