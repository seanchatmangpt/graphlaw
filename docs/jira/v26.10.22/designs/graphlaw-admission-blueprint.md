# graphlaw Admission Blueprint — CONSTRUCT-OPERATIONAL-BLUEPRINT

Lane P1, v26.10.22, operator FOND decomposition: the Claude Code PreToolUse
hook is the first grounded increment of the graphlaw ledger contract.
Every tool call is an RDF transition proposal admitted (or refused,
typed, before disk alteration) by ONE executable law.

Status: 3 of 4 FOND subtasks grounded, CONSTRUCT started (this artifact is the
CONSTRUCT output). The PostToolUse receipt hook is registered. Signing still
needs `GRAPHLAW_ATTEST_KEY_SEED_HEX`.

## 1. Tool call → graphlaw mapping

| Concept | Claude Code / graphlaw realization |
|---|---|
| transition proposal | hook input JSON (`tool_name`, `tool_input`) → one-node Turtle `act:ToolCall` |
| admission (μ) | `shacl` ABI op in the compiled wasm module vs the seed policy graph |
| refusal (typed) | `conforms:false` → deny JSON + named rule IRI, exit 1 |
| ledger contract | PostToolUse receipt fold via `graphlaw_receipt_attest.py` |
| law kernel | graphlaw itself; Python is transport only (the hook builds the proposal, never decides) |

The decision surface is the same artifact the repo ships:
`graphlaw-run` (wasmi host) driving `priv/graphlaw.wasm` over packet-4 stdio,
one frame in, one reply frame out. No SHACL logic lives in the hook.

## 2. Seed policy graph — scope and honest limits

Graph: `.claude/policy/graphlaw-seed-shapes.ttl`. Three rule shapes, each a
named `sh:NodeShape` over `act:ToolCall` with one `sh:not` compound condition,
so a violation names its rule IRI (typed refusal):

- `policy:seed#R1ManifestCoordinatorOwned` — file tools on root
  `Cargo.toml`/`deny.toml` (mirrors `.claude/hooks/manifest_guard.py`).
- `policy:seed#R2WasmPinImmutable` — `Edit` on `priv/graphlaw.wasm`.
- `policy:seed#R3ReceiptSurfaceGenerated` — file tools on `registry/*.json`,
  `registry/*.ttl`, `registry/ARTIFACTS.blake3`.

Limits (stated, not hidden):

1. **Three rules, not the full kh: pattern.** Caller identity, lease
   standing, per-lane authority and refusal-code provenance are not modeled;
   the proposal node has no `act:caller` standing yet.
2. **Bash escape hatch** — unchanged from the witnessed guards: Bash
   transitions are not intercepted (the coordinator's scripted-edit path is
   deliberate; a hostile lane could also use it — the Bash-policy increment
   closes this).
3. **`/.claude/` exclusion dropped** vs `manifest_guard.py`: the seed R1 regex
   would refuse a hypothetical `…/.claude/Cargo.toml`; today no such path
   exists, but the divergence is noted rather than papered over.
4. **sh:message not surfaced** by the purrdf report inside `sh:not`; refusal
   reasons currently carry the rule IRI, not the prose message. Sequenced:
   either surface messages in the ABI or key refusals off rule IRIs (done —
   the hook reports the IRI).
5. **Fail-open on infra failure** (runner/wasm/shapes missing), matching
   `manifest_guard.py` precedent; a fail-closed increment needs a staged
   rollout so a missing artifact cannot freeze all lanes.

## 3. Non-vacuity court (standalone, pre-registration) — RECEIPT

Real invocations, real artifact, 2026-10-22 (P1 lane; runner built from
`src/bin/graphlaw_run.rs` @ HEAD 615dbfa, module = pinned
`priv/graphlaw.wasm`):

