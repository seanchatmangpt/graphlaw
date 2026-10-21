//! Architectural Case Study: Regulated Life-Sciences Cold-Chain Biologics Verification
//!
//! Demonstrates FDA 21 CFR Part 11 / EU GMP Annex 11 compliant audit execution:
//! 1. Telemetry ingestion as an Object-Centric Event Log (OCEL v2) in partitioned Parquet.
//! 2. Declarative SQL AST representation with multi-table JOINs in Turtle RDF.
//! 3. SHACL pre-flight gate guaranteeing anti-injection and query shape invariants.
//! 4. Vectorized pushdown via embedded `frozen-duckdb` scanning Parquet views with zone maps.
//! 5. Fixed-point rule deduction deriving `bio:ReleasedBatch` vs `bio:QuarantinedBatch`
//!    committed to an immutable cryptographic BLAKE3 receipt.

use graphlaw::duckdb_eval::GraphLawDuckDB;
use graphlaw::sql_ast::{
    validate_ast_shacl, CompiledQuery, RdfAstGraphReader,
};
use purrdf::TermValue;
use std::collections::HashMap;
use std::fs;

struct RdfDatasetReader {
    outgoing: HashMap<String, HashMap<String, Vec<RdfTerm>>>,
}

#[derive(Clone, Debug)]
enum RdfTerm {
    Iri(String),
    Literal { lexical: String, datatype: Option<String> },
}

impl RdfDatasetReader {
    fn from_turtle(turtle: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let dataset = purrdf::parse_dataset(turtle.as_bytes(), "text/turtle", None)
            .map_err(|e| format!("Failed to parse turtle: {:?}", e))?;
        let mut outgoing: HashMap<String, HashMap<String, Vec<RdfTerm>>> = HashMap::new();

        for quad in dataset.quads() {
            let subj = match dataset.term_value(quad.s) {
                TermValue::Iri(i) => i,
                TermValue::Blank { label, scope } => format!("_:{}_{}", label, scope.0),
                _ => continue,
            };

            let pred = match dataset.term_value(quad.p) {
                TermValue::Iri(i) => i,
                _ => continue,
            };

            let term = match dataset.term_value(quad.o) {
                TermValue::Iri(i) => RdfTerm::Iri(i),
                TermValue::Blank { label, scope } => RdfTerm::Iri(format!("_:{}_{}", label, scope.0)),
                TermValue::Literal { lexical_form, datatype, .. } => RdfTerm::Literal {
                    lexical: lexical_form,
                    datatype: Some(datatype),
                },
                _ => continue,
            };

            outgoing
                .entry(subj)
                .or_default()
                .entry(pred)
                .or_default()
                .push(term);
        }

        Ok(Self { outgoing })
    }
}

impl RdfAstGraphReader for RdfDatasetReader {
    type Node = String;

    fn get_single_object(&self, subject: &Self::Node, predicate: &str) -> Option<Self::Node> {
        self.outgoing
            .get(subject)?
            .get(predicate)?
            .first()
            .and_then(|term| match term {
                RdfTerm::Iri(i) => Some(i.clone()),
                _ => None,
            })
    }

    fn get_datatype_literal(
        &self,
        subject: &Self::Node,
        predicate: &str,
    ) -> Option<(String, Option<String>)> {
        self.outgoing
            .get(subject)?
            .get(predicate)?
            .first()
            .and_then(|term| match term {
                RdfTerm::Literal { lexical, datatype } => Some((lexical.clone(), datatype.clone())),
                _ => None,
            })
    }

