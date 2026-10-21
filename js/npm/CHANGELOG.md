# changelog

This file mirrors the Rust workspace CHANGELOG at the repository root. The npm
package version always equals the crates.io `graphlaw` version it wraps; release
entries are copied verbatim from the root CHANGELOG.

## [Unreleased]

### Changed

- sha256->BLAKE3 migration: artifact integrity is now BLAKE3 (`WASM_BLAKE3` in
  `src/generated/artifact-meta.ts`, re-pinned by `scripts/build.mjs` via `b3sum`
  cross-checked against the new dependency-free pure-TypeScript implementation
  `src/blake3.ts`; validated against the `b3sum` CLI in `test/blake3.test.ts`).
  `initGraphlaw` option renamed `skipShaCheck` -> `skipDigestCheck`.

### Added

- Initial JavaScript/TypeScript distribution: npm package `graphlaw` wrapping
  the pinned wasm32-wasip1 artifact.
- Typed client for all 14 ops, plus a `graphlaw/raw` passthrough surface.
- Types generated from `registry/capability-registry.json`.
- Vitest golden-vector test suite over `registry/op-examples.json`.

## [26.10.13] - 2026-10-10

Published as crates.io `graphlaw` 26.10.13.

ERRC release: Eliminate/Reduce/Raise pass over the repo; no semantic changes to src/.

### Added
- tests/readme_first_ten_minutes.rs pins the README's example outputs (hashes,
  refusal text); docs_release_claims.rs pins the doc version lines to
  CARGO_PKG_VERSION.
### Reduced
- ci.yml: dropped the duplicated chicago/qualification steps and the clippy-subsumed
  check step.
### Eliminated
- extract-from-graphlaw.yml (destructive tree-replacing footgun; EXTRACTION.md updated);
  malformed generated docs/reference-generated/{how_to,explanation}.md; orphaned
  ggen.toml; dead .gitignore rules.
