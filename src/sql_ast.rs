//! Declarative SQL AST representation and deterministic compiler for GraphLaw.
//!
//! Provides in-memory AST terms matching the `ontologies/sql-ast.ttl` ontology,
//! identifier validation eliminating structural injection, parameterized SQL serialization,
//! and recursive bounded RDF collection unrolling.

#![allow(missing_docs)]

use std::collections::HashSet;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum AstError {
    InvalidIdentifier(String),
    MissingProperty(String, String),
    RecursionLimitExceeded(usize),
    MalformedList,
    UnsupportedTerm(String),
    MalformedNode(String),
}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier(s) => write!(f, "Invalid SQL identifier: {s}"),
            Self::MissingProperty(prop, node) => write!(f, "Missing required property '{prop}' on node {node}"),
            Self::RecursionLimitExceeded(limit) => write!(f, "Recursion depth limit exceeded ({limit}) while traversing AST"),
            Self::MalformedList => write!(f, "Malformed RDF list: circular reference or unclosed node"),
            Self::UnsupportedTerm(term) => write!(f, "Unsupported operator or function IRI: {term}"),
            Self::MalformedNode(node) => write!(f, "Unexpected terminal node: {node}"),
        }
    }
}

impl std::error::Error for AstError {}

/// Validates strict SQL identifier semantics to eliminate structural injection.
/// Conforms to `^[a-zA-Z_][a-zA-Z0-9_]*$`.
pub fn validate_identifier(ident: &str) -> Result<&str, AstError> {
    if ident.is_empty() {
        return Err(AstError::InvalidIdentifier(ident.to_string()));
    }
    let mut chars = ident.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(AstError::InvalidIdentifier(ident.to_string()));
    }
    for c in chars {
        if !(c.is_ascii_alphanumeric() || c == '_') {
            return Err(AstError::InvalidIdentifier(ident.to_string()));
        }
    }
    Ok(ident)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEq,
    LessThan,
    LessThanOrEq,
    And,
    Or,
    Like,
}