    fn get_iri(&self, node: &Self::Node) -> Option<String> {
        Some(node.clone())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("===============================================================================");
    println!("CASE STUDY: REGULATED LIFE-SCIENCES COLD-CHAIN BIOLOGICS VERIFICATION");
    println!("Compliance Context: FDA 21 CFR Part 11 / EU GMP Annex 11");
    println!("Substrates: OCEL v2 (Parquet) -> GraphLaw AST -> frozen-duckdb -> BLAKE3 Receipt");
    println!("===============================================================================\n");

    // 1. Setup temporary directory and mock OCEL v2 partitioned Parquet logs
    let base_dir = format!("/tmp/biopharma_ocel_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let _ = fs::remove_dir_all(&base_dir);
    fs::create_dir_all(&base_dir)?;

    let db = GraphLawDuckDB::open_in_memory()?;

    println!("=== Phase 1: Ingestion & Substrate Binding (OCEL v2 Columnar Layout) ===");
    // Generate events table (partitioned by facility_id)
    let gen_events = format!(
        "COPY (
            -- BATCH-MAB-9042: Excursions above -20C: duration 140s, max temp -18.2C (COMPLIANT)
            SELECT 'FAC-US-01' AS facility_id, 'EVT-101' AS event_id, 'ColdChainExcursion' AS event_type, -18.2 AS temp_celsius, 140 AS duration_seconds
            UNION ALL
            SELECT 'FAC-US-01' AS facility_id, 'EVT-102' AS event_id, 'ColdChainExcursion' AS event_type, -25.0 AS temp_celsius, 500 AS duration_seconds
            UNION ALL
            -- BATCH-MAB-9043: Excursions above -20C: duration 450s, max temp -12.4C (BREACH / QUARANTINE)
            SELECT 'FAC-US-01' AS facility_id, 'EVT-201' AS event_id, 'ColdChainExcursion' AS event_type, -12.4 AS temp_celsius, 450 AS duration_seconds
        ) TO '{}/events' (FORMAT PARQUET, PARTITION_BY (facility_id));",
        base_dir
    );
    db.raw_connection().execute_batch(&gen_events)?;

    // Generate event-to-object relationships table
    let gen_rel = format!(
        "COPY (
            SELECT 'EVT-101' AS event_id, 'BATCH-MAB-9042' AS object_id, 'contains_product' AS qualifier
            UNION ALL
            SELECT 'EVT-102' AS event_id, 'BATCH-MAB-9042' AS object_id, 'contains_product' AS qualifier
            UNION ALL
            SELECT 'EVT-201' AS event_id, 'BATCH-MAB-9043' AS object_id, 'contains_product' AS qualifier
        ) TO '{}/event_object_map.parquet' (FORMAT PARQUET);",
        base_dir
    );
    db.raw_connection().execute_batch(&gen_rel)?;

    // Mount virtual zero-copy views
    db.register_parquet_view("ocel_events", &format!("{}/events/**/*.parquet", base_dir), true)?;
    db.register_parquet_view("ocel_event_object_rel", &format!("{}/event_object_map.parquet", base_dir), false)?;
    println!("  -> Mounted /events/**/*.parquet as view 'ocel_events' (Hive partition: facility_id)");
    println!("  -> Mounted /event_object_map.parquet as view 'ocel_event_object_rel'");

    println!("\n=== Phase 2: Structural Verification via SHACL Pre-Flight Gate ===");
    let shapes_ttl = include_str!("../ontologies/sql-ast.ttl");

    let query_ast_ttl = r#"
        @prefix sql:  <http://graphlaw.org/sql#> .
        @prefix fn:   <http://graphlaw.org/functions#> .
        @prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
        @prefix ex:   <http://biopharma.org/queries#> .

        ex:ColdChainExcursionQuery a sql:SelectStatement ;
            sql:fromTable [
                sql:tableName "ocel_events" ;
                sql:tableAlias "e"
            ] ;
            sql:joinClause [
                sql:joinType sql:InnerJoin ;
                sql:joinTable [
                    sql:tableName "ocel_event_object_rel" ;
                    sql:tableAlias "r"
                ] ;
                sql:joinCondition [
                    sql:operator sql:Equals ;
                    sql:leftOperand  [ sql:columnName "event_id" ; sql:tableAlias "e" ] ;
                    sql:rightOperand [ sql:columnName "event_id" ; sql:tableAlias "r" ]
                ]
            ] ;
            sql:selectList (
                [ sql:columnName "object_id" ; sql:tableAlias "r" ]
                [
                    sql:aliasName "max_excursion_temp" ;
                    sql:expression [
                        sql:function fn:MAX ;
                        sql:argument [ sql:columnName "temp_celsius" ; sql:tableAlias "e" ]
                    ]
                ]
                [
                    sql:aliasName "total_excursion_duration" ;
                    sql:expression [
                        sql:function fn:SUM ;
                        sql:argument [ sql:columnName "duration_seconds" ; sql:tableAlias "e" ]
                    ]
                ]
            ) ;
            sql:whereClause [
                sql:operator sql:And ;
                sql:leftOperand [
                    sql:operator sql:GreaterThan ;
                    sql:leftOperand  [ sql:columnName "temp_celsius" ; sql:tableAlias "e" ] ;
                    sql:rightOperand [ sql:literalValue "-20.0"^^xsd:double ]
                ] ;
                sql:rightOperand [
                    sql:operator sql:Equals ;
                    sql:leftOperand  [ sql:columnName "qualifier" ; sql:tableAlias "r" ] ;
                    sql:rightOperand [ sql:literalValue "contains_product"^^xsd:string ]
                ]
            ] ;
            sql:groupBy (
                [ sql:columnName "object_id" ; sql:tableAlias "r" ]
            ) .
    "#;

