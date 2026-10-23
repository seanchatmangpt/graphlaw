# Stale WASI/frozen-duckdb reference sweep — lane L142

Date: 2026-10-22. Read-only sweep; no repo files edited (this report is the
only file written). Exclusions honored: import-pass files, lanes 1-11 owned
files, dated receipts, and historical-marked text.

## Scope + method

Searched: `src/ tests/ benches/ examples/ wasm/ scripts/ docs/ .github/ ggen/
ontologies/ Cargo.toml` for three claim classes:

- (a) frozen-duckdb **2.0.1** (stale version — repo pins 2.0.2,
  `Cargo.toml:35 frozen-duckdb = { version = "2.0.2" }`)
- (b) a path `[patch]` / patch instruction pointing at
  `/Users/sac/frozen-duckdb`
- (c) the engine cache directory (`~/.frozen-duckdb/cache/...`) used as
  `WASI_SDK_PATH` in an instructional sense

## Receipts (real grep commands + exits)

```
$ grep -rn "2\.0\.1" src tests benches examples wasm scripts docs .github ggen ontologies
  | grep -iv receipt | head -50
…only hits: tests/fixtures/w3c/*/reports/raptor2012-earl-*.ttl:23
  doap:name "Raptor 2.0.12"   (unrelated fixture DOAP metadata)
EXIT1=0

$ grep -rn "/Users/sac/frozen-duckdb\|\[patch" src tests benches examples wasm \
  scripts docs .github ggen ontologies | grep -iv receipt | head -50
EXIT2=0

$ grep -rn "WASI_SDK_PATH" src tests benches examples wasm scripts docs .github \
  ggen ontologies Cargo.toml | grep -vi receipt | head -40
EXIT3=0

$ grep -rn "\.frozen-duckdb" src tests benches examples wasm scripts docs .github \
  ggen ontologies | grep -vi receipt | head -30
EXIT4=0
```

## Findings: LIVE stale hits = 0

**Class (a) frozen-duckdb 2.0.1 — 0 live hits.** Only fixture DOAP text
("Raptor 2.0.12"). The pin is 2.0.2 everywhere in scope.

**Class (b) /Users/sac/frozen-duckdb path patch — 0 live hits.** All
`/Users/sac/frozen-duckdb` occurrences outside excluded files are either:

- dated-receipt / historical narrative, e.g.
  `docs/jira/v26.10.22/SQLRDF-15.md:54,74`,
  `docs/jira/v26.10.22/RELEASE-NOTES-DRAFT-26.10.21.md:27,47,69` (historical
  release notes), `docs/jira/v26.10.22/designs/sweep-pre-integration-2.md:24`
  (dated audit receipt), `docs/jira/v26.10.22/designs/wasi-sdk-reproducibility.md:4,9,29`
  (dated receipt: "Subject staging … == verify staging"), or
- **guards asserting the patch's ABSENCE** (inverses, not instructions):
  `tests/wasi_patch_guard.rs:3` ("[patch.crates-io] purrdf-sparql-eval).
  Until now nothing verified that…"),
  `tests/authority_manifest.rs:109-110` ("[patch] entries are ignored on
  publish by design"),
  `docs/reference-generated/reference.md:2909,5952` (generated projection of
  grammar GRAM paths — regenerated artifact, not an instruction).

One borderline: `docs/refusal-playbook.md:113` — "…build-script compile error,
pre-existing state of the `/Users/sac/frozen-duckdb` working tree, not this
repo. The probe was run against the shared `target/` build cache instead;
disclosed per the compile-freeze disclosure rule." This is a dated, past-tense
environment note about a probe run ("the first attempt…"), not a live
instruction to patch; classified historical, not a hit.

**Class (c) engine cache used as WASI_SDK_PATH instructionally — 0 live
stale hits.** Every instructional `WASI_SDK_PATH` text in scope says the
OPPOSITE (correctly): the cache dir is NOT an SDK.

- Live instructional sites are clean: `tests/wasm_duckdb_op_wired.rs:48`
  ("build it with `WASI_SDK_PATH=<wasi-sdk> cargo build -p graphlaw-wasm
  --target wasm32-wasip1 --profile wasm`") — instructs a wasi-sdk root, not
  the cache; `docs/refusals.md:91` (lane 1: "the engine cache is not a
  wasi-sdk"); `docs/WASM.md:19-21` (excluded file: "the engine archive, not
  that SDK").
- Cache-path hits remaining in scope are inside dated receipts only:
  `docs/jira/v26.10.22/designs/refusals-register-reconciliation.md:64`,
  `designs/wasi-sdk-reproducibility.md:5,29`,
  `designs/sweep-pre-integration-2.md:18`.

Excluded-file hits (listed for completeness, out of lane scope):
`src/abi_manifest.rs:137`, `tests/wasm_duckdb_op.rs:8,10`,
`docs/WASM.md`, `docs/wasm-integration.md`,
`docs/wasm-duckdb-integration.md`, `docs/wasm-duckdb-op15-design.md`,
`docs/jira/v26.10.22/THESIS.md:46`, `SQLRDF-08.md:13`,
`.github/workflows/ci.yml:27-42,177-182`, `release.yml:52-119`
(all consistent 2.0.2 / wasi-sdk-root guidance).

## Verdict

Zero live stale instructions for frozen-duckdb 2.0.1, a
/Users/sac/frozen-duckdb path `[patch]`, or cache-dir-as-WASI_SDK_PATH.
The live surfaces (Cargo.toml pin, guard tests, wired-test panic message,
workflow cache staging) already state 2.0.2 and wasi-sdk-root semantics.
