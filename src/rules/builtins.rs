//! Builtins bridging external evaluation substrates into GraphLaw reasoning and state admission.
//!
//! Provides `builtin_eval_sql` which invokes `frozen-duckdb` columnar execution from
//! declarative query AST nodes, generating BLAKE3 `ExecutionReceipt`s and materializing
//! triples back into the deductive working memory.

#![allow(missing_docs)]

use std::fmt;

use crate::duckdb_eval::{EvaluatorError, GraphLawDuckDB, RdfTriple};
use crate::sql_ast::RdfAstGraphReader;
use purrdf::TermValue;

#[derive(Debug)]
pub enum BuiltinError {
    Eval(EvaluatorError),
    InvalidArguments { expected: String, got: String },
    AstError(String),
}

impl fmt::Display for BuiltinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eval(e) => write!(f, "Evaluation error: {e}"),
            Self::InvalidArguments { expected, got } => {
                write!(f, "Invalid argument shape: expected {expected}, got {got}")
            }
            Self::AstError(msg) => write!(f, "AST resolution failure: {msg}"),
        }
    }
}

impl std::error::Error for BuiltinError {}

impl From<EvaluatorError> for BuiltinError {
    fn from(e: EvaluatorError) -> Self {
        Self::Eval(e)
    }
}

pub struct BuiltinContext<'a> {
    pub db: &'a GraphLawDuckDB,
}

pub struct BuiltinResult {
    /// Variables to bind in the rule environment: (Variable Name -> Bound TermValue)
    pub bindings: Vec<(String, TermValue)>,
    /// New statements to materialize directly into the reasoner's knowledge base
    pub derived_triples: Vec<RdfTriple>,
    /// The canonical query hash (BLAKE3) committing to the executed query and constants
    pub query_hash: [u8; 32],
    /// Row count produced by DuckDB execution
    pub rows_emitted: usize,
}

/// Dynamic evaluator builtin for `gl:evalSql`.
///
/// Inputs:
/// - `args`: `[query_ast_iri, subject_prefix, vocab_prefix]`
/// - `output_vars`: Names of target variables, e.g. `["receipt", "rowsEmitted", "queryHash"]`
pub fn builtin_eval_sql<R: RdfAstGraphReader<Node = String>>(
    args: &[&str],
    output_vars: &[&str],
    ctx: &BuiltinContext,
    reader: &R,
) -> Result<BuiltinResult, BuiltinError> {
    if args.len() != 3 {
        return Err(BuiltinError::InvalidArguments {
            expected: "[queryAst, subjectPrefix, vocabPrefix]".to_string(),
            got: format!("{} arguments", args.len()),
        });
    }

    let query_node = args[0].to_string();
    let subject_prefix = args[1];
    let vocab_prefix = args[2];

    // 1. Extract AST using reader
    let ast = reader
        .extract_select_statement(&query_node)
        .map_err(|e| BuiltinError::AstError(e.to_string()))?;

    // 2. Compile to parameterized SQL
    let compiled = ast
        .compile()
        .map_err(|e| BuiltinError::AstError(e.to_string()))?;

    // 3. Vectorized pushdown to embedded frozen-duckdb
    let receipt = ctx
        .db
        .eval_ast_query(&query_node, &compiled, subject_prefix, vocab_prefix)?;

    // 4. Bind return variables: (?receipt, ?rowsEmitted, ?queryHash)
    let hex_hash = hex::encode(receipt.query_hash);
    let receipt_iri = format!("{}/receipt_{}", subject_prefix, &hex_hash[0..16]);

    let mut bindings = Vec::new();
    if let Some(&var) = output_vars.first() {
        bindings.push((var.to_string(), TermValue::Iri(receipt_iri)));
    }
    if let Some(&var) = output_vars.get(1) {
        bindings.push((
            var.to_string(),
            TermValue::Literal {
                lexical_form: receipt.rows_emitted.to_string(),
                datatype: "http://www.w3.org/2001/XMLSchema#integer".to_string(),
                language: None,
                direction: None,
            },
        ));
    }
    if let Some(&var) = output_vars.get(2) {
        bindings.push((
            var.to_string(),
            TermValue::Literal {
                lexical_form: hex_hash,
                datatype: "http://www.w3.org/2001/XMLSchema#hexBinary".to_string(),
                language: None,
                direction: None,
            },
        ));
    }

    Ok(BuiltinResult {
        bindings,
        derived_triples: receipt.result_triples,
        query_hash: receipt.query_hash,
        rows_emitted: receipt.rows_emitted,
    })
}
