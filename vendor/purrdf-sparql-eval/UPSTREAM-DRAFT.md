# Upstream submission draft — purrdf-sparql-eval WASI clock gate

Status: READY_UNSUBMITTED. Nothing has been sent to any remote.
Lane: L91 (/Users/sac/graphlaw). Base: pristine crates.io `purrdf-sparql-eval` 2.0.2.
Patch: `vendor/upstream/purrdf-sparql-eval.patch`.

## Problem

`src/clock.rs` (`wall_clock_now`, `entropy_seed`) and `src/governor/mod.rs`
(`host_millis`) gate their browser implementations (`js_sys::Date::now()`,
`js_sys::Math::random()`) on `target_arch = "wasm32"` alone. That predicate also
matches `wasm32-wasip1`, where WASI provides a real clock through
`std::time::SystemTime`/`Instant` and a real RNG through `getrandom` 0.3
(`random_get`). On WASI the `js-sys` imports have no host, so the first `NOW()`
read, `RAND()`/`UUID()` entropy seed, or wall-deadline governor poll traps with
`unreachable`.

## Fix (cfg only — no logic changes, no new features)

Replace `target_arch = "wasm32"` with
`all(target_arch = "wasm32", target_os = "unknown")` for the browser branch and
its negation for the std branch:

| File | Hunks | Lines |
| --- | --- | --- |
| `Cargo.toml` | 2 | `getrandom` dep gate; `js-sys` dep gate → `all(wasm32, os="unknown")` / negation |
| `src/clock.rs` | 4 | 2× `#[cfg(not(...))]` (std branch), 2× `#[cfg(...)]` (browser branch) |
| `src/governor/mod.rs` | 2 | `host_millis()` std + browser `#[cfg]` |

Total: 3 files, 8 `#[cfg]`/dep-gate lines. No behavior change on native targets
(`not(all(wasm32, unknown))` ≡ `not(wasm32)` there) nor on
`wasm32-unknown-unknown` (`all(wasm32, unknown)` ≡ old `wasm32`); only
`wasm32-wasip1`/`wasip2`/emscripten change branch.

## Upstream state (checked 2026-10-22, lane L91)

- crates.io: latest is **3.0.1** (published 2026-10-02; 33 versions; default
  version 3.0.1). API query exit 0.
- **3.0.1 does NOT fix the WASI paths.** Verified by downloading
  `purrdf-sparql-eval-3.0.1.crate` and inspecting the tree:
  - `src/clock.rs` lines 23/33/40/48: still `#[cfg(not(target_arch = "wasm32"))]`
    / `#[cfg(target_arch = "wasm")]`-class gates (grep exit 0, four browser-only
    gates present).
  - `src/governor/mod.rs` lines 212/227: same.
  - `Cargo.toml` line 466+: `[target.'cfg(not(target_arch = "wasm32"))'.dependencies.getrandom]`
    and a `wasm-bindgen` gate on bare `target_arch = "wasm32"`.
  So the 2.0.2 patch remains necessary against 3.x; the diff is unchanged in
  intent (same six sites). The 3.0.1 tree adds a `wasm-bindgen` (non-js-sys-only)
  dep gate that will need the same treatment at migration time.
- Repository: https://github.com/Blackcat-Informatics/purrdf
  (CONTRIBUTING.md fetched, HTTP 200).

## License / CONTRIBUTING requirements

- Crate license (2.0.2 and 3.0.1): **`MIT OR Apache-2.0 OR MulanPSL-2.0`**.
  graphlaw's `deny.toml` already carries the allowance for eventual 3.x
  adoption: `deny.toml` `[licenses] allow` includes `"MulanPSL-2.0"` (line 35,
  with the comment "purrdf 3.x family (RUST-8 lane L7): keep a purrdf 3.x
  migration unblocked"). No deny.toml change is needed for a 3.x bump.
- CONTRIBUTING constraints relevant to this PR:
  - **"No Cargo features"** — workspace enforces zero feature flags
    (`helper-census --no-features`). This patch adds none; it is cfg-only,
    consistent with the constraint.
  - Conformance corpora are the contract; run `make check` **and**
    `make conformance` before claiming done.
  - Nightly toolchain via `rust-toolchain.toml`; Node.js 24 required for the
    pattern-emission oracle (`make doctor` to see local skips).
  - Bug-fix PRs against `main`.

## Test evidence (graphlaw wasi_patch_guard receipts)

Real run this session (exit 0):

```
$ CARGO_TARGET_DIR=target-lane91 cargo test --test wasi_patch_guard --features wasi-patched-deps
test wasi_patched_deps_feature_implies_vendored_patch_present ... ok
test wasi_clock_patch_cfg_is_applied ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

- `tests/wasi_patch_guard.rs` asserts the vendored `clock.rs` carries
  `all(target_arch = "wasm32", target_os = "unknown")` and that the unpatched
  upstream gate `#[cfg(target_arch = "wasm32")]` is absent (typed
  `WASI_PATCH_UNVERIFIED` flunks), plus a fail-closed cfg court on
  `graphlaw_wasi_clock_patched` with a negative arm so a cfg-gated test can
  never silently vanish.
- Patch applicability re-verified this session against the pristine 2.0.2 tree
  (downloaded from crates.io):

```
$ patch --dry-run -p1 < vendor/upstream/purrdf-sparql-eval.patch
patching file Cargo.toml
patching file 'src/clock.rs'
patching file 'src/governor/mod.rs'
DRYRUN_EXIT=0        (apply also exit 0)
$ diff -u src/clock.rs /Users/sac/graphlaw/vendor/purrdf-sparql-eval/src/clock.rs
CLOCKDIFF_EXIT=0     (patched pristine clock.rs is byte-identical to the vendored copy)
```

## Submission checklist

- [x] Patch file applies clean to pristine 2.0.2 (`patch --dry-run -p1`, exit 0)
- [x] Patched tree byte-matches the vendored copy (clock.rs diff exit 0)
- [x] Guard court green with the feature armed (`cargo test --test wasi_patch_guard --features wasi-patched-deps`, exit 0, 2 passed)
- [x] Upstream 3.x checked — 3.0.1 still carries the browser-only gates (crates.io download + grep, exit 0)
- [x] License/CONTRIBUTING constraints identified; no feature flags added (complies with "No Cargo features")
- [x] deny.toml MulanPSL-2.0 allowance already in place for 3.x adoption
- [ ] Apply the same re-gating to the 3.0.1 tree (incl. its new `wasm-bindgen` dep gate) and re-verify before rebasing the patch onto 3.x
- [ ] Fork Blackcat-Informatics/purrdf, branch, apply patch
- [ ] Run upstream `make check` + `make conformance` (Node.js 24 required)
- [ ] Open PR against `main` (title and body drafted in `vendor/upstream/purrdf-sparql-eval.md`)
- [ ] On merge + release carrying the gate: delete `vendor/purrdf-sparql-eval/`, its `[patch.crates-io]` entry, and the `wasi-patched-deps` feature (vendor/README.md instructions)
