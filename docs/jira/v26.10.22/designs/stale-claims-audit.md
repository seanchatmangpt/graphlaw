# Stale-claims audit — v26.10.22 (skeptic lane, read-only)

Date: 2026-10-22. The REFUSAL-04 trigger, the `feature duckdb` sentences, the 25 MiB ceiling, and the CLAUDE.md default-features comment were corrected later the same day. `registry/capability-registry.json` and `.ttl` still carry the old DuckDb field sentence until `graphlaw-registry` is regenerated (no build was run for that). Scope of the original pass: stale doc claims, dead architecture references,
unverifiable assertions, wrong-premise tests. Read-only on code; this report is
the only write. Concurrent lanes are editing this tree — every finding carries
a lane-in-flight note where applicable.

---

## CONFIRMED-STALE

### C1. `docs/wasm-integration.md` — artifact pin section describes a dead source state

- **File:line:** `docs/wasm-integration.md:107-131`
- **Stale text:** "digest `cbc80f07346f025d4e1cd001f21d15474928cb055d4b1a9c0ac7c31c5938dc6a` … 6,654,496 bytes … Minted at the **26.10.13** source state … `wasm/Cargo.toml` reads `version = "26.10.13"`" (historical)
- **Correction:** The live pin is `332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c`, 6,793,617 B, `pin-source=26.10.21`; `wasm/Cargo.toml` reads `version = "26.10.21"`.
- **Evidence:** `cat registry/ARTIFACTS.blake3` → `wasm 332ea218…2c30c 6793617 graphlaw_wasm.wasm`, header `pin-source=26.10.21`; `grep '^version' wasm/Cargo.toml` → `26.10.21`; `ls -l priv/graphlaw.wasm` → 6,793,617 B Oct 22 12:56; `b3sum priv/graphlaw.wasm` → `332ea218…2c30c` (matches pin).
- Note: `docs/WASM.md` step 7 of the upgrade checklist already names this file's
  pin section as a pending update; still stale as read.

### C2. `wasm/Cargo.toml` header comment — cites superseded digest and size

- **File:line:** `wasm/Cargo.toml:3`
- **Stale text:** "The digest pins in registry/ARTIFACTS.blake3 (cbc80f07…, 6,654,496 B) were minted at the 26.10.21 source state"
- **Correction:** The pins on disk are `332ea218…`, 6,793,617 B (both minted at
  the 26.10.21 source state per the ARTIFACTS header — the digest/size quoted
  in the comment are the old cbc80f07/6,654,496 values).
- **Evidence:** same as C1: `b3sum priv/graphlaw.wasm` → `332ea218…2c30c`; `ls -l priv/graphlaw.wasm` → 6,793,617 B.

### C3. README — artifact size "6,654,496 bytes" and "REFUSAL-04 … engine executes host-side"

- **File:line:** `README.md:67`
- **Stale text:** "The core `graphlaw.wasm` artifact is **~6.3 MiB** (6,654,496 bytes, pinned by `registry/ARTIFACTS.blake3`) … (The DuckDB engine executes host-side; the wasm boundary refuses SQL by typed `REFUSAL-04` capability refusal, not by embedding the engine.)"
- **Correction (both halves):**
  1. The pinned artifact is 6,793,617 B (~6.5 MiB); 6,654,496 is the
     superseded cbc80f07-era size.
  2. The current design is the opposite of "executes host-side":
     `contracts/abi-manifest.json` notes: "op `duckdb` (op-15) is dispatched by
     the wasm module itself … frozen-duckdb does not compile to wasm32, so the
     host op table stays the 14-op set"; `docs/WASM.md:24-27`: "frozen-duckdb
     is a normal dependency … dispatched by the wasm module itself … compiled
     into the same module"; `src/registry.rs:889-899` `NATIVE_ONLY_OPS` comment:
     "the wasm module dispatches the duckdb op itself through the compiled-in
     frozen-duckdb engine". REFUSAL-04 is a different thing entirely
     (`docs/refusals.md:45`: non-document-backed builtin SQL-eval refusal; the
     wasm module admits and compiles SQL-in-RDF through `compile_turtle`).
