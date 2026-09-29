# Contributing

Rust 1.96 (pinned by `rust-toolchain.toml`). Rules: normal branches, merge (no rebase, no force
push); tests use real collaborators, not mocks.

## Gate (mirrors `.github/workflows/ci.yml`)

```sh
cargo fmt --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm
GRAPHLAW_WASM=$PWD/target/wasm32-wasip1/wasm/graphlaw_wasm.wasm cargo test --all-targets --all-features
cargo package -p graphlaw-eyeron -p graphlaw
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo check --lib --target wasm32-unknown-unknown
```

Install targets first: `rustup target add wasm32-wasip1 wasm32-unknown-unknown`.

## WebAssembly module

`cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` produces
`target/wasm32-wasip1/wasm/graphlaw_wasm.wasm`. `tests/wasm_abi.rs` runs it in a real wasm runtime
when `GRAPHLAW_WASM` points at it. Building `graphlaw` for WASI outside this workspace is a
compile error by design (the clock fix lives in `vendor/`).

## Examples

`cargo build --examples --all-features` must stay green; each example in `examples/` must run.
