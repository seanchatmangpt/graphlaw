#![allow(missing_docs)]
#![allow(clippy::result_large_err)]

use super::refusals::{AstError, RefusalCode, refusal_arm};
use super::serializer::SelectStatement;

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
    ILike,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
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
            Self::ILike => "ILIKE",
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "%",
        }
    }

    /// Legacy axiom IRIs (`sql:Equals`, ...) stay legal (SQLRDF-01 alias window).
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
            _ => Self::from_token(iri),
        }
    }

    /// Section 4 operator token enum:
    /// `"=" "!=" "<" "<=" ">" ">=" "AND" "OR" "+" "-" "*" "/" "%" "LIKE" "ILIKE"`.
    /// Any other token is REFUSAL-01 (`MalformedAstRefusal`).
    pub fn from_token(token: &str) -> Result<Self, AstError> {
        match token {
            "=" => Ok(Self::Equals),
            "!=" => Ok(Self::NotEquals),
            "<" => Ok(Self::LessThan),
            "<=" => Ok(Self::LessThanOrEq),
            ">" => Ok(Self::GreaterThan),
            ">=" => Ok(Self::GreaterThanOrEq),
            "AND" => Ok(Self::And),
            "OR" => Ok(Self::Or),
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Subtract),
            "*" => Ok(Self::Multiply),
            "/" => Ok(Self::Divide),
            "%" => Ok(Self::Modulo),
            "LIKE" => Ok(Self::Like),
            "ILIKE" => Ok(Self::ILike),
            other => Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::OPERATOR_TOKEN)
                    .with_ident(other),
            ),
        }
    }
}

/// Operational scope of a term (sql-rdf Rec section 1.3/3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    View,
    Process,
}

impl Scope {
    pub fn from_iri(iri: &str) -> Result<Self, AstError> {
        match iri {
            "http://graphlaw.org/sql#ViewScope" => Ok(Self::View),
            "http://graphlaw.org/sql#ProcessScope" => Ok(Self::Process),
            other => Err(
                AstError::new(RefusalCode::MalformedAst, refusal_arm::SCOPE_INDIVIDUAL)
                    .with_ident(other),
            ),
        }
    }
}

/// Syntactic position of an expression, driving scope-integrity enforcement
/// (section 1.3/5, REFUSAL-03): aggregates are illegal in predicates and
/// GROUP BY; nested aggregates are illegal everywhere; an explicit
/// sql:ProcessScope column may not surface in a projection list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopePos {
    SelectList,
    OrderBy,
    Having,
    Predicate,
    GroupBy,
    AggregateArgument,
    /// Aggregate argument beneath a PREDICATE-rooted aggregate (WHERE/JOIN/
    /// HAVING/GROUP BY): same aggregate-nesting semantics as AggregateArgument,
    /// but an explicit sql:scope on the argument is a laundering marker
    /// (TC-MUT-01 arm B) — under an output aggregate (SELECT/ORDER BY) the
    /// section 7 example (ProcessScope amount inside a projection-list
    /// aggregate) stays lawful.
    PredicateAggregateArgument,
}

impl ScopePos {
    pub(super) fn allows_aggregate(self) -> bool {
        matches!(self, Self::SelectList | Self::OrderBy | Self::Having)
    }

    pub(super) fn is_view_surface(self) -> bool {
        matches!(self, Self::SelectList | Self::OrderBy)
    }
}

