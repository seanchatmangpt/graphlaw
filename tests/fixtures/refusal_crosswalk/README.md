# refusal_crosswalk fixtures

Byte-identical pinned copies of the marketplace pack
`/Users/sac/ggen-marketplace/packs/refusal-vocabulary-pack/`:

- `ontology.ttl` — crosswalk ontology (51 erlmcp codes -> 18 ash_a2a S42
  classes -> 6 strata-valve classes -> A2A wire column).
- `gates/010..060*.rq` — the pack's six closure-style totality gates
  (G1 exactly-one-class, G2 class coverage, G3 JSON-RPC law set, G4 macro
  bijection totality, G5 valve totality, G6 A2A wire totality).
- `queries/crosswalk.rq` — canonical projection (51+6+18+2 = 77 rows).

`manifest.blake3` is the pin of record. Regenerate pins by re-copying from
the pack and re-running `shasum -a 256` over the same file set. The drift
meta-test (`refusal_crosswalk_pin_matches_live_sibling` in
`tests/refusal_crosswalk_differential.rs`) compares pin vs the live sibling
and BLOCKS (skips) when the sibling checkout is absent.

Court: `tests/refusal_crosswalk_differential.rs`.
