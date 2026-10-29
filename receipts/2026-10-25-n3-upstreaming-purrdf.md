# Receipt: Notation3 engine upstreaming — `graphlaw-eyeron` → `purrdf::n3`

- **Date**: 2026-10-25
- **Subject**: canonical checkout `/Users/sac/graphlaw`, branch `main`, base `a00770b` (dirty
  tree; the whole session lineage is uncommitted — no commit was requested).
- **Operator order**: "replace the old way n3 was handled in the app. It should just work out
  of the box like the other purrdf capabilities."
- **Mode**: 20-lane same-checkout fan-out (upstream crate build at `~/purrdf`), then
  coordinator integration + 5-lane pin sweep (T1 engine courts, T2 docs courts, T3 CI/release,
  T4 prose docs, T5 thesis doc) with disjoint file leases; coordinator owned manifests, git,
  pin files, generated artifacts.

## What was manufactured

1. **Upstream crate built first** (`~/purrdf`, canonical purrdf checkout, uncommitted):
   `crates/n3` (`purrdf-n3` 3.0.1) — full eyeron port (ast/error/lexer/parser/reasoner/
   printing/proof_check/proof_check_n3/proof_writer), rewritten `rdf_compat` bridge onto
   upstream `purrdf-rdf` types (typed `ConversionError`, lossy edges refused never flattened),
   umbrella re-export `purrdf::n3` (glob, same as geo/text), docs, corpora (32 files,
   9 N3 conformance cases), bench, example. Convention parity: layers.toml, Makefile wasm
   gate, release lane, dependency-ledger (num-bigint/num-traits/num-integer/autocfg), README,
   mdBook, CHANGELOG. Gates green there: 46 crate tests, umbrella 46, conformance 1, bench
   6 measured, check-layers OK, ledger OK, check-versions OK.
2. **Graphlaw switch**: `crates/graphlaw-eyeron` DELETED (workspace member removed, dir
   removed); vendored `vendor/purrdf-n3` (cargo-packaged 3.0.1, crates.io-normalized) feeds
   the vendored umbrella by path dep; graphlaw consumes ONLY `purrdf::n3` through the
   existing `purrdf = "=3.0.1"` dep; curated module `pub mod n3 { pub use purrdf::n3::*; }`
   (`src/lib.rs`) is the sole `graphlaw::n3` path (escape hatch `pub use eyeron;` deleted).
3. **Identity collapse**: `Engine::Eyeron` variant REMOVED — engines `{PurRdf, FrozenDuckdb}`;
   `Dialect::N3 => Engine::PurRdf` (receipts/refusals now report `"engine":"PurRdf"`, same as
   SPARQL/datalog); `BACKEND_AUTHORITIES` Notation3 row → `authority: "purrdf::n3",
   revision: "3.0.1"`. Registry `ENGINES = ["PurRdf", "FrozenDuckdb"]`.
4. **Pin sweep** (~40 pins): authority/api-stability/wasm-abi/chicago/ocel/law-state courts,
   docs-drift + provenance-consts courts (rewritten to pin the new truth, vacuity controls
   kept), CI workflow legs (`cargo package -p graphlaw` only), release.yml publish planning
   (graphlaw alone), deny.toml/.cargo comments, ~19 prose docs + thesis Ch.15 rewrite
   (module map grounded on disk; one historical lineage sentence kept, past tense).
5. **Vector-gen pipeline**: `graphlaw-eyeron` bin (already dead — manifest declared it,
   src deleted) replaced by `src/bin/n3_derive.rs` (reads N3, prints derived triples as
   prefixed N3, fail-closed on incomplete fixpoints); `scripts/gen_wasm4pm_graphlaw_vectors.py`
   repointed (`N3_BIN`, `n3_engine: purrdf-n3/3.0.1`, `purrdf 3.0.1`); end-to-end witnessed
   (6 vectors generated through the new engine, provenance verified).
6. **Generated artifacts regenerated** (never hand-edited): `registry/capability-registry
   .json/.ttl` + `ARTIFACTS.blake3` (registry/surface/receipt-surface rows) via
   `graphlaw-registry --write` (rebuilt release bin first — the first --write ran from a
   stale binary and was caught by `capabilities_digest`, fixed by rebuild+re-emit);
   `js/npm` generated types (gen-types + tsc + assemble; dist-*/generated/types.d.ts now
   `PurRdf | FrozenDuckdb`); `docs/reference-generated/` (doc-hdit scaffold after patching
   the marketplace generator to skip `target*` dirs — the stale `target-lane-pkg2` lane root
   was injecting deleted-crate sections; generator patch in `~/ggen-marketplace`, disclosed);
   `courts/code-partition/{manifest.json,axioms.blake3,.next}` (eyeron roots dropped, 20
   staging `_split/` ghost paths remapped to wired paths, 4 new egress/bin files classified,
   axiom digests re-pinned from disk — G1/G2/G4 PASS, G3 red by design: 41 unadmitted
   projection leaks is the codegen roadmap tracker, `admitted-leaks.json` empty by charter).
7. **dl_dualroute vector pins re-pinned** with reason (provenance metadata change:
   `n3_engine`/`purrdf 3.0.1`): a-paper-baader-el `62bf42ab…`, b-chain3-conj `570a9a22…`,
   c-exists-pair `0486fa91…`, d-exists-conj-interlock `05478a0b…`; fixtures re-emitted into
   `~/wasm4pm/.../fixtures/graphlaw/` (sibling canonical checkout, machine-pinned by design).
8. **`ASSETS.blake3`** regenerated via the court's sanctioned `UPDATE_ASSET_MANIFEST=1`:
   the in-flight sniffer work now labels `schema_all_https.nt` as `Turtle` (same bytes,
   same digest `d501ffc9…`; N-Triples ⊂ Turtle). Pre-existing delta from the parallel
   in-flight wave, not the N3 migration.

