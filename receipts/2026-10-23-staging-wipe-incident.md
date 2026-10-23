# Incident: canonical frozen-duckdb staging wiped mid-wave

- **When:** discovered 2026-10-23 01:47; wipe mtime 01:34 on
  `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/` (emptied: 88,161,726 B libduckdb.a,
  sysroot, STAGING-NOTES.md all gone).
- **Survivor:** `v2.0.0-dev-wasm32-wasip1.legacy-eh.bak/` (00:18) — the PREVIOUS
  legacy-EH archive set, superseded this wave; usable as reference only.
- **Suspected cause:** unattributed deletion during the concurrent wave (no lane claimed
  it; cleanup scripts exist in the tree). No receipt exists for the deletion — by repo law
  a wiped staging without a receipt is an unreceipted destructive transition.
- **Impact:** the new-EH `libduckdb.a` (built 00:05, verify exit 0) is lost; every wasm
  build needing `WASI_SDK_PATH` is BLOCKED until the staging rebuild lands.
- **Recovery in flight:** `build_wasi_duckdb.sh --force` (new-EH flags
  `-fwasm-exceptions -mllvm -wasm-use-legacy-eh=0`) rebuilding the full staging; the
  uniformly new-EH `libunwind` candidate (variant B, sha256 `6a5f79f7…`, wasmtime EH-OK
  witnessed) integrates afterward so the whole module is uniform new-EH.
- **Law going forward:** staging transitions must be receipted and coordinator-owned;
  concurrent lanes must never mutate `~/.frozen-duckdb` — same discipline as the
  coordinator-owned root manifest.

## RESOLUTION (appended 2026-10-23)

- **Staging rebuilt** 2026-10-23 01:52 via `build_wasi_duckdb.sh --force`:
  staging verify **VERIFY-OK, exit 0**; `libduckdb.a` restored at
  **88,161,726 B** (matches the W1 GREEN build in
  `docs/jira/v26.10.22/RECEIPTS.md`). The BLOCKED-on-staging impact is
  closed.
- **libunwind candidate B integrated:** the uniformly new-EH candidate
  (sha256 `6a5f79f7…`) installed into `lib/wasm32-wasip1/` plus the
  `share/wasi-sysroot` copy. Pre-candidate backup kept at
  `libunwind.a.pre-candidate.bak` (reversible option preserved).
- **Canonical expectation updated:** the build script's expected libunwind
  digest now pins the candidate sha (`6a5f79f7…`) with a typed FAIL naming
  the legacy variant (`7ab65b4c…`) — a legacy-EH object can no longer pass
  the build silently.
- **EH-flavor mixing RULED OUT as sole trap cause:** with the uniform
  new-EH module (candidate B integrated end to end), the rebuilt DuckDB-linked
  artifact measures **46,069,809 B / Tag(13) × 1** and still traps on
  uninitialized-element at the dtor runner. Mixed-EH linkage is therefore not
  the whole story; the **LTO matrix + dtor-dump experiments are in flight**
  (open rows, no verdict claimed here).
- **Standing:** RESOLUTION covers staging restoration + libunwind integration
  only. The dtor-runner trap remains OPEN pending the LTO matrix / dtor-dump
  results; no flip of any ledger row rests on this section.
