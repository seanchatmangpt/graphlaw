# Security Policy

## Supported versions

| version | supported |
|---|---|
| 26.10.26 (latest released) | yes — the current workspace release line on `main` |
| 26.10.22 (crates.io; no GitHub tag) | yes |
| 26.10.21 (crates.io; no GitHub tag) | yes |
| v26.10.13 (newest published GitHub release / crate) | yes |
| 26.10.14 – 26.10.20 | intermediate workspace versions; fixed forward on `main`, no per-version patches |
| `main` (development) | fixes land on `main` only; no backports, no LTS branches |
| all earlier published releases | no |

There is no long-term-support policy: only the current release line receives security
fixes, and `main` is fixed forward. The version string is gated to the workspace version
(`tests/docs_release_claims.rs::doc_version_lines_match_workspace_version`), so this table
moves with every workspace bump; published artifacts may lag the workspace line (as of
this writing the newest published release is still v26.10.13 — this note is refreshed at
publish time, not per-commit). Pin your own dependency versions accordingly.

## Reporting a vulnerability

Use GitHub private vulnerability reporting on `seanchatmangpt/graphlaw` (Security tab, "Report a
vulnerability"). Do not open a public issue for an undisclosed vulnerability.

## Scope

In scope: untrusted plan, policy, and JSON ABI input; receipt forgery; resource exhaustion
(memory, CPU, unbounded loops); egress via `COPY ... TO` and `ATTACH`; sandbox escapes from
the EH host (see "EH host sandbox" below). Vulnerabilities in PurRDF, purrdf-n3, or DuckDB
(`frozen-duckdb`) themselves belong upstream.

Egress specifics in scope:

- `COPY ... TO` writes whatever path the admitted statement names, as the host process user.
  If your process can write a path, so can an admitted COPY. Treat the statement graph as
  the egress authority.
- Scalar-fallback COPY: a `sql:CopyStatement` with no `sql:copyOperation` renders a
  hardcoded destination `/tmp/graphlaw-rdf-<table>.csv` (`src/sql_ast.rs:2255-2258`). That
  path is NOT covered by any attach allowlist and is written without a witness digest on the
  receipt. It is a known, lawful egress hole; prefer the expression form
  (`docs/how-to/egress-and-attach-policy.md`, "Scalar-fallback caveat").

## Known limits (as of the current `git log`)

- Plain receipts (`receipt::record` / `receipt::require`) prove presence only: a receipt is a
  content-addressed record, and anyone who can write a state can record one. Authenticated use needs
  the signed API: Ed25519 attestations (`attest`), `receipt::require_signed` /
  `Step::RequireSignedReceipt`, signed leases judged against a verifier-supplied `Clock`, and the
  `graphlaw-verify` binary. Key custody, distribution and revocation are outside the library.
- The wasm ABI has no signing op; hosts issue and sign leases and receipts natively. A request that
  sets `unverified_lease: true` opts out of lease signature checks and is not authenticated.
- Resource caps exist on the JSON ABI (`abi::MAX_*`); the native Rust API does not enforce them.

## EH host sandbox (`graphlaw-run-eh`, wasmtime 48)

What the bundled EH runner (`src/bin/graphlaw_run_eh.rs`, feature `runner-eh`) does provide:

- Import allowlist is fail-closed: any import outside `wasi_snapshot_preview1` refuses
  instantiation (`src/bin/graphlaw_run_eh.rs:222-231`). No raw host functions, no
  custom-import surface.
- No filesystem access by default: the WASI context never preopens a directory
  (no `preopen_dir` call in the runner). An in-sandbox COPY/ATTACH cannot reach the host
  filesystem unless YOU preopen a directory in your own host embedding.
- Request-size wall: requests above `WALL_BYTES` (64 MiB,
  `src/bin/graphlaw_run_eh.rs:28`) are rejected before `gl_alloc`.
- Frame bound: a frame whose 4-byte length prefix exceeds
  `graphlaw::abi::MAX_REQUEST_BYTES` is refused with exit 92 before any body
  allocation (`read_frame`; court `tests/wasm_frame_bound_court.rs` 8/0 —
  at-cap admitted, cap+1 → 92, 0xFFFFFFFF → 92 pre-body, zero-len → 0;
  `docs/reference/wasm-gap-register.md` G4 CLOSED, 03:20 flip log).
- Sterile environment: the WASI context is built with no `inherit_env` /
  `env` / `envs` call at all — the guest environment is empty, so no host
  variable (keys, tokens, paths) reaches the module
  (`src/bin/graphlaw_run_eh.rs:292-300`; court
  `tests/wasm_env_isolation_court.rs` 3/0 with a positive-control leg,
  gap-register 03:20 entry).
- Fuel metering: every guest request runs under a 100,000,000,000-unit
  wasmtime fuel budget (`FUEL_PER_REQUEST`,
  `src/bin/graphlaw_run_eh.rs:46`); exhaustion is a typed exit 98, never a
  raw trap or an unbounded hang. The budget was calibrated against the
  pinned artifact on a red→red→green ladder (10M red, 1B red, 100B green;
  `receipts/2026-10-24-fuel-calibration.md`).

### Imported WASI capability inventory

The shipped artifact (`priv/graphlaw.wasm`, b3sum
`0858a760f9df87c66cdc01a04f6f4ce15926f400cb974b67a7d14513a5ddd2f1`,
60,895,924 B — the live pin, 4-surface verified) imports 28 functions, all
`wasi_snapshot_preview1`. Re-witnessed on this exact pin with
`wasm-objdump -x priv/graphlaw.wasm` on 2026-10-26 (`Import[28]`; all 28
names below match the live section, no non-WASI import modules):

`random_get`, `environ_get`, `environ_sizes_get`, `clock_time_get`,
`fd_close`, `fd_datasync`, `fd_fdstat_get`, `fd_fdstat_set_flags`,
`fd_filestat_get`, `fd_filestat_set_size`, `fd_pread`,
`fd_prestat_get`, `fd_prestat_dir_name`, `fd_pwrite`, `fd_read`,
`fd_readdir`, `fd_seek`, `fd_write`, `path_create_directory`,
`path_filestat_get`, `path_open`, `path_readlink`,
`path_remove_directory`, `path_rename`, `path_unlink_file`,
`poll_oneoff`, `proc_exit`, `sched_yield`.

The file-descriptor and `path_*` imports are the full ambient file-I/O
surface (open, read, write, seek, rename, unlink, directory create/
remove/readdir), and `environ_get` reads the host environment. This is
what the module is *capable of requesting* from any host; the EH
runner's zero-preopen posture is the containment — with no preopened
directory, `path_open` and the fd calls have no reachable filesystem
root, and the import allowlist still fail-closes anything outside
`wasi_snapshot_preview1`. Hosts that preopen a directory (or inherit
the env) grant exactly these calls real reach.

What it does NOT provide — do not over-assume:

- NO wall-clock bound that can fire. The runner arms an epoch deadline
  (`store.set_epoch_deadline`, `src/bin/graphlaw_run_eh.rs:306`), but no
  watchdog calls `Engine::increment_epoch`, so the epoch counter never
  advances and the deadline cannot fire today. Fuel (above) is the live CPU
  bound; a host needing a time-based bound must drive `increment_epoch`
  from its own timer thread.
- NO linear-memory ceiling is enforced on the store. `WALL_BYTES` caps the request frame,
  not guest heap growth; the module was only measured against a 64 MiB wall
  (`docs/WASM.md`, "smalt sandbox spec"). Do not feed it adversarial inputs expecting an
  OOM trap.
- `GRAPHLAW_ATTACH_ROOTS` is a HOST-LIBRARY control: `enforce_attach_policy` runs in
  `src/duckdb_op.rs` before the engine sees a statement, on the host side of the
  boundary. It is NOT enforced inside the wasm sandbox; if you preopen the database
  directory into the guest, the allowlist does not follow it in.
- Stdio is inherited (`inherit_stdio`): the guest can write to the runner's stdout/stderr.

## Disclosure timeline (target, not a promise)

- Acknowledgement: target 7 days.
- Triage and severity: target 14 days.
- Fix and coordinated disclosure: target 90 days.
