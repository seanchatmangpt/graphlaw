# Carried upstream patches

These are unmodified copies of upstream crates with **one class of change**:
clock and entropy reads that upstream gates on `target_arch = "wasm32"` (and
implements with `js_sys::Date::now()` / `Math::random()` or a `wasm-bindgen`
import) are re-gated on `all(target_arch = "wasm32", target_os = "unknown")`.

Why: on `wasm32-wasip1` (the target Elixir/Wasmex-style hosts use) there is a
real clock and RNG through WASI, which `std::time` and `getrandom` already use.
Upstream's gate sends WASI down the browser path, where `wasm-bindgen` stubs
panic (`unreachable` trap) the first time a SPARQL query or an N3 time builtin
reads the clock.

| Crate | Upstream | Files changed |
| --- | --- | --- |
| `purrdf-sparql-eval` | crates.io 2.0.2 | `src/clock.rs`, `src/governor/mod.rs`, `Cargo.toml` (cfg only) |

Eyeron is vendored as a real workspace crate in `crates/graphlaw-eyeron` (see its `UPSTREAM.md`).

Tests, examples and benches were dropped from the copy. Delete this directory (and its `[patch]` entry in the root `Cargo.toml`) as soon as upstream carries the same gate.
