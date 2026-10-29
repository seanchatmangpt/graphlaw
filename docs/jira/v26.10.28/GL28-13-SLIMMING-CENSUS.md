All evidence gathered. Here is the GL28-13 analysis.

## GL28-13: artifact analysis — priv/graphlaw.wasm (74,097,211 B)

### Section budget (wasm-objdump -h)

| Section | Bytes | Share |
|---|---|---|
| Code | 69,662,174 | 94.0% |
| Data | 4,253,753 | 5.7% |
| Function decls | 96,851 | 0.13% |
| Elem | 75,561 | 0.1% |
| Type/Import/Export/… | ~8,800 | trivial |
| **Custom sections** | **0** | — |

twiggy is not installed; there is also nothing for it to symbolicate — **the artifact has zero custom sections** (no `name`, no debug): the file ends exactly at the Data section end, so `strip = true` in `[profile.wasm]` is already fully effective and **debug/custom sections contribute 0 recoverable bytes**. Function-count-by-crate from symbols is impossible; I reconstructed attribution from the build graph and the pin history instead.

### Attribution — who owns the 74 MB

The build graph is explicit (`/Users/sac/graphlaw/Cargo.toml` line 80): `frozen-duckdb = { version = "2.0.2" }` is a **non-optional dependency linked into the one unit on every target, wasm32 included** ("DuckDB is not a feature (repo law)"). The wasm link consumes `/Users/sac/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/`:

- `libduckdb.a` — 84 MB archive, 88.5 MB member payload (248 objects, biggest single member 10.7 MB)
- `libcore_functions_extension.a` — 16.5 MB payload
- `libicu_extension.a` — 3.3 MB payload (33 objects incl. tz_data, collation data)

Pin history in `/Users/sac/graphlaw/registry/ARTIFACTS.blake3` gives exact attribution:

| Pin | Date | Size | Delta | Cause |
|---|---|---|---| frozen-duckdb fork | graphlaw wasm profile | Notes |
|---|---|---|---|---|---|---|
| Remove ICU from the wasm link | ~−3.0 MB | **frozen-duckdb fork** (delete `libicu_extension.a` from the cache dir — build.rs only links what is present) + **graphlaw wasm** `wasm/src/lib.rs` (delete the `icu_describe` extern + its registration) | 71.1 MB. graphlaw src has **zero** ICU/collation usage (verified: no COLLATE, no tz). ICU is pulled in only by `duckdb_static_extensions` in `wasm/src/lib.rs` lines ~63–77. Landed in graphlaw too small |
| Rebuild `libduckdb.a` at -Oz/MinSizeRel for wasm32 only | −10 to −18 MB | **frozen-duckdb fork**: `frozen-duckdb-builder/src/lib.rs` `compile_wasip1_static()` — currently `-DCMAKE_BUILD_TYPE=Release` (−O3). Use `MinSizeRel` or per-file `-Oz` for the wasip1 target; keep host build at Release | ~61 MB |
| Prune core_functions archive | −3 to −6 MB | **frozen-duckdb fork**: rebuild without unneeded function macro groups (window/dated/aggregates graphlaw never calls) | ~55–58 MB |
| libc++ static trimming (-fno-exceptions variants, --strip in ar members) | −0.5 to −1.5 MB | fork | marginal |
| graphlaw/purrdf itself | — | already ≈7.1 MB total pre-DuckDB (pin f149433c); not the problem | — |

### Recommended sequence

1. **Drop ICU (graphlaw wasm + fork cache hygiene)** — 2-line change in `wasm/src/lib.rs` (remove `icu_describe` reference/registration) plus removing the archive from the link set. Lowest risk, immediate −3 MB. Verifiable: `duckdb_extension_icu_describe` disappears from the artifact and the "using the ICU library" string leaves the Data section.
2. **-Oz/MinSizeRel wasm32 archive rebuild in the fork** — the single biggest lever, −10 to −18 MB, and it is a one-line cmake change in `frozen-duckdb-builder`. Combined with (1): **≈51–59 MB, under the 64 MiB target.**
3. Optional third lever (core_functions pruning) only if (1)+(2) don't clear the bar.

### Answering the three named recovery buckets

- **ICU collation tables**: ~2.5–3.3 MB recoverable, entirely — graphlaw never uses collation; registration is an explicit GC root in the wasm crate.
- **Unused DuckDB core functions**: bounded by the core_functions archive (16.5 MB); realistically −3 to −6 MB after pruning unused function families. Not fully droppable — aggregates (SUM/COUNT) are required.
- **Debug/custom sections**: **0 bytes** — the artifact already contains none.