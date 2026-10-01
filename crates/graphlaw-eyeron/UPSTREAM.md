# Provenance

This crate is [Eyeron](https://github.com/eyereasoner/eyeron) by Jos De Roo
(KNoWS office of IDLab, Ghent University - imec), MIT licensed (see
`LICENSE.md`), vendored so GraphLaw can be published to crates.io without a git
dependency.

- Upstream commit: `d6568f657c19805b64223acf28d74234156bb837` (crate version 0.7.7)
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
