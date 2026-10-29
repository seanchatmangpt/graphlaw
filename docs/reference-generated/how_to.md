# How to: Using graphlaw

## Prerequisites


- @seanchatmangpt/graphlaw::build (script)

- @seanchatmangpt/graphlaw::gen (script)

- @seanchatmangpt/graphlaw::lint:pkg (script)

- @seanchatmangpt/graphlaw::sync-version (script)

- @seanchatmangpt/graphlaw::test (script)

- @seanchatmangpt/graphlaw::test:browser (script)

- @seanchatmangpt/graphlaw::typecheck (script)

- GraphlawEx.Application::name (str_key)

- GraphlawEx.Application::start (function)

- GraphlawEx.Application::strategy (str_key)

- GraphlawEx::abi_not_wired (str_key)

- GraphlawEx::admit (function)

- GraphlawEx::handle_call (function)

- GraphlawEx::init (function)

- GraphlawEx::instance (str_key)

- GraphlawEx::module (str_key)

- GraphlawEx::name (str_key)

- GraphlawEx::start_link (function)

- GraphlawEx::stop (function)

- GraphlawEx::store (str_key)

- GraphlawEx::wasm_load_failed (str_key)

- GraphlawEx::wasm_path (str_key)

- benches/shacl_boundary_cliff.rs::../ontologies/sql-ast.ttl (str_key)

- benches/shacl_boundary_cliff.rs::@prefix ex: <http://example.org/> . (str_key)

- benches/shacl_boundary_cliff.rs::@prefix sh: <http://www.w3.org/ns/shacl#> . (str_key)

- benches/sieve_cliff.rs::urn:s/ (str_key)

- benches/sieve_cliff.rs::urn:v/ (str_key)

- benches/sql_rdf_engine.rs::@prefix sql: <http://graphlaw.org/sql#> . (str_key)

- benches/sql_rdf_engine.rs::CREATE TABLE t(id INTEGER, name VARCHAR);  (str_key)

- benches/statutory_boundary_50ms.rs::@prefix : <http://example.org/sb#> .
@prefix math: <http://www.w3.org/2000/10/swap/math#> .
:trade_47 a :ProposedTrade .
:trade_47 :notional 25000000 ;
          :book :emea_deriv .
{ ?t a :ProposedTrade ;
     :notional ?n .
  ?n math:greaterThan 10000000 .
  ?t :book :emea_deriv }
  => { ?t :violates :Rule15c3_1 ; :requires :pretrade_review } .
{ ?t a :ProposedTrade ; :violates :Rule15c3_1 }
  => { :desk :halt :trade_47 } .
 (str_key)

- benches/statutory_boundary_50ms.rs::@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/sb#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
ex:TradeShape a sh:NodeShape ;
  sh:targetClass ex:Order ;
  sh:closed true ;
  sh:property [
    sh:path rdf:type ; sh:minCount 1 ; sh:maxCount 1 ; sh:hasValue ex:Order
  ] ;
  sh:property [
    sh:path ex:id ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] ;
  sh:property [
    sh:path ex:acct ; sh:minCount 1 ; sh:maxCount 2 ; sh:nodeKind sh:IRI
  ] ;
  sh:property [
    sh:path ex:qty ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] .
 (str_key)

- benches/statutory_boundary_50ms.rs::SB_DEBUG (env_key)

- benches/tc_hot_01.rs::TC_HOT_FULL (env_key)

- benches/tc_hot_01.rs::TC_HOT_INJECT_OUTLIER_NS (env_key)

- benches/tc_hot_01.rs::TC_HOT_SPC_EXPORT (env_key)

- benches/tc_hot_02.rs::TC_HOT_CORPUS_PATH (env_key)

- benches/tc_hot_02.rs::TC_HOT_FULL (env_key)

- benches/tc_hot_02.rs::thread_affinity=unavailable_stable_rust_macos (core_affinity crate not added) (str_key)

- examples/allen_datalog.rs::p (str_key)

- examples/asp_tableaux_vector_gen.rs::3.0.1 (str_key)


## Steps


1. Use `build` from `@seanchatmangpt/graphlaw`.

2. Use `gen` from `@seanchatmangpt/graphlaw`.

3. Use `lint:pkg` from `@seanchatmangpt/graphlaw`.

4. Use `sync-version` from `@seanchatmangpt/graphlaw`.

5. Use `test` from `@seanchatmangpt/graphlaw`.

6. Use `test:browser` from `@seanchatmangpt/graphlaw`.

7. Use `typecheck` from `@seanchatmangpt/graphlaw`.

8. Use `admit` from `GraphlawEx`.

9. Use `handle_call` from `GraphlawEx`.

10. Use `init` from `GraphlawEx`.

11. Use `start_link` from `GraphlawEx`.

12. Use `stop` from `GraphlawEx`.


## Verified snippet

<!-- The snippet slot carries code copied from the extracted code surface -->
<!-- (doc:Claim rows whose doc:attribute is "snippet"), never agent prose. -->

```rust
// GraphlawEx :: admit
admit/2
```

<!-- AGENT-COMMENTARY-BEGIN -->
<!-- The ONLY region an agent may write into. Bounds: <= 12 lines,    -->
<!-- <= 100 chars/line, no new code facts (any new symbol mentioned   -->
<!-- must exist in queries/ast_extract.rq output; the doc_quality     -->
<!-- court fails Phi_halluc > 0.001 otherwise). No tables, no         -->
<!-- signatures, no parameters, no error lists — AGENT-FORBIDDEN      -->
<!-- everywhere.                                                      -->
<!-- AGENT-COMMENTARY-END -->
