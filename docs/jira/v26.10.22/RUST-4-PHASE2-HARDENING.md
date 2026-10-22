# RUST-4 Phase-2 Hardening Backlog — evidence_ring.rs concurrency audit findings

Status: OPEN (recorded 2026-10-22, post-integration; directive: record, do not
hot-fix mid-wave). Source: RUST-4 concurrency audit (coordinator dispatch).
Subject file: `src/evidence_ring.rs` (lane-owned). All line numbers as of
2026-10-22 14:5x PDT tree; re-verify before fixing.

## Finding 1 — HIGH: NDJSON write-fail advances `parent` anyway

- **Where**: `src/evidence_ring.rs` — slot-loop `parent = child` at :591 and
  :610 (in-memory chain head advances inside the pop loop), vs the batch write
  at :623: `if out.write_all(buf.as_bytes()).is_ok()` — failure is swallowed,
  `let _ = out.flush()` likewise; `ledger_error` is never set on this path.
- **Failure mode**: on write error, the whole batch's lines chain off a parent
  that never hit disk. On restart the resume parser takes the last on-disk
  child as head; every subsequent line then fails digest re-verification —
  permanent `BrokenChain` (typed, but the ledger is dead), and the failed
  batch's records are silently gone from NDJSON while
  `QueryLedger::record_batch` may still hold them — the two surfaces diverge
  (conservation: the exact quantity the ASC 450 aggregation sums).
- **Fix shape (per audit)**: advance `parent` only after successful
  write+flush (track a batch-local pending-parent, commit on success), or
  surface `ledger_error` and stop the drainer (fail-stop instead of fail-silent).

## Finding 2 — HIGH: OnceLock double-init corrupts the shared ledger

- **Where**: `init_global` :316-318 and `init_global_with_ledger` :326-331 —
  `Self::init[_with_ledger](...)` performs the full pre-set I/O (create dirs,
  spawn drainer, open `evidence.ndjson` append) BEFORE `GLOBAL.set`; `OnceLock`
  serializes only the set, not the I/O.
- **Failure mode**: two racing `init_global*` callers both fully initialize:
  two drainers interleave BLAKE3 chains into one `evidence.ndjson` →
  `BrokenChain`; the loser's `close()` writes a finalizing drop line into the
  corrupted file.
- **Fix shape (per audit)**: cover the init I/O with a process-wide init mutex
  (static `Mutex` + double-check `GLOBAL.get()`, or `get_or_try_init` pattern
  — note `get_or_try_init` is unstable; the mutex + double-check is the
  portable shape), or refuse a second init via a `try: GLOBAL.get() else init
  under lock` sequence so the loser discards its pre-set I/O before any
  ledger byte is written.

## Finding 3 — LOW: doc/API mismatch on "wait-free hot path"

- **Where**: module docs + `EvidenceRing::push` doc claim wait-free/never-block
  hot path; `push` takes the producer `Mutex` every call (de-facto
  MPMC-serialized, correctness intact).
- **Fix shape**: either replace the `Mutex<Option<Producer>>` with an
  unsafe-but-audited `UnsafeCell`/`MaybeUninit`-style single-producer handoff,
  or correct the docs to claim "non-blocking (drop-newest), Mutex-serialized
  SPSC-by-contract" and note the court is a ratio fence on the median, not a
  strict wait-free claim. Docs-only change is the honest minimal fix.
