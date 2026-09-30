# GraphLaw extraction receipt

GraphLaw was extracted from `seanchatmangpt/praxis` at exact source
commit `15504cd791cc13e117ff7cd02fd662be7a1f8631`.

Bootstrap run `36222181361` completed both `cargo metadata --no-deps`
and `cargo check -p praxis-graphlaw --lib` successfully. That run failed
only when generated `target/` compiler output was staged for publication;
the canonical extraction excludes compiler output and delegates continuing
verification to this repository's CI.

Canonical source boundary:

- `crates/praxis-graphlaw` -> repository root
- `crates/praxis-graphlaw-wasm` -> `crates/praxis-graphlaw-wasm`
- `crates/powl2-decompose` -> `crates/powl2-decompose` (direct local dependency)

The engine code, tests, benches, ontology corpus, and conformance fixtures
are copied byte-for-byte before boundary-only manifest/repository rewrites.
Praxis is not the canonical home after this extraction.
