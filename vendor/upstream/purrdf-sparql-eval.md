# purrdf-sparql-eval: use std clock and getrandom on wasm32-wasip1

Status: READY_UNSUBMITTED (nothing has been sent to any remote).
Pristine base: crates.io `purrdf-sparql-eval` 2.0.2 (verified against
`~/.cargo/registry/src/*/purrdf-sparql-eval-2.0.2`). Patch: `purrdf-sparql-eval.patch`.

## Problem

`src/clock.rs` (`wall_clock_now`, `entropy_seed`) and `src/governor/mod.rs` (`host_millis`)
gate their browser implementation (`js_sys::Date::now()`, `js_sys::Math::random()`) on
`target_arch = "wasm32"`. That also matches `wasm32-wasip1`, where WASI provides a real clock
(`std::time::SystemTime` / `Instant`) and RNG (`getrandom` 0.3 via `random_get`). On WASI the
`js-sys` imports have no host, so the first `NOW()`, `RAND()`, `UUID()`/`STRUUID()`-style
entropy read, or wall-deadline poll traps with `unreachable`.

## Change (cfg only)

Replace `target_arch = "wasm32"` with `all(target_arch = "wasm32", target_os = "unknown")` for
the browser branch, and its negation for the std branch, in:

- `Cargo.toml`: `getrandom` dependency gate and `js-sys` dependency gate
- `src/clock.rs`: four `#[cfg]` attributes
- `src/governor/mod.rs`: two `#[cfg]` attributes

Total: 3 files, 8 lines, no logic changes.

## Why behavior-preserving off wasm32-wasip1

For every target other than wasm32 the predicate `not(all(wasm32, unknown))` equals
`not(wasm32)`, so native targets select the same std/getrandom code as before. For
`wasm32-unknown-unknown` the predicate `all(wasm32, unknown)` equals the old
`wasm32`, so the browser path and `js-sys` dependency are unchanged. Only
`wasm32-wasip1`/`wasip2`/emscripten (`target_os != "unknown"`) change branch.

## Verification

```sh
tar xzf ~/.cargo/registry/cache/*/purrdf-sparql-eval-2.0.2.crate && cd purrdf-sparql-eval-2.0.2
patch --dry-run -p1 < purrdf-sparql-eval.patch && patch -p1 < purrdf-sparql-eval.patch
rustup target add wasm32-wasip1
cargo build --lib --target wasm32-wasip1        # patched: succeeds, no js-sys in the dep tree
cargo tree --target wasm32-wasip1 -e normal | grep -c js-sys   # 0
cargo build --lib                                # native unchanged
```

Runtime check: run any `SELECT (NOW() AS ?t) {}` query under a WASI runtime (wasmtime/Wasmex);
pristine traps, patched returns a dateTime. graphlaw exercises this via `tests/wasm_abi.rs`.

## Draft PR

Title: `Use std clock and getrandom on wasm32-wasip1 (gate js-sys path on target_os = "unknown")`

Body:
> `wasm32-wasip1` currently selects the `js_sys` clock/RNG path because the gates test only
> `target_arch = "wasm32"`. On WASI there are no JS imports, so `NOW()`, entropy seeding and the
> wall-deadline governor trap at runtime. This narrows the browser path to
> `all(target_arch = "wasm32", target_os = "unknown")` (Cargo.toml deps, `clock.rs`,
> `governor/mod.rs`); WASI uses `std::time` and `getrandom`, both WASI-capable.
> No behavior change on native or `wasm32-unknown-unknown`. Verified: `cargo build --lib`
> for native and `wasm32-wasip1`.
