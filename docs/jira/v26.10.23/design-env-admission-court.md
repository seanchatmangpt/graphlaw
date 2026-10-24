# Design: Pre-Wave Environment-Admission Court

- **Milestone**: v26.10.23
- **Lane**: no-code (this file only; markdown only, no code execution)
- **Status**: DRAFT
- **Provenance**: ERRC process-mining intervention I-3. ~20 recurring BLOCKED rows in the
  v26.10.22 wave ledger were environmental, not logical: lanes were dispatched into a
  substrate that was already broken, discovered the break mid-gate, and burned a wave slot
  reporting it. The intervention moves that discovery to BEFORE dispatch, as a typed
  admission decision.

## 1. Context — witnessed blockers

| Class | Witness (v26.10.22 window) | Cost |
|---|---|---|
| ENOSPC | `df -h /` fell below the 20 GiB floor; gates killed twice mid-run | 2 dead gate runs |
| Engine-cache wipe | `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/` staging wiped; the `.legacy*.bak` siblings in the cache dir are the residue of those wipes | wasm boundary unrunnable |
| Missing `WASI_SDK_PATH` | `WASI_SDK_PATH` empty in a fresh shell; `/opt/wasi-sdk` absent on this host at design time (probed 2026-10-23: `ls /opt/wasi-sdk` → No such file or directory). CLAUDE.md: "typed build.rs refusal without it" | wasm build refused at build.rs |
| Sysroot mismatch | CI pins wasi-sdk **34** (`bin/clang`, `share/cmake/wasi-sdk-p1.cmake`); a wrong-major local install fails at link time, after minutes of compile | debug at the worst rung |
| Missing tool | `b3sum` / `python3` / `cargo-mutants`-class binary absent from PATH; court scripts die on first missing tool | court exits for the wrong reason |
| `src/` dirt | Concurrent lanes reading each other's in-flight `src/` edits as their baseline | false-red compiles |

Environmental causes are re-observed every wave because nothing fails closed on them
before dispatch. The court is that fail-closed step.

## 2. Mechanism

A single wave-admission script — `scripts/env_admission_court.sh` — executed by the
coordinator **before** dispatching any wave. Zero side effects; it only observes and
emits a typed verdict on stdout, exit 0 (PASS) or exit 1 (REFUSED).

### Checks

Each check emits one line: `CHECK <name>: <PASS|REFUSED> <detail>`.

1. **disk-floor** — `df -h /` available ≥ 20 GiB (CLAUDE.md disk discipline).
2. **duckdb-cache** — `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/libduckdb.a`
   exists and `b3sum` matches the pinned expectation. Pin lives in the court's companion
   pin file (e.g. `courts/env-admission/expected.blake3`), updated only via the same
   receipt-gated flip as `courts/code-partition/axioms.blake3`.
3. **wasi-sdk** — `WASI_SDK_PATH` resolvable (env or a known default such as
   `/opt/wasi-sdk`): `$WASI_SDK_PATH/bin/clang` executable AND
   `$WASI_SDK_PATH/share/cmake/wasi-sdk-p1.cmake` present; version banner must contain
   the pinned major (34), else REFUSED for sysroot mismatch.
4. **tools** — `b3sum`, `python3`, `cargo-mutants` each resolvable on PATH.
5. **src-clean** — `git status --porcelain -- src/` empty (coordinator may pass
   `--allow-src-dirt` with a receipt, since coordinator owns `src/` transitions and the
   mutation court runs solo).

### Verdict format (typed)

```
ENV-ADMISSION PASS (7/7 checks) — wave dispatch authorized
```

```
ENV-ADMISSION REFUSED: wasi-sdk — /opt/wasi-sdk/bin/clang missing
  fix: install wasi-sdk 34 to /opt/wasi-sdk, or export WASI_SDK_PATH=<root>
  court: scripts/env_admission_court.sh
```

- A REFUSED verdict names the exact failing check and the recovery. One check fails →
  the whole verdict is REFUSED (fail-closed; no partial admission).
- Loud-SKIP discipline carries over unchanged: a check that cannot run (e.g. `b3sum`
  missing so the cache digest cannot be computed) emits
  `CHECK duckdb-cache: SKIP [typed reason]` — and a SKIP is **not** a pass: any SKIP
  forces the overall verdict to REFUSED with that reason named.

## 3. Integration

Two integration points, either is sufficient, both are cheap:

1. **Dispatch precondition hook** — the coordinator's wave-dispatch step invokes the
   court first and aborts on exit 1. If the coordinator dispatches through a script,
   this is the first line; if through a hook, a `PreToolUse`-class gate.
2. **Wave runbook first line** — every `docs/jira/v<version>/` wave runbook opens with:

   ```bash
   bash scripts/env_admission_court.sh || { echo "ENV-ADMISSION REFUSED — do not dispatch"; exit 1; }
   ```

The court runs **solo and read-only** (same serialization discipline as
`scripts/mutation_court.sh` applies trivially since it writes nothing, but it must run
before, not concurrently with, dispatch). Coordinator-owned; lanes never invoke it as a
gate against each other.

## 4. Anti-vacuity self-test

A gate that cannot refuse is the failure. Self-test procedure (the implementing lane
must embed the real transcript as the receipt):

1. **Baseline**: run the court on a healthy tree → expect `PASS`, exit 0.
2. **Mutation 1 (cache)**: rename `libduckdb.a` → `libduckdb.a.vacuity` → run → expect
   exit 1 and a REFUSED naming `duckdb-cache`. Restore, verify `b3sum` unchanged.
3. **Mutation 2 (wasi)**: `WASI_SDK_PATH=/tmp/empty-root` → expect REFUSED naming
   `wasi-sdk` with the missing `bin/clang` path.
4. **Mutation 3 (disk)**: point the floor check at a tiny tmpfs/volume (or temporarily
   lower the floor above current usage via a `-f` override flag whose use is logged) →
   expect REFUSED naming `disk-floor`.
5. **Mutation 4 (skip-as-pass)**: run with PATH stripped of `b3sum` → expect
   `SKIP [b3sum not on PATH]` AND overall REFUSED. A court that reports PASS here is
   `VACUOUS GATE` and fails the self-test.
6. **Restore proof**: after each mutation, verify restore (digest or path check) —
   same restore-detection discipline as the smalt shim-court protocol.

Acceptance for the implementing lane: all four mutations flip the verdict, restore is
byte-exact, and the transcript (real output, exit codes) is embedded in
`docs/jira/v26.10.23/RECEIPTS.md`.

## 5. Acceptance

- `scripts/env_admission_court.sh` exists, is executable, side-effect-free.
- On the current host it must currently **REFUSED** on `wasi-sdk` (probed 2026-10-23:
  `/opt/wasi-sdk` absent) — that refusal IS the first real receipt, proving the gate
  can exit 1 on day one.
- After the operator installs wasi-sdk 34, a full `PASS` transcript with real `df`,
  `b3sum`, and `git status` output is the standing receipt.
- Self-test (Section 4) embedded as a real transcript, not described.

## 6. Status vocabulary

- `PASS` / `REFUSED` are the only verdicts. No silent skip, no partial admission.
- Refusal codes live in `docs/refusals.md` as `ENV-REFUSED(<check>)` — append-only,
  never renumbered.

## See Also

- `docs/jira/v26.10.22/` wave ledger (the ~20 environmental BLOCKED rows)
- `docs/refusals.md` — typed refusal register
- `CLAUDE.md` — disk discipline, wasi-sdk env requirement, mutation-court solo rule
- `~/.claude/rules/testing-chicago-style.md` — shim-court / loud-SKIP discipline
