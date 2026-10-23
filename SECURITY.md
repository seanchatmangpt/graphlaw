# Security Policy

## Supported versions

| version | supported |
|---|---|
| 26.10.22 (latest released) | yes — the current workspace release line on `main` |
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
the EH host (see "EH host sandbox" below). Vulnerabilities in PurRDF, Eyeron, or DuckDB
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

What it does NOT provide — do not over-assume:

- NO fuel or epoch metering. `graphlaw-run-eh` sets neither `Config::consume_fuel` nor
  epoch-deadline trapping; a wasm-infinite loop pins the thread until the operator kills it.
  Hosts needing CPU bounds must add their own metering.
- NO linear-memory ceiling is enforced on the store. `WALL_BYTES` caps the request frame,
  not guest heap growth; the module was only measured against a 64 MiB wall
  (`docs/WASM.md`, "smalt sandbox spec"). Do not feed it adversarial inputs expecting an
  OOM trap.
- `inherit_env()` passes the FULL host environment to the guest
  (`src/bin/graphlaw_run_eh.rs:236`). Any secret in the process env is readable by the
  module. Strip the environment (build your own `WasiCtxBuilder` without `inherit_env`)
  before running untrusted graphs.
- `GRAPHLAW_ATTACH_ROOTS` is a HOST-LIBRARY control: `enforce_attach_policy` runs in
  `src/duckdb_op.rs` before the engine sees a statement, on the host side of the
  boundary. It is NOT enforced inside the wasm sandbox; if you preopen the database
  directory into the guest, the allowlist does not follow it in.
- Stdio is inherited (`inherit_stdio`): the guest can write to the runner's stdout/stderr.

## Disclosure timeline (target, not a promise)

- Acknowledgement: target 7 days.
- Triage and severity: target 14 days.
- Fix and coordinated disclosure: target 90 days.
