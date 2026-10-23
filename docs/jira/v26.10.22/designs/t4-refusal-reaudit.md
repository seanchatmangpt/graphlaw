# T4 refusal-register re-audit (post-switchover drift check)

- Date: 2026-10-22, lane T4, repo `/Users/sac/graphlaw`, one canonical checkout.
- Baseline: V-L11 pre-switchover audit (anchors already carried V-L11 dated corrections in-register).
- Scope: `docs/refusals.md` file anchors re-verified after the SQLRDF-18 switchover wiring (facades + dirs).

## Anchor table (pre = V-L11 baseline, post = this audit, 2026-10-22)

| anchor | claim in register | pre (V-L11) | post (T4 grep, exact command) | verdict |
|---|---|---|---|---|
| `src/sql_ast/refusals.rs` REFUSAL-01..03 constants | `sql_ast::REFUSAL_01_MALFORMED_AST` etc. live here post-switchover | resolves | lines 9, 10, 11 — `grep -n "REFUSAL_0[123]_" src/sql_ast/refusals.rs` | UNCHANGED |
| `src/sql_ast.rs` re-export | `pub use refusals::*` keeps `sql_ast::REFUSAL_*` paths resolving | resolves | line 24 (`pub use refusals::refusal_arm;` line 23) | UNCHANGED |
| `src/sql_ast/refusals.rs` `RefusalCode = 36/37` | `LiteralNotInt = 36`, `LiteralNotFloat = 37` | resolves | lines 112, 115 | UNCHANGED |
| `src/sql_ast/refusals.rs` `refusal_arm::JOIN_TYPE_IRI = 37` | arm-namespace constant | resolves | line 59 | UNCHANGED |
| `src/tier.rs` `REFUSAL_04_CAPABILITY_REFUSAL` | typed constant, value `REFUSAL-04` | resolves | line 25, value `"REFUSAL-04"` | UNCHANGED |
| `src/rules/builtins.rs` REFUSAL-04 call site | literal string, "verified 2026-10-22 lane V-L11" | literal | line 89, still the literal marker (same text) | UNCHANGED |
| `src/duckdb_eval.rs` `NoCanonicalGraph` Display arm | `no canonical graph bytes — … (query {iri})` | resolves | lines 91 (variant), 102-104 (Display), 673 (raise site) | UNCHANGED |
| `src/attest.rs` `AttestError` | `#[non_exhaustive]`, `BadSignature`/`UnknownKey`/`Malformed` | resolves | lines 38-46; Display arms 52-53 | UNCHANGED |

Shifted anchors: **0**. No dated corrections were required; the register text is
byte-identical except for the dated re-audit note appended to the
"Message ergonomics" section.

## Probe receipt (real output)

```
$ CARGO_TARGET_DIR=target-lane-t4 cargo run --example refusal_probe
...
ALL REFUSAL PROBES MATCHED
```

- `grep -c MATCH` on probe stdout: **13** (exit 0)
- `grep -c MISMATCH`: **0**
- probe process exit: **0**

## Conclusion

Post-switchover drift: none. The V-L11 anchor table stands as of 2026-10-22.
Register flips remain receipt-gated; no status was hand-edited this audit.