| payload | exit | decision | reason names |
|---|---|---|---|
| Edit `/Users/sac/graphlaw/Cargo.toml` | 1 | deny | `R1ManifestCoordinatorOwned` |
| Write `Cargo.toml` | 1 | deny | `R1ManifestCoordinatorOwned` |
| Edit `deny.toml` | 1 | deny | `R1ManifestCoordinatorOwned` |
| Edit `priv/graphlaw.wasm` | 1 | deny | `R2WasmPinImmutable` |
| Edit `registry/capability-registry.json` | 1 | deny | `R3ReceiptSurfaceGenerated` |
| MultiEdit `registry/ARTIFACTS.blake3` | 1 | deny | `R3ReceiptSurfaceGenerated` |
| Edit `README.md` | 0 | allow | — |
| Edit `src/law.rs` | 0 | allow | — |
| Edit `.claude/settings.json` | 0 | allow | — |
| Bash (any) | 0 | allow (passthrough) | — |

A court that cannot exit 1 is the failure; this one exits 1 on five refusing
payloads and 0 on four admitting ones.

### 3.1 Correction (lane N29, 2026-10-22)

Pyright flagged possibly-unbound `payload` and `reply` in
`.claude/hooks/graphlaw_admission_guard.py`: `fail_open` ends in `sys.exit`
but was typed `-> None`, so the type checker could not prove the early-return
paths yielded a defined state. Fix: `fail_open` (and `refuse`) are now typed
`-> NoReturn`; no control flow changed. Re-run court (same real invocations,
same runner/artifact): Edit `Cargo.toml` → exit 1 with the
`R1ManifestCoordinatorOwned` deny JSON; Edit `README.md` → exit 0; empty and
garbage stdin → typed INFRA-FAIL fail-open line on stderr, exit 0, no
NameError/traceback. `python3 -m py_compile` and `pyright` both clean
(0 errors, 0 warnings). settings.json registration untouched.

## 4. Sequenced increments (not silently skipped)

1. **PostToolUse attestation** — registered on
   `Edit|Write|MultiEdit|NotebookEdit` as
   `.claude/hooks/graphlaw_receipt_attest.py`. A mutating edit writes a
   transition receipt and folds `receipts/CHAIN.blake3`. Signing still
   requires `GRAPHLAW_ATTEST_KEY_SEED_HEX`; without it the receipt records
   `UNSIGNED-NO-KEY`.
2. **WASM bridge** — replace the host-side `graphlaw-run` subprocess with
   wasmex hosting the same module inside a BEAM runner (composition C14);
   prototype honesty: this prototype routes through the host runner, and the
   bridge is a transport swap, not a semantic one.
3. **WASI sandbox hardening** — run the module under
   `cargo build --target wasm32-wasip1 --profile wasm` re-pin discipline with
   `--expect-blake3` digest verification in the runner (already supported:
   exit 84 on mismatch).
4. **Caller standing** — `act:caller` + lease shapes so rules can distinguish
   coordinator vs lane, retiring the Bash escape hatch.
5. **Bash policy shapes** — model guarded-path Bash commands as proposals.
6. **Fail-closed infra policy** — staged refusal when the runner/artifact is
   absent.

## 5. Environment receipts

- Runner: `cargo build --features runner --bin graphlaw-run` →
  `target/debug/graphlaw-run` (exit 0, 2m37s).
- Fresh wasm32-wasip1 artifact build: **DISK-BLOCKED** (lane disk guard: 2.8
  GiB free < 20 GiB; `df -h /` → `926Gi size, 2.7Gi avail, 81% capacity` at
  21:52 PDT). The prototype executed the pinned `priv/graphlaw.wasm`
  (2026-10-22 12:56, 6,793,617 bytes) via `GRAPHLAW_WASM`, which is the real
  module, just not freshly rebuilt this session. Re-pin/rebuild is the
  coordinator's next free-disk window.
- Stale lane build roots (~70 GiB `target-lane*`) were identified for the
  coordinator's cleanup law; purge permission was denied to this lane.