impl Operator {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Equals => "=",
            Self::NotEquals => "<>",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEq => ">=",
            Self::LessThan => "<",
            Self::LessThanOrEq => "<=",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Like => "LIKE",
        }
    }

    pub fn from_iri(iri: &str) -> Result<Self, AstError> {
        match iri {
            "http://graphlaw.org/sql#Equals" => Ok(Self::Equals),
            "http://graphlaw.org/sql#NotEquals" => Ok(Self::NotEquals),
            "http://graphlaw.org/sql#GreaterThan" => Ok(Self::GreaterThan),
            "http://graphlaw.org/sql#GreaterThanOrEq" => Ok(Self::GreaterThanOrEq),
            "http://graphlaw.org/sql#LessThan" => Ok(Self::LessThan),
            "http://graphlaw.org/sql#LessThanOrEq" => Ok(Self::LessThanOrEq),
            "http://graphlaw.org/sql#And" => Ok(Self::And),
            "http://graphlaw.org/sql#Or" => Ok(Self::Or),
            "http://graphlaw.org/sql#Like" => Ok(Self::Like),
            _ => Err(AstError::UnsupportedTerm(iri.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

impl AggregateFunction {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Count => "COUNT",
            Self::Sum => "SUM",
            Self::Avg => "AVG",
            Self::Min => "MIN",
            Self::Max => "MAX",
        }
    }

    pub fn from_iri(iri: &str) -> Result<Self, AstError> {
        match iri {
            "http://graphlaw.org/functions#COUNT" => Ok(Self::Count),
            "http://graphlaw.org/functions#SUM" => Ok(Self::Sum),
            "http://graphlaw.org/functions#AVG" => Ok(Self::Avg),
            "http://graphlaw.org/functions#MIN" => Ok(Self::Min),
            "http://graphlaw.org/functions#MAX" => Ok(Self::Max),
            _ => Err(AstError::UnsupportedTerm(iri.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SqlLiteral {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SqlExpr {
    ColumnRef {
        column: String,
        table_alias: Option<String>,
    },
    Literal(SqlLiteral),
    Binary {
        op: Operator,
        left: Box<SqlExpr>,
        right: Box<SqlExpr>,
    },
    FunctionCall {
        function: AggregateFunction,
        argument: Box<SqlExpr>,
    },
    Alias {
        expr: Box<SqlExpr>,
        alias: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRef {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub from: TableRef,
    pub select_list: Vec<SqlExpr>,
    pub where_clause: Option<SqlExpr>,
    pub group_by: Vec<SqlExpr>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledQuery {
    pub sql: String,
    pub params: Vec<SqlLiteral>,
}

impl SelectStatement {
    const MAX_DEPTH: usize = 64;

    /// Compiles the AST into parameterized SQL compatible with DuckDB prepared statements ($1, $2, ...).
    pub fn compile(&self) -> Result<CompiledQuery, AstError> {
        validate_identifier(&self.from.name)?;
        if let Some(alias) = &self.from.alias {
            validate_identifier(alias)?;
        }

        let mut params = Vec::new();
        let mut sql = String::with_capacity(256);

        sql.push_str("SELECT ");
        if self.select_list.is_empty() {
            sql.push('*');
        } else {
            for (idx, item) in self.select_list.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(", ");
                }
                Self::serialize_expr(item, &mut sql, &mut params, 0)?;
            }
        }

        sql.push_str(" FROM ");
        sql.push_str(&self.from.name);
        if let Some(alias) = &self.from.alias {
            sql.push(' ');
            sql.push_str(alias);
        }

        if let Some(where_clause) = &self.where_clause {
            sql.push_str(" WHERE ");
            Self::serialize_expr(where_clause, &mut sql, &mut params, 0)?;
        }

        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            for (idx, item) in self.group_by.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(", ");
                }
                Self::serialize_expr(item, &mut sql, &mut params, 0)?;
            }
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        Ok(CompiledQuery { sql, params })
    }

    fn serialize_expr(
        expr: &SqlExpr,
        sql: &mut String,
        params: &mut Vec<SqlLiteral>,
        depth: usize,
    ) -> Result<(), AstError> {
        if depth > Self::MAX_DEPTH {
            return Err(AstError::RecursionLimitExceeded(Self::MAX_DEPTH));
        }

        match expr {
            SqlExpr::ColumnRef { column, table_alias } => {
                validate_identifier(column)?;
                if let Some(alias) = table_alias {
                    validate_identifier(alias)?;
                    sql.push_str(alias);
                    sql.push('.');
                }
                sql.push_str(column);
            }
            SqlExpr::Literal(lit) => {
                params.push(lit.clone());
                sql.push('$');
                sql.push_str(&params.len().to_string());
            }
            SqlExpr::Binary { op, left, right } => {
                sql.push('(');
                Self::serialize_expr(left, sql, params, depth + 1)?;
                sql.push(' ');
                sql.push_str(op.as_sql());
                sql.push(' ');
                Self::serialize_expr(right, sql, params, depth + 1)?;
                sql.push(')');
            }
            SqlExpr::FunctionCall { function, argument } => {
                sql.push_str(function.as_sql());
                sql.push('(');
                Self::serialize_expr(argument, sql, params, depth + 1)?;
                sql.push(')');
            }
            SqlExpr::Alias { expr, alias } => {
                validate_identifier(alias)?;
                Self::serialize_expr(expr, sql, params, depth + 1)?;
                sql.push_str(" AS ");
                sql.push_str(alias);
            }
        }
        Ok(())
    }
}

// =============================================================================
// Graph Unrolling Helper (purrdf / RDF list extraction)
// =============================================================================

pub trait RdfAstGraphReader {
    type Node: Clone + fmt::Display;

    fn get_single_object(&self, subject: &Self::Node, predicate: &str) -> Option<Self::Node>;
    fn get_datatype_literal(&self, subject: &Self::Node, predicate: &str) -> Option<(String, Option<String>)>;
    fn get_iri(&self, node: &Self::Node) -> Option<String>;

    /// Deterministically unwraps an rdf:List (rdf:first / rdf:rest chain) up to max_items.
    fn unroll_rdf_list(&self, head: &Self::Node, max_items: usize) -> Result<Vec<Self::Node>, AstError> {
        let rdf_first = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
        let rdf_rest = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
        let rdf_nil = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";

        let mut items = Vec::new();
        let mut curr = head.clone();
        let mut visited = HashSet::new();

        while let Some(iri) = self.get_iri(&curr) {
            if iri == rdf_nil {
                break;
            }

            if items.len() >= max_items {
                return Err(AstError::RecursionLimitExceeded(max_items));
            }

            if !visited.insert(iri.clone()) {
                return Err(AstError::MalformedList);
            }

            let first = self.get_single_object(&curr, rdf_first)
                .ok_or_else(|| AstError::MissingProperty(rdf_first.to_string(), curr.to_string()))?;
            items.push(first);

            curr = self.get_single_object(&curr, rdf_rest)
                .ok_or_else(|| AstError::MissingProperty(rdf_rest.to_string(), curr.to_string()))?;
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_identifier_compilation() {
        let stmt = SelectStatement {
            from: TableRef {
                name: "telemetry_events".to_string(),
                alias: Some("t".to_string()),
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
            limit: Some(100),
            offset: None,
        };

        let compiled = stmt.compile().unwrap();
        assert_eq!(
            compiled.sql,
            "SELECT batch_id, SUM(cost) AS total_cost FROM telemetry_events t WHERE (latency > $1) GROUP BY batch_id LIMIT 100"
        );
        assert_eq!(compiled.params, vec![SqlLiteral::Integer(500)]);
    }

    #[test]
    fn test_injection_attempt_rejected() {
        let stmt = SelectStatement {
            from: TableRef {
                name: "events; DROP TABLE users; --".to_string(),
                alias: None,
            },
            select_list: vec![],
            where_clause: None,
            group_by: vec![],
            limit: None,
            offset: None,
        };

        let result = stmt.compile();
        assert!(matches!(result, Err(AstError::InvalidIdentifier(_))));
    }
}
