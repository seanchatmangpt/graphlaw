# graphlaw

Rust law-admission kernel: RDF/SHACL graphs admit through one executable law
(`src/law.rs`), project to SQL/DuckDB (`src/sql_ast.rs`, `src/duckdb_eval.rs`),
sign receipts (Ed25519, `src/attest.rs`), and run the same artifact on host and
wasm (`wasm/`, `graphlaw-run-eh` wasmtime packet-4 stdio runner).

## Agents

Never dispatch the built-in generic agents (`general-purpose`, `claude`).
Only use agents defined in `.claude/agents/` plus read-only Explore/Plan.
Subagent prompts always carry the absolute repo path (`/Users/sac/graphlaw`)
and a lane file-ownership list.

## Commands

```bash
cargo check                                     # fastest compile gate
cargo test                                      # full host suite (default feature: abi; DuckDB is a required dependency)
cargo test --test chicago_validation            # Chicago receipt/execution courts
cargo test --test sql_rdf_dialect               # SQL-in-RDF compile + admission courts
cargo test --test duckdb_statement_kinds        # 35 PEG statement kinds on real frozen-duckdb
cargo test --test trust_attest                  # Ed25519 attestation courts
bash scripts/mutation_court.sh                  # anti-vacuity: gates must flip on mutation
bash scripts/spc_court.sh                       # SPC gate: Cpk ≥ 1.33 on the receipt fold path (P28-SIL program)
# WASI_SDK_PATH (/opt/wasi-sdk) is a wasi-sdk root (bin/clang, share/cmake/wasi-sdk-p1.cmake);
# CI installs wasi-sdk 34 there.
# ~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1 is the engine archive, not a wasi-sdk.
WASI_SDK_PATH="/opt/wasi-sdk" \
cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm   # wasm boundary (env required; typed build.rs refusal without it)
WASI_SDK_PATH="/opt/wasi-sdk" \
cargo test -p graphlaw-wasm                     # host contract around wasm/src/lib.rs
./target/debug/graphlaw-verify                  # receipt verify CLI
```

### Test ladder

Cheapest rung that answers the question; escalate only on failure. After
fixing a failure, rerun the boundary rung first, then expand — never rerun an
unchanged failing command without a new hypothesis.

1. `cargo check` — compile gate (~fast)
2. `cargo test --test <target>` — single court file
3. `cargo test` — full host suite
4. `bash scripts/mutation_court.sh` — anti-vacuity sweep
5. `bash scripts/spc_court.sh` — SPC gate: Cpk ≥ 1.33 on the receipt fold path
6. `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` + `cargo test -p graphlaw-wasm` — wasm boundary courts

Exit discipline: a loud SKIP with a typed reason is not a pass; a gate that
cannot exit 1 is the failure.

## Layout

- `src/` — law kernel: `law.rs` (admission), `sql_ast.rs` (sql-rdf compiler),
  `duckdb_eval.rs` (frozen-duckdb eval + receipts), `attest.rs` (Ed25519),
  `tier.rs` (admission tiers), `abi.rs` (op ABI, feature `abi`), `dialect.rs`
- `ontologies/` — canonical graphs + SHACL shapes (`sql-ast.ttl`); never
  hand-edit generated projections (see Generated artifacts)
- `ggen/` — dialect packs (`sql-dialect/`: statement classes, SQL statement
  kinds, extensions)
- `tests/` — courts; `chicago_validation.rs`, `sql_rdf_dialect.rs`,
  `duckdb_statement_kinds.rs`, `trust_attest.rs` are the sql-rdf acceptance set
- `wasm/` — `graphlaw-wasm` is the unit. It links this crate, including
  frozen-duckdb. SQL-in-RDF admits and evaluates inside that module.
- `crates/graphlaw-eyeron/` — workspace-member crate (see root `Cargo.toml`)
- `docs/refusals.md` — the typed refusal register (REFUSAL-01..04 etc.),
  every refusal has a stable code + trigger + recovery; flips are receipt-gated
- `docs/jira/v<version>/` — active milestone window (newest dir); tickets,
  receipts, INDEX.md
- `receipts/` — durable receipts (per-file blake3 digests; inter-receipt chaining
  is a pending decision — the chain rule lives in `src/receipt_chain.rs` for
  in-memory OCEL frames, not yet serialized into `receipts/`)

