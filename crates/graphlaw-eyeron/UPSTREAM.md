# Provenance

This crate is [Eyeron](https://github.com/eyereasoner/eyeron) by Jos De Roo
(KNoWS office of IDLab, Ghent University - imec), MIT licensed (see
`LICENSE.md`), vendored so GraphLaw can be published to crates.io without a git
dependency.

- Upstream commit: `d6568f657c19805b64223acf28d74234156bb837` (crate version 0.7.7)
- This crate's own version is 0.7.8: upstream 0.7.7 plus the GraphLaw changes below. The
  `BACKEND_AUTHORITIES` revision in `graphlaw` records the upstream version and commit.
- Changed in `src/reasoner.rs` / `src/main.rs` / `src/sudoku.rs` (resource safety, GraphLaw 26.9.29):
  run-wide `ReasonerOptions` caps (`max_total_bytes`, `max_closure_facts`, `max_total_steps`) and
  the matching `ReasonerLimit::ClosureSize` / `ClosureFacts` (the enum is now `#[non_exhaustive]`);
  bounded `log:includes` / `list:append` search, `string:format` precision, exact `BigInt`
  literal/product sizes and tabled backward goals; nested `log:conclusion` shares the run-wide
  step budget; input-size caps in the CLI and a node cap in the sudoku solver.
- Source files: `src/` copied unmodified except as listed below.
- Removed from the copy: tests, examples, docs, tooling and extra binaries.
- Changed in `src/reasoner.rs` (cfg only): the wasm-bindgen `Date.now` import
  and the clock read are gated on `target_arch = "wasm32"` **and**
  `target_os = "unknown"`, so `wasm32-wasip1` uses `std::time` (WASI clock)
  instead of a browser-only call that traps.
- Package name is `graphlaw-eyeron` (the library is still `eyeron`) so it
  cannot collide with the upstream crate name.

Drop this crate and depend on upstream `eyeron` once it publishes a release
carrying the same gate.
