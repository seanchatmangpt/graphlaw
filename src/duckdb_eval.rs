//! Embedded execution engine bridging GraphLaw's AST compiler with `frozen-duckdb`.
//!
//! Evaluates compiled relational queries inside an embedded, memory-bounded DuckDB instance,
//! collects cryptographic execution receipts, and projects tabular results into RDF statements.

#![allow(missing_docs)]

use std::fmt;
use std::time::Instant;

use crate::sql_ast::{CompiledQuery, SqlLiteral};

// Re-exported or native types from purrdf
use purrdf::TermValue;

#[derive(Debug, Clone, PartialEq)]
pub struct RdfTriple {
    pub subject: String,
    pub predicate: String,
    pub object: TermValue,
}

#[derive(Debug)]
pub enum EvaluatorError {
    DuckDb(frozen_duckdb::DuckDBError),
    ConversionError {
        column: String,
        row: usize,
        details: String,
    },
    ReceiptError(String),
}

impl fmt::Display for EvaluatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuckDb(e) => write!(f, "DuckDB execution failure: {e}"),
            Self::ConversionError { column, row, details } => {
                write!(f, "Column conversion error for '{column}' at row {row}: {details}")
            }
            Self::ReceiptError(msg) => write!(f, "Receipt generation error: {msg}"),
        }
    }
}

impl std::error::Error for EvaluatorError {}

impl From<frozen_duckdb::DuckDBError> for EvaluatorError {
    fn from(err: frozen_duckdb::DuckDBError) -> Self {
        Self::DuckDb(err)
    }
}

/// Cryptographic and semantic proof of an executed relational query.
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionReceipt {
    /// IRI identifying the executed query AST node
    pub query_iri: String,
    /// Canonical BLAKE3 hash across the serialized SQL and its bound parameter vector
    pub query_hash: [u8; 32],
    /// Parameterized SQL string sent to frozen-duckdb
    pub sql_statement: String,
    /// Number of tabular rows emitted
    pub rows_emitted: usize,
    /// Execution wall-clock time in milliseconds
    pub execution_ms: f64,
    /// Downstream RDF statements materialized from the result vectors
    pub result_triples: Vec<RdfTriple>,
}

/// Embedded execution context wrapping a sovereign `frozen-duckdb` connection.
pub struct GraphLawDuckDB {
    conn: frozen_duckdb::Connection,
}