- **Evidence:** `python3 -c` read of `contracts/abi-manifest.json` `notes`
  (op-15 note quoted above); `b3sum priv/graphlaw.wasm` (6,793,617 B);
  `docs/WASM.md:22-27`; `docs/refusals.md:45`.

### C4. Dead `duckdb` feature references (feature no longer exists)

- **Sites (all quoting "feature `duckdb`"):**
  - `docs/refusals.md:25` — "The embedded DuckDB substrate (ABI op `duckdb`,
    feature `duckdb`, native-only; …)"
  - `docs/refusals.md:72` — "`DuckDb` (the native-only embedded DuckDB
    substrate failed; op `duckdb`, feature `duckdb`)."
  - `src/registry.rs:794` — "Diagnostic text from the embedded DuckDB substrate
    (op `duckdb`, feature `duckdb`, native-only)."
  - `src/dialect.rs:144-145` — "The embedded DuckDB execution substrate (ABI op
    `duckdb`, feature `duckdb`, native-only) failed …"
  - `tests/registry_dispatch.rs:19` — "the registered native-only
    (feature-gated, compiled …"
- **Correction:** `[features]` in root `Cargo.toml` is exactly
  `abi, default, evidentiary-ring, pack-tools, runner, wasi-patched-deps` — no
  `duckdb` feature; frozen-duckdb is an unconditional dependency; zero
  `cfg(feature = "duckdb")` arms exist in `src/`, `wasm/`, `tests/`.
- **Evidence:** `cargo metadata --no-deps` feature list (above);
  `grep -rn 'cfg(feature = "duckdb")' src/ wasm/ tests/` → 0 hits;
  `cargo check --features duckdb` → `error: the package 'graphlaw' does not
  contain this feature: duckdb`.

### C5. CLAUDE.md — "(abi+duckdb default features)"

- **File:line:** `/Users/sac/graphlaw/CLAUDE.md:19`
- **Stale text:** "cargo test  # full host suite (abi+duckdb default features)"
- **Correction:** default features = `["abi"]` only; there is no duckdb feature
  (see C4).
- **Evidence:** `cargo metadata --no-deps` → `features: ['abi', 'default',
  'evidentiary-ring', 'pack-tools', 'runner', 'wasi-patched-deps']`.

### C6. `docs/jira/v26.10.22/SQLRDF-08.md` — Current-behavior section contradicts the tree

- **File:line:** `docs/jira/v26.10.22/SQLRDF-08.md:15` (Current behavior)
- **Stale text:** "…depends on `graphlaw` with `default-features = false` …
  The `duckdb` feature is off. [`src/lib.rs`] lines 191–199 compile
  `duckdb_eval` and `duckdb_op` only under `feature = "duckdb"` …"
- **Correction:** `wasm/Cargo.toml:19` has no `default-features = false`;
  `src/lib.rs:211-214` compiles `duckdb_eval`/`duckdb_op` unconditionally
  ("Linked into the wasm module with the rest of the kernel"); no `duckdb`
  feature exists (C4). The INDEX.md row says the gap is "closed by the
  capability refusal in wasm/src/lib.rs, courted by the wasm refusal courts
  (7/7)" — but the wasm-crate test count is now 9 (C7) and `wasm/src/lib.rs`
  carries no refusal code; REFUSAL-04 lives in `src/rules/builtins.rs:89`
  (SQLRDF-17 wording), not in the wasm unit.
- **Evidence:** read of `wasm/Cargo.toml`, `src/lib.rs:211-214`;
  `grep -rn 'REFUSAL-04' src/ wasm/` → only `src/sql_ast.rs`,
  `src/tier.rs:25`, `src/rules/builtins.rs:89`, `docs/refusals.md:45`.

### C7. "wasm courts 7/7" — count is stale; actual is 9

- **Sites:**
  - `docs/jira/v26.10.22/INDEX.md:33` (SQLRDF-08 gap column)
  - `docs/jira/v26.10.22/CHARTER-DFLSS.md:63` and `:106` ("wasm 7/7")
  - `docs/jira/v26.10.22/DFLSS-DOSSIER.md:128` ("wasm refusal courts 7/7
    (`cargo test -p graphlaw-wasm`)") and `:161`
- **Correction:** `cargo test -p graphlaw-wasm` (lane-isolated
  `CARGO_TARGET_DIR=target-lane-skeptic`) → `test result: ok. 9 passed;
  0 failed; 0 ignored` (exit 0). The court family grew to 9 after the 7/7 was
  written; the claim should read 9/9 or cite the court list.
- **Evidence:** command output above; `grep -c '#\[test\]' wasm/src/*.rs` →
  lib.rs 4 + ffi.rs 5 = 9.

### C8. CHARTER-DFLSS §3 — sql_ast recursion-guard line pointers moved

- **File:line:** `docs/jira/v26.10.22/CHARTER-DFLSS.md:69-71`
- **Stale text:** "RecursionLimitExceeded(32) at `src/sql_ast.rs:719-720`;
  serialize_expr MAX_DEPTH(64) at `src/sql_ast.rs:573-574`"
- **Correction:** MAX_DEPTH(64) const is now `src/sql_ast.rs:953`, depth check
  `:1068-1069`; the `RecursionLimitExceeded(32)` checks are at
  `src/sql_ast.rs:1239` and `:2694`. Lines 573-574/719-720 now land in
  unrelated code (token enum / match arm).
- **Evidence:** `grep -n 'MAX_DEPTH\|RecursionLimitExceeded' src/sql_ast.rs`;
  `sed -n '570,576p;715,724p' src/sql_ast.rs` (shows `_ => Self::from_token(iri)`
  at 719-720).

### C9. `registry/ARTIFACTS.blake3` — capability-registry.json digest pin does not match the file on disk

- **File:line:** `registry/ARTIFACTS.blake3` (registry row)
- **Stale text:** "registry blake3:7db561aa315e86f3f21022697276ef3382b808ea1bcef4e4245159b755f819b4 - capability-registry.json" (historical)
- **Correction:** `b3sum registry/capability-registry.json` →
  `5d31d47270ddb7758e86e3223eebd91cd4d59b5dedcee17f875f8660ba97dff3`. The pin (historical)
  is behind the file.
- **Evidence:** `b3sum registry/capability-registry.json` (above). **Lane in
  flight:** both `src/registry.rs` and `registry/capability-registry.json` are
  git-modified in the working tree; a registry sync/re-pin is presumably
  pending. Marked CONFIRMED on disk-state, with lane-in-flight note.

---

## PLAUSIBLE-STALE

### P1. `docs/WASM.md` — "the LINK is red" (pending Lane 1)

- **File:line:** `docs/WASM.md:29-36` and `:73-78`
- **Stale text:** "**Current build state: the typecheck is green and the LINK
  is red.** … the final link fails with `rust-lld: unable to find library
  -lduckdb` pending the Lane 1 link fix." / "the wasm32-wasip1 link currently
  red … the pins must be treated as historical until a real build succeeds"
- **Correction (probable):** `docs/jira/v26.10.22/RECEIPTS.md:58` (Successor
  §2) records the boundary build "re-executed green lane-isolated … wasm
  profile 4m25s" with the artifact re-pinned (`332ea218`, verified by
  `b3sum` against the on-disk `priv/graphlaw.wasm`, mtime Oct 22 12:56). A
  matching-digest artifact on disk contradicts a "link is red" present tense.
- **Why not CONFIRMED:** my own verification build in the shared
  `target/` aborted with a concurrent-lane artifact collision
  (`error: failed to build archive … No such file or directory` — not a link
  error), so I could not re-witness the link color firsthand. The file is
  git-modified; lane in flight.
- **Evidence:** RECEIPTS.md:58, :118-126; `b3sum priv/graphlaw.wasm` matches
  pin; my build log (collision abort, exit nonzero output above).

### P2. `docs/jira/v26.10.22/INDEX.md:42` / `SQLRDF-17.md` — gap prose describes the pre-fix state while the fix has landed

- **File:line:** `docs/jira/v26.10.22/INDEX.md:42`
- **Stale text:** "`src/duckdb_op.rs:198-219` accepts `ast.text` as raw SQL
  (legacy `(None,None)` hash arm at `src/duckdb_eval.rs:183-188`)"
- **Correction (probable):** the code now refuses and the legacy arm is
  deleted: `src/duckdb_op.rs:211-218` — "SQLRDF-17 refused: legacy `ast.text`
  raw SQL is not an admitted admission path"; `src/duckdb_eval.rs:526` — "the
  legacy … hash arm and …" deleted ("the legacy fallback arm is deleted, not
  gated", `src/duckdb_eval.rs:1118`). The INDEX row still presents acceptance
  of raw SQL as current behavior.
- **Why not CONFIRMED:** INDEX Status column reads "Open (follow-up,
  BLOCKED-BY: v26.10.22 integration commit)" and RECEIPTS.md records the
  integration commit as coordinator-owned/pending — the prose may be
  intentionally pre-integration. Lane in flight.
