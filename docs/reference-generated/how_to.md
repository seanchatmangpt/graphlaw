# How to: Using graphlaw

## Prerequisites


- @seanchatmangpt/graphlaw::build (script)

- @seanchatmangpt/graphlaw::gen (script)

- @seanchatmangpt/graphlaw::lint:pkg (script)

- @seanchatmangpt/graphlaw::sync-version (script)

- @seanchatmangpt/graphlaw::test (script)

- @seanchatmangpt/graphlaw::test:browser (script)

- @seanchatmangpt/graphlaw::typecheck (script)

- benches/tc_hot_01.rs::TC_HOT_FULL (env_key)

- benches/tc_hot_02.rs::TC_HOT_FULL (env_key)

- crates/graphlaw-eyeron/src/ast.rs::CRYPTO_SHA (const)

- crates/graphlaw-eyeron/src/ast.rs::CRYPTO_SHA (const)

- crates/graphlaw-eyeron/src/ast.rs::DT_DATATYPE (const)

- crates/graphlaw-eyeron/src/ast.rs::DT_DATATYPE (const)

- crates/graphlaw-eyeron/src/ast.rs::DT_LEXICAL_FORM (const)

- crates/graphlaw-eyeron/src/ast.rs::DT_LEXICAL_FORM (const)

- crates/graphlaw-eyeron/src/ast.rs::Document (struct)

- crates/graphlaw-eyeron/src/ast.rs::Document (struct)

- crates/graphlaw-eyeron/src/ast.rs::EYELING_DT_DATATYPE (const)

- crates/graphlaw-eyeron/src/ast.rs::EYELING_DT_DATATYPE (const)

- crates/graphlaw-eyeron/src/ast.rs::EYELING_DT_LEXICAL_FORM (const)

- crates/graphlaw-eyeron/src/ast.rs::EYELING_DT_LEXICAL_FORM (const)

- crates/graphlaw-eyeron/src/ast.rs::EYERON_UNQUOTE (const)

- crates/graphlaw-eyeron/src/ast.rs::EYERON_UNQUOTE (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_APPEND (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_APPEND (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_FIRST (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_FIRST (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_FIRST_REST (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_FIRST_REST (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_IN (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_IN (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_ITERATE (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_ITERATE (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_LAST (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_LAST (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_LENGTH (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_LENGTH (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_MAP (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_MAP (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_MEMBER (const)


## Steps


1. Use `build` from `@seanchatmangpt/graphlaw`.

2. Use `gen` from `@seanchatmangpt/graphlaw`.

3. Use `lint:pkg` from `@seanchatmangpt/graphlaw`.

4. Use `sync-version` from `@seanchatmangpt/graphlaw`.

5. Use `test` from `@seanchatmangpt/graphlaw`.

6. Use `test:browser` from `@seanchatmangpt/graphlaw`.

7. Use `typecheck` from `@seanchatmangpt/graphlaw`.

8. Use `TC_HOT_FULL` from `benches/tc_hot_01.rs`.

9. Use `TC_HOT_FULL` from `benches/tc_hot_02.rs`.

10. Use `CRYPTO_SHA` from `crates/graphlaw-eyeron/src/ast.rs`.

11. Use `DT_DATATYPE` from `crates/graphlaw-eyeron/src/ast.rs`.

12. Use `DT_LEXICAL_FORM` from `crates/graphlaw-eyeron/src/ast.rs`.


## Verified snippet

<!-- The snippet slot carries code copied from the extracted code surface -->
<!-- (doc:Claim rows whose doc:attribute is "snippet"), never agent prose. -->

```rust
// crates/graphlaw-eyeron/src/ast.rs :: as_str
as_str(&self) -> &str
```

<!-- AGENT-COMMENTARY-BEGIN -->
<!-- The ONLY region an agent may write into. Bounds: <= 12 lines,    -->
<!-- <= 100 chars/line, no new code facts (any new symbol mentioned   -->
<!-- must exist in queries/ast_extract.rq output; the doc_quality     -->
<!-- court fails Phi_halluc > 0.001 otherwise). No tables, no         -->
<!-- signatures, no parameters, no error lists — AGENT-FORBIDDEN      -->
<!-- everywhere.                                                      -->
<!-- AGENT-COMMENTARY-END -->
