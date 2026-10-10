# How to: Using graphlaw

## Prerequisites


- crates/graphlaw-eyeron/src/ast.rs::Document (struct)

- crates/graphlaw-eyeron/src/ast.rs::Document (struct)

- crates/graphlaw-eyeron/src/ast.rs::Literal (struct)

- crates/graphlaw-eyeron/src/ast.rs::Literal (struct)

- crates/graphlaw-eyeron/src/ast.rs::Name (struct)

- crates/graphlaw-eyeron/src/ast.rs::Name (struct)

- crates/graphlaw-eyeron/src/ast.rs::Rule (struct)

- crates/graphlaw-eyeron/src/ast.rs::Rule (struct)

- crates/graphlaw-eyeron/src/ast.rs::SourceRef (struct)

- crates/graphlaw-eyeron/src/ast.rs::SourceRef (struct)

- crates/graphlaw-eyeron/src/ast.rs::Term (enum)

- crates/graphlaw-eyeron/src/ast.rs::Term (enum)

- crates/graphlaw-eyeron/src/ast.rs::Triple (struct)

- crates/graphlaw-eyeron/src/ast.rs::Triple (struct)

- crates/graphlaw-eyeron/src/ast.rs::as_str (function)

- crates/graphlaw-eyeron/src/ast.rs::as_str (function)

- crates/graphlaw-eyeron/src/ast.rs::blank (function)

- crates/graphlaw-eyeron/src/ast.rs::blank (function)

- crates/graphlaw-eyeron/src/ast.rs::default_prefixes (function)

- crates/graphlaw-eyeron/src/ast.rs::default_prefixes (function)

- crates/graphlaw-eyeron/src/ast.rs::formula (function)

- crates/graphlaw-eyeron/src/ast.rs::formula (function)

- crates/graphlaw-eyeron/src/ast.rs::fuse (function)

- crates/graphlaw-eyeron/src/ast.rs::fuse (function)

- crates/graphlaw-eyeron/src/ast.rs::iri (function)

- crates/graphlaw-eyeron/src/ast.rs::iri (function)

- crates/graphlaw-eyeron/src/ast.rs::is_ground (function)

- crates/graphlaw-eyeron/src/ast.rs::is_ground (function)

- crates/graphlaw-eyeron/src/ast.rs::is_variable (function)

- crates/graphlaw-eyeron/src/ast.rs::is_variable (function)

- crates/graphlaw-eyeron/src/ast.rs::list (function)

- crates/graphlaw-eyeron/src/ast.rs::list (function)

- crates/graphlaw-eyeron/src/ast.rs::literal (function)

- crates/graphlaw-eyeron/src/ast.rs::literal (function)

- crates/graphlaw-eyeron/src/ast.rs::merge (function)

- crates/graphlaw-eyeron/src/ast.rs::merge (function)

- crates/graphlaw-eyeron/src/ast.rs::new (function)

- crates/graphlaw-eyeron/src/ast.rs::new (function)

- crates/graphlaw-eyeron/src/ast.rs::plain (function)

- crates/graphlaw-eyeron/src/ast.rs::plain (function)


## Steps


1. Use `Term` from `crates/graphlaw-eyeron/src/ast.rs`.

2. Use `as_str` from `crates/graphlaw-eyeron/src/ast.rs`.

3. Use `blank` from `crates/graphlaw-eyeron/src/ast.rs`.

4. Use `default_prefixes` from `crates/graphlaw-eyeron/src/ast.rs`.

5. Use `formula` from `crates/graphlaw-eyeron/src/ast.rs`.

6. Use `fuse` from `crates/graphlaw-eyeron/src/ast.rs`.

7. Use `iri` from `crates/graphlaw-eyeron/src/ast.rs`.

8. Use `is_ground` from `crates/graphlaw-eyeron/src/ast.rs`.

9. Use `is_variable` from `crates/graphlaw-eyeron/src/ast.rs`.

10. Use `list` from `crates/graphlaw-eyeron/src/ast.rs`.

11. Use `literal` from `crates/graphlaw-eyeron/src/ast.rs`.

12. Use `merge` from `crates/graphlaw-eyeron/src/ast.rs`.


## Verified snippet

<!-- The snippet slot carries code copied from the extracted code surface -->
<!-- (doc:Claim rows whose doc:attribute is "snippet"), never agent prose. -->

```rust
// crates/graphlaw-eyeron/src/ast.rs :: as_str
as_str(&self)
```

<!-- AGENT-COMMENTARY-BEGIN -->
<!-- The ONLY region an agent may write into. Bounds: <= 12 lines,    -->
<!-- <= 100 chars/line, no new code facts (any new symbol mentioned   -->
<!-- must exist in queries/ast_extract.rq output; the doc_quality     -->
<!-- court fails Phi_halluc > 0.001 otherwise). No tables, no         -->
<!-- signatures, no parameters, no error lists — AGENT-FORBIDDEN      -->
<!-- everywhere.                                                      -->
<!-- AGENT-COMMENTARY-END -->