- **Evidence:** greps of `src/duckdb_op.rs` / `src/duckdb_eval.rs` quoted
  above.

### P3. `tests/wasm_duckdb_op.rs:4-8` header comment — "the wasm link is pending (rust-lld -lduckdb)"

- **File:line:** `tests/wasm_duckdb_op.rs:4-8`
- **Stale text:** "The wasm link is pending (rust-lld -lduckdb), so this court
  exercises the op on the native ABI pipeline"
- **Correction (probable):** same as P1 — the docs lane records a green
  lane-isolated boundary build and re-pinned artifact; this comment is the
  same "link is red" snapshot.
- **Why not CONFIRMED:** link color not firsthand-witnessed by this lane
  (shared-target collision); test file may be mid-edit by its owning lane.
- **Evidence:** RECEIPTS.md:58; artifact digest match (C1 evidence).

### P4. `docs/wasm-duckdb-op15-design.md:7` — "The 14-op wasm ABI has no duckdb/sql op"

- **File:line:** `docs/wasm-duckdb-op15-design.md:7`
- **Stale text:** "The 14-op wasm ABI has **no duckdb/sql op**;
  `gl:evalSql` … is native-only via frozen-duckdb"
- **Correction (probable):** superseded by the op15 landing: the manifest
  notes say op `duckdb` "is dispatched by the wasm module itself" (op-15);
  `docs/wasm-duckdb-integration.md` and `tests/wasm_duckdb_op.rs` treat the op
  as real and courted.
