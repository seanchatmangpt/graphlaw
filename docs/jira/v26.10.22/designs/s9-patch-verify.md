# S9 patch verify — L172 staged 6-entry surface patch (skeptic, read-only)

Date: 2026-10-22. Subject: /Users/sac/graphlaw @ main 615dbfa (dirty tree).
Ran: test -f, sed, grep, shasum, git diff. Exit 0 on all listed commands.

## Verdicts

### Check 1 — include paths resolve from src/registry/ — PASS
All six resolve (test -f, exit 0 each):
- src/registry/../duckdb_eval.rs            OK
- src/registry/../sql_ast.rs                OK
- src/registry/../sql_ast/ast.rs            OK
- src/registry/../sql_ast/refusals.rs       OK
- src/registry/../sql_ast/serializer.rs     OK
- src/registry/../../ontologies/sql-ast.ttl OK

### Check 2 — 2→6 bump site + loop genericity — PASS
- Bump site confirmed: tests/registry_artifacts.rs:905 (patch doc says
  "~907"; actual is 905 — cosmetic off-by-two, do not chase).
  `assert_eq!(files.len(), 2, "duckdb_eval + sql_ast pinned");`
- Per-file loop (lines ~909-933) is path-generic: reads `f["path"]` from
  disk via CARGO_MANIFEST_DIR.join(path), recomputes blake3, byte count,
  flip-one-byte drift control. No per-file count coupling.
- grep `== 2|len() == ` in tests/registry_artifacts.rs: only lines 248
  (hex-char check, unrelated), 893 (pin.len()==71, unrelated), 905 (the
  bump site). No other 2-assumptions.

### Check 3 — ttl byte-identity invariant — PASS WITH CAVEAT
- include_bytes! embeds a compile-time snapshot; the drift court reads the
  SAME on-disk file, so readback is consistent with whatever bytes compile.
  shasum-256 stable across re-check (3s apart):
  340afb5519dc67974a6f230f1da57a2ff2c38fc5a8d6aff9fb3aabbb24b798ca (historical)
- CAVEAT (session instability, not patch defect): ontologies/sql-ast.ttl is
  git-MODIFIED in the working tree (99 insertions, 1 deletion vs HEAD
  5606594) and mtime is 2026-10-22 17:56:33 — the file changed during this
  session. The 6-entry pin is therefore only reproducible AS OF the bytes
  present at patch application; if the ttl is edited again before the
  quiescent window, ARTIFACTS.blake3 must be regenerated after it (--write
  ordering in patch §4 already enforces this). Coordinator should confirm
  ttl ownership before applying.

### Check 4 — other files.len() assumptions — PASS
- tests/registry_artifacts.rs: only line 905 (the bump site).
- Broad tests/ sweep for len()/files.len() near registry/surface:
  tests/corpus_conformance.rs:109 and tests/wasm_abi.rs:897 are unrelated
  (failure-message count; 50-ttl vector court). query_ledger_court.rs:161
  is a hex-length filter. No other court asserts receipt-surface arity.
  Also no `len() == 6` pre-existing elsewhere that would double-count.

## Patch-doc accuracy spot-checks
- emit_json.rs:74-77 current 2-entry FILES const matches the patch's
  "replace" block byte-for-byte (verified sed 72-80).
- Patch's path-resolution note (../ = src/, ../../ = repo root) matches
  actual include_bytes! semantics from src/registry/emit_json.rs.

## Overall: PATCH VERIFIED, staged apply sound. One operational caveat (Check 3): pin ttl ownership/quiescence before --write.