## Verification ladder (real exits)

| Rung | Command | Result |
|---|---|---|
| 1 | `cargo check` (+ `--tests`, `--examples`) | clean; 0 errors |
| 2 | Targeted courts | authority courts 2+6+8+8+2+12 green; N3 courts (chicago_n3_datalog 6, n3_fuse 5, wasm_n3_admission 3) green; docs courts (ci_features 3, docs_drift 3, docs_release 2, provenance_consts 4) green; sql-rdf acceptance set (sql_rdf_dialect 36, chicago_validation 12, columns/pivot/star/set-op 27) green; capabilities_digest 6; corpus_conformance 12; wasm_size 4 |
| 3 | `cargo test` (full suite, CI env) | 37 binaries ok, 38 verdict lines: every binary `test result: ok` except 1 known court re-run after its sanctioned regen (see above); final targeted rerun of that court green. 360+ tests passing across the suite with zero failures at close |
| 4 | Grep gate | zero `eyeron`/`graphlaw-eyeron` outside `vendor/` history, `docs/jira/`, `docs/archive/`, `receipts/`, CHANGELOG dated history, negative-court assertions, and the one past-tense lineage sentence (each intentional) |
| 5 | `WASI_SDK_PATH=/Users/sac/wasi-sdks/wasi-sdk-34 CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS="-C target-feature=+exception-handling" cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` + wasm courts | **ALIVE**: module links at 60,880,445 B (< 64 MiB ceiling, size court 4/4); wasm_abi 71, chicago_cross_authority 25, wasm_n3_admission 3 — purrdf::n3 executes N3 inside the linked module |
| 6 | `bash scripts/mutation_court.sh --force` | **INTERRUPTED at the 2 h harness cap** (max window; not restarted): unmutated baseline PASS (31s build + 55s test), 204 mutants enumerated, >=1 kill witnessed pre-interrupt (mutant flipped `law_state::transition_with_unknown_capability_refuses_typed`), tree restored byte-exact (sha256-verified). Full 204-mutant sweep remains an open rung for the next window |

## Environment repairs made to close rung 5 (disclosed)

- `/opt/wasi-sdk` is ABSENT on this workstation; the real SDK is
  `/Users/sac/wasi-sdks/wasi-sdk-34` (found by lane T1).
- The local `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1` archive was stale for the
  in-flight wasi recipe. Rebuilt from source per `action.yml` at the pinned duckdb commit
  `b6013f7a…` with wasi-sdk-34; first build failed deterministically in
  `http_transport_manager.cpp` (`struct rlimit` incomplete — wasi-libc carries the rlimit
  surface behind `__wasilibc_unmodified_upstream`). **Completed `wasi_compat34.h`** with the
  rlimit compat block (`struct wasi_compat_rlimit` + `rlimit`/`RLIMIT_NOFILE`/`RLIM_INFINITY`
  + `getrlimit(int, void*)` decl; symbol provided at link by frozen-duckdb-sys
  `wasi_posix.c:54`). EH-flavor gate ADMITTED (246 members, Σtry_table=174551, legacy=0 on
  every member); old cache preserved at `…pre-n3-migration.bak`.
- Rust target `wasm32-wasip1` added to the active toolchain (T1, environment only).

## Generated vs handwritten

- Generated (regenerated by flows): registry json/ttl/ARTIFACTS rows, js/npm types + dist,
  docs/reference-generated, code-partition manifest/pins, ASSETS.blake3, dl_dualroute
  vectors (bin), wasm4pm oracle vectors (script), mdBook page copy.
- Handwritten: `src/bin/n3_derive.rs` (new, ~60 lines), shim rlimit block (~15 lines),
  prose/docs edits, court expectation updates.

## Standing

- **N3 pipeline: ALIVE** — purrdf::n3 3.0.1 (vendored) is the only Notation3 engine; parses,
  reasons, emits proofs, executes in WASM; vector pipeline produces engine-stamped vectors.
- **Wave standing: PARTIAL_ALIVE** — every executed rung green at close; the 204-mutant
  sweep (court rung 6) is the single open rung, interrupted at the harness's max window
  after a witnessed baseline PASS + kill flip + byte-exact restore.

## Falsifiers (how to falsify this receipt)

1. `grep -rn "graphlaw::eyeron" src/ tests/` → any hit falsifies the identity collapse.
2. `cargo test --test backend_authorities --test authority_manifest` → red falsifies the
   authority row.
3. Delete the `rlimit` block from `wasi_compat34.h` and rebuild duckdb wasm → the http TU
   must fail again (shim non-vacuity).
4. `WASI_SDK_PATH=/Users/sac/wasi-sdks/wasi-sdk-34 … cargo build -p graphlaw-wasm --target
   wasm32-wasip1 --profile wasm` → link failure falsifies rung 5.

## Typed residues (disclosed, not laundered)

- `scripts/partition_court.sh` G3: 41 unadmitted projection leaks — pre-existing by-design
  codegen roadmap tracker (`admitted-leaks.json` empty by charter); not a regression.
- 41 orphaned `target-*` lane build roots from prior waves remain on disk (deletion denied
  by the permission system this session); `target-lane-pkg2` still pollutes nothing after
  the generator-side `target*` skip, but disk hygiene (~tens of GB) awaits operator action.
- The in-flight parallel waves' uncommitted state (wasi recipe, sniffers, mutants configs)
  rides in the same working tree; the N3-migration slice is enumerated above.
- `/tmp/n3probe/` (Lane 19 scratch) and session scratch under /tmp remain (cleanup denied).