- **Why not CONFIRMED:** the design doc may be an intentionally historical
  design record (pre-landing); it is not marked historical, so it cannot be
  skipped under the rules.
- **Evidence:** `contracts/abi-manifest.json` notes; `tests/wasm_duckdb_op.rs`
  header.

### P5. `docs/jira/v26.10.22/DFLSS-DOSSIER.md:161` — "no `duckdb` feature (`wasm/Cargo.toml:19`)" listed PENDING-LANE

- **File:line:** `docs/jira/v26.10.22/DFLSS-DOSSIER.md:161`
- **Stale text:** "wasm boundary | capability refusal | wasm courts 7/7 | no
  `duckdb` feature (`wasm/Cargo.toml:19`) | PENDING-LANE"
- **Correction (probable):** court count is 9/9 (C7) and the row's remaining
  open item cites a feature model that no longer exists in the form implied.
- **Why not CONFIRMED:** the row is self-marked PENDING-LANE — it discloses
  its own currency.
- **Evidence:** C7 output; wasm/Cargo.toml read.

### P6. Registry digest/surface pins vs `graphlaw-registry` regeneration (C9 continuation)

If the in-flight registry lane re-syncs, the `receipt-surface` row
(`blake3:78ebb149…` over `src/duckdb_eval.rs` + `src/sql_ast.rs`) must be
re-verified against the post-integration files; both files are git-modified.
UNVERIFIED whether the current receipt-surface pin matches. **Evidence
command:** `cargo run --features abi --bin graphlaw-registry -- --print-digests` (not run by this lane — cargo lock/target contention with in-flight lanes; running it would build against a moving tree).

---

## UNVERIFIED (could not check; not silently dropped)

