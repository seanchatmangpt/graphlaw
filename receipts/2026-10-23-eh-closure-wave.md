# EH-closure wave receipt (2026-10-23)

Witnessed integration receipt for the DFLSS-GL EH-closure wave. Every value
below traces to a witnessed command output recorded in
`docs/jira/v26.10.22/RECEIPTS.md:1169-1340` (waves W1–W53) or to the staged
wave record; no placeholders survive. This file replaces the earlier
draft, which was moved verbatim to
`docs/jira/v26.10.22/designs/eh-closure-draft.md` after the chain re-baseline
(see `receipts/2026-10-23-chain-rebaseline-note.md`).

- Date: 2026-10-23 (local)
- Subject: `/Users/sac/graphlaw` @ main, working tree dirty (wave in flight).
- Wave ledger: `docs/jira/v26.10.22/RECEIPTS.md:1169-1340`.

## 1. Staging rebuilds + opcode EH gate

- frozen-duckdb archive rebuilt with `-fwasm-exceptions -mllvm -wasm-use-legacy-eh=0`
  (duckdb pin `b6013f7aa2`); staging verify **exit 0**;
  `libduckdb.a` **88,161,726 B** (wasm32 magic);
  `libcore_functions_extension.a` **16,389,152 B**;
  `libicu_extension.a` **3,312,408 B** (W1).
- Opcode EH gate on the built artifact: **try_table=99, legacy=0** —
  **PASS** (uniform new-EH encoding; legacy `try`/`delegate` absent; also the
  exoneration arm for the EH-flavor-mixing hypothesis, W30/W53).
- wasmtime 48.0.1 new-EH hello-world probe: output `"EH-OK probe"`,
  **exit 0** (W2). The hosting wall is closed.
- Env law (local wasm builds): `/opt/wasi-sdk` does not exist on this host;
  local builds use
  `WASI_SDK_PATH=/Users/sac/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1`
  (wave 2 addendum).

## 2. Manifest transitions + gate removals (W3, W4)

- `Cargo.toml`: frozen-duckdb unconditional dependency; `runner-eh` feature
  (wasmtime/wasmtime-wasi 48); `graphlaw-run-eh` bin.
- Gate removals in `src/lib.rs` + `src/abi.rs` (both-target `cargo check`
  exit 0); registry `abi::ops::meta` ungated, exit 0.

## 3. libunwind candidate B integrated

- Candidate B (new-EH libunwind variant, EH-OK under wasmtime+node in the
  W30 probe) integrated into staging by the coordinator re-run
  (`--force` new-EH rebuild + candidate-B libunwind integration + verify);
  staging digest prefix **`6a5f79f7`** (wave 2 addendum, W31/W30 record).
- Staging incident: canonical staging wiped twice (~01:34, ~02:11); incident
  receipt `receipts/2026-10-23-staging-wipe-incident.md`; law: all lanes
  verify staging before building and NEVER mutate `~/.frozen-duckdb` (W31).

## 4. Manifest transitions, gate removals, op promotion, session surface, document sequences (courts)

All counts are real test-runner output as witnessed in the ledger rows cited.

| Court | Result | Ledger row |
|---|---|---|
| MERGE mutating classification | 12 passed / 0 failed (+ anti-vacuity flip) | W5 |
| sql_rdf_dialect | 34/0 then 36/0 | W6, W18/W38 |
| registry dispatch / `registry_dispatch` | 7/0; 14/0 | W7 |
| SHACL WHERE shapes (`ggen sync run`) + `ast_governance` | green | W6 |
| G6 ATTACH allowlist | 13/0 | W15 |
| wired-op court | landed (413-line rewrite) | W16 |
| registry ascii em-dash | 1/0 | W17 |
| `REFUSAL-05` pins | 36/0 | W18 |
| conformance arm registration | 8/0 registered | W19, W44 |
| session ABI | 9/9, 0 failed | W20 |
| host-side | 8/0 | W21 |
| memory wall | 8/0 | W22 |
| EH-class preflight | exit 96 (typed refusal end to end) | W24 |
| duckdb_op | 16/0 | W46 |
| duckdb_eval | 18/0 | W48 |
| sql_ast | 32/0 | W48 (post-purge 25/0 at W37) |
| chicago_validation | 12/0 | W37 |
| dialect | 36/0 | W36 |
| statement_kinds | 8/0 | wave record |
| query_cache | 9/0 | W47 |
| refusal_crosswalk_differential | 6/0 | W45 |
| boundary fuzz (wasmtime port) | 1172 hostile frames / 0 traps | W35 |
| refusal family | 75/0 (conformance 8, laundering 23, k5 8, dialect 36) | W36 |
| npm | 198/198 | W51 |
| npm dual-class re-band | 4/0 | W29 |
| js pack-smoke | PASS | W28 |
| pm4pytest conformance legs | 3/0 | wave record |
| pm4pytest fitness/precision | fitness 1.0; precision 1.0 (non-vacuity: unmodeled side-effect → fitness 1.0, precision 0.8 violation witnessed at W40) | W40 |

