# Vector-pin receipt — DL dualroute (Lane M10)

Defect class (K10): `src/bin/dl_dualroute.rs` silently added a
`graphlaw_version` provenance field — emitted vector bytes changed with no
pin to catch it.

## Construct

- `tests/dl_dualroute_vector_pin.rs` (new, lane-owned): runs the real
  generator binary (`cargo run --bin dl_dualroute`, Chicago — no
  reimplementation of emission logic), reads back the four emitted vectors
  from `/Users/sac/wasm4pm/crates/wasm4pm-cognition/tests/fixtures/graphlaw/`,
  normalizes ONLY the per-commit-volatile `graphlaw_rev` field (field presence
  is still pinned; its value is not), and asserts per-vector BLAKE3 pins over
  the full serialization: `vector_id`, `breed`,
  `provenance.{graphlaw_version, purrdf, eyeron, generator, stratified}`,
  `breed_input.{intent, facts, rules, goals, candidates, cases, state}`,
  `expected.{derived_facts_sorted, verdicts, notes}`.
- Re-pin protocol coded in the failure message: intentional serialization
  changes re-pin in the same diff as the generator change.

## Pins (rev-normalized BLAKE3)

| slug | digest |
|---|---|
| a-paper-baader-el | `f12d295ffc9209a2c8a7e6f7bc8258daa239e49b29ff6d65d06a102f5d91309c` | (historical)
| b-chain3-conj | `6e920dcc011ba8c665440b691d0e7fa1b11ac27cbc13a03205d2dba06e1d329e` | (historical)
| c-exists-pair | `2ba717f1109389dec0733f812966dbdc0e7ee52d653117214b94942a415465d9` | (historical)
| d-exists-conj-interlock | `c22c4ee38d0e6c47eedbd05c49f65ae818d4cb718b2a28eaa2d56abaf5aee345` | (historical)

## Verification (real output)

```
$ CARGO_TARGET_DIR=target-m10 cargo test --test dl_dualroute_vector_pin
running 1 test
test dl_dualroute_vector_bytes_are_pinned ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 67.49s
exit=0
```

## Anti-vacuity witness

Injected `"extra_field": true` after `"breed"` into the emitted bytes and
re-hashed with the same normalization:

```
6799ace43411a625dac72b833aa7fd29308020bc09a64294e911743fea255aa1  (≠ pin f12d295f…) # historical
```

The court flips on exactly the K10 mutation (field addition). Emission state
regenerated pristine afterwards (`cargo run --bin dl_dualroute`, all four OK).

## Notes

- `graphlaw_version` is INSIDE the digest on purpose: a version bump flips
  the pin and forces a receipted re-pin, which is the correct behavior for
  the K10 class (provenance changes must be loud).
- `graphlaw_rev` alone is normalized (changes every commit; pinning it would
  red on every HEAD move, which is drift noise, not the defect class).
- The test writes to the generator's real output directory (its designed
  behavior); no mocks, no sleeps.