- **U1. `README.md:39` "drives all 14 ops" (`cargo test --test wasm_abi`)** —
  could not run: shared target dir is being thrashed by concurrent lanes
  (dependency `.d`/rlib writes colliding, see P1 evidence). Not contradicted:
  `contracts/abi-manifest.json` hashed OPS list is exactly 14.
- **U2. `README.md:22-34` example receipts** (`quickstart_plan` blake3
  prefixes, "2 rows in 3.806ms", query hash `8901beb8…`) — unverifiable in
  this session (target-dir contention); note `tests/readme_first_ten_minutes.rs`
  and `tests/docs_release_claims.rs` exist as courts over these claims, so
  they are receipt-gated rather than free prose.
- **U3. DFLSS-DOSSIER §7 "41 unique = 16 fixed + 6 ticketed-open +
  3 open-by-design + 16 open-unticketed"** — arithmetic verified (16+6+3+16 =
  41); underlying defect ledger not re-counted by this lane.
- **U4. CHARTER §5.2 / `designs/cpk-correction.md` Cpk figures** — the charter
  already REFUTES the dispatch-carried 1.44 and cites the correction doc;
  internal consistency verified, measurements not re-run.
- **U5. `receipts/gws-16-wasm-duckdb-op15.md`** — describes `--features
  duckdb` runs; file is a dated receipt (dated, historical by the rules) —
  skipped as historical, listed here for census completeness.

## DISMISSED

- **D1. `CHANGELOG.md:227` `capability_intake` mention** — changelog entries
  are dated historical records (documents the module's removal); not stale.
- **D2. `Cargo.toml:44` comment block on sha2 transitive pins** — verified
  current against the dependency table; not stale.
- **D3. `src/lib.rs:204` "native-only" on the evidentiary ring** — accurate:
  `evidentiary-ring` is a real, still-existing feature (`Cargo.toml:63`),
  native-only by design.
- **D4. `tests/gws4_ipc_injection_court.rs:88,101` "native-only today"** —
  phrasing marked "today"/conditional; consistent with current op-15 notes;
  borderline, dismissed as not asserting a dead feature gate.

---

## Census

| Class | CONFIRMED-STALE | PLAUSIBLE-STALE | DISMISSED | UNVERIFIED |
|---|---|---|---|---|
| 1. Stale doc claims | C1, C2, C3, C9 | P1, P5 | D1, D2 | U1–U3 |
| 2. Dead architecture refs | C4, C5, C6, C8 | P2, P3, P4 | D3, D4 | U5 |
| 3. Unverifiable assertions | C7 | P6 | — | U2, U4 |
| 4. Wrong-premise tests | — | — | — | — |

Class 4: no wrong-premise test found. The one candidate — the "link is red"
premise embedded in `tests/wasm_duckdb_op.rs`'s native-only exercise (P3) —
is a comment premise, not an assertion the test makes; the test asserts
native-ABI behavior, which is true on its face. Courts that would catch the
stalest claims exist and pass: `tests/abi_manifest_court.rs`,
`tests/docs_release_claims.rs`, `tests/registry_artifacts.rs` (the last is
expected to be red against the C9 pin drift until the registry lane re-pins).

## Commands run (exit codes)

| Command | Exit |
|---|---|
| `cargo metadata --no-deps` (feature list) | 0 |
| `cargo check --features duckdb` | nonzero (error: package 'graphlaw' does not contain this feature: duckdb) |
| `cargo build --locked -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` | abort: concurrent-lane target-dir collision (rlib "No such file or directory"); not a link verdict |
| `CARGO_TARGET_DIR=target-lane-skeptic cargo test -p graphlaw-wasm` | 0 — 9 passed, 0 failed |
| `b3sum priv/graphlaw.wasm` | 0 — `332ea218…2c30c`, 6,793,617 B |
| `b3sum registry/capability-registry.json` | 0 — `5d31d472…` vs pin `7db561aa…` |
| `grep`/`sed`/`cat` over docs, src, wasm, tests, registry, contracts | 0 |
| `rm -rf target-lane-skeptic` (lane build-root cleanup) | 0 |
