# R-2 discharge — cargo-deny residual-advisory operator decision (2026-10-24)

- Date: 2026-10-24
- Subject: `/Users/sac/graphlaw` @ main, HEAD `6da1d33`, working tree dirty
  (in-flight wasmi-retirement wave).
- Decision authority: **operator directive 2026-10-24 (publish order)** —
  `docs/jira/v26.10.23/compliance-final-verdict.md:374-388`
  (R-2 DISCHARGE section: per-advisory dispositions, all dated-ignored in
  `deny.toml` with expiry 2026-11-23, re-review required on expiry).
- Lane: finD execution lane (file lease includes this receipt);
  coordinator owns git.

## Decision

The three residual advisories in `deny.toml` `[advisories]` are **accepted
by explicit operator decision**, each dated-ignored with expiry
**2026-11-23** and re-review required on expiry:

| advisory | crate/version | disposition | rationale (from deny.toml + verdict §R-2) |
|---|---|---|---|
| RUSTSEC-2026-0258 | h2 0.3.27 (unbounded empty DATA frames, DoS) | **ACCEPTED RISK** | transitive via frozen-duckdb → reqwest ^0.11; **host-side tooling only, not on the in-WASM execution/receipt-signing path**; h2 0.4 is semver-incompatible with the frozen reqwest pin; upstream bump tracked |
| RUSTSEC-2025-0134 | rustls-pemfile (unmaintained) | **ACCEPTED** | host-side TLS config plumbing only |
| RUSTSEC-2024-0449 | unsafe-libyaml (unmaintained) | **ACCEPTED** | serde_yaml transitive, config parsing only |

Expiry **2026-11-23** (30-day window for upstream reconciliation); the
ignores must be re-reviewed on expiry, not silently renewed.

## Embedded live output (this lane, 2026-10-24)

Command: `cargo deny check advisories` (workspace root,
`deny.toml [advisories]` carries the 3 dated ignores above).

```
CARGO_DENY_EXIT=0
advisories ok
```

(`CARGO_DENY_EXIT` is the bare cargo-deny exit captured without a pipe
tail. An earlier same-command run through `tee` captured the tee exit —
same verdict, defective capture; the bare-exit re-run above is the receipt
of record.)

Note: `cargo audit` exit 1 is the **expected divergence** — `cargo audit`
does not read `deny.toml`, so it still reports the three advisories while
cargo-deny passes under the dated ignores. This divergence is documented,
not a failure.

## Standing

`cargo deny check advisories` = exit 0 on this tree under the operator
decision. R-2 status: **DECIDED-2026-10-24** (was: open ask in
`docs/jira/v26.10.22/RELEASE-NOTES-26.10.22.md` PRE-PUBLISH CHECKLIST item
2 — flipped to a DECIDED pointer this pass). No git transition; the
coordinator owns commits.
