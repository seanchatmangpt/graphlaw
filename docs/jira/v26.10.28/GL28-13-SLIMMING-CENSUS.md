# GL28-13 — WASM Slimming Census (74,097,211 B artifact)

All evidence from wasm-objdump on priv/graphlaw.wasm plus the build graph
(Cargo.toml:80 — frozen-duckdb is a non-optional dependency linked into the
one unit on every target).

## Section budget

| Section | Bytes | Share |
| --- | --- | --- |
| Code | 69,662,174 | 94.0% |
| Data | 4,253,753 | 5.7% |
| Function decls | 96,851 | 0.13% |
| Elem | 75,561 | 0.1% |
| Custom sections | 0 | — |

The artifact has zero custom sections: `strip = true` in [profile.wasm] is
fully effective and debug/custom sections contribute 0 recoverable bytes.

## Attribution — who owns the 74 MB

The wasm link consumes /Users/sac/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/:
libduckdb.a (84 MB archive, 88.5 MB member payload, 248 objects),
libcore_functions_extension.a (16.5 MB payload), libicu_extension.a
(3.3 MB payload incl. tz/collation data). Pin history in
registry/ARTIFACTS.blake3 attributes growth: graphlaw/purrdf itself is
~7.1 MB total pre-DuckDB (pin f149433c) — DuckDB archives are the growth.

## Ranked slimming levers

| Lever | Est. savings | Lands in | Risk |
| --- | --- | --- | --- |
| 1. Drop ICU from the wasm link — remove libicu_extension.a from the cache link set + delete the icu_describe registration in wasm/src/lib.rs (graphlaw has zero COLLATE/tz usage) | −3.0 MB | graphlaw wasm + fork cache hygiene | low |
| 2. Rebuild libduckdb.a at MinSizeRel/-Oz for wasm32 only (frozen-duckdb-builder compile_wasip1_static() is currently Release/-O3; keep host at Release) | −10 to −18 MB | frozen-duckdb fork | medium |
| 3. Prune core_functions function-macro groups graphlaw never calls (window/dated families; aggregates SUM/COUNT required) | −3 to −6 MB | frozen-duckdb fork | medium |
| 4. libc++ static trimming | −0.5 to −1.5 MB | fork | marginal |

Levers 1+2 put the artifact at ~51-59 MB — under the 64 MiB gate. Lever 3
only if 1+2 fall short.

## Named recovery buckets

- ICU collation tables: ~2.5-3.3 MB, entirely recoverable (explicit GC root
  in the wasm crate).
- Unused DuckDB core functions: bounded by the 16.5 MB archive; realistic
  −3 to −6 MB.
- Debug/custom sections: 0 bytes — already stripped.
