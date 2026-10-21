# Contributing

Rust 1.96 (pinned by `rust-toolchain.toml`). Rules: normal branches, merge (no rebase, no force
push); tests use real collaborators, not mocks.

## Gate (mirrors `.github/workflows/ci.yml`)

```sh
cargo fmt --check
cargo check --all-targets --all-features
cargo run --locked --features abi --bin graphlaw-registry -- --check
cargo clippy --all-targets --features pack-tools,abi,wasi-patched-deps -- -D warnings
cargo build --locked -p graphlaw-wasm --target wasm32-wasip1 --profile wasm
GRAPHLAW_WASM=$PWD/target/wasm32-wasip1/wasm/graphlaw_wasm.wasm \
    cargo test --locked --all-targets --features pack-tools,abi,wasi-patched-deps
cargo package -p graphlaw-eyeron -p graphlaw
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --features pack-tools,abi,wasi-patched-deps
cargo check --lib --target wasm32-unknown-unknown
```

The `graphlaw-registry --check` line fails when the committed `registry/` files differ from what
`src/registry.rs` emits. The registry-vs-dispatch court (a test that parses the `"<op>" => ...`
arms of `dispatch` in `src/abi.rs` and compares them with the registry op list) runs inside
`cargo test --all-targets --all-features`; keep one op per `dispatch` arm line so it can parse.

Install targets first: `rustup target add wasm32-wasip1 wasm32-unknown-unknown`.

## Embedded DuckDB 2.0 Substrate

GraphLaw unconditionally embeds `frozen-duckdb 2.0`. Prebuilt native binaries are cached automatically in `~/.frozen-duckdb/cache/` (e.g., `libduckdb_arm64.dylib` on macOS).
Run DuckDB-specific integration suites:
```sh
cargo test --test test_edge_sovereign_ingestion --test test_cold_chain_ocel --test test_parquet_and_builtin_eval
```

## WebAssembly module

`cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` produces
`target/wasm32-wasip1/wasm/graphlaw_wasm.wasm` (the `wasm` member is pinned at
26.10.13; the same commit must carry `registry/ARTIFACTS.blake3` and
`priv/graphlaw.wasm.blake3` for any version bump). `tests/wasm_abi.rs` runs it in a real wasm runtime
when `GRAPHLAW_WASM` points at it. The `graphlaw` lib facade must also compile
for `wasm32-unknown-unknown` (no WASI imports allowed in the facade); the
artifact itself is WASI-only. Building `graphlaw` for WASI outside this workspace is a
compile error by design (the clock fix lives in `vendor/`).

## WASM and dialect court suites

CI's `wasm-court` job (`.github/workflows/ci.yml`) runs, besides `wasm_abi`
and `wasi_patch_guard`, these landed suites — each loud-skips with a counted
`[SKIP]` line if its subject is absent, never a silent pass:

```sh
for t in wasm_size_court abi_manifest_court abi_bounds_court \
         canonical_cross_dialect parse_refusal_court tier_court \
         parser_capability_probe wasm_native_differential; do
    cargo test --locked --test $t
done
```

`wasm_native_differential` additionally needs `GRAPHLAW_WASM` (or will build
the module once): it replays N >= 30 canonical, keyless/clockless cases both
natively (`graphlaw::abi::call`) and through the compiled module, asserting
raw-byte equality. The dialect-side courts (`tier_court`,
`parse_refusal_court`, `canonical_cross_dialect`, `parser_capability_probe`)
and `w3c_conformance` run with plain `cargo test --test <name>` — no env or
artifact needed. `w3c_conformance` drives the vendored W3C RDF 1.1 syntax
suites (Turtle, N-Triples, N-Quads) listed in `tests/w3c_manifest.rs`.

After adding or removing fixtures under `tests/fixtures/w3c/`, regenerate the
manifest (it is GENERATED; never edit `tests/w3c_manifest.rs` by hand):

```sh
scripts/gen_w3c_manifest.sh   # from repo root; rewrites tests/w3c_manifest.rs
```

The artifact size/header court is a shell script, not a cargo test:

```sh
scripts/wasm_audit.sh   # derives the version from wasm/Cargo.toml, checks
                        # \0asm header magic + 25 MiB ceiling on the artifact
                        # at $GRAPHLAW_WASM or the default target path
```

Non-vacuity: `WASM_AUDIT_FATFIXTURE=<path>` points it at an oversized file
that MUST exit 1.

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
