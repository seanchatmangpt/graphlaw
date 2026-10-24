# Artifact build flags — pre-staged landing spots

Status: PRE-STAGE (no winning candidate yet). This doc is written before the
trap probe names the winning flag; when the probe lands, apply the matching
section here and update Status to LANDED with a receipt.

Related: `wasm-link-root-cause.md`, `wasmi-eh-research.md`,
`g2-repin-prep.md`, `wasm-repin-receipt.md` (same designs dir).

## 1. Current environment (as-of this writing)

- Local staging: `WASI_SDK_PATH` points at a **staging** wasi-sdk root.
- CI: `/opt/wasi-sdk` (wasi-sdk 34, installed by the wasm job from the
  official release tarball).
- Rustflags (both local and CI):
  `CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS="-C target-feature=+exception-handling"`
  — required at link time by the new-EH DuckDB wasm archive; wasmtime 48
  hosts it, wasmi cannot (legacy wasmi ABI court is a loud skip).
- `.cargo/config.toml` currently sets only
  `rustflags = ["-C", "link-arg=-zstack-size=16777216"]` under
  `[target.wasm32-wasip1]`. No dead-code / GC-section flags anywhere yet.
- There is **no `build_wasi_duckdb.sh`** in this repo today; the libduckdb
  wasm32 archive is built inline in `.github/workflows/ci.yml` (wasm job,
  `DUCKDB_PIN=b6013f7...` block, ~lines 72-104). Candidate (c) therefore
  lands as a ci.yml change unless the script is extracted first.

## 2. Size / Tag / trap test matrix template

Run for each candidate before landing; the winner must beat baseline on all
three axes or the baseline is re-confirmed and no flag lands.

| Candidate | Byte size | producers dump | EH court rung | Verdict |
|---|---|---|---|---|
| baseline (+EH only) | | | | |
| (a) + link-dead-code=true | | | | |
| (b) + --no-gc-sections | | | | |
| (c) emsdk-object fix | | | | |

Fill per row:

1. **Size**: `stat -f%z` (macOS) / `stat -c%s` (Linux) on
   `target/wasm32-wasip1/wasm/graphlaw_wasm.wasm`.
2. **Tag/producers**: `strings ... | grep producers` or the wasm object dump
   used by the trap probe; must show no emsdk-object remnants for (c).
3. **Trap/ABI rung**: `cargo test --test wasm_abi` (expected to still fail
   under wasmi until the dual-artifact policy lands) plus the wasmtime EH
   courts:
   `cargo test --test wasi_patch_guard --test wasm_duckdb_lifecycle_eh \
   --test wasm_duckdb_op_wired --features abi,wasi-patched-deps,runner-eh`.
4. **Verdict**: winner named, receipt path, applied landing spot (§3).

## 3. Per-candidate landing spots

### (a) `-C link-dead-code=true` wins

CI: add to both env blocks (`.github/workflows/ci.yml` ~116-123 and
~250-258):

```yaml
        env:
          CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS: >-
            -C target-feature=+exception-handling -C link-dead-code=true
```

`.cargo/config.toml` (coordinator applies; keeps local parity):

```toml
[target.wasm32-wasip1]
rustflags = [
  "-C", "link-arg=-zstack-size=16777216",
  "-C", "link-dead-code=true",
]
```

Note: rustflags env var and config.toml rustflags do not merge — the env var
wins. CI uses the env var only; config.toml exists so local builds (which
may not export the env var) get the same flags. Keep the two in sync or
local and CI artifacts diverge.

### (b) `--no-gc-sections` wins

Same two spots, as a link-arg:

```yaml
          CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS: >-
            -C target-feature=+exception-handling
            -C link-arg=--no-gc-sections
```

Exact TOML for (b):
[target.wasm32-wasip1]
rustflags = [
  "-C", "link-arg=-zstack-size=16777216",
  "-C", "link-arg=--no-gc-sections",
]
```

### (c) emsdk-object fix (libduckdb rebuild flag)

Landing spot: the inline libduckdb build in `.github/workflows/ci.yml`
(~lines 72-104, `DUCKDB_PIN` block). Add the flag to the `cmake -S ... -B
...` configure invocation, e.g. a CMake flag on the configure line after
`-DDISABLE_THREADS=ON`. Expected effect: rebuilt `libduckdb.a` contains no
emsdk-object producers section; verify with the producers dump in §2 before
re-pin. Staging verify expectation: after the rebuild, the local staging
`WASI_SDK_PATH` build must produce an artifact whose producers dump matches
CI's byte-for-byte (both consumed from the same `~/.frozen-duckdb/cache`
style cache admission — CI admits the cached archive via `is_wasm32_ar`).

Apply-then-pin order: apply the flag, rebuild the archive, clear the CI
cache key by bumping the cache `key:` (else the stale archive is re-admitted
— the cache admits on `is_wasm32_ar` alone, not on build flags).

## 4. Re-pin runbook pointer

Re-pin follows the existing runbooks in this directory:
`wasm-repin-receipt.md` (receipt format + ASSETS.blake3 re-pin) and
`g2-repin-prep.md` (pre-pin checklist). The winning artifact's new digest is
pinned via the normal registry sync flow (never hand-edit
`registry/ARTIFACTS.blake3` or `registry/capability-registry.json`).

## 5. CI wiring notes

- Env blocks to touch: `.github/workflows/ci.yml` lines ~116-123 (native
  job "Build WASI module") and ~250-258 (wasm job "Build WASI module").
  Grep anchor: `CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS` appears at lines 121
  and 252.
- Both jobs must change together; a single-job edit leaves CI internally
  inconsistent (artifact uploaded by one job fails the other's courts).
- `--locked` on the build step is unaffected (flags are not manifest
  changes); `Cargo.toml` stays coordinator-owned.
- The legacy wasmi ABI court stays a loud skip (`continue-on-error: true`)
  unless the winner also shrinks/removes the EH section — re-check
  `wasmi-eh-research.md` after the probe lands.