- Op promotion + session surface + document-sequence wire surface verified
  end to end: duckdb_op 16/0, duckdb_eval 18/0, sql_ast 32/0 (W48);
  session ABI 9/9 (W20); docs registered (W49).
- G1 DECIMAL 3 courts + G4 multi-column 6 courts folded into the sql_ast
  total (W39).

## 5. REFUSAL-05 + G5 arms registered

- `REFUSAL-05` register row landed (W7) and pins re-landed 36/0 (W18);
  refusal-register rows 119-121 appended to `docs/refusals.md`, append-only,
  no renumbering (W25).
- G5 arms `MIXED_DOCUMENT_ROOTS` / `INDEX_SEQUENCE_DISORDER` /
  `SEQUENCE_EMPTY` registered at `docs/refusals.md:96-110`;
  **PINNED_GAPS → REGISTERED_ARMS** flip witnessed (W44).

## 6. Docs / receipt chain

- Docs registered (W26, W49): how-to run-the-wasm-module, DIALECTS session +
  document-sequence sections, SQLRDF-21/22/23/24 progress, refusals rows
  96-110 + 119-121 + 124, index, CHANGELOG/MIGRATION.
- Receipt chain re-baselined and VERIFIED: **773 receipts**, head
  **`d4447281…`** (W52); chain head as of this update:
  `chain_head 718063bf1ba6926360fc216aa1df16a5d0b1417571a02cf5a7d166427e8d9d97`
  (`receipts/CHAIN.blake3:905`); rebaseline note receipted
  (`receipts/2026-10-23-chain-rebaseline-note.md`). `receipt_chain_sync` is
  NOT run by this lane — the coordinator runs it at integration.

## 7. Built artifact (witnessed values; canonical re-pin landed — see §8)

- Artifact blake3 (BUILT artifact,
  `target/wasm32-wasip1/wasm/graphlaw_wasm.wasm`):
  **`259cf5b0650c94f310e516fbccefabed5896b6a1872d915a3273729f427ed7c9`**.
- Artifact size: **46,069,809 B**.
- Tag(13) section count (inline section walk, 2026-10-23): **1** (> 0 — the
  DuckDB engine did link).
- wasmtime 48.0.1 `compile` validation of the built artifact: **exit 0**.
- Pre-pin baseline being replaced: `332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c`,
  7,104,147 B.
- **Re-pin HELD (superseded)**: at the time of writing the re-pin was held;
  it has since LANDED on the closure wave — final canonical pin values in
  §8. W9/W27 re-pin held (wave 2 addendum 2) is likewise superseded.

## 8. Prior OPEN rows — resolution (2026-10-23 closure wave)

- **dtor-runner trap (SQLRDF-24): CONFIRMED + FIXED.** Root cause witnessed
  as two defects, both fixed: (1) musl atexit walker fn-read offsets +
  `slot` re-attach — `atexit.c.obj` replaced with the current wasi-libc
  member (finished_atexit present, sha256 prefix `a1874498…`);
  (2) reactor-contract completion in `wasm/src/lib.rs`: static extension
  registration via `.init_array` constructors + atexit-family no-op
  overrides (destructors never run mid-module-life). Size delta explained:
  statically-linked core_functions/icu extension code + wave source delta
  under LTO (libc swap −106 B measured).
- **Re-pin: DONE.** Final canonical pin
  `priv/graphlaw.wasm` blake3
  **`fa1558306690493249b176ec312a4f01e97798937e05cd49f2946d61a7e6d107`**,
  **57,781,384 B** (b3sum re-verified on disk at this update), Tag(13)×1,
  wasm_audit PASS, registry `--check` exit 0, trap table clean
  (capabilities/duckdb open/close exit 0). Replaces pre-pin baseline
  `332ea2188faf…` (7,104,147 B).
- **Qualification witnessed**: lifecycle courts **3/3 GREEN** (full
  in-wasm CREATE→INSERT→SUM→UPDATE + REFUSAL-05); battery 9/10 suites
  green (fuzz 2/0, gws12 2/0, n3 3/0, memory wall 8/0 re-lawed
  per-cycle-bounded Δ≤2, wasm_abi 25/0, wired 4/0, eh exit-code 7/0);
  FIBO final run vs final pin in flight at receipt time.
- **wasm-trapped courts** (`wasm_abi`/gws4-style, W43): **GREEN** per
  witnessed rows above.
- **pm4pytest wasm legs**: **UNBLOCKED** — pending final rerun on the
  final pin (host conformance legs 3/0 are witnessed above).
- **W10/W11/W12/W13/W14**: open on their own ledger rows; W14 unblocks at
  next coordinator commit.

## 9. Standing

GREEN rows CONFORMANT-ON-OBSERVED-OUTPUT (courts in §4, §5); built-artifact
values in §7 are witnessed on the target build; the canonical pin standing
lives in §8 (final pin values, b3sum-verified on disk). Residual OPEN: the
FIBO final run vs final pin and the pm4pytest wasm-leg final rerun are
in flight at receipt time; W10–W14 remain open on their own ledger rows.
The coordinator runs
`receipt_chain_sync` and commits at integration; no receipt, no flip.
