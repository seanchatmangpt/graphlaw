# Receipt: refusal register post-switchover anchor audit (lane V-L11, 2026-10-22)

Subject: `docs/refusals.md`, `docs/refusal-playbook.md` anchor audit after the
SQLRDF-18 dialect/registry switchover.

## Probe run (real output)

```
$ cargo run --example refusal_probe
...
=== direct validate_ast_shacl (missing index, no context fix-up) ===
SHACL AST structural violation: Node <<http://x>>: [REFUSAL-01] MalformedAstRefusal: a SelectStatement MUST contain exactly one projection list (sql:select, or legacy alias sql:selectList — never zero, never both; an empty rdf:nil list is ne (+1 more violations)
[exit-class MATCH]

ALL REFUSAL PROBES MATCHED
EXIT=0
```

(Verbatim tail of the run; 13/13 exit-class MATCH, exit 0.)

## Per-anchor verification (grep, all exit 0)

| anchor | resolves |
|---|---|
| `sql_ast::REFUSAL_01/02/03` constants | `src/sql_ast/refusals.rs:9-11` (re-exported by `src/sql_ast.rs:24 pub use refusals::*`) |
| `tier::REFUSAL_04_CAPABILITY_REFUSAL` | `src/tier.rs:25` |
| `RefusalCode` LiteralNotInt=36 / LiteralNotFloat=37 | `src/sql_ast/refusals.rs:112,115` |
| `refusal_arm::JOIN_TYPE_IRI = 37` | `src/sql_ast/refusals.rs:59` |
| `duckdb_eval::EvaluatorError::NoCanonicalGraph` + Display arm text | `src/duckdb_eval.rs:91,102-104` (exact string match) |
| `attest::AttestError` `#[non_exhaustive]`, BadSignature/UnknownKey/Malformed, sign path | `src/attest.rs:38-46` |
| `ggen/sql-dialect/sql_statement_kinds.rs` Connect/DisconnectStatement | lines 74,76 |
| `rules::builtins` `BuiltinError::AstError` + REFUSAL-04 literal | `src/rules/builtins.rs:19,87-89` |
| `examples/refusal_probe.rs` | exists; probe re-run 13/13 MATCH, exit 0 |
| `abi::MAX_*` limit constants | `src/abi.rs:48-56` |
| `law::N3_*` constants + `n3_*` limit names | `src/law.rs:440-448,542-558` |
| `src/law.rs` `Step` enum (AdmitShacl/DeriveN3) | `src/law.rs:50-57` |
| `src/duckdb_op.rs` | exists |
| playbook probe pointer `examples/refusal_probe.rs` | exists, re-run today |

## Corrections applied (stale anchors)

1. `docs/refusals.md` §SQL AST refusals: constants anchor `src/sql_ast.rs` →
   `src/sql_ast/refusals.rs` (switchover move; `sql_ast::` paths still resolve
   via re-export), dated 2026-10-22 lane V-L11.
2. `docs/refusals.md` `AstError` `details` row: "Variants in src" → AstError is
   a pure-data struct `{code, ctx}` post SQLRDF-18; the names are constructor
   functions, Display from a const table.
3. `LITERAL_NOT_INT` / `LITERAL_NOT_FLOAT` rows: the rendered message carries
   the `[REFUSAL-01]` marker + lexical, not a `LITERAL_NOT_INT` token —
   verified against `src/sql_ast/refusals.rs:338-345` and live probe P2.
4. REFUSAL-04 carrier cell: call site `src/rules/builtins.rs` uses the literal
   string, not the `tier::` constant (same text), dated correction.
5. `docs/refusal-playbook.md` findings section: Display-string anchor
   `src/sql_ast.rs` → `src/sql_ast/refusals.rs`, dated correction.

No refusal status was flipped; no code added or removed; register content
(append-only codes) untouched.