## Generated artifacts (never hand-edit)

| Artifact | Regenerate |
|---|---|
| `registry/ARTIFACTS.blake3`, `registry/capability-registry.json` | registry sync flow (`src/registry.rs` / `graphlaw-registry` bin) |
| conformance vectors | `scripts/gen_w3c_manifest.sh` |
| generated docs under `docs/reference-generated/` | their generators, never patch |

## Conventions

- **DuckDB is not a feature**: frozen-duckdb is a normal dependency; never add a
  `duckdb` feature or `#[cfg(target_arch)]`-gate the SQL surface out of any
  target — graphlaw-wasm is one unit and must typecheck AND link it.
  Refusal codes are append-only (`docs/refusals.md`); never renumber.
- **Disk discipline**: check `df -h /` (≥20 GiB free) before full suites/benches;
  lane `CARGO_TARGET_DIR=target-lane*` roots are leases — the coordinator deletes
  every lane's build root at integration. ENOSPC has killed all-lane waves.
- **Coordinator-owned files**: `Cargo.toml` and git transitions are
  coordinator-only; lanes never edit the manifest (concurrent edits revert
  each other — witnessed 3×).
- **Mutation court runs solo**: `scripts/mutation_court.sh` mutates the tree in
  place; serialize it against all lanes or its inversions false-red concurrent
  lanes' compiles.
- **Chicago (classicist) testing**: real frozen-duckdb connections
  (`GraphLawDuckDB::open_in_memory[_allowing_unsigned_extensions]`), assert on
  real table readbacks; mocks/stubs banned by default. Anti-vacuity: a court
  must flip under the operator inversion/mutation, or it is FAIL.
- **No-sleep**: no `Process.sleep`/`std::thread::sleep` as coordination; poll
  observables or use structural sync. A sleep is a defect.
- **Receipts**: embed REAL command output (exit code, counts, hashes). A
  receipt with no embedded output is not a receipt. Receipt hash for sql-rdf
  is BLAKE3(Canon(G_AST) || P_params) via `purrdf::try_canonicalize`.
- **Refusals**: typed and stable-coded (`docs/refusals.md`). Never crash or
  raise a generic DB error where a typed refusal belongs.
- **Lanes**: one canonical checkout, no worktrees/clones. Lane file ownership
  is disjoint and stated in the dispatch prompt; coordinator owns git.
  Lane build isolation: `CARGO_TARGET_DIR=target-lane<N>`. No `git stash` in
  shared checkouts — snapshot via `git show HEAD:path > /tmp/...`. Compile-freeze
  SLA: fix or revert a shared-compile break within ~10 min; minimal unblock
  fixes disclosed in both lanes' reports.
- **Commits**: `git commit -F <file>`, never inline `-m`.
- **Never** rebase, force-push, or `git reset --hard`; merge only, fix forward.
- **Generated files** are regenerated, never patched by hand.
- **Conformance text**: the sql-rdf Recommendation lives at
  `docs/reference/sql-rdf-rec.md` — cite by section; its W3C status claims are
  document text, not facts asserted by this repo.

## Lithography

Loaded again on session start by `.claude/hooks/lithography_card.py`.

- A pinned axiom is re-pinned only with a receipt. A generated projection
  (`docs/reference-generated/`, `registry/ARTIFACTS.blake3`,
  `registry/capability-registry.json`, `registry/capability-registry.ttl`)
  is regenerated, never patched. A leaf edit names one owned path and one court.
- Every dispatch carries `/Users/sac/graphlaw` and a disjoint file lease.
- Tool refusals: `GENERIC_AGENT_REFUSED`, `REFUSED(topology)`,
  `MANIFEST-REFUSED`, `DISK-REFUSED`,
  `LITHO-REFUSED(generated|registry-artifact|pin-file|staging|unreadable)`.
  Pin files are `courts/code-partition/axioms.blake3`,
  `axioms.blake3.next`, and `manifest.json`. A path containing `_split/`
  is staging. File tools refuse those; Bash remains the coordinator hatch.
- Cheapest court rung first. A loud SKIP is not a pass. A deny is not a
  receipt. Do not rephrase a refused tool into a second attempt.
- `court-runner` stays read-only. Axiom `.rs` sources stay editable: the
  partition manifest is not a live allow-list.
