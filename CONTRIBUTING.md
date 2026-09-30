# Contributing

Rust 1.96 (pinned by `rust-toolchain.toml`). Rules: normal branches, merge (no rebase, no force
push); tests use real collaborators, not mocks.

## Gate (mirrors `.github/workflows/ci.yml`)

```sh
cargo fmt --check
cargo check --all-targets --all-features
cargo run --features abi --bin graphlaw-registry -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm
GRAPHLAW_WASM=$PWD/target/wasm32-wasip1/wasm/graphlaw_wasm.wasm cargo test --all-targets --all-features
cargo package -p graphlaw-eyeron -p graphlaw
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo check --lib --target wasm32-unknown-unknown
```

The `graphlaw-registry --check` line fails when the committed `registry/` files differ from what
`src/registry.rs` emits. The registry-vs-dispatch court (a test that parses the `"<op>" => ...`
arms of `dispatch` in `src/abi.rs` and compares them with the registry op list) runs inside
`cargo test --all-targets --all-features`; keep one op per `dispatch` arm line so it can parse.

Install targets first: `rustup target add wasm32-wasip1 wasm32-unknown-unknown`.

## WebAssembly module

`cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` produces
`target/wasm32-wasip1/wasm/graphlaw_wasm.wasm`. `tests/wasm_abi.rs` runs it in a real wasm runtime
when `GRAPHLAW_WASM` points at it. Building `graphlaw` for WASI outside this workspace is a
compile error by design (the clock fix lives in `vendor/`).

## Examples

`cargo build --examples --all-features` must stay green; each example in `examples/` must run.

## Registry regeneration

After any change to an op, field, dialect, limit or refusal code, edit `src/registry.rs` (never
the emitted files), then run:

```sh
cargo run --features abi --bin graphlaw-registry -- --write
cargo run --features abi --bin graphlaw-registry -- --check
```

Commit `registry/` with the source change. See `docs/capability-registry.md`.
