# DEPENDENCY-STANDING (2026-11-17, lane 17)

Static read of Cargo.toml, Cargo.lock, deny.toml, rust-toolchain.toml, wasm/Cargo.toml, CHANGELOG.md, registry/*.json, ci.yml. `cargo deny`, `cargo tree`, `cargo build --locked` NOT run. Cargo.toml is coordinator-owned; nothing here is an applied edit.

## 1. Pins
| Item | Value |
|---|---|
| Toolchain | 1.98.0 (rustfmt, clippy, minimal); rust-version 1.98; edition 2024 |
| Workspace | ".", "wasm"; excludes courts/castle-diff (nightly) |
| purrdf, purrdf-columnar | git seanchatmangpt/purrdf rev 2223f8945e19eb9aed750fe038b0fa833bd17c83; family 3.0.1 in lock |
| frozen-duckdb | 2.0.2 (+ -builder, -sys 2.0.2) |
| wasmtime, wasmtime-wasi | 48 optional; locked 48.0.7 |
| ed25519-dalek | =2.2.0 exact |
| blake3 | 1 (locked 1.8.7) |
| Lock | 530 packages; no [patch] tables |

purrdf drift: `refs/heads/graphlaw-carried` is at a1cd8dd113ab902531db4b9236371c649f6ca947; pin 2223f894 is NOT the head. Ancestry unverified (compare API 403). Upstream main 0ed3d09. Branch name in Cargo.toml is documentary; manifest pins a rev only.

## 2. Features
default = ["abi"] (no-op marker); runner-eh = wasmtime(+wasi); evidentiary-ring = rtrb (off); pack-tools = regex (off). wasm crate: cdylib, publish=false, license "MIT AND LicenseRef-P28-SIL-2.1.0". Not published to crates.io. serde_json declared twice (deps and dev-deps); manifest tables interleaved ([[bin]]/[[test]]/[profile.dev] among dev-deps) — cosmetic.

## 3. Duplicates (deny multiple-versions = warn)
~55 names carry multiple versions (~20 windows-*). Substantive: sha2 0.10.9/0.11.1 (+digest, block-buffer, crypto-common, cpufeatures; 0.11 from purrdf-core), rand 0.8/0.9/0.10, zstd 0.13.3/0.14.3 + zstd-safe 7.3.0/8.2.0, base64 0.21/0.22, hashbrown 0.15/0.16/0.17, syn 2/3, thiserror 1/2, nom 7/8, itertools, rustix, strum, wasm-encoder/parser/wast. None blocking.

## 4. deny.toml
all-features; yanked = deny. Six advisory ignores, all "host-tooling-only":
| Advisory | Crate | Expiry |
|---|---|---|
| RUSTSEC-2026-0258 | h2 0.3.27 (frozen-duckdb -> reqwest 0.11) | 2026-11-23 |
| RUSTSEC-2025-0134 | unmaintained, transitive | 2026-11-23 |
| RUSTSEC-2024-0449 | unmaintained, transitive | 2026-11-23 |
| RUSTSEC-2026-0391 | zstd 0.13.3 (wasmtime-internal-cache 48.0.7) | 2026-11-28 |
| RUSTSEC-2026-0398 | zstd-safe 7.3.0 | 2026-11-28 |
| RUSTSEC-2026-0403 | zstd-safe 7.3.0 | 2026-11-28 |
**Expiry watch:** 3 ignores expire in 6 days (2026-11-23), 3 in 11 days (2026-11-28); expiry is comment-only, not enforced. Holders still in the lock.
Licenses: confidence 0.9, 17 permissive entries incl. MulanPSL-2.0; LGPL not allowed. Bans: wildcards warn (exact-rev git deps), allow-wildcard-paths. Sources: unknown-registry/git deny; only crates.io-index and github.com/seanchatmangpt/purrdf. Enforcement layers: ci.yml:76-86 "Purrdf resolves from the pinned fork", tests/authority_manifest.rs (note: lane 05 found this file absent — verify), tests/wasi_patch_guard.rs. deny.toml cites "Cargo.toml:50-51" (line-number drift risk).

## 5. Version strings
26.10.28 in: Cargo.toml, wasm/Cargo.toml, Cargo.lock, python/pyproject.toml:7, CHANGELOG `## [26.10.28]`, contracts/abi-manifest.json, mcp-manifest.json, capability-registry.json, graphlaw_ex MANIFEST.json. Divergent: wasm/Cargo.toml digest-pin comment says "minted with version 26.10.22" (f149433c...2e7e, 7,104,147 B) — confirm ARTIFACTS.blake3 and priv/graphlaw.wasm.blake3 re-pinned at 26.10.28; CHANGELOG:177 mentions npm sync at 26.10.26; graphlaw_ex mix.exs 0.1.0 (independent). No 26.11.x string exists; a bump must touch all 26.10.28 sites plus the digest re-pin.

## 6. Open items
1. purrdf pin vs a1cd8dd: re-pin or record why not (`git merge-base --is-ancestor 2223f894 a1cd8dd1` in a purrdf checkout).
2. Re-review the 6 advisory ignores before 2026-11-23 / 2026-11-28.
3. Confirm wasm digest pins match 26.10.28.
4. Run `cargo deny check` and `cargo build --locked`.
5. Optional: dedupe serde_json, group manifest tables.
