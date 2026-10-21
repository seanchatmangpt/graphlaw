//! End-to-end demonstration: Parsing a SQL AST from a Turtle RDF graph,
//! validating structural invariants, compiling to parameterized SQL,
//! executing against embedded `frozen-duckdb`, and generating an `ExecutionReceipt`.
//!
//! Run with:
//! ```bash
//! cargo run --features duckdb --example sql_in_rdf_duckdb
//! ```

use graphlaw::duckdb_eval::GraphLawDuckDB;
use graphlaw::sql_ast::{
    AggregateFunction, AstError, CompiledQuery, Operator, RdfAstGraphReader, SelectStatement,
    SqlExpr, SqlLiteral, TableRef,
};
use purrdf::TermValue;
use std::collections::HashMap;

/// In-memory graph adapter implementing `RdfAstGraphReader` over `purrdf::RdfDataset`.
struct RdfDatasetReader {
    // Index: subject IRI -> predicate IRI -> objects (IRIs or Literal values with datatypes)
    outgoing: HashMap<String, HashMap<String, Vec<RdfTerm>>>,
}

#[derive(Clone, Debug)]
enum RdfTerm {
    Iri(String),
    Literal {
        lexical: String,
        datatype: Option<String>,
    },
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

    /// Recursively extracts a `SqlExpr` from an AST node in the graph.
    fn extract_expr(&self, node_iri: &String, depth: usize) -> Result<SqlExpr, AstError> {
        if depth > 32 {
            return Err(AstError::RecursionLimitExceeded(32));
        }

        // Check if node is a ColumnRef
        if let Some((col_name, _)) = self.get_datatype_literal(node_iri, "http://graphlaw.org/sql#columnName") {
            let alias = self
                .get_datatype_literal(node_iri, "http://graphlaw.org/sql#tableAlias")
                .map(|(a, _)| a);
            return Ok(SqlExpr::ColumnRef {
                column: col_name,
                table_alias: alias,
            });
        }

        // Check if node is a Literal
        if let Some((val_str, dtype)) = self.get_datatype_literal(node_iri, "http://graphlaw.org/sql#literalValue") {
            let lit = match dtype.as_deref() {
                Some("http://www.w3.org/2001/XMLSchema#integer") => {
                    SqlLiteral::Integer(val_str.parse().unwrap_or(0))
                }
                Some("http://www.w3.org/2001/XMLSchema#double") => {
                    SqlLiteral::Float(val_str.parse().unwrap_or(0.0))
                }
                Some("http://www.w3.org/2001/XMLSchema#boolean") => {
                    SqlLiteral::Boolean(val_str == "true" || val_str == "1")
                }
                _ => SqlLiteral::String(val_str),
            };
            return Ok(SqlExpr::Literal(lit));
        }

        // Check if node is an Alias
        if let Some((alias_name, _)) = self.get_datatype_literal(node_iri, "http://graphlaw.org/sql#aliasName") {
            let target_expr_node = self
                .get_single_object(node_iri, "http://graphlaw.org/sql#expression")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#expression".to_string(),
                        node_iri.to_string(),
                    )
                })?;
            let inner_expr = self.extract_expr(&target_expr_node, depth + 1)?;
            return Ok(SqlExpr::Alias {
                expr: Box::new(inner_expr),
                alias: alias_name,
            });
        }

        // Check if node is a FunctionCall
        if let Some(func_node) = self.get_single_object(node_iri, "http://graphlaw.org/sql#function") {
            let function = AggregateFunction::from_iri(&func_node)?;
            let arg_node = self
                .get_single_object(node_iri, "http://graphlaw.org/sql#argument")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#argument".to_string(),
                        node_iri.to_string(),
                    )
                })?;
            let argument = self.extract_expr(&arg_node, depth + 1)?;
            return Ok(SqlExpr::FunctionCall {
                function,
                argument: Box::new(argument),
            });
        }

        // Check if node is a BinaryExpression
        if let Some(op_node) = self.get_single_object(node_iri, "http://graphlaw.org/sql#operator") {
            let op = Operator::from_iri(&op_node)?;

            let left_node = self
                .get_single_object(node_iri, "http://graphlaw.org/sql#leftOperand")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#leftOperand".to_string(),
                        node_iri.to_string(),
                    )
                })?;
            let right_node = self
                .get_single_object(node_iri, "http://graphlaw.org/sql#rightOperand")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#rightOperand".to_string(),
                        node_iri.to_string(),
                    )
                })?;

            let left = self.extract_expr(&left_node, depth + 1)?;
            let right = self.extract_expr(&right_node, depth + 1)?;

            return Ok(SqlExpr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Err(AstError::MalformedNode(node_iri.to_string()))
    }

    /// Extracts a full `SelectStatement` AST from a target query node.
    pub fn extract_select_statement(&self, query_node: &String) -> Result<SelectStatement, AstError> {
        let table_ref_node = self
            .get_single_object(query_node, "http://graphlaw.org/sql#fromTable")
            .ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#fromTable".to_string(),
                    query_node.to_string(),
                )
            })?;

        let (table_name, _) = self
            .get_datatype_literal(&table_ref_node, "http://graphlaw.org/sql#tableName")
            .ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#tableName".to_string(),
                    table_ref_node.clone(),
                )
            })?;

        let table_alias = self
            .get_datatype_literal(&table_ref_node, "http://graphlaw.org/sql#tableAlias")
            .map(|(a, _)| a);

        let from = TableRef {
            name: table_name,
            alias: table_alias,
        };

        // Unroll projection list (rdf:List)
        let select_head = self
            .get_single_object(query_node, "http://graphlaw.org/sql#selectList")
            .ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#selectList".to_string(),
                    query_node.to_string(),
                )
            })?;
        let select_nodes = self.unroll_rdf_list(&select_head, 64)?;
        let mut select_list = Vec::with_capacity(select_nodes.len());
        for node in select_nodes {
            select_list.push(self.extract_expr(&node, 0)?);
        }

        // Where clause (optional)
        let where_clause = if let Some(where_node) =
            self.get_single_object(query_node, "http://graphlaw.org/sql#whereClause")
        {
            Some(self.extract_expr(&where_node, 0)?)
        } else {
            None
        };

        // Group by (optional rdf:List)
        let group_by = if let Some(group_head) =
            self.get_single_object(query_node, "http://graphlaw.org/sql#groupBy")
        {
            let group_nodes = self.unroll_rdf_list(&group_head, 64)?;
            let mut list = Vec::with_capacity(group_nodes.len());
            for node in group_nodes {
                list.push(self.extract_expr(&node, 0)?);
            }
            list
        } else {
            Vec::new()
        };

        // Limit & offset
        let limit = self
            .get_datatype_literal(query_node, "http://graphlaw.org/sql#limit")
            .and_then(|(l, _)| l.parse().ok());
        let offset = self
            .get_datatype_literal(query_node, "http://graphlaw.org/sql#offset")
            .and_then(|(o, _)| o.parse().ok());

        Ok(SelectStatement {
            from,
            joins: vec![],
            select_list,
            where_clause,
            group_by,
            limit,
            offset,
        })
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
    println!("=== 1. Initializing Embedded frozen-duckdb Substrate ===");
    let db = GraphLawDuckDB::open_in_memory()?;

    // Seed mock telemetry state directly into the columnar store
    db.raw_connection().execute_batch(
        "CREATE TABLE telemetry_events (
            batch_id VARCHAR,
            service_name VARCHAR,
            cost BIGINT,
            latency_ms INTEGER
        );
        INSERT INTO telemetry_events VALUES 
            ('batch-alpha', 'auth-service', 140, 480),
            ('batch-alpha', 'auth-service', 85, 520),
            ('batch-beta',  'payment-gw',   310, 610),
            ('batch-gamma', 'ledger-sync',  95, 300);",
    )?;
    println!("  -> Ingested seed telemetry into 'telemetry_events' table.\n");

    println!("=== 2. Parsing Declarative SQL AST Turtle Subgraph ===");
    let turtle_data = r#"
        @prefix sql: <http://graphlaw.org/sql#> .
        @prefix fn:  <http://graphlaw.org/functions#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix ex:  <http://example.org/queries#> .

        ex:AdmissibilityQuery a sql:SelectStatement ;
            sql:fromTable [
                sql:tableName "telemetry_events" ;
                sql:tableAlias "t"
            ] ;
            sql:selectList (
                [ sql:columnName "batch_id" ]
                [
                    sql:aliasName "total_cost" ;
                    sql:expression [
                        sql:function fn:SUM ;
                        sql:argument [ sql:columnName "cost" ]
                    ]
                ]
            ) ;
            sql:whereClause [
                sql:operator sql:GreaterThan ;
                sql:leftOperand  [ sql:columnName "latency_ms" ] ;
                sql:rightOperand [
                    sql:literalValue "400"^^xsd:integer
                ]
            ] ;
            sql:groupBy (
                [ sql:columnName "batch_id" ]
            ) ;
            sql:limit 10 .
    "#;

    let reader = RdfDatasetReader::from_turtle(turtle_data)?;
    println!("  -> Parsed Turtle dataset successfully into RdfDatasetReader.");

    let query_node = "http://example.org/queries#AdmissibilityQuery".to_string();

    println!("=== 3. Extracting and Validating AST in GraphLaw ===");
    let ast = reader.extract_select_statement(&query_node)?;
    let compiled: CompiledQuery = ast.compile()?;

    println!("  -> Validated AST conformance (identifiers safe from injection).");
    println!("  -> Serialized SQL : {}", compiled.sql);
    println!("  -> Bound Params   : {:?}", compiled.params);

    println!("\n=== 4. Executing via frozen-duckdb & Producing ExecutionReceipt ===");
    let receipt = db.eval_ast_query(
        &query_node,
        &compiled,
        "http://example.org/data",
        "http://example.org/vocab",
    )?;

    println!("  -> Rows Emitted    : {}", receipt.rows_emitted);
    println!("  -> Execution Time  : {:.3} ms", receipt.execution_ms);
    println!("  -> Query Hash (BLAKE3): {:02x?}", &receipt.query_hash[0..8]);
    println!("\n=== 5. Materialized RDF Statements (Grounding Tabular State) ===");
    for stmt in &receipt.result_triples {
        println!("  <{}> <{}> {:?}", stmt.subject, stmt.predicate, stmt.object);
    }

    Ok(())
}