impl GraphLawDuckDB {
    /// Creates an in-memory embedded execution instance.
    pub fn open_in_memory() -> Result<Self, EvaluatorError> {
        let conn = frozen_duckdb::Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    /// Wraps an existing connection to a static/frozen catalog.
    pub fn from_connection(conn: frozen_duckdb::Connection) -> Self {
        Self { conn }
    }

    /// Accesses the underlying raw `frozen_duckdb::Connection` for table creation or seeding.
    pub fn raw_connection(&self) -> &frozen_duckdb::Connection {
        &self.conn
    }

    /// Executes a `CompiledQuery`, binds typed parameters to placeholders, computes the
    /// cryptographic execution receipt, and projects tabular rows into RDF triples.
    pub fn eval_ast_query(
        &self,
        query_iri: &str,
        compiled: &CompiledQuery,
        subject_iri_prefix: &str,
        predicate_iri_prefix: &str,
    ) -> Result<ExecutionReceipt, EvaluatorError> {
        let start_time = Instant::now();

        // 1. Compute deterministic query hash via BLAKE3 (SQL + canonical parameter representation)
        let mut hasher = blake3::Hasher::new();
        hasher.update(compiled.sql.as_bytes());
        for param in &compiled.params {
            match param {
                SqlLiteral::String(s) => {
                    hasher.update(&[0x01]);
                    hasher.update(s.as_bytes());
                }
                SqlLiteral::Integer(i) => {
                    hasher.update(&[0x02]);
                    hasher.update(&i.to_le_bytes());
                }
                SqlLiteral::Float(f) => {
                    hasher.update(&[0x03]);
                    hasher.update(&f.to_bits().to_le_bytes());
                }
                SqlLiteral::Boolean(b) => {
                    hasher.update(&[0x04]);
                    hasher.update(&[*b as u8]);
                }
                SqlLiteral::Null => {
                    hasher.update(&[0x00]);
                }
            }
        }
        let query_hash: [u8; 32] = *hasher.finalize().as_bytes();

        // 2. Prepare statement and bind positional parameters ($1, $2, ...)
        let mut stmt = self.conn.prepare(&compiled.sql)?;
        let duck_params: Vec<Box<dyn frozen_duckdb::ToSql>> = compiled
            .params
            .iter()
            .map(|p| -> Box<dyn frozen_duckdb::ToSql> {
                match p {
                    SqlLiteral::String(s) => Box::new(s.clone()),
                    SqlLiteral::Integer(i) => Box::new(*i),
                    SqlLiteral::Float(f) => Box::new(*f),
                    SqlLiteral::Boolean(b) => Box::new(*b),
                    SqlLiteral::Null => Box::new(Option::<String>::None),
                }
            })
            .collect();

        let param_refs: Vec<&dyn frozen_duckdb::ToSql> =
            duck_params.iter().map(|b| b.as_ref()).collect();

        // 3. Vectorized execution and columnar projection to RDF statements
        let mut rows = stmt.query(param_refs.as_slice())?;
        let col_count = rows.as_ref().map(|r| r.column_count()).unwrap_or(0);
        let column_names: Vec<String> = (0..col_count)
            .map(|i| {
                rows.as_ref()
                    .and_then(|r| r.column_name(i).ok().map(|s| s.to_string()))
                    .unwrap_or_else(|| format!("col_{i}"))
            })
            .collect();

        let mut result_triples = Vec::new();
        let mut row_index = 0;

        while let Some(row) = rows.next()? {
            row_index += 1;
            let row_subject = format!("{}/row_{}", subject_iri_prefix, row_index);

            for (col_idx, col_name) in column_names.iter().enumerate() {
                let predicate = format!("{}/{}", predicate_iri_prefix, col_name);

                let term: Option<TermValue> = if let Ok(val) = row.get::<_, i64>(col_idx) {
                    Some(TermValue::Literal {
                        lexical_form: val.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                        language: None,
                        direction: None,
                    })
                } else if let Ok(val) = row.get::<_, f64>(col_idx) {
                    Some(TermValue::Literal {
                        lexical_form: val.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#double".to_string(),
                        language: None,
                        direction: None,
                    })
                } else if let Ok(val) = row.get::<_, bool>(col_idx) {
                    Some(TermValue::Literal {
                        lexical_form: val.to_string(),
                        datatype: "http://www.w3.org/2001/XMLSchema#boolean".to_string(),
                        language: None,
                        direction: None,
                    })
                } else if let Ok(val) = row.get::<_, String>(col_idx) {
                    Some(TermValue::Literal {
                        lexical_form: val,
                        datatype: "http://www.w3.org/2001/XMLSchema#string".to_string(),
                        language: None,
                        direction: None,
                    })
                } else {
                    None
                };

                if let Some(object_term) = term {
                    result_triples.push(RdfTriple {
                        subject: row_subject.clone(),
                        predicate,
                        object: object_term,
                    });
                }
            }
        }

        let elapsed = start_time.elapsed();

        Ok(ExecutionReceipt {
            query_iri: query_iri.to_string(),
            query_hash,
            sql_statement: compiled.sql.clone(),
            rows_emitted: row_index,
            execution_ms: elapsed.as_secs_f64() * 1000.0,
            result_triples,
        })
    }

    /// Mounts a directory of partitioned Parquet files or a glob pattern as a 
    /// zero-copy virtual view.
    ///
    /// - `view_name`: Relational table identifier exposed to the AST (e.g., "telemetry_events").
    /// - `glob_pattern`: Path pattern (e.g., "data/telemetry/tenant_id=*/*.parquet").
    /// - `hive_partitioning`: Enables automatic partition column extraction from paths.
    pub fn register_parquet_view(
        &self,
        view_name: &str,
        glob_pattern: &str,
        hive_partitioning: bool,
    ) -> Result<(), EvaluatorError> {
        crate::sql_ast::validate_identifier(view_name)
            .map_err(|e| EvaluatorError::ReceiptError(e.to_string()))?;

        // Sanitize glob pattern string to prevent escaping the single-quoted literal
        let sanitized_pattern = glob_pattern.replace('\'', "''");

        let sql = format!(
            "CREATE OR REPLACE VIEW {} AS SELECT * FROM read_parquet('{}', hive_partitioning = {});",
            view_name,
            sanitized_pattern,
            if hive_partitioning { "true" } else { "false" }
        );

        self.conn.execute_batch(&sql)?;
        Ok(())
    }

    /// Verifies zone map pruning by retrieving DuckDB's physical execution plan.
    pub fn explain_ast_query(&self, compiled: &CompiledQuery) -> Result<String, EvaluatorError> {
        let mut stmt = self.conn.prepare(&format!("EXPLAIN {}", compiled.sql))?;
        let duck_params: Vec<Box<dyn frozen_duckdb::ToSql>> = compiled
            .params
            .iter()
            .map(|p| -> Box<dyn frozen_duckdb::ToSql> {
                match p {
                    SqlLiteral::String(s) => Box::new(s.clone()),
                    SqlLiteral::Integer(i) => Box::new(*i),
                    SqlLiteral::Float(f) => Box::new(*f),
                    SqlLiteral::Boolean(b) => Box::new(*b),
                    SqlLiteral::Null => Box::new(Option::<String>::None),
                }
            })
            .collect();

        let param_refs: Vec<&dyn frozen_duckdb::ToSql> =
            duck_params.iter().map(|b| b.as_ref()).collect();

        let mut rows = stmt.query(param_refs.as_slice())?;

        let mut plan = String::new();
        while let Some(row) = rows.next()? {
            let line: String = row.get(1)?;
            plan.push_str(&line);
            plan.push('\n');
        }
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql_ast::{
        AggregateFunction, Operator, SelectStatement, SqlExpr, TableRef,
    };

    #[test]
    fn test_eval_ast_query_in_memory() {
        let db = GraphLawDuckDB::open_in_memory().expect("Failed to initialize DuckDB");

        // Prepare test schema & seed data
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE telemetry (
                    batch_id VARCHAR,
                    cost BIGINT,
                    latency INTEGER
                );
                INSERT INTO telemetry VALUES ('batch-001', 120, 450);
                INSERT INTO telemetry VALUES ('batch-001', 80, 600);
                INSERT INTO telemetry VALUES ('batch-002', 250, 720);",
            )
            .expect("Failed to insert seed telemetry");

        // SELECT batch_id, SUM(cost) AS total_cost FROM telemetry WHERE latency > $1 GROUP BY batch_id
        let ast = SelectStatement {
            from: TableRef {
                name: "telemetry".to_string(),
                alias: None,
            },
            select_list: vec![
                SqlExpr::ColumnRef {
                    column: "batch_id".to_string(),
                    table_alias: None,
                },
                SqlExpr::Alias {
                    expr: Box::new(SqlExpr::FunctionCall {
                        function: AggregateFunction::Sum,
                        argument: Box::new(SqlExpr::ColumnRef {
                            column: "cost".to_string(),
                            table_alias: None,
                        }),
                    }),
                    alias: "total_cost".to_string(),
                },
            ],
            where_clause: Some(SqlExpr::Binary {
                op: Operator::GreaterThan,
                left: Box::new(SqlExpr::ColumnRef {
                    column: "latency".to_string(),
                    table_alias: None,
                }),
                right: Box::new(SqlExpr::Literal(SqlLiteral::Integer(500))),
            }),
            group_by: vec![SqlExpr::ColumnRef {
                column: "batch_id".to_string(),
                table_alias: None,
            }],
            limit: None,
            offset: None,
        };

        let compiled = ast.compile().expect("Compilation failed");
        let query_iri = "http://example.org/queries/q1";

        let receipt = db
            .eval_ast_query(
                &query_iri,
                &compiled,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .expect("Execution failed");

        assert_eq!(receipt.rows_emitted, 2);
        assert_eq!(
            receipt.sql_statement,
            "SELECT batch_id, SUM(cost) AS total_cost FROM telemetry WHERE (latency > $1) GROUP BY batch_id"
        );
        assert!(!receipt.result_triples.is_empty());
        assert_ne!(receipt.query_hash, [0u8; 32]);
    }
}
