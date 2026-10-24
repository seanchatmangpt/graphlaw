# Design: read_frame length-prefix bound (G4, both hosts)

Milestone: v26.10.23 · Lane: no-code docs lane · Status: SPEC (unimplemented)

## Problem

Both stdio runners (`src/bin/graphlaw_run.rs`, `src/bin/graphlaw_run_eh.rs`)
read a packet-4 frame as a 4-byte BE length prefix followed by a body that is
allocated at the prefix's full value *before* a single body byte is read
(`src/bin/graphlaw_run.rs:457-471`, `src/bin/graphlaw_run_eh.rs:336-350` —
the two functions are textually identical). A malicious or broken driver can
name `0xFFFFFFFF` (~4 GiB) in the prefix and force the runner to allocate
~4 GiB of zeros before the `read_exact` fails with `FrameEof` (exit 90) only
after stdin is exhausted. The bound must fire on the prefix alone, before any
body allocation, with a typed exit code, identically on both hosts.

## Current behavior (verified on disk)

- `src/bin/graphlaw_run.rs:457-471` — `read_frame`: reads the 4-byte BE
  prefix; line 465 `let mut body = vec![0u8; len]` allocates `len` bytes
  unconditionally, then `read_exact(&mut body)`. No upper bound on `len`.
- `src/bin/graphlaw_run_eh.rs:336-350` — byte-identical `read_frame`.
- `src/abi.rs:48` — `pub const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024`
  (16 MiB). This is the existing guest-side request cap, enforced in
  `abi.rs:165` (`check_limit("request_bytes", ...)`); the runners currently
  do not consult it at the frame layer.
- `wasm/src/abi_meta.rs:20` — rendered `MAX_REQUEST_BYTES: usize = 16777216`;
  `wasm/src/lib.rs:32-33` asserts it equals `graphlaw::abi::MAX_REQUEST_BYTES`.
  The single-source property already holds for the wasm side; the runners
  adopt the same hook.
- `docs/reference/runner-exit-codes.md` — exit table has no oversize row;
  90/91 are the only frame-layer codes today.

## Required behavior

### Single-source hook

`read_frame` must bound the prefix by `graphlaw::abi::MAX_REQUEST_BYTES`
(`src/abi.rs:48`) referenced directly — never a locally duplicated
`16 * 1024 * 1024`. The precedent is the wasm-side equality assertion at
`wasm/src/lib.rs:32-33`. A divergent local constant would split the limit
into N independently drifting copies; the frame bound and the guest alloc
bound must move together under one definition.

### Oversize prefix → typed error, exit 92

When `len > MAX_REQUEST_BYTES`, `read_frame` returns a new
`RunError::FrameOversize { len: usize, cap: usize }` *immediately*, before
the `vec![0u8; len]` allocation and before reading any body byte. The
diagnostic must name both numbers, e.g.

```text
graphlaw-run-eh: frame length 4294967295 exceeds cap 16777216
```

(prefix `graphlaw-run: ` on the wasmi host). The stream is then aborted —
the runner exits 92 and does not attempt to resynchronize or drain stdin.

### The cap is the request cap, not the wall

Startup must assert `MAX_REQUEST_BYTES <= WALL_BYTES` (the 64 MiB runner
memory wall; see the exit-86 "memory wall" row in
`docs/reference/runner-exit-codes.md`). These are deliberately *different*
limits serving different layers: the frame cap bounds an untrusted wire
prefix; the wall bounds total guest memory. The assert is a coherence guard
(a frame the cap admits must be allocatable under the wall), not a demand
that the constants be equal — do not force-equal them. A frame may be
under the cap yet still fail inside the guest; that is exit 86/97
territory, not 92.

### Wire-up

1. `RunError::FrameOversize` variant added to both binaries' `RunError`
   enums (the enums are per-binary; both get the variant).
2. `code()` tables (`graphlaw_run.rs:134-149`, `graphlaw_run_eh.rs:58-72`)
   gain `RunError::FrameOversize { .. } => 92`.
3. `diagnostic()` gains the len+cap arm shown above.
4. Both `read_frame` bodies gain the bound check between prefix decode and
   allocation. Keep the two functions textually identical (they are today);
   the diff is mirrored in both files.
5. `docs/reference/runner-exit-codes.md`: new row `| 92 | Frame length
   prefix exceeds MAX_REQUEST_BYTES | frame length <len> exceeds cap <cap> |
   Driver: chunk or shrink the frame. |` in BOTH host tables, plus the exact
   diagnostic strings in both fenced blocks.

## Acceptance (courts, `tests/graphlaw_run_eh_court.rs`)

Anti-vacuity twins — each leg must flip if the bound is removed or
mis-wired:

1. **Exactly-at-cap succeeds end to end.** A frame whose body length is
   exactly `MAX_REQUEST_BYTES` (16 MiB) is read, dispatched, and answered
   (a `capabilities` request is sufficient; assert on the parsed response,
   real artifact, real subprocess via the existing `run_host` harness at
   `graphlaw_run_eh_court.rs:69`). This proves the bound is `>` not `>=`
   and that no off-by-one rejects lawful traffic.
2. **`0xFFFFFFFF` prefix → exit 92, zero body bytes read.** stdin supplies
   the 4-byte prefix `FF FF FF FF` and then a bounded amount of padding
   bytes. Assert exit code 92 AND that the runner consumed fewer bytes than
   a body read would require (e.g. a pipe that blocks/counts: write only the
   prefix plus one sentinel byte through a `CountingReader`-style fixture, or
   assert the short-read count < cap). If the allocation-first bug returned,
   this leg flips to exit 90/OOM — it cannot silently pass.
3. **Zero-len frame + EOF → exit 0.** stdin is exactly 4 zero bytes
   (`00 00 00 00`). `read_exact` of an empty body succeeds, `read_frame`
   returns `Some(vec![])` once, and the next boundary read hits clean EOF →
   exit 0, silent. Proves the bound does not reject degenerate-but-lawful
   frames.
4. **Wasmi-host mirror.** Legs 2 and 3 are duplicated against
   `graphlaw-run` (the wasmi binary) to prove the diff was mirrored, not
   applied to one host only. (Leg 1 may be eh-only if the legacy artifact
   fixture class cannot carry a 16 MiB frame; state the skip loudly if so.)

Court plumbing constraints: the 16 MiB fixture bodies are built in-memory
per test and cleaned up with the process; no temp-file leaks; no sleeps —
subprocess completion is awaited via the existing `run_host` wait, and any
pipe-count fixture uses bounded reads with a deadline, never a fixed sleep.

## Exit-code stability

92 is unused in both tables today (verified: 0,2,84,86,90,91,93,94,95,96,97
in `graphlaw_run.rs`; same set plus 96-EH in `graphlaw_run_eh.rs`). Assigning
it is append-only per `docs/refusals.md` discipline; no existing code is
renumbered.

## Non-goals

- No chunked/framed-input resynchronization after an oversize prefix: the
  connection is aborted, exit 92, driver's problem to retry smaller.
- No change to the guest-side cap enforcement (`src/abi.rs:165`) or to
  `MAX_OUTSTANDING_ALLOC_BYTES`.
- No change to `write_frame` (egress side is host-controlled, not
  attacker-lengthed).

## See Also

- `docs/reference/runner-exit-codes.md`
- `src/abi.rs` (`MAX_REQUEST_BYTES`, `check_limit`)
- `wasm/src/lib.rs:32-33` (single-source equality precedent)
- `tests/graphlaw_run_eh_court.rs`
