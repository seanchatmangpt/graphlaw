# Proposal: normalize wasm/ggen.toml paths (L160)

Status: PROVEN (relative-path form renders byte-identical output; no ggen change required)
Date: 2026-10-22
Lane: L160 · repo: /Users/sac/graphlaw

## Problem

`wasm/ggen.toml` hardcodes four `/Users/sac` absolute paths
(`ontology.source`, `ontology.imports[0]`, rule `query.file`,
rule `template.file`). Any other checkout location breaks `ggen sync run`.

## Mechanism survey (ggen 26.10.11 source, ~/ggen)

- **No `${VAR}` env-var interpolation** exists in ggen.toml parsing.
  `ggen-config` `load_with_env` only applies `[env.NAME]` *override tables*
  keyed by named environments ("development"/"production") — not env-var
  expansion. REQUIRES-UPSTREAM if env interpolation is wanted; NOT needed here.
- **Relative paths ARE resolved against the project root** (the ggen.toml
  directory / cwd): `ggen-engine/src/generation_rules.rs`
  `root.join(&manifest.ontology.source)` (line 257), `root.join(import)`
  (line 264), `root.join(rel)` for rule files. So relative paths are the
  native portable mechanism.
- `[[packs]]` + `QuerySource::Pack`/`TemplateSource::Pack` also exist
  (`resolve_pack_root`, line 911) — pack base is declared once in
  `[[packs]]` with `path` (also root-joined). Viable alternative; not needed
  since only two pack files are referenced.

## Proposed config (applied to wasm/ggen.toml, untracked — operator decides commit-as-is vs normalize)

Only the four path lines change; comments document the normalization:

```toml
[ontology]
source = "../ontologies/graphlaw-wasm.ttl"
imports = ["../../ggen-marketplace/packs/rust-wasi-wasmex-pack/ontology.ttl"]

[[generation.rules]]
name = "relative-proof"
query = { file = "../../ggen-marketplace/packs/rust-wasi-wasmex-pack/queries/ffi.rq" }
template = { file = "../../ggen-marketplace/packs/rust-wasi-wasmex-pack/templates/abi/ffi.rs.tmpl" }
```

(The `[[generation.rules]]` keeps `name = "ffi"`, `mode = "Overwrite"`,
`output_dir = "src"`, `output_file = "ffi.rs"` unchanged.)

**Layout assumption (declared, not hidden)**: `ggen-marketplace` is the
canonical sibling checkout at `~/ggen-marketplace`, adjacent to `~/graphlaw`.
`../../ggen-marketplace/...` is portable across usernames and machines with
that layout, and portable across the repo location. If the marketplace path
itself varies, the fallback is a wrapper (`scripts/ggen_sync_wasm.sh`)
exporting `GGEN_PACK_PATH` and materializing a concrete config — but ggen has
no env interpolation, so that would require upstream support; relative paths
make it unnecessary.

## Proof (executed 2026-10-23, ggen 26.10.11, /Users/sac/graphlaw/wasm)

1. Wrote the relative-path config into `wasm/ggen.toml`.
2. `ggen sync run` → `"written": []` (idempotent re-run; file unchanged) —
   `b3sum src/ffi.rs` =
   `9dcab4e4099ede0e605b2f619c8791e8f19bf656f3c6a8bd552eeade6d9e46a9` (known-good). (historical)
3. Delete-and-regen: `mv src/ffi.rs /tmp/ffi.rs.bak && ggen sync run` →
   `"written": ["src/ffi.rs"]`, `graph_hash_hex`
   `8821e277608452cd44baac350c850e4d96759b17a64bec9b08f2b644b0c07ae2` (matches (historical)
   the L49 determinism receipt), and regenerated
   `b3sum src/ffi.rs` =
   `9dcab4e4099ede0e605b2f619c8791e8f19bf656f3c6a8bd552eeade6d9e46a9` — (historical)
   byte-identical to the known-good artifact.

## REQUIRES-UPSTREAM

None. Relative paths are natively supported in ggen 26.10.11.

## Non-goals

- No wrapper script proposed (unnecessary given native relative-path support).
- `abi_meta.rs` remains out of scope of this pin (L49 decision, unchanged).
