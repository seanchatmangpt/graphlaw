# DOC-STALENESS (v26.11.17, lane 15)

Method: ran `bash scripts/doc_staleness_gate.sh` (read-only); link-checked docs/index.md, README.md, CLAUDE.md; resolved backticked repo paths across living docs. No cargo run. Wasm pin bytes and counts inside other docs not checked.

## Verdict
- docs/index.md, README.md, CLAUDE.md: every relative link resolves.
- Headline versions match the tree (Rust 1.98.0, frozen-duckdb 2.0.2, 26.10.28; README:35 "15 ops, src/abi.rs:319-333" true).
- Gate: FINDINGs from R4 and R5; R1 clean.

## Findings
| # | doc | stale claim | tree truth | where |
|---|---|---|---|---|
| 1 | CLAUDE.md | active window "currently docs/jira/v26.10.28/" | newest is v26.11.16 (now v26.11.17); v26.10.22 still present though "archived" | CLAUDE.md:80-82 |
| 2 | docs/index.md | "Active window: jira/v26.10.28/" | newest is v26.11.16 | index.md:136 |
| 3 | docs/index.md | links jira/v26.10.22/* as live | CHANGELOG/CLAUDE say archived; tree still has them | index.md:110-118,160-166 |
| 4 | docs/index.md | relocation map left column cites jira/v26.10.23/... | dir absent; fine as a map | index.md:173-176 |
| 5 | docs/index.md | anchor wasm/src/ffi.rs:7 (R5) | no symbol token on line 7 | index.md:194 |
| 6 | docs/refusals.md | anchors sql_ast.rs:284, 284-295, 304-311 | no symbol token; drift | refusals.md:78 |
| 7 | docs/refusals.md | anchor refusals.rs:150, 150-165 | no symbol token | refusals.md:150 |
| 8 | docs/refusals.md | anchor abi-reference.md#13 | no matching symbol line | refusals.md:253 |
| 9 | docs/refusals.md | GRAPHLAW-RUN-EH-REQUIRED cites src/bin/graphlaw_run.rs | file gone; wasmi runner retired 2026-10-24 (ADR); only graphlaw_run_eh.rs | refusals.md:206 |
| 10 | docs/refusals.md | cites examples/l85_refusal_probe.rs ("renamed from") | absent; confirm new name | refusals.md:19 |
| 11 | design-read-frame-bound.md | cites src/bin/graphlaw_run.rs | removed | line 9 |
| 12 | CONTRIBUTING.md | tests/docs_release_claims.rs court | file does not exist | CONTRIBUTING.md:34 |
| 13 | capability-registry.md | scripts/vendor_registry.sh | not in scripts/ | line 156 |
| 14 | docs/WASM.md | purrdf fork `src/reasoner.rs` | lives in the fork; bare path reads as local | line 201 |
| 15 | docs/WASM.md | jira/v26.10.21/WASM-SANDBOX-SPEC.md | absent, says "not vendored here"; acceptable | line 132 |
| 16 | typescript-api.md | src/generated/artifact-meta.ts, src/runtime.ts, etc. | no top-level src/*.ts; probably js/npm/src; prefix paths | lines 16,33-34,89,101 |
| 17 | wasm-gap-register.md | docs/wasm-duckdb-integration.md, jira/v26.10.23/ROADMAP.md | both absent | lines 4,27,48,180 |
| 18 | design-sniff-fixes.md | jira/v26.10.24/ROADMAP-v26.10.24.md | absent | line 13 |
| 19 | jira/v26.10.28/RECEIPTS.md | 24 count/digest claims without receipts/ pointer (R4) | gate fails | lines 65-66,106-110,170,181-185,221,223,226-227,230,422-423,474,551-552 |
| 20 | README.md | publish state "verified 2026-11-09": npm 26.10.14 | 8 days old | README.md:98-102 |
| 21 | CHANGELOG.md | [Unreleased] "refreshed to the 26.10.28 tree" | confirm dated 26.10.28 section | ~line 14 |

## Not stale (do not re-file)
Rust 1.98.0, frozen-duckdb 2.0.2, purrdf rev pin (Cargo.toml:50-51), 15 dispatched ops, README/CLAUDE fenced commands (R1 clean). Cross-repo paths (smalt, wasm4pm) are external by design.

## Fixes by priority
1. Rows 1-2: bump active-window pointer to v26.11.17; reconcile row 3.
2. Row 12: write the court or delete CONTRIBUTING.md:34-36.
3. Rows 9, 11, 13, 16, 17: repoint or drop.
4. Rows 5-8, 19: refresh anchors / add receipts pointers to clear the gate.
