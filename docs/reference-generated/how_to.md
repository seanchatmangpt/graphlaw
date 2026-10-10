# How to: Using graphlaw

## Prerequisites


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

- crates/graphlaw-eyeron/src/ast.rs::LIST_MEMBER (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_MEMBER_AT (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_MEMBER_AT (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_NOT_MEMBER (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_NOT_MEMBER (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_REMOVE (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_REMOVE (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_REST (const)

- crates/graphlaw-eyeron/src/ast.rs::LIST_REST (const)


## Steps


1. Use `CRYPTO_SHA` from `crates/graphlaw-eyeron/src/ast.rs`.

2. Use `DT_DATATYPE` from `crates/graphlaw-eyeron/src/ast.rs`.

3. Use `DT_LEXICAL_FORM` from `crates/graphlaw-eyeron/src/ast.rs`.

4. Use `EYELING_DT_DATATYPE` from `crates/graphlaw-eyeron/src/ast.rs`.

5. Use `EYELING_DT_LEXICAL_FORM` from `crates/graphlaw-eyeron/src/ast.rs`.

6. Use `EYERON_UNQUOTE` from `crates/graphlaw-eyeron/src/ast.rs`.

7. Use `LIST_APPEND` from `crates/graphlaw-eyeron/src/ast.rs`.

8. Use `LIST_FIRST` from `crates/graphlaw-eyeron/src/ast.rs`.

9. Use `LIST_FIRST_REST` from `crates/graphlaw-eyeron/src/ast.rs`.

10. Use `LIST_IN` from `crates/graphlaw-eyeron/src/ast.rs`.

11. Use `LIST_ITERATE` from `crates/graphlaw-eyeron/src/ast.rs`.

12. Use `LIST_LAST` from `crates/graphlaw-eyeron/src/ast.rs`.


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