/// Position lattice for an aggregate ARGUMENT: under an output-position
/// aggregate (SELECT list / ORDER BY — the section 7 example, where a
/// ProcessScope column inside a projection-list aggregate is lawful view
/// output of an internal computation) the argument keeps the plain
/// AggregateArgument position; under a predicate-rooted aggregate
/// (WHERE/JOIN/HAVING/GROUP BY) the argument is
/// PredicateAggregateArgument, where an explicit sql:scope marker is
/// REFUSAL-03 (TC-MUT-01 arm B).
pub(super) fn aggregate_argument_pos(root: ScopePos) -> ScopePos {
    if root.is_view_surface() {
        ScopePos::AggregateArgument
    } else {
        ScopePos::PredicateAggregateArgument
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

/// LANE L5 minimal unblock (compile-freeze SLA, disclosed): name-form
/// constructor referenced by lane L2's in-flight section 7 function form
/// (`sql_ast.rs:703`). Accepts the SQL function name (case-insensitive) or
/// its `graphlaw.org/functions#` IRI; anything else is the typed
/// `AstError::UnsupportedTerm` refusal. Remove when lane L2 lands their own.
pub(super) fn aggregate_from_name(name: &str) -> Result<AggregateFunction, AstError> {
    let upper = name.to_ascii_uppercase();
    let token = upper
        .strip_prefix("HTTP://GRAPHLAW.ORG/FUNCTIONS#")
        .unwrap_or(&upper);
    match token {
        "COUNT" => Ok(AggregateFunction::Count),
        "SUM" => Ok(AggregateFunction::Sum),
        "AVG" => Ok(AggregateFunction::Avg),
        "MIN" => Ok(AggregateFunction::Min),
        "MAX" => Ok(AggregateFunction::Max),
        _ => Err(AstError::UnsupportedTerm(name.to_string())),
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
    /// Optional schema qualifier (`sql:schemaName`); rendered as `schema.name`.
    pub schema: Option<String>,
}

/// Validates a dotted identifier (`main.regional_sales`): every dot-separated
/// part must satisfy [`validate_identifier`].
pub fn validate_qualified_identifier(ident: &str) -> Result<(), AstError> {
    if ident.is_empty() {
        return Err(AstError::InvalidIdentifier(ident.to_string()));
    }
    for part in ident.split('.') {
        validate_identifier(part)?;
    }
    Ok(())
}

/// One ORDER BY term: expression plus direction (`sql:direction`, ASC default).
#[derive(Debug, Clone, PartialEq)]
pub struct OrderTerm {
    pub expr: SqlExpr,
    pub descending: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    LeftOuter,
}

impl JoinType {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Inner => "JOIN",
            Self::LeftOuter => "LEFT JOIN",
        }
    }

    pub fn from_iri(iri: &str) -> Result<Self, AstError> {
        match iri {
            "http://graphlaw.org/sql#InnerJoin" => Ok(Self::Inner),
            "http://graphlaw.org/sql#LeftJoin" => Ok(Self::LeftOuter),
            _ => Err(AstError::UnsupportedTerm(iri.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    pub join_type: JoinType,
    pub table: TableRef,
    pub condition: SqlExpr,
}

#[derive(Debug, Clone)]
pub struct CompiledQuery {
    pub sql: String,
    pub params: Vec<SqlLiteral>,
    /// The admitted AST graph (Turtle source) this query was compiled from,
    /// when the compile was graph-backed. When present, the execution receipt
    /// hash is BLAKE3(Canon(G_AST) || P_params) (sql-rdf Rec. section 1.4);
    /// when `None` (hand-built ASTs), the legacy SQL-string hash is used.
    /// LANE L5 minimal seam (SQLRDF-09), disclosed: field added by lane L5.
    pub ast_graph_turtle: Option<String>,
    /// RDFC-1.0 canonical byte form of the admitted AST graph
    /// (`Canon(G_AST)`), computed exactly ONCE per `compile_turtle` and
    /// carried here so the eval/receipt path never re-parses or
    /// re-canonicalizes per receipt (R1 moonshot). Receipt law is unchanged:
    /// `H_query = BLAKE3(Canon(G_AST) || P_params)` (sql-rdf Rec. section
    /// 1.4). `None` for hand-built ASTs (legacy SQL-string hash applies).
    /// Failure to canonicalize at compile time is a typed
    /// `AstError::ShaclViolation` carrying the REFUSAL-01 marker.
    pub canon_graph_bytes: Option<Vec<u8>>,
    /// Receipt anchor `h_ast` (P28-SIL §3.1, TC-HOT-01 two-node formula):
    /// `h_ast = BLAKE3(Canon(G_AST))`, computed exactly once at
    /// `compile_turtle` time. Per receipt, eval hashes
    /// `H_query = BLAKE3(h_ast || P_params)` (sql-rdf Rec. section 1.4
    /// first formula, two-node form) with ONE fresh hasher + two updates +
    /// finalize — a 32-byte Copy instead of a ~2KB hasher clone, and no
    /// O(|Canon|) re-absorb per receipt. `None` for hand-built ASTs
    /// (legacy paths apply). Bit-identical to the
    /// `admitted_statement_hash` reference by construction (same formula).
    pub receipt_state: Option<[u8; 32]>,
    /// C1-a classification bit (designs/C1-cache.md): whether this compiled
    /// statement may EVER be served from a compiled-plan cache. Set `true`
    /// only on the `SelectStatement` arm of `compile_turtle`; every
    /// hand-built or raw constructor is `false`. The compiler's statement
    /// classification gates caching — a cache store (post-integration) must
    /// never admit a query with `cacheable == false`. Pure classification:
    /// no cache store exists yet, and the bit does not alter compile/eval
    /// behavior (sql-rdf Rec. section 1.4 receipt law unchanged).
    pub cacheable: bool,
    /// Private admission seal (RUST-5 sole-admission direction, appended
    /// v26.10.22 lane L4): `blake3(sql bytes ‖ cacheable byte ‖
    /// receipt_state 32B-or-zero-marker ‖ canon_graph_bytes ‖ presence-flag
    /// bytes)`, computed by [`CompiledQuery::reseal`] at the end of every
    /// admitted compile. `pub(super)` so the parent court can read the
    /// seal; a struct literal is still impossible outside `sql_ast`.
    /// Eval is the sole admission gate and refuses a broken seal with a
    /// typed error before any engine call.
    pub(super) admission_seal: [u8; 32],
}

impl CompiledQuery {
    /// Sole in-crate constructor for non-graph-backed (raw/merge/command)
    /// compiled queries. Seals before returning.
    pub(crate) fn raw_compiled(sql: String, params: Vec<SqlLiteral>) -> Self {
        let mut q = Self {
            sql,
            params,
            ast_graph_turtle: None,
            canon_graph_bytes: None,
            receipt_state: None,
            cacheable: false,
            admission_seal: [0u8; 32],
        };
        q.reseal();
        q
    }

    /// Recomputes the admission seal over the current state.
    pub(crate) fn reseal(&mut self) {
        self.admission_seal = self.compute_seal();
    }

    fn compute_seal(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(self.sql.as_bytes());
        h.update(&[self.cacheable as u8]);
        h.update(&self.receipt_state.unwrap_or([0u8; 32]));
        if let Some(canon) = &self.canon_graph_bytes {
            h.update(canon);
        }
        // Presence-flag bytes: receipt_state present, canon bytes present.
        h.update(&[
            self.receipt_state.is_some() as u8,
            self.canon_graph_bytes.is_some() as u8,
        ]);
        *h.finalize().as_bytes()
    }

    /// True exactly when the query still carries a valid admission seal,
    /// i.e. its sealed fields are unmodified since the admitted compile.
    /// Bound parameters (`params`) are not sealed: binding different params
    /// to the same admitted statement is lawful.
    pub fn verify_seal(&self) -> bool {
        self.admission_seal == self.compute_seal()
    }

    /// The current seal bytes (court/ABI observation accessor).
    pub fn seal_bytes(&self) -> [u8; 32] {
        self.admission_seal
    }
}

// TC-HOT-01: `receipt_state` now carries `h_ast = BLAKE3(Canon(G_AST))` as a
// 32-byte Copy digest, so `PartialEq` compares it directly (a plain
// `[u8; 32]` equality — same law as before, cheaper mechanics).
impl PartialEq for CompiledQuery {
    fn eq(&self, other: &Self) -> bool {
        self.sql == other.sql
            && self.params == other.params
            && self.ast_graph_turtle == other.ast_graph_turtle
            && self.canon_graph_bytes == other.canon_graph_bytes
            && self.receipt_state == other.receipt_state
            && self.cacheable == other.cacheable
            && self.admission_seal == other.admission_seal
    }
}

// Number of RDFC-1.0 canonicalizations performed by this module. Instrument
// for the R1 receipt-cost reduction: canonicalization is amortized to once
// per compile, not once per receipt. Eval reads the carried
// `CompiledQuery::canon_graph_bytes` instead of re-canonicalizing.
thread_local! {
    pub static CANONICALIZATION_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Computes `Canon(G_AST)` — the RDFC-1.0 canonical byte form of an admitted
/// AST graph — using the same `purrdf::try_canonicalize` law as
/// `duckdb_eval::canonical_ast_graph_bytes`. Called exactly once per
/// `compile_turtle`; failure is a typed REFUSAL-01 `AstError`, never a panic.
pub(super) fn canonical_graph_bytes(turtle: &str) -> Result<Vec<u8>, AstError> {
    let dataset = purrdf::parse_dataset(turtle.as_bytes(), "text/turtle", None)
        .map_err(|_| AstError::new(RefusalCode::MalformedAst, refusal_arm::CANONICALIZATION))?;
    let canon = purrdf::try_canonicalize(&dataset)
        .map_err(|_| AstError::new(RefusalCode::MalformedAst, refusal_arm::CANONICALIZATION))?;
    CANONICALIZATION_COUNT.set(CANONICALIZATION_COUNT.get() + 1);
    Ok(canon.nquads.into_bytes())
}

/// Section 7 analytical MERGE statement (rec 7/7.1): the target table, the
/// grouped USING sub-select, the ON condition, and the WHEN MATCHED update
/// assignments. Compiled by [`MergeStatement::compile_merge`] with the
/// sub-select literal (section 7.1: `$1`) extracted into the parameter
/// buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct MergeStatement {
    pub target: TableRef,
    pub source: SelectStatement,
    pub on: SqlExpr,
    pub assignments: Vec<(String, SqlExpr)>,
}

impl MergeStatement {
    /// Compiles the section 7 graph to the section 7.1 parameterized MERGE.
    /// The sub-select literal is extracted into `params` ($1 = "SETTLED" for
    /// the section 7 graph), never string-interpolated.
    pub fn compile_merge(&self) -> Result<CompiledQuery, AstError> {
        validate_identifier(&self.target.name)?;
        if let Some(schema) = &self.target.schema {
            validate_qualified_identifier(schema)?;
        }
        if let Some(alias) = &self.target.alias {
            validate_identifier(alias)?;
        }

        let mut params = Vec::new();
        // Inner sub-select body (the leading "SELECT " of the clause compiler
        // output is stripped for embedding).
        let inner = self.source.compile()?;
        let body = inner.sql;
        params.extend(inner.params);

        let mut sql = String::with_capacity(body.len() + 160);
        sql.push_str("MERGE INTO ");
        if let Some(schema) = &self.target.schema {
            sql.push_str(schema);
            sql.push('.');
        }
        sql.push_str(&self.target.name);
        sql.push_str(" USING (");
        sql.push_str(&body);
        sql.push_str(") AS src ON ");
        let mut on_sql = String::new();
        SelectStatement::serialize_expr(
            &self.on,
            &mut on_sql,
            &mut params,
            0,
            ScopePos::Predicate,
        )?;
        sql.push_str(&on_sql);
        sql.push_str(" WHEN MATCHED THEN UPDATE SET ");
        for (idx, (column, value)) in self.assignments.iter().enumerate() {
            if idx > 0 {
                sql.push_str(", ");
            }
            validate_identifier(column)?;
            sql.push_str(column);
            sql.push_str(" = ");
            SelectStatement::serialize_expr(value, &mut sql, &mut params, 0, ScopePos::Predicate)?;
        }
        sql.push(';');
        // C1-a: merge compile is not the SelectStatement arm; never cacheable.
        Ok(CompiledQuery::raw_compiled(sql, params))
    }
}