    validate_ast_shacl(query_ast_ttl, shapes_ttl)?;
    println!("  -> AST conformance verified: adherence to sql:SelectStatementShape and token safety rules.");

    println!("\n=== Phase 3: Extracting AST & Compiling Parameterized SQL Pushdown ===");
    let reader = RdfDatasetReader::from_turtle(query_ast_ttl)?;
    let query_node = "http://biopharma.org/queries#ColdChainExcursionQuery".to_string();
    let ast = reader.extract_select_statement(&query_node)?;
    let compiled: CompiledQuery = ast.compile()?;

    println!("  -> Serialized SQL:\n     {}", compiled.sql);
    println!("  -> Parameter Vector: {:?}", compiled.params);

    println!("\n=== Phase 4: Vectorized Execution via frozen-duckdb & Receipt Minting ===");
    let receipt = db.eval_ast_query(
        &query_node,
        &compiled,
        "http://biopharma.org/data",
        "http://biopharma.org/vocab",
    )?;

    println!("  -> Rows Emitted       : {}", receipt.rows_emitted);
    println!("  -> Pushdown Duration  : {:.3} ms", receipt.execution_ms);
    println!("  -> BLAKE3 Query Hash  : {}", hex::encode(receipt.query_hash));

    println!("\n=== Phase 5: Materialized Triples (Bridged to Deductive State) ===");
    for stmt in &receipt.result_triples {
        println!("  <{}> <{}> {:?}", stmt.subject, stmt.predicate, stmt.object);
    }

    println!("\n=== Phase 6: Deductive GxP State Admission (FDA 21 CFR 211.165) ===");
    // Invariants: Total excursion duration <= 300s AND Max temp < -15.0C
    let batches = vec!["BATCH-MAB-9042", "BATCH-MAB-9043"];

    for target_batch in batches {
        let mut row_subj = None;
        for t in &receipt.result_triples {
            if t.predicate == "http://biopharma.org/vocab/object_id" {
                if let TermValue::Literal { lexical_form, .. } = &t.object {
                    if lexical_form == target_batch {
                        row_subj = Some(t.subject.clone());
                        break;
                    }
                }
            }
        }

        let subj = row_subj.expect("Batch row should exist in result");
        let mut duration = 0i64;
        let mut max_temp = 0.0f64;

        for t in &receipt.result_triples {
            if t.subject == subj {
                if t.predicate == "http://biopharma.org/vocab/total_excursion_duration" {
                    if let TermValue::Literal { lexical_form, .. } = &t.object {
                        duration = lexical_form.parse().unwrap_or(0);
                    }
                }
                if t.predicate == "http://biopharma.org/vocab/max_excursion_temp" {
                    if let TermValue::Literal { lexical_form, .. } = &t.object {
                        max_temp = lexical_form.parse().unwrap_or(0.0);
                    }
                }
            }
        }

        print!("  Checking Batch {}: duration={}s, max_temp={:.1}C -> ", target_batch, duration, max_temp);
        if duration <= 300 && max_temp < -15.0 {
            println!("PASS [bio:ReleasedBatch]");
            println!("    Assert: <http://biopharma.org/batches/{}> a bio:ReleasedBatch ;", target_batch);
            println!("            bio:releaseApproval \"FDA 21 CFR 211.165 Compliant\" ;");
            println!("            gl:auditProof \"{}\"^^xsd:hexBinary .\n", hex::encode(receipt.query_hash));
        } else {
            println!("FAIL [bio:QuarantinedBatch] (FAIL-CLOSED)");
            println!("    Assert: <http://biopharma.org/batches/{}> a bio:QuarantinedBatch ;", target_batch);
            println!("            bio:quarantineReason \"Cold-chain cumulative duration/temp breach\" ;");
            println!("            gl:auditProof \"{}\"^^xsd:hexBinary .\n", hex::encode(receipt.query_hash));
        }
    }

    let _ = fs::remove_dir_all(&base_dir);
    println!("=== GxP Verification Execution Completed Successfully ===");
    Ok(())
}
