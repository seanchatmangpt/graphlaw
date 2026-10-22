//! Declarative SQL AST representation and deterministic compiler for GraphLaw.
//!
//! Provides in-memory AST terms matching the `ontologies/sql-ast.ttl` ontology,
//! identifier validation eliminating structural injection, parameterized SQL serialization,
//! and recursive bounded RDF collection unrolling.

#![allow(missing_docs)]

use std::collections::{HashMap, HashSet};
use std::fmt;

/// Typed refusal codes (`docs/refusals.md`). These are the stable message
/// markers carried inside `AstError` text; only REFUSAL-04 lives in
/// `src/tier.rs` (capability refusals).
pub const REFUSAL_01_MALFORMED_AST: &str = "[REFUSAL-01] MalformedAstRefusal:";
pub const REFUSAL_02_UNIMPLEMENTED: &str = "[REFUSAL-02] UnimplementedStatementRefusal:";
pub const REFUSAL_03_SCOPE_INTEGRITY: &str = "[REFUSAL-03] ScopeIntegrityRefusal:";

/// Internal message-arm discriminants (`RefusalContext::shape_id`). Each
/// stable refusal code has a fixed table of rendering arms; the arm selects
/// which `&'static str` template the cold Display path interpolates.
///
/// OPEN DRIFT NOTE (SQLRDF-18 pure-data refusal restructure): these are
/// internal, never serialized over the ABI; new arms append, never renumber.
pub mod refusal_arm {
    pub const OPERATOR_TOKEN: u16 = 1;
    pub const SCOPE_INDIVIDUAL: u16 = 2;
    pub const CANONICALIZATION: u16 = 3;
    pub const DUP_CLAUSE_PREDICATE: u16 = 4;
    pub const SCOPE_CARDINALITY: u16 = 5;
    pub const SCOPE_WRONG_NODE: u16 = 6;
    pub const DISCRIMINATORS: u16 = 7;
    pub const EXPRESSION_WRAPPER: u16 = 8;
    pub const ALIAS_IN_AGG_ARG: u16 = 9;
    pub const DUAL_LEFT_OPERAND: u16 = 10;
    pub const DUAL_RIGHT_OPERAND: u16 = 11;
    pub const DUAL_OPERATOR_FORMS: u16 = 12;
    pub const SQL_ALIAS_IN_AGG_ARG: u16 = 13;
    pub const SCOPE_ON_TABLE_REF: u16 = 14;
    pub const DUAL_FROM: u16 = 15;
    pub const EMPTY_FROM: u16 = 16;
    pub const NO_FROM: u16 = 17;
    pub const DUAL_SELECT: u16 = 18;
    pub const NO_PROJECTION: u16 = 19;
    pub const EMPTY_PROJECTION: u16 = 20;
    pub const DUAL_WHERE: u16 = 21;
    pub const DUAL_HAVING: u16 = 22;
    pub const DIRECTION_LEX: u16 = 23;
    pub const DIRECTION_IRI: u16 = 24;
    pub const LIMIT_MALFORMED: u16 = 25;
    pub const LIMIT_DUP: u16 = 26;
    pub const LIMIT_NOT_INT: u16 = 27;
    pub const OFFSET_MALFORMED: u16 = 28;
    pub const OFFSET_DUP: u16 = 29;
    pub const OFFSET_NOT_INT: u16 = 30;
    pub const INDEX_MISSING: u16 = 31;
    pub const INDEX_DUP: u16 = 32;
    pub const INDEX_NOT_INT: u16 = 33;
    pub const INDEX_BAD_DTYPE: u16 = 34;
    pub const INDEX_NEGATIVE: u16 = 35;
    pub const UNKNOWN_STATEMENT_PREDICATE: u16 = 36;
    // ScopeIntegrity (REFUSAL-03) arms.
    pub const AGG_IN_POSITION: u16 = 1;
    pub const ALIASED_AGG_ARG: u16 = 2;
    pub const AGG_IN_PREDICATE: u16 = 3;
    pub const PROCESS_SCOPE_IN_SELECT: u16 = 4;
    pub const VIEW_SCOPE_IN_PREDICATE: u16 = 5;
    pub const SCOPE_IN_PRED_AGG_ARG: u16 = 6;
    // Internal MalformedAst-family arms (ShaclViolation carriers without a
    // stable [REFUSAL-nn] marker today — the docs/refusals.md known gap).
    pub const SHACL_DEPTH: u16 = 100;
    pub const PARSE_DATASET: u16 = 101;
    pub const PARSE_SHAPES: u16 = 102;
    pub const SHACL_ENGINE: u16 = 103;
    pub const SHACL_VIOLATIONS: u16 = 104;
    // UnsupportedTerm internal arms.
    pub const UNSUPPORTED_GENERIC: u16 = 0;
    // LANE L2 disclosed compat arm (SQLRDF-18 restructure unblock): free-form
    // legacy ShaclViolation text renders raw through the ident buffer.
    pub const LEGACY_MESSAGE: u16 = 255;
    pub const UNIMPLEMENTED_STATEMENT: u16 = 1;
    pub const NO_SQL_TEMPLATE: u16 = 2;
}

/// Stable sql-rdf refusal codes (`docs/refusals.md`, REFUSAL-01..04) plus the
/// internal discriminants for the malformed-AST family. Pure data: `repr(u8)`,
/// stable values never renumbered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RefusalCode {
    /// REFUSAL-01 `MalformedAstRefusal`.
    MalformedAst = 1,
    /// REFUSAL-02 `UnimplementedStatementRefusal`.
    UnimplementedStatement = 2,
    /// REFUSAL-03 `ScopeIntegrityRefusal`.
    ScopeIntegrity = 3,
    /// REFUSAL-04 `CapabilityRefusal` (document-backed builtin eval; see `docs/refusals.md`).
    Capability = 4,
    /// Internal: identifier fails the strict ident regex.
    InvalidIdentifier = 16,
    /// Internal: required property absent on a node.
    MissingProperty = 17,
    /// Internal: recursion/list depth cap breached.
    RecursionLimitExceeded = 18,
    /// Internal: circular or unclosed RDF list.
    MalformedList = 19,
    /// Internal: unsupported operator/function/statement IRI or token.
    UnsupportedTerm = 20,
    /// Internal: expression node matches no discriminator.
    MalformedNode = 21,
}

impl RefusalCode {
    /// The stable `[REFUSAL-nn]` marker for the stable codes, `None` for the
    /// internal discriminants.
    pub const fn marker(self) -> Option<&'static str> {
        match self {
            Self::MalformedAst => Some(REFUSAL_01_MALFORMED_AST),
            Self::UnimplementedStatement => Some(REFUSAL_02_UNIMPLEMENTED),
            Self::ScopeIntegrity => Some(REFUSAL_03_SCOPE_INTEGRITY),
            Self::Capability => Some(crate::tier::REFUSAL_04_CAPABILITY_REFUSAL),
            _ => None,
        }
    }
}

/// Max bytes packed per identifier. Anything longer truncates (the cold
/// Display path renders the truncated prefix).
pub const REFUSAL_IDENT_BYTES: usize = 192;

/// Fixed-size packed identifier: up to `REFUSAL_IDENT_BYTES` bytes copied at
/// construction (memcpy, no heap), rendered lossily on the cold Display path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedIdent {
    len: u8,
    bytes: [u8; REFUSAL_IDENT_BYTES],
}

impl Default for PackedIdent {
    fn default() -> Self {
        Self { len: 0, bytes: [0u8; REFUSAL_IDENT_BYTES] }
    }
}

impl PackedIdent {
    /// Packs a Display value into the fixed buffer without heap allocation.
    pub fn pack(v: impl fmt::Display) -> Self {
        let mut p = Self::default();
        p.push(v);
        p
    }

    /// Packs two Display values concatenated (`a` then `b`).
    pub fn packed2(a: impl fmt::Display, b: impl fmt::Display) -> Self {
        let mut p = Self::default();
        p.push(a);
        p.push(b);
        p
    }

    /// Appends a Display value (truncating at capacity).
    pub fn push(&mut self, v: impl fmt::Display) {
        let mut w = FixedWriter { buf: &mut self.bytes, len: self.len as usize };
        // Formatting can only fail when the underlying writer fails; the
        // fixed buffer never fails, it truncates.
        let _ = fmt::write(&mut w, format_args!("{v}"));
        self.len = w.len.min(REFUSAL_IDENT_BYTES) as u8;
    }

    /// The packed bytes as a (possibly lossy) string, cold path.
    pub fn as_str_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.bytes[..self.len as usize])
    }
}

struct FixedWriter<'a> {
    buf: &'a mut [u8; REFUSAL_IDENT_BYTES],
    len: usize,
}

impl fmt::Write for FixedWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let rem = REFUSAL_IDENT_BYTES - self.len;
        let n = s.len().min(rem);
        self.buf[self.len..self.len + n].copy_from_slice(&s.as_bytes()[..n]);
        self.len += n;
        Ok(())
    }
}

/// Fixed-size refusal context: shape id, predicate index, numeric arguments,
/// and packed idents. `Copy`, no `String`, no heap — construction is a pair
/// of bounded memcpys (hot path budget <10ns).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefusalContext {
    /// Internal static-message-template selector (see [`refusal_arm`]).
    pub shape_id: u16,
    /// Statement index / ordinal position the refusal attaches to.
    pub predicate_idx: u16,
    /// Primary numeric argument (count or limit).
    pub num: u32,
    /// Secondary numeric argument.
    pub aux_num: u32,
    /// Primary packed identifier.
    pub ident: PackedIdent,
    /// Secondary packed identifier.
    pub aux_ident: PackedIdent,
}

impl Default for RefusalContext {
    fn default() -> Self {
        Self {
            shape_id: 0,
            predicate_idx: 0,
            num: 0,
            aux_num: 0,
            ident: PackedIdent::default(),
            aux_ident: PackedIdent::default(),
        }
    }
}

/// A typed sql-rdf refusal. Pure data (SQLRDF-18 restructure): `code` +
/// fixed-size `ctx`, with message rendering deferred entirely to the cold
/// Display/JSON-at-ABI path via the const code→template table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AstError {
    /// Stable (or internal) refusal code.
    pub code: RefusalCode,
    /// Fixed-size rendering context.
    pub ctx: RefusalContext,
}

impl AstError {
    #[inline]
    fn new(code: RefusalCode, shape_id: u16) -> Self {
        Self { code, ctx: RefusalContext { shape_id, ..RefusalContext::default() } }
    }
    #[inline]
    fn with_num(mut self, num: usize) -> Self {
        self.ctx.num = num as u32;
        self
    }
    #[inline]
    fn with_aux_num(mut self, aux_num: usize) -> Self {
        self.ctx.aux_num = aux_num as u32;
        self
    }
    #[inline]
    fn with_ident(mut self, ident: impl fmt::Display) -> Self {
        self.ctx.ident.push(ident);
        self
    }
    #[inline]
    fn with_aux_ident(mut self, aux: impl fmt::Display) -> Self {
        self.ctx.aux_ident.push(aux);
        self
    }

}

// LANE L2 disclosed compat shim (SQLRDF-18 restructure unblock): the two
// concurrent shim sets collided; this is the constructor surface the legacy
// call sites use. Rendering goes through the arm table / LEGACY_MESSAGE
// fallback; free-form tails truncate at the REFUSAL_IDENT_BYTES (192) buffer.
// N2 closure: no construction site in this file uses LEGACY_MESSAGE anymore;
// the only producer is `ShaclViolation` (arbitrary caller-supplied free-form
// text — no typed arm can represent it without a String field on AstError).
impl AstError {
    #[allow(non_snake_case)]
    pub fn ShaclViolation(msg: String) -> Self {
        let code = if msg.contains(REFUSAL_03_SCOPE_INTEGRITY) {
            RefusalCode::ScopeIntegrity
        } else {
            RefusalCode::MalformedAst
        };
        AstError::new(code, refusal_arm::LEGACY_MESSAGE).with_ident(msg)
    }
    #[allow(non_snake_case)]
    pub fn MissingProperty(a: String, b: String) -> Self {
        AstError::new(RefusalCode::MissingProperty, 0).with_ident(a).with_aux_ident(b)
    }
    #[allow(non_snake_case)]
    pub fn InvalidIdentifier(a: String) -> Self {
        AstError::new(RefusalCode::InvalidIdentifier, 0).with_ident(a)
    }
    #[allow(non_snake_case)]
    pub fn UnsupportedTerm(a: String) -> Self {
        AstError::new(RefusalCode::UnsupportedTerm, refusal_arm::UNSUPPORTED_GENERIC).with_ident(a)
    }
    #[allow(non_snake_case)]
    pub fn RecursionLimitExceeded(n: usize) -> Self {
        AstError::new(RefusalCode::RecursionLimitExceeded, 0).with_num(n)
    }
    #[allow(non_snake_case)]
    pub fn MalformedNode(a: String) -> Self {
        AstError::new(RefusalCode::MalformedNode, 0).with_ident(a)
    }
}

impl std::error::Error for AstError {}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use refusal_arm as arm;
        let id = self.ctx.ident.as_str_lossy();
        let aux = self.ctx.aux_ident.as_str_lossy();
        // Cold path only: code → static template from the const table, with
        // context interpolation. Byte-compatible with the pre-restructure
        // format!-built messages (needle courts pin these fragments).
        match self.code {
            RefusalCode::InvalidIdentifier => {
                write!(f, "Invalid SQL identifier: {id}")
            }
            RefusalCode::MissingProperty => {
                write!(f, "Missing required property '{id}' on node {aux}")
            }
            RefusalCode::RecursionLimitExceeded => write!(
                f,
                "Recursion depth limit exceeded ({}) while traversing AST",
                self.ctx.num
            ),
            RefusalCode::MalformedList => write!(
                f,
                "Malformed RDF list: circular reference or unclosed node"
            ),
            RefusalCode::MalformedNode => {
                write!(f, "Unexpected terminal node: {id}")
            }
            RefusalCode::UnsupportedTerm | RefusalCode::UnimplementedStatement => match self.ctx.shape_id {
                arm::UNIMPLEMENTED_STATEMENT => write!(
                    f,
                    "compile refusal: http://graphlaw.org/sql#{id} {REFUSAL_02_UNIMPLEMENTED} not implemented by this DuckDB build"
                ),
                arm::NO_SQL_TEMPLATE => write!(f,
                    "Unsupported operator or function IRI: compile refusal: {id} has no SQL template"),
                _ => write!(f, "Unsupported operator or function IRI: {id}"),
            },
            RefusalCode::MalformedAst => {
                // ShaclViolation carriers render the structural prefix, the
                // stable marker, then the arm template.
                let marker = self.code.marker().unwrap_or("");
                match self.ctx.shape_id {
                    arm::OPERATOR_TOKEN => write!(f,
                        "SHACL AST structural violation: {marker}operator token '{id}' is not in the section 4 enum (= != < <= > >= AND OR + - * / % LIKE ILIKE)"),
                    arm::SCOPE_INDIVIDUAL => write!(f,
                        "SHACL AST structural violation: {marker}'{id}' is not a sql:Scope individual (sql:ViewScope | sql:ProcessScope)"),
                    arm::CANONICALIZATION => write!(f,
                        "SHACL AST structural violation: {marker} admitted AST graph canonicalization refused"),
                    arm::DUP_CLAUSE_PREDICATE => write!(f,
                        "SHACL AST structural violation: {marker}{id} on {aux} appears more than once; clause predicates are single-cardinality"),
                    arm::SCOPE_CARDINALITY => write!(f,
                        "SHACL AST structural violation: {marker}node {id} carries {} sql:scope values; scope is single-cardinality", self.ctx.num),
                    arm::SCOPE_WRONG_NODE => write!(f,
                        "SHACL AST structural violation: {marker}sql:scope on {id} is only defined on column or output-expression nodes, not on literals or operator nodes"),
                    arm::DISCRIMINATORS => write!(f,
                        "SHACL AST structural violation: {marker}node {id} carries {} expression discriminators (columnName/literalValue/value/aliasName/function/functionName/operator); exactly one is allowed", self.ctx.num),
                    arm::EXPRESSION_WRAPPER => write!(f,
                        "SHACL AST structural violation: {marker}node {id} carries sql:expression alongside another expression discriminator; a bare sql:expression wrapper must be the node's only term"),
                    arm::ALIAS_IN_AGG_ARG => write!(f,
                        "SHACL AST structural violation: {marker}aliased expression {id} inside an aggregate argument is not serializable SQL (fuzz 3: SUM(a AS col))"),
                    arm::DUAL_LEFT_OPERAND => write!(f,
                        "SHACL AST structural violation: {marker}binary expression {id} carries both sql:leftOperand and sql:left; mixed operand names launder a term past review"),
                    arm::DUAL_RIGHT_OPERAND => write!(f,
                        "SHACL AST structural violation: {marker}binary expression {id} carries both sql:rightOperand and sql:right; mixed operand names launder a term past review"),
                    arm::DUAL_OPERATOR_FORMS => write!(f,
                        "SHACL AST structural violation: {marker}binary expression {id} carries sql:operator as both a string token and an IRI; exactly one operator form is allowed"),
                    arm::SQL_ALIAS_IN_AGG_ARG => write!(f,
                        "SHACL AST structural violation: {marker}sql:alias on {id} inside an aggregate argument is not serializable SQL"),
                    arm::SCOPE_ON_TABLE_REF => write!(f,
                        "SHACL AST structural violation: {marker}sql:scope on table-reference node {id} is a term out of place; scope is defined on column or output-expression nodes, never on sql:fromTable/sql:joinTable table references"),
                    arm::DUAL_FROM => write!(f,
                        "SHACL AST structural violation: {marker}statement {id} carries both sql:from and sql:fromTable; exactly one data source clause is allowed"),
                    arm::EMPTY_FROM => write!(f,
                        "SHACL AST structural violation: {marker}sql:from list on {id} is empty"),
                    arm::NO_FROM => write!(f,
                        "SHACL AST structural violation: {marker}statement {id} has no sql:from (or legacy sql:fromTable) clause"),
                    arm::DUAL_SELECT => write!(f,
                        "SHACL AST structural violation: {marker}SelectStatement {id} carries both sql:selectList and sql:select; exactly one projection list is allowed"),
                    arm::NO_PROJECTION => write!(f,
                        "SHACL AST structural violation: {marker}SelectStatement {id} has no projection list (sql:select or sql:selectList)"),
                    arm::EMPTY_PROJECTION => write!(f,
                        "SHACL AST structural violation: {marker}SelectStatement {id} has an empty projection list; projection is never zero-column (SELECT * is not implied by rdf:nil)"),
                    arm::DUAL_WHERE => write!(f,
                        "SHACL AST structural violation: {marker}SelectStatement {id} carries both sql:whereClause and sql:where; exactly one filter clause is allowed"),
                    arm::DUAL_HAVING => write!(f,
                        "SHACL AST structural violation: {marker}SelectStatement {id} carries both sql:having and sql:havingClause; exactly one filter clause is allowed"),
                    arm::DIRECTION_LEX => write!(f,
                        "SHACL AST structural violation: {marker}sql:direction '{id}' on {aux} is not ASC or DESC"),
                    arm::DIRECTION_IRI => write!(f,
                        "SHACL AST structural violation: {marker}sql:direction on {id} must be an ASC or DESC string literal, not an IRI"),
                    arm::LIMIT_MALFORMED | arm::OFFSET_MALFORMED => write!(f,
                        "SHACL AST structural violation: {marker}sql:{} '{id}' on {aux} is not a valid non-negative integer",
                        if self.ctx.shape_id == arm::LIMIT_MALFORMED { "limit" } else { "offset" }),
                    arm::LIMIT_DUP | arm::OFFSET_DUP => write!(f,
                        "SHACL AST structural violation: {marker}sql:{} on {id} appears more than once; clause predicates are single-cardinality",
                        if self.ctx.shape_id == arm::LIMIT_DUP { "limit" } else { "offset" }),
                    arm::LIMIT_NOT_INT | arm::OFFSET_NOT_INT => write!(f,
                        "SHACL AST structural violation: {marker}sql:{} on {id} must be an xsd:integer literal",
                        if self.ctx.shape_id == arm::LIMIT_NOT_INT { "limit" } else { "offset" }),
                    arm::INDEX_MISSING => write!(f,
                        "SHACL AST structural violation: {marker}statement {id} is missing required sql:statementIndex (section 4 StatementBaseShape)"),
                    arm::INDEX_DUP => write!(f,
                        "SHACL AST structural violation: {marker}statement {id} carries {} sql:statementIndex values, maxCount is 1", self.ctx.num),
                    arm::INDEX_NOT_INT => write!(f,
                        "SHACL AST structural violation: {marker}sql:statementIndex on {id} is not an xsd:integer literal (section 4 datatype)"),
                    arm::INDEX_BAD_DTYPE => write!(f,
                        "SHACL AST structural violation: {marker}sql:statementIndex on {id} has a non-integer datatype, expected xsd:integer"),
                    arm::INDEX_NEGATIVE => write!(f,
                        "SHACL AST structural violation: {marker}sql:statementIndex '{id}' on {aux} is not a non-negative xsd:integer"),
                    arm::UNKNOWN_STATEMENT_PREDICATE => write!(f,
                        "SHACL AST structural violation: {marker}statement {id} carries unknown sql: predicate '{aux}'; term out of place — the sql: predicate set is a closed enum, a rename is not a silent-ignore escape hatch"),
                    arm::SHACL_DEPTH => write!(f,
                        "SHACL AST structural violation: SHACL nesting depth exceeded: observed {}, max {}",
                        self.ctx.num, self.ctx.aux_num),
                    arm::PARSE_DATASET => write!(f,
                        "SHACL AST structural violation: Failed to parse AST dataset"),
                    arm::PARSE_SHAPES => write!(f,
                        "SHACL AST structural violation: Failed to parse SHACL shapes"),
                    arm::SHACL_ENGINE => write!(f,
                        "SHACL AST structural violation: Validation engine failure"),
                    arm::LEGACY_MESSAGE => write!(f,
                        "SHACL AST structural violation: {marker}{id}"),
                    arm::SHACL_VIOLATIONS => {
                        if self.ctx.num > 1 {
                            write!(f,
                                "SHACL AST structural violation: Node <{id}>: {aux} (+{} more violations)",
                                self.ctx.num - 1)
                        } else {
                            write!(f, "SHACL AST structural violation: Node <{id}>: {aux}")
                        }
                    }
                    _ => write!(f,
                        "SHACL AST structural violation: {marker}node {id}"),
                }
            }
            RefusalCode::ScopeIntegrity => {
                // REFUSAL-03 arms: per-code arm namespace (same numeric ids
                // as the MalformedAst namespace, different code).
                let marker = self.code.marker().unwrap_or("");
                match self.ctx.shape_id {
                    arm::AGG_IN_POSITION => write!(f,
                        "SHACL AST structural violation: {marker}aggregate {} is not allowed in this syntactic position (section 1.3/5: WHERE, JOIN, GROUP BY are process-only)",
                        aggregate_debug_name(self.ctx.num)),
                    arm::ALIASED_AGG_ARG => write!(f,
                        "SHACL AST structural violation: {marker}aliased aggregate argument is not serializable SQL"),
                    arm::AGG_IN_PREDICATE => write!(f,
                        "SHACL AST structural violation: {marker}aggregate in a WHERE/JOIN/GROUP BY position is refused (section 1.3/5); re-scope the expression"),
                    arm::PROCESS_SCOPE_IN_SELECT => write!(f,
                        "SHACL AST structural violation: {marker}column '{id}' is sql:ProcessScope but sits in the projection list (section 1.3: a selected column crosses into the consumer's view)"),
                    arm::VIEW_SCOPE_IN_PREDICATE => write!(f,
                        "SHACL AST structural violation: {marker}column '{id}' is sql:ViewScope but sits inside a predicate (section 1.3: WHERE/JOIN/HAVING columns are processed, not viewed)"),
                    arm::SCOPE_IN_PRED_AGG_ARG => write!(f,
                        "SHACL AST structural violation: {marker}column '{id}' is {aux} inside an aggregate argument in a predicate position (section 1.3: the predicate-side argument is process-only by construction; an explicit sql:scope marker re-scopes the aggregate past review)"),
                    arm::LEGACY_MESSAGE => write!(f,
                        "SHACL AST structural violation: {marker}{id}"),
                    _ => write!(f,
                        "SHACL AST structural violation: {marker}node {id}"),
                }
            }
            RefusalCode::Capability => write!(f, "{}", crate::tier::REFUSAL_04_CAPABILITY_REFUSAL),
        }
    }
}

/// Debug-style aggregate name for the AGG_IN_POSITION arm (cold path).
fn aggregate_debug_name(idx: u32) -> &'static str {
    match idx {
        0 => "Count",
        1 => "Sum",
        2 => "Avg",
        3 => "Min",
        4 => "Max",
        _ => "Aggregate",
    }
}

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
            other => Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::OPERATOR_TOKEN).with_ident(other)),
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
            other => Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::SCOPE_INDIVIDUAL).with_ident(other)),
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
    fn allows_aggregate(self) -> bool {
        matches!(
            self,
            Self::SelectList | Self::OrderBy | Self::Having
        )
    }

    fn is_view_surface(self) -> bool {
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
fn aggregate_argument_pos(root: ScopePos) -> ScopePos {
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
fn aggregate_from_name(name: &str) -> Result<AggregateFunction, AstError> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub from: TableRef,
    pub joins: Vec<JoinClause>,
    pub select_list: Vec<SqlExpr>,
    pub where_clause: Option<SqlExpr>,
    pub group_by: Vec<SqlExpr>,
    /// `sql:having` / `sql:havingClause` predicate (aggregates legal here).
    pub having: Option<SqlExpr>,
    /// `sql:orderBy` terms (aggregates legal here).
    pub order_by: Vec<OrderTerm>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
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
fn canonical_graph_bytes(turtle: &str) -> Result<Vec<u8>, AstError> {
    let dataset = purrdf::parse_dataset(turtle.as_bytes(), "text/turtle", None).map_err(|_| {
        AstError::new(RefusalCode::MalformedAst, refusal_arm::CANONICALIZATION)
    })?;
    let canon = purrdf::try_canonicalize(&dataset).map_err(|_| {
        AstError::new(RefusalCode::MalformedAst, refusal_arm::CANONICALIZATION)
    })?;
    CANONICALIZATION_COUNT.set(CANONICALIZATION_COUNT.get() + 1);
    Ok(canon.nquads.into_bytes())
}

impl SelectStatement {
    const MAX_DEPTH: usize = 64;

    /// Compiles the AST into parameterized SQL compatible with DuckDB prepared statements ($1, $2, ...).
    pub fn compile(&self) -> Result<CompiledQuery, AstError> {
        validate_identifier(&self.from.name)?;
        if let Some(schema) = &self.from.schema {
            validate_qualified_identifier(schema)?;
        }
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
                Self::serialize_expr(item, &mut sql, &mut params, 0, ScopePos::SelectList)?;
            }
        }

        sql.push_str(" FROM ");
        if let Some(schema) = &self.from.schema {
            sql.push_str(schema);
            sql.push('.');
        }
        sql.push_str(&self.from.name);
        if let Some(alias) = &self.from.alias {
            sql.push(' ');
            sql.push_str(alias);
        }

        for join in &self.joins {
            validate_identifier(&join.table.name)?;
            if let Some(schema) = &join.table.schema {
                validate_qualified_identifier(schema)?;
            }
            if let Some(alias) = &join.table.alias {
                validate_identifier(alias)?;
            }
            sql.push(' ');
            sql.push_str(join.join_type.as_sql());
            sql.push(' ');
            if let Some(schema) = &join.table.schema {
                sql.push_str(schema);
                sql.push('.');
            }
            sql.push_str(&join.table.name);
            if let Some(alias) = &join.table.alias {
                sql.push(' ');
                sql.push_str(alias);
            }
            sql.push_str(" ON ");
            Self::serialize_expr(&join.condition, &mut sql, &mut params, 0, ScopePos::Predicate)?;
        }

        if let Some(where_clause) = &self.where_clause {
            sql.push_str(" WHERE ");
            Self::serialize_expr(where_clause, &mut sql, &mut params, 0, ScopePos::Predicate)?;
        }

        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            for (idx, item) in self.group_by.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(", ");
                }
                Self::serialize_expr(item, &mut sql, &mut params, 0, ScopePos::GroupBy)?;
            }
        }

        if let Some(having) = &self.having {
            sql.push_str(" HAVING ");
            Self::serialize_expr(having, &mut sql, &mut params, 0, ScopePos::Having)?;
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            for (idx, term) in self.order_by.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(", ");
                }
                Self::serialize_expr(&term.expr, &mut sql, &mut params, 0, ScopePos::OrderBy)?;
                if term.descending {
                    sql.push_str(" DESC");
                }
            }
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        // C1-a: SelectStatement::compile() alone is not graph-backed; only the
        // `compile_turtle` SelectStatement arm classifies as cacheable.
        Ok(CompiledQuery { sql, params, ast_graph_turtle: None, canon_graph_bytes: None, receipt_state: None, cacheable: false })
    }

    fn serialize_expr(
        expr: &SqlExpr,
        sql: &mut String,
        params: &mut Vec<SqlLiteral>,
        depth: usize,
        pos: ScopePos,
    ) -> Result<(), AstError> {
        if depth > Self::MAX_DEPTH {
            return Err(AstError::RecursionLimitExceeded(Self::MAX_DEPTH));
        }

        match expr {
            SqlExpr::ColumnRef { column, table_alias } => {
                validate_identifier(column)?;
                if let Some(alias) = table_alias {
                    // Qualified references (`main.regional_sales.region_id`)
                    // validate per dot-separated part.
                    validate_qualified_identifier(alias)?;
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
                Self::serialize_expr(left, sql, params, depth + 1, pos)?;
                sql.push(' ');
                sql.push_str(op.as_sql());
                sql.push(' ');
                Self::serialize_expr(right, sql, params, depth + 1, pos)?;
                sql.push(')');
            }
            SqlExpr::FunctionCall { function, argument } => {
                if !pos.allows_aggregate() {
                    // Typed arm: `num` carries the aggregate discriminant that
                    // `aggregate_debug_name` renders on the cold Display path.
                    return Err(AstError::new(RefusalCode::ScopeIntegrity, refusal_arm::AGG_IN_POSITION)
                        .with_num(match function {
                            AggregateFunction::Count => 0,
                            AggregateFunction::Sum => 1,
                            AggregateFunction::Avg => 2,
                            AggregateFunction::Min => 3,
                            AggregateFunction::Max => 4,
                        }));
                }
                sql.push_str(function.as_sql());
                sql.push('(');
                Self::serialize_expr(argument, sql, params, depth + 1, ScopePos::AggregateArgument)?;
                sql.push(')');
            }
            SqlExpr::Alias { expr, alias } => {
                if pos == ScopePos::AggregateArgument {
                    return Err(AstError::new(
                        RefusalCode::ScopeIntegrity,
                        refusal_arm::ALIASED_AGG_ARG,
                    ));
                }
                validate_identifier(alias)?;
                Self::serialize_expr(expr, sql, params, depth + 1, pos)?;
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

    /// Number of objects on `(node, predicate)`, across IRIs and literals.
    fn object_count(&self, node: &Self::Node, predicate: &str) -> usize;

    /// Counts only IRI-valued objects of `predicate` (dual-form
    /// discriminator refusals must not count string-literal objects).
    fn iri_object_count(&self, node: &Self::Node, predicate: &str) -> usize;

    /// Predicate IRIs asserted on `node` — backs the unknown-statement-
    /// predicate gate (D1/RUST-7: a rename must not be a silent-ignore
    /// escape hatch past the closed sql: predicate enum). Default: empty,
    /// so the gate degrades to a no-op for readers that cannot enumerate
    /// predicates (never a false refusal on a lawful document).
    fn predicate_iris(&self, _node: &Self::Node) -> Vec<String> {
        Vec::new()
    }

    /// SQLRDF-17: the admitted Turtle AST document this reader is backed
    /// by, when the reader is document-backed. Builtin eval admits only
    /// graph-backed queries compiled via `compile_turtle`, so a reader that
    /// cannot supply the document (default: `None`) is a typed REFUSAL-04
    /// at the builtin, never a hand-built-AST admission.
    fn query_document_turtle(&self) -> Option<String> {
        None
    }

    /// Cardinality-checked object read (audit E3/fuzz-4): more than one
    /// object on a single-cardinality clause predicate is REFUSAL-01; a
    /// duplicate must not win by document order.
    fn single_object_checked(
        &self,
        node: &Self::Node,
        predicate: &str,
    ) -> Result<Option<Self::Node>, AstError> {
        if self.object_count(node, predicate) > 1 {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DUP_CLAUSE_PREDICATE).with_ident(predicate).with_aux_ident(node));
        }
        Ok(self.get_single_object(node, predicate))
    }

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
                return Err(AstError::new(RefusalCode::MalformedList, 0));
            }

            let first = self.get_single_object(&curr, rdf_first)
                .ok_or_else(|| AstError::MissingProperty(rdf_first.to_string(), curr.to_string()))?;
            items.push(first);

            curr = self.get_single_object(&curr, rdf_rest)
                .ok_or_else(|| AstError::MissingProperty(rdf_rest.to_string(), curr.to_string()))?;
        }

        Ok(items)
    }

    /// Recursively extracts a `SqlExpr` from an AST node in the graph.
    /// Legacy entry point: expression is assumed to sit in a permissive
    /// position (see [`Self::extract_expr_scoped`] for the scope-integrity
    /// aware path used by the clause compiler).
    fn extract_expr(&self, node: &Self::Node, depth: usize) -> Result<SqlExpr, AstError> {
        self.extract_expr_scoped(node, depth, ScopePos::SelectList)
    }

    /// Scope-integrity aware extraction (rec 1.3/3.3/5). `pos` is the
    /// syntactic position of `node`; aggregates are only legal in SELECT
    /// list, HAVING and ORDER BY; an explicit `sql:ProcessScope` column may
    /// not surface in a projection list; an explicit `sql:ViewScope` column
    /// may not filter inside a predicate. A property-stuffing node carrying
    /// more than one expression discriminator is REFUSAL-01 (never silently
    /// extract one term and let the aggregate vanish).
    fn extract_expr_scoped(
        &self,
        node: &Self::Node,
        depth: usize,
        pos: ScopePos,
    ) -> Result<SqlExpr, AstError> {
        if depth > 32 {
            return Err(AstError::RecursionLimitExceeded(32));
        }

        let sql_ns = "http://graphlaw.org/sql#";

        // sql:scope is single-cardinality and only meaningful on column
        // references; a scope annotation on any other expression node is
        // REFUSAL-01, never silently ignored (fuzz 1/4).
        let scope_objects = self.object_count(node, "http://graphlaw.org/sql#scope");
        if scope_objects > 1 {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::SCOPE_CARDINALITY)
                .with_ident(node)
                .with_num(scope_objects));
        }
        let explicit_scope = self
            .get_single_object(node, "http://graphlaw.org/sql#scope")
            .and_then(|n| self.get_iri(&n))
            .or_else(|| {
                self.get_datatype_literal(node, "http://graphlaw.org/sql#scope")
                    .map(|(l, _)| l)
            });
        if explicit_scope.is_some() {
            // Scope is meaningful on column references and on output
            // expressions (the section 7 graph annotates the aliased
            // aggregate). It is meaningless on literals and binary/operator
            // nodes: carrying it there is REFUSAL-01, never silence.
            let is_column = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#columnName")
                .is_some();
            let is_output_expr = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#aliasName")
                .is_some()
                || self.get_datatype_literal(node, "http://graphlaw.org/sql#alias").is_some()
                || self.get_datatype_literal(node, "http://graphlaw.org/sql#functionName").is_some()
                || self.get_single_object(node, "http://graphlaw.org/sql#function").is_some();
            if !is_column && !is_output_expr {
                return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::SCOPE_WRONG_NODE)
                    .with_ident(node));
            }
        }

        // Ambiguity gate: at most one expression discriminator per node.
        let has_column = self.get_datatype_literal(node, "http://graphlaw.org/sql#columnName").is_some();
        let has_literal = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#literalValue")
            .or_else(|| self.get_datatype_literal(node, "http://graphlaw.org/sql#value"))
            .is_some();
        let has_alias_wrapper = self.get_datatype_literal(node, "http://graphlaw.org/sql#aliasName").is_some();
        let has_func_iri = self.get_single_object(node, "http://graphlaw.org/sql#function").is_some();
        let has_func_name = self.get_datatype_literal(node, "http://graphlaw.org/sql#functionName").is_some();
        let has_operator = self.get_single_object(node, "http://graphlaw.org/sql#operator").is_some()
            || self.get_datatype_literal(node, "http://graphlaw.org/sql#operator").is_some();
        let discriminators = [
            has_column, has_literal, has_alias_wrapper, has_func_iri, has_func_name, has_operator,
        ]
        .iter()
        .filter(|b| **b)
        .count();
        if discriminators > 1 {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DISCRIMINATORS)
                .with_ident(node)
                .with_num(discriminators));
        }

        // D3: `sql:expression` is the property of the alias wrapper
        // (sql:aliasName + sql:expression). Without sql:aliasName it is a
        // transparent expression wrapper (e.g. an sql:orderBy item annotating
        // an aggregate with sql:direction plus sql:expression). Carrying it
        // alongside any other discriminator is a laundering marker
        // (REFUSAL-01), never first-key-wins.
        let has_expr_wrapper = self
            .get_single_object(node, "http://graphlaw.org/sql#expression")
            .is_some();
        if has_expr_wrapper && !has_alias_wrapper && discriminators > 0 {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::EXPRESSION_WRAPPER)
                .with_ident(node));
        }

        // Check if node is a ColumnRef
        if has_column {
            let (col_name, _) = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#columnName")
                .expect("checked above");
            let alias = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#tableAlias")
                .map(|(a, _)| a);
            // sql:qualifier (section 7) renders as a dotted prefix.
            let qualifier = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#qualifier")
                .map(|(q, _)| q)
                .or(alias);

            // Scope integrity on explicit scope annotations (section 1.3/5).
            if let Some(scope_iri) = explicit_scope
            {
                let scope = Scope::from_iri(&scope_iri)?;
                if scope == Scope::Process && pos.is_view_surface() {
                    return Err(AstError::new(
                        RefusalCode::ScopeIntegrity,
                        refusal_arm::PROCESS_SCOPE_IN_SELECT,
                    )
                    .with_ident(&col_name));
                }
                if scope == Scope::View
                    && matches!(
                        pos,
                        ScopePos::Predicate | ScopePos::GroupBy | ScopePos::Having
                    )
                {
                    return Err(AstError::new(
                        RefusalCode::ScopeIntegrity,
                        refusal_arm::VIEW_SCOPE_IN_PREDICATE,
                    )
                    .with_ident(&col_name));
                }
                // TC-MUT-01 arm B: an explicit scope annotation on an
                // aggregate ARGUMENT beneath a predicate-rooted aggregate is
                // a laundering marker — the argument position is process-only
                // by construction, so the marker can only re-scope the
                // aggregate's output past the gate. The aggregate's own
                // legality (e.g. HAVING) must not inherit. Under an
                // output-position aggregate the section 7 example stays
                // lawful (AggregateArgument, no refusal).
                if pos == ScopePos::PredicateAggregateArgument {
                    return Err(AstError::new(
                        RefusalCode::ScopeIntegrity,
                        refusal_arm::SCOPE_IN_PRED_AGG_ARG,
                    )
                    .with_ident(&col_name)
                    .with_aux_ident(&scope_iri));
                }
            }

            let expr = SqlExpr::ColumnRef {
                column: col_name,
                table_alias: qualifier,
            };
            return Self::finish_alias(self, node, expr, sql_ns, pos);
        }

        // Check if node is a Literal
        if has_literal {
            let (val_str, dtype) = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#literalValue")
                .or_else(|| self.get_datatype_literal(node, "http://graphlaw.org/sql#value"))
                .expect("checked above");
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
            return Self::finish_alias(self, node, SqlExpr::Literal(lit), sql_ns, pos);
        }

        // Check if node is an Alias wrapper
        if has_alias_wrapper {
            if matches!(
                pos,
                ScopePos::AggregateArgument | ScopePos::PredicateAggregateArgument
            ) {
                return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::ALIAS_IN_AGG_ARG)
                    .with_ident(node));
            }
            let (alias_name, _) = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#aliasName")
                .expect("checked above");
            let target_expr_node = self
                .get_single_object(node, "http://graphlaw.org/sql#expression")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#expression".to_string(),
                        node.to_string(),
                    )
                })?;
            let inner_expr = self.extract_expr_scoped(&target_expr_node, depth + 1, pos)?;
            return Ok(SqlExpr::Alias {
                expr: Box::new(inner_expr),
                alias: alias_name,
            });
        }

        // Check if node is a FunctionCall (axiom IRI form: sql:function + sql:argument)
        if has_func_iri {
            let func_node = self
                .get_single_object(node, "http://graphlaw.org/sql#function")
                .expect("checked above");
            let func_iri = self.get_iri(&func_node).ok_or_else(|| {
                AstError::MissingProperty("http://graphlaw.org/sql#function".to_string(), node.to_string())
            })?;
            let function = AggregateFunction::from_iri(&func_iri)?;
            let arg_node = self
                .get_single_object(node, "http://graphlaw.org/sql#argument")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#argument".to_string(),
                        node.to_string(),
                    )
                })?;
            let argument = self.extract_expr_scoped(
                &arg_node,
                depth + 1,
                aggregate_argument_pos(pos),
            )?;
            let expr = Self::check_aggregate_position(
                SqlExpr::FunctionCall { function, argument: Box::new(argument) },
                pos,
            )?;
            return Self::finish_alias(self, node, expr, sql_ns, pos);
        }

        // FunctionCall (section 7 literal form: sql:functionName + sql:arguments list)
        if has_func_name {
            let (fname, _) = self
                .get_datatype_literal(node, "http://graphlaw.org/sql#functionName")
                .expect("checked above");
            let function = aggregate_from_name(&fname)?;
            let args_head = self
                .get_single_object(node, "http://graphlaw.org/sql#arguments")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#arguments".to_string(),
                        node.to_string(),
                    )
                })?;
            let args = self.unroll_rdf_list(&args_head, 8)?;
            let arg_node = args.first().ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#arguments".to_string(),
                    node.to_string(),
                )
            })?;
            let argument = self.extract_expr_scoped(
                arg_node,
                depth + 1,
                aggregate_argument_pos(pos),
            )?;
            let expr = Self::check_aggregate_position(
                SqlExpr::FunctionCall { function, argument: Box::new(argument) },
                pos,
            )?;
            return Self::finish_alias(self, node, expr, sql_ns, pos);
        }

        // Check if node is a BinaryExpression
        if has_operator {
            let op = self
                .read_operator(node)?
                .ok_or_else(|| {
                    AstError::MissingProperty("http://graphlaw.org/sql#operator".to_string(), node.to_string())
                })?;

            let left_legacy = self.single_object_checked(node, "http://graphlaw.org/sql#leftOperand")?;
            let left_norm = self.single_object_checked(node, "http://graphlaw.org/sql#left")?;
            if left_legacy.is_some() && left_norm.is_some() {
                return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_LEFT_OPERAND)
                    .with_ident(node));
            }
            let right_legacy = self.single_object_checked(node, "http://graphlaw.org/sql#rightOperand")?;
            let right_norm = self.single_object_checked(node, "http://graphlaw.org/sql#right")?;
            if right_legacy.is_some() && right_norm.is_some() {
                return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_RIGHT_OPERAND)
                    .with_ident(node));
            }
            let left_node = self
                .get_single_object(node, "http://graphlaw.org/sql#leftOperand")
                .or_else(|| self.get_single_object(node, "http://graphlaw.org/sql#left"))
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#leftOperand".to_string(),
                        node.to_string(),
                    )
                })?;
            let right_node = self
                .get_single_object(node, "http://graphlaw.org/sql#rightOperand")
                .or_else(|| self.get_single_object(node, "http://graphlaw.org/sql#right"))
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#rightOperand".to_string(),
                        node.to_string(),
                    )
                })?;

            let left = self.extract_expr_scoped(&left_node, depth + 1, pos)?;
            let right = self.extract_expr_scoped(&right_node, depth + 1, pos)?;

            return Ok(SqlExpr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        // D3: a bare sql:expression wrapper (no sql:aliasName, no other
        // discriminator — laundering guarded above) is transparent: extract
        // the inner expression in the SAME syntactic position, so an
        // sql:orderBy item carrying sql:expression -> aggregate extracts the
        // aggregate under ScopePos::OrderBy (which allows aggregates, rec 5)
        // instead of dying as an untyped "Unexpected terminal node".
        if let Some(inner) = self.get_single_object(node, "http://graphlaw.org/sql#expression") {
            return self.extract_expr_scoped(&inner, depth + 1, pos);
        }

        Err(AstError::MalformedNode(node.to_string()))
    }

    /// REFUSAL-03 gate: an aggregate is legal only in SELECT list, HAVING and
    /// ORDER BY; nested aggregates (COUNT(SUM(x))) refuse everywhere.
    fn check_aggregate_position(expr: SqlExpr, pos: ScopePos) -> Result<SqlExpr, AstError> {
        if !pos.allows_aggregate() {
            return Err(AstError::new(RefusalCode::ScopeIntegrity, refusal_arm::AGG_IN_PREDICATE));
        }
        Ok(expr)
    }

    /// Wraps an extracted expression in an output alias when the node carries
    /// `sql:alias` (section 7 attribute form; not an expression discriminator).
    fn finish_alias(
        this: &Self,
        node: &Self::Node,
        expr: SqlExpr,
        sql_ns: &str,
        pos: ScopePos,
    ) -> Result<SqlExpr, AstError> {
        if let Some((alias, _)) = this.get_datatype_literal(node, &format!("{sql_ns}alias")) {
            if matches!(
                pos,
                ScopePos::AggregateArgument | ScopePos::PredicateAggregateArgument
            ) {
                return Err(AstError::new(
                    RefusalCode::MalformedAst,
                    refusal_arm::SQL_ALIAS_IN_AGG_ARG,
                )
                .with_ident(node));
            }
        
            return Ok(SqlExpr::Alias {
                expr: Box::new(expr),
                alias,
            });
        }
        Ok(expr)
    }

    /// Reads `sql:operator` as either a section 4 string token (xsd:string
    /// literal) or a legacy axiom IRI. Disallowed tokens are REFUSAL-01.
    fn read_operator(&self, node: &Self::Node) -> Result<Option<Operator>, AstError> {
        // D5: dual-carry (string token AND IRI operator) is REFUSAL-01 — the
        // string used to win silently and the IRI term laundered past review.
        let has_string_operator = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#operator")
            .is_some();
        // MINIMAL UNBLOCK (compile-freeze SLA, lane R1, disclosed): count
        // only IRI objects — `object_count` includes string-literal
        // operators, which made every literal-operator expression a false
        // dual-carry refusal. Repaired (repair lane): order-independent via
        // a dedicated `iri_object_count` trait method — `get_single_object`
        // only inspects the first term, so the string-first ordering slipped
        // past the dual-carry gate.
        let iri_operator_objects = self
            .iri_object_count(node, "http://graphlaw.org/sql#operator");
        if has_string_operator && iri_operator_objects > 0 {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_OPERATOR_FORMS).with_ident(node));
        }
        if let Some((token, _)) = self.get_datatype_literal(node, "http://graphlaw.org/sql#operator") {
            return Operator::from_token(&token).map(Some);
        }
        if let Some(op_node) = self.get_single_object(node, "http://graphlaw.org/sql#operator") {
            let iri = self.get_iri(&op_node).ok_or_else(|| {
                AstError::MissingProperty("http://graphlaw.org/sql#operator".to_string(), node.to_string())
            })?;
            return Operator::from_iri(&iri).map(Some);
        }
        Ok(None)
    }

    /// Reads a data source: either the legacy `sql:fromTable` alias (single
    /// TableRef node) or the normative `sql:from` rdf:List of table
    /// identifiers with optional `sql:schemaName` (rec 3.3).
    fn extract_table_ref(&self, node: &Self::Node) -> Result<TableRef, AstError> {
        // D1 (fuzz law, rec 1.3/3.3): sql:scope is defined on column and
        // output-expression nodes only. A scope marker on a table-reference
        // node (sql:fromTable / sql:joinTable target) is a term out of
        // place — REFUSAL-01, never silently ignored.
        let scope_marker = self.object_count(node, "http://graphlaw.org/sql#scope") > 0
            || self
                .get_datatype_literal(node, "http://graphlaw.org/sql#scope")
                .is_some();
        if scope_marker {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::SCOPE_ON_TABLE_REF)
                .with_ident(node));
        }
        let (name, _) = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#tableName")
            .ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#tableName".to_string(),
                    node.to_string(),
                )
            })?;
        let alias = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#tableAlias")
            .map(|(a, _)| a);
        let schema = self
            .get_datatype_literal(node, "http://graphlaw.org/sql#schemaName")
            .map(|(s, _)| s);
        Ok(TableRef { name, alias, schema })
    }

    /// Reads the FROM clause: `sql:from` rdf:List first, `sql:fromTable`
    /// alias second. Neither present is REFUSAL-01.
    fn extract_from(&self, query_node: &Self::Node) -> Result<TableRef, AstError> {
        // Mixed-source documents are REFUSAL-01: the validated sql:fromTable
        // must not be a decoy for an unreviewed sql:from list (audit E2).
        let from_list = self.single_object_checked(query_node, "http://graphlaw.org/sql#from")?;
        let from_table = self.single_object_checked(query_node, "http://graphlaw.org/sql#fromTable")?;
        if from_list.is_some() && from_table.is_some() {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_FROM)
                .with_ident(query_node));
        }
        if let Some(from_head) = from_list {
            let items = self.unroll_rdf_list(&from_head, 64)?;
            let first = items.first().ok_or_else(|| {
                AstError::new(RefusalCode::MalformedAst, refusal_arm::EMPTY_FROM)
                    .with_ident(query_node)
            })?;
            return self.extract_table_ref(first);
        }
        if let Some(table_ref_node) = from_table {
            return self.extract_table_ref(&table_ref_node);
        }
        Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::NO_FROM).with_ident(query_node))
    }

    /// Extracts a full `SelectStatement` AST from a target query node.
    /// Normative property names (rec 3.3: sql:select/from/where/having/
    /// orderBy/groupBy) are read first; the SQLRDF-01 aliases (selectList/
    /// fromTable/whereClause/havingClause) stay legal.
    fn extract_select_statement(&self, query_node: &Self::Node) -> Result<SelectStatement, AstError> {
        // D1/RUST-7 unknown-predicate gate: every sql:-namespaced predicate on
        // the statement node must be in the closed enum the reader knows how
        // to review. A renamed clause (sql:whereX) must refuse typed
        // REFUSAL-01, never be dropped on the floor and admit the statement
        // without the filter the author wrote.
        const SQL_NS: &str = "http://graphlaw.org/sql#";
        const KNOWN_SQL_PREDICATES: &[&str] = &[
            "statementIndex", "selectList", "select", "from", "fromTable",
            "joinClause", "whereClause", "where", "having", "havingClause",
            "groupBy", "orderBy", "limit", "offset",
        ];
        for pred in self.predicate_iris(query_node) {
            if let Some(local) = pred.strip_prefix(SQL_NS) {
                if !KNOWN_SQL_PREDICATES.contains(&local) {
                    return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::UNKNOWN_STATEMENT_PREDICATE)
                        .with_ident(query_node)
                        .with_aux_ident(&pred));
                }
            }
        }
        let from = self.extract_from(query_node)?;

        // Joins (optional)
        let mut joins = Vec::new();
        if let Some(join_node) = self.get_single_object(query_node, "http://graphlaw.org/sql#joinClause") {
            let join_type = if let Some(type_node) = self.get_single_object(&join_node, "http://graphlaw.org/sql#joinType") {
                let type_iri = self.get_iri(&type_node).unwrap_or_default();
                JoinType::from_iri(&type_iri).unwrap_or(JoinType::Inner)
            } else {
                JoinType::Inner
            };

            let join_table_node = self.get_single_object(&join_node, "http://graphlaw.org/sql#joinTable")
                .ok_or_else(|| AstError::MissingProperty("http://graphlaw.org/sql#joinTable".to_string(), join_node.to_string()))?;
            let join_table = self.extract_table_ref(&join_table_node)?;

            let join_cond_node = self.get_single_object(&join_node, "http://graphlaw.org/sql#joinCondition")
                .ok_or_else(|| AstError::MissingProperty("http://graphlaw.org/sql#joinCondition".to_string(), join_node.to_string()))?;

            let join_condition = self.extract_expr_scoped(&join_cond_node, 0, ScopePos::Predicate)?;

            joins.push(JoinClause {
                join_type,
                table: join_table,
                condition: join_condition,
            });
        }

        // Unroll projection list (rdf:List): sql:selectList or sql:select.
        // Carrying both is REFUSAL-01: the reviewer must see the list the
        // engine executes (audit E1).
        let select_legacy = self.single_object_checked(query_node, "http://graphlaw.org/sql#selectList")?;
        let select_norm = self.single_object_checked(query_node, "http://graphlaw.org/sql#select")?;
        if select_legacy.is_some() && select_norm.is_some() {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_SELECT)
                .with_ident(query_node));
        }
        let select_head = select_legacy.or(select_norm);
        let select_head = select_head.ok_or_else(|| {
            AstError::new(RefusalCode::MalformedAst, refusal_arm::NO_PROJECTION)
                .with_ident(query_node)
        })?;
        let select_nodes = self.unroll_rdf_list(&select_head, 64)?;
        let mut select_list = Vec::with_capacity(select_nodes.len());
        for node in select_nodes {
            select_list.push(self.extract_expr_scoped(&node, 0, ScopePos::SelectList)?);
        }
        // D1: an empty projection list (rdf:nil head) is malformed, never
        // SELECT *: the ontology says projection is never zero-column
        // (both sql:selectList and sql:select paths funnel through here).
        if select_list.is_empty() {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::EMPTY_PROJECTION)
                .with_ident(query_node));
        }

        // Where clause (optional): sql:whereClause or sql:where; both is REFUSAL-01.
        let where_legacy = self.single_object_checked(query_node, "http://graphlaw.org/sql#whereClause")?;
        let where_norm = self.single_object_checked(query_node, "http://graphlaw.org/sql#where")?;
        if where_legacy.is_some() && where_norm.is_some() {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_WHERE)
                .with_ident(query_node));
        }
        let where_clause = if let Some(where_node) = where_legacy.or(where_norm) {
            Some(self.extract_expr_scoped(&where_node, 0, ScopePos::Predicate)?)
        } else {
            None
        };

        // Group by (optional rdf:List)
        let group_head = self.single_object_checked(query_node, "http://graphlaw.org/sql#groupBy")?;
        let group_by = if let Some(group_head) = group_head {
            let group_nodes = self.unroll_rdf_list(&group_head, 64)?;
            let mut list = Vec::with_capacity(group_nodes.len());
            for node in group_nodes {
                list.push(self.extract_expr_scoped(&node, 0, ScopePos::GroupBy)?);
            }
            list
        } else {
            Vec::new()
        };

        // Having (optional): sql:having or sql:havingClause; both is REFUSAL-01.
        let having_norm = self.single_object_checked(query_node, "http://graphlaw.org/sql#having")?;
        let having_alias = self.single_object_checked(query_node, "http://graphlaw.org/sql#havingClause")?;
        if having_norm.is_some() && having_alias.is_some() {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DUAL_HAVING)
                .with_ident(query_node));
        }
        let having = if let Some(having_node) = having_norm.or(having_alias) {
            Some(self.extract_expr_scoped(&having_node, 0, ScopePos::Having)?)
        } else {
            None
        };

        // Order by (optional rdf:List): sql:orderBy; items carry optional
        // sql:direction ("ASC" | "DESC", ASC default).
        let order_head = self.single_object_checked(query_node, "http://graphlaw.org/sql#orderBy")?;
        let order_by = if let Some(order_head) = order_head {
            let order_nodes = self.unroll_rdf_list(&order_head, 64)?;
            let mut list = Vec::with_capacity(order_nodes.len());
            for node in order_nodes {
                let expr = self.extract_expr_scoped(&node, 0, ScopePos::OrderBy)?;
                // A malformed direction is REFUSAL-01, never a silent fold to
                // ASC (same law as limit/offset below): only ASC/DESC admitted.
                // D4-adjacent: a present-but-IRI sql:direction is REFUSAL-01,
                // never a silent fold to ASC (distinguish present-but-IRI
                // from absent via object_count).
                let descending = if self.object_count(&node, "http://graphlaw.org/sql#direction") > 0 {
                    match self
                        .get_datatype_literal(&node, "http://graphlaw.org/sql#direction")
                    {
                        Some((d, _)) => {
                        if d.eq_ignore_ascii_case("DESC") {
                            true
                        } else if d.eq_ignore_ascii_case("ASC") {
                            false
                        } else {
                            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DIRECTION_LEX)
                                .with_ident(&d)
                                .with_aux_ident(query_node));
                        }
                    }
                    None => {
                        return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DIRECTION_IRI)
                            .with_ident(query_node))
                    }
                }
            } else {
                false
            };
                list.push(OrderTerm { expr, descending });
            }
            list
        } else {
            Vec::new()
        };

        // Limit & offset: a malformed integer is REFUSAL-01, never a silent
        // clause drop (audit E4: unbounded scan from a SHACL-clean doc).
        let parse_clause_int = |lex: String, prop: &str| -> Result<usize, AstError> {
            let arm = if prop == "limit" { refusal_arm::LIMIT_MALFORMED } else { refusal_arm::OFFSET_MALFORMED };
            lex.parse::<usize>().map_err(|_| {
                AstError::new(RefusalCode::MalformedAst, arm)
                    .with_ident(&lex)
                    .with_aux_ident(query_node)
            })
        };
        // E4 (lane E1-court defect fix, disclosed): sql:limit/sql:offset are
        // literal-valued, and `get_single_object` only surfaces IRI objects —
        // a literal clause previously read as ABSENT, silently dropping LIMIT
        // and never refusing malformed integers. Drive both clauses off
        // `get_datatype_literal` + `object_count` instead.
        let limit_clause = |prop: &str| -> Result<Option<usize>, AstError> {
            let count = self.object_count(query_node, &format!("http://graphlaw.org/sql#{prop}"));
            if count > 1 {
                return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::DUP_CLAUSE_PREDICATE).with_ident(prop).with_aux_ident(query_node));
            }
            match self.get_datatype_literal(query_node, &format!("http://graphlaw.org/sql#{prop}")) {
                Some((lex, _)) => parse_clause_int(lex, prop).map(Some),
                None if count > 0 => {
                    let arm = if prop == "limit" { refusal_arm::LIMIT_NOT_INT } else { refusal_arm::OFFSET_NOT_INT };
                    Err(AstError::new(RefusalCode::MalformedAst, arm).with_ident(query_node))
                }
                None => Ok(None),
            }
        };
        let limit = limit_clause("limit")?;
        let offset = limit_clause("offset")?;

        Ok(SelectStatement {
            from,
            joins,
            select_list,
            where_clause,
            group_by,
            having,
            order_by,
            limit,
            offset,
        })
    }

    /// Extracts a section 7 `MergeIntoStatement`: target table identifier,
    /// grouped USING sub-select, merge ON condition, and WHEN MATCHED update
    /// assignments. Any other shape falls back to the legacy scalar template.
    fn extract_merge_statement(&self, node: &Self::Node) -> Result<MergeStatement, AstError> {
        let target_node = self
            .single_object_checked(node, "http://graphlaw.org/sql#targetTable")?
            .ok_or_else(|| {
                AstError::MissingProperty("http://graphlaw.org/sql#targetTable".to_string(), node.to_string())
            })?;
        let target = self.extract_table_ref(&target_node)?;

        let using_node = self
            .single_object_checked(node, "http://graphlaw.org/sql#usingSource")?
            .ok_or_else(|| {
                AstError::MissingProperty("http://graphlaw.org/sql#usingSource".to_string(), node.to_string())
            })?;
        let source = self.extract_select_statement(&using_node)?;

        let on_node = self
            .single_object_checked(node, "http://graphlaw.org/sql#mergeOn")?
            .ok_or_else(|| {
                AstError::MissingProperty("http://graphlaw.org/sql#mergeOn".to_string(), node.to_string())
            })?;
        let on = self.extract_expr_scoped(&on_node, 0, ScopePos::Predicate)?;

        let when_node = self
            .single_object_checked(node, "http://graphlaw.org/sql#whenMatched")?
            .ok_or_else(|| {
                AstError::MissingProperty("http://graphlaw.org/sql#whenMatched".to_string(), node.to_string())
            })?;
        let assigns_head = self
            .get_single_object(&when_node, "http://graphlaw.org/sql#setAssignments")
            .ok_or_else(|| {
                AstError::MissingProperty(
                    "http://graphlaw.org/sql#setAssignments".to_string(),
                    when_node.to_string(),
                )
            })?;
        let mut assignments = Vec::new();
        for assign in self.unroll_rdf_list(&assigns_head, 64)? {
            let (column, _) = self
                .get_datatype_literal(&assign, "http://graphlaw.org/sql#targetColumn")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#targetColumn".to_string(),
                        assign.to_string(),
                    )
                })?;
            let value_node = self
                .get_single_object(&assign, "http://graphlaw.org/sql#valueExpression")
                .ok_or_else(|| {
                    AstError::MissingProperty(
                        "http://graphlaw.org/sql#valueExpression".to_string(),
                        assign.to_string(),
                    )
                })?;
            let value = self.extract_expr_scoped(&value_node, 0, ScopePos::Predicate)?;
            assignments.push((column, value));
        }

        Ok(MergeStatement { target, source, on, assignments })
    }

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
        SelectStatement::serialize_expr(&self.on, &mut on_sql, &mut params, 0, ScopePos::Predicate)?;
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
        Ok(CompiledQuery { sql, params, ast_graph_turtle: None, canon_graph_bytes: None, receipt_state: None, cacheable: false })
    }
}

/// Validates an RDF AST dataset in Turtle syntax against the canonical SHACL rules in `ontologies/sql-ast.ttl`.
pub fn validate_ast_shacl(
    turtle_dataset: &str,
    shapes_ttl: &str,
) -> Result<(), AstError> {
    if let Err(over) = crate::shacl_guard::check_shape_depth(shapes_ttl) {
        return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::SHACL_DEPTH)
            .with_num(over.observed)
            .with_aux_num(crate::shacl_guard::MAX_SHACL_VALIDATION_DEPTH));
    }

    let ds = purrdf::parse_dataset(turtle_dataset.as_bytes(), "text/turtle", None)
        .map_err(|_| AstError::new(RefusalCode::MalformedAst, refusal_arm::PARSE_DATASET))?;

    let shapes = crate::shacl::engine::parse_shapes(shapes_ttl, None)
        .map_err(|_| AstError::new(RefusalCode::MalformedAst, refusal_arm::PARSE_SHAPES))?;

    let report = crate::shacl::engine::validate_dataset(&ds, &shapes)
        .map_err(|_| AstError::new(RefusalCode::MalformedAst, refusal_arm::SHACL_ENGINE))?;

    if !report.conforms {
        // Typed SHACL_VIOLATIONS carrier: first violation renders on the cold
        // path, `num` counts the remainder ("(+N more violations)").
        let first = report.results.first().ok_or_else(|| {
            AstError::new(RefusalCode::MalformedAst, refusal_arm::SHACL_VIOLATIONS)
        })?;
        return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::SHACL_VIOLATIONS)
            .with_ident(&first.focus_node)
            .with_aux_ident(first.message.as_deref().unwrap_or("violation"))
            .with_num(report.results.len()));
    }

    Ok(())
}

/// SHACL-check `turtle` against the SQL AST shapes and compile its statement
/// into SQL. `SelectStatement` uses the expression compiler. Other admitted
/// statement classes compile from `sql:targetTable` and assignment literals.
/// `ConnectStatement` and `DisconnectStatement` refuse here, before any engine call.
/// Charter-A04 seam: process-wide `h_ast → Arc<CompiledQuery>` memoization.
/// A repeated identical admitted graph (same `h_ast = BLAKE3(Canon(G_AST))`)
/// must never re-run `serialize_expr` (or SHACL validation / RDFC-1.0
/// canonicalization for the re-render): the memo returns the already-admitted
/// compiled form. Arc-fed so the stored compiled query is never re-built —
/// callers clone out of the Arc. Capped (flush-on-overflow at the same 256
/// capacity class as the prepared-statement cache) and keyed by the digest,
/// so a flipped literal in the graph forks to a distinct entry by
/// construction.
static AST_COMPILE_MEMO: std::sync::OnceLock<
    std::sync::Mutex<HashMap<[u8; 32], std::sync::Arc<CompiledQuery>>>,
> = std::sync::OnceLock::new();

const AST_COMPILE_MEMO_CAPACITY: usize = 256;

fn ast_compile_memo() -> &'static std::sync::Mutex<HashMap<[u8; 32], std::sync::Arc<CompiledQuery>>> {
    AST_COMPILE_MEMO.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

pub fn compile_turtle(turtle: &str) -> Result<CompiledQuery, AstError> {
    let shapes = format!(
        "{}\n{}",
        include_str!("../ontologies/sql-ast.ttl"),
        include_str!("../ggen/sql-dialect/statement_classes.ttl")
    );
    validate_ast_shacl(turtle, &shapes)?;
    let reader = DatasetReader::from_turtle(turtle)?;
    let (node, kind) = reader.statement_kind()?;
    // Section 4 (StatementBaseShape): every statement MUST carry exactly one
    // integer sql:statementIndex. Missing, duplicated, wrongly-typed, or
    // negative is REFUSAL-01 and refuses before any engine call (fuzz D1).
    let require_index = |reader: &DatasetReader, n: &String| -> Result<(), AstError> {
        match reader.object_count(n, "http://graphlaw.org/sql#statementIndex") {
            0 => {
                return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::INDEX_MISSING).with_ident(&n))
            }
            c if c > 1 => {
                return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::INDEX_DUP)
                    .with_ident(n)
                    .with_num(c))
            }
            _ => {}
        }
        let (index_lex, datatype) = reader
            .get_datatype_literal(n, "http://graphlaw.org/sql#statementIndex")
            .ok_or_else(|| {
                AstError::new(RefusalCode::MalformedAst, refusal_arm::INDEX_NOT_INT).with_ident(n)
            })?;
        if datatype.as_deref() != Some("http://www.w3.org/2001/XMLSchema#integer") {
            return Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::INDEX_BAD_DTYPE)
                .with_ident(n));
        }
        match index_lex.parse::<i64>() {
            Ok(v) if v >= 0 => Ok(()),
            _ => Err(AstError::new(RefusalCode::MalformedAst, refusal_arm::INDEX_NEGATIVE)
                .with_ident(&index_lex)
                .with_aux_ident(n)),
        }
    };
    require_index(&reader, &node)?;
    // Fuzz D2: nested statement nodes are statements too — the §7 usingSource
    // sub-select carries its own index per §7's example graph.
    if let Some(using) = reader.get_single_object(&node, "http://graphlaw.org/sql#usingSource") {
        require_index(&reader, &using)?;
    }
    // R1: canonicalize ONCE at compile time (not per receipt). Failure is a
    // typed REFUSAL-01 AstError, never a crash.
    let canon_bytes = canonical_graph_bytes(turtle)?;
    // Charter-A04 memoization: identical h_ast → reuse the compiled form,
    // skipping serialize_expr entirely. The memo is consulted only on the
    // SelectStatement (cacheable-class) arm; merge/command/refusal arms are
    // never memoized. `ast_graph_turtle` is restamped from THIS call's
    // source (canonically identical graphs may differ textually).
    let h_ast = *blake3::hash(&canon_bytes).as_bytes();
    if kind.as_str() == "SelectStatement" {
        if let Some(arc) = ast_compile_memo().lock().ok().and_then(|m| m.get(&h_ast).cloned()) {
            let mut cq = (*arc).clone();
            cq.ast_graph_turtle = Some(turtle.to_string());
            return Ok(cq);
        }
    }
    let compiled = match kind.as_str() {
        "SelectStatement" => reader.extract_select_statement(&node)?.compile(),
        // REFUSAL-02 (UnimplementedStatementRefusal): registered carrier is
        // the typed AstError::UnsupportedTerm carrying the statement-class
        // IRI, with the "compile refusal" marker the execution court pins.
        "ConnectStatement" | "DisconnectStatement" => Err(AstError::new(
            RefusalCode::UnimplementedStatement,
            refusal_arm::UNIMPLEMENTED_STATEMENT,
        )
        .with_ident(&kind)),
        "MergeIntoStatement"
            if reader
                .get_single_object(&node, "http://graphlaw.org/sql#usingSource")
                .is_some() =>
        {
            reader.extract_merge_statement(&node)?.compile_merge()
        }
        other => reader.compile_command(&node, other),
    };
    let mut compiled = compiled.map(|mut cq| {
        // P28-SIL §3.1 + TC-HOT-01 two-node formula: pin the receipt anchor
        // `h_ast = BLAKE3(Canon(G_AST))` exactly ONCE at compile time —
        // BEFORE moving the bytes into `canon_graph_bytes`. Eval then
        // computes `H_query = BLAKE3(h_ast || P_params)` with one fresh
        // hasher per receipt.
        cq.receipt_state = Some(h_ast);
        cq.ast_graph_turtle = Some(turtle.to_string());
        cq.canon_graph_bytes = Some(canon_bytes);
        // C1-a classification (designs/C1-cache.md): only the SelectStatement
        // arm of `compile_turtle` is ever cacheable. Merge and command arms
        // (and every hand-built constructor) compile with `cacheable: false`;
        // no cache store exists yet — this bit only gates future admission.
        cq.cacheable = kind.as_str() == "SelectStatement";
        cq
    })?;
    // Charter-A04 memoization store: only cacheable-class (SelectStatement)
    // compiles enter the memo. Flush-on-overflow at the pinned capacity so
    // the memo never grows unbounded.
    if kind.as_str() == "SelectStatement" {
        if let Ok(mut memo) = ast_compile_memo().lock() {
            if memo.len() >= AST_COMPILE_MEMO_CAPACITY && !memo.contains_key(&h_ast) {
                memo.clear();
            }
            memo.insert(h_ast, std::sync::Arc::new(compiled.clone()));
        }
    }
    Ok(compiled)
}

struct DatasetReader {
    outgoing: HashMap<String, HashMap<String, Vec<RdfTerm>>>,
    /// The admitted Turtle document this reader was parsed from
    /// (SQLRDF-17: lets document-backed callers compile via `compile_turtle`
    /// so every admission carries canon bytes).
    turtle: String,
}

#[derive(Clone)]
enum RdfTerm {
    Iri(String),
    Literal { lexical: String, datatype: Option<String> },
}

impl DatasetReader {
    fn from_turtle(turtle: &str) -> Result<Self, AstError> {
        let dataset = purrdf::parse_dataset(turtle.as_bytes(), "text/turtle", None)
            .map_err(|_| AstError::new(RefusalCode::MalformedAst, refusal_arm::PARSE_DATASET))?;
        let mut outgoing: HashMap<String, HashMap<String, Vec<RdfTerm>>> = HashMap::new();
        for quad in dataset.quads() {
            let subj = match dataset.term_value(quad.s) {
                purrdf::TermValue::Iri(i) => i,
                purrdf::TermValue::Blank { label, scope } => format!("_:{label}_{}", scope.0),
                _ => continue,
            };
            let pred = match dataset.term_value(quad.p) {
                purrdf::TermValue::Iri(i) => i,
                _ => continue,
            };
            let term = match dataset.term_value(quad.o) {
                purrdf::TermValue::Iri(i) => RdfTerm::Iri(i),
                purrdf::TermValue::Blank { label, scope } => {
                    RdfTerm::Iri(format!("_:{label}_{}", scope.0))
                }
                purrdf::TermValue::Literal { lexical_form, datatype, .. } => RdfTerm::Literal {
                    lexical: lexical_form,
                    datatype: Some(datatype),
                },
                _ => continue,
            };
            outgoing.entry(subj).or_default().entry(pred).or_default().push(term);
        }
        Ok(Self {
            outgoing,
            turtle: turtle.to_string(),
        })
    }

    /// Finds the top-level statement node. Nested statement-class nodes
    /// (e.g. the section 7 `sql:usingSource` sub-select) are objects of other
    /// subjects, so a candidate that is nobody's object wins; ties break on
    /// the lexicographically smallest subject for determinism.
    fn statement_kind(&self) -> Result<(String, String), AstError> {
        let ty = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        let prefix = "http://graphlaw.org/sql#";
        let is_object: HashSet<String> = self
            .outgoing
            .values()
            .flat_map(|preds| preds.values().flatten())
            .filter_map(|t| match t {
                RdfTerm::Iri(i) if i.starts_with("_:") => Some(i.clone()),
                _ => None,
            })
            .collect();
        let mut candidates: Vec<(String, String)> = Vec::new();
        for (subj, preds) in &self.outgoing {
            let Some(objs) = preds.get(ty) else { continue };
            for term in objs {
                let RdfTerm::Iri(iri) = term else { continue };
                if let Some(kind) = iri.strip_prefix(prefix) {
                    if kind.ends_with("Statement") && !is_object.contains(subj) {
                        candidates.push((subj.clone(), kind.to_string()));
                    }
    }
            }
        }
        candidates.sort();
        candidates
            .into_iter()
            .next()
            .ok_or_else(|| AstError::MissingProperty(ty.into(), "sql:*Statement".into()))
    }

    fn ident(&self, node: &str, pred: &str) -> Result<String, AstError> {
        let node = node.to_string();
        let (lex, _) = self.get_datatype_literal(&node, pred).ok_or_else(|| {
            AstError::MissingProperty(pred.into(), node.into())
        })?;
        validate_identifier(&lex).map(str::to_string)
    }

    fn lit(&self, node: &str, pred: &str) -> Result<String, AstError> {
        let node = node.to_string();
        let (lex, dtype) = self.get_datatype_literal(&node, pred).ok_or_else(|| {
            AstError::MissingProperty(pred.into(), node.into())
        })?;
        let numeric = dtype
            .as_deref()
            .is_some_and(|d| d.ends_with("integer") || d.ends_with("double"))
            || lex.chars().all(|c| c.is_ascii_digit());
        if numeric {
            Ok(lex)
        } else {
            Ok(format!("'{}'", lex.replace('\'', "''")))
        }
    }

    /// Compiles the expression-level form of a command class from its
    /// structured graph (nested operand nodes, rdf:Lists, nested SELECT
    /// expressions). Returns `Ok(None)` when the statement does not use the
    /// expression-level vocabulary, so the scalar template vocabulary remains
    /// a lawful fallback (rec 3.2: both forms are the same admitted class).
    fn compile_expression_form(&self, node: &str, kind: &str) -> Result<Option<String>, AstError> {
        let node = node.to_string();
        let p = |name: &str| format!("http://graphlaw.org/sql#{name}");
        match kind {
            "CreateStatement" => {
                let Some(def) = self.get_single_object(&node, &p("tableDefinition")) else {
                    return Ok(None);
                };
                let table = self.ident(&def, &p("tableName"))?;
                let cols_head = self
                    .get_single_object(&def, &p("columns"))
                    .ok_or_else(|| AstError::MissingProperty(p("columns"), def.clone()))?;
                let mut cols = Vec::new();
                for col in self.unroll_rdf_list(&cols_head, 64)? {
                    let name = self.ident(&col, &p("columnName"))?;
                    let ty = self.column_type(&col)?;
                    cols.push(format!("{name} {ty}"));
                }
                Ok(Some(format!("CREATE TABLE {table} ({})", cols.join(", "))))
            }
            "DropStatement" => {
                let Some(target) = self.get_single_object(&node, &p("dropTarget")) else {
                    return Ok(None);
                };
                let table = self.ident(&target, &p("tableName"))?;
                let if_exists = self
                    .get_datatype_literal(&target, &p("ifExists"))
                    .map(|(v, _)| v == "true" || v == "1")
                    .unwrap_or(false);
                Ok(Some(
                    if if_exists {
                        format!("DROP TABLE IF EXISTS {table}")
                    } else {
                        format!("DROP TABLE {table}")
                    },
                ))
            }
            "AlterStatement" => {
                let Some(action) = self.get_single_object(&node, &p("alterAction")) else {
                    return Ok(None);
                };
                let table = self.ident(&action, &p("tableName"))?;
                let col = self.ident(&action, &p("columnName"))?;
                let ty = self.column_type(&action)?;
                Ok(Some(format!(
                    "ALTER TABLE {table} ADD COLUMN {col} {ty}"
                )))
            }
            "TruncateStatement" => {
                let Some(target) = self.get_single_object(&node, &p("truncateTarget")) else {
                    return Ok(None);
                };
                let table = self.ident(&target, &p("tableName"))?;
                Ok(Some(format!("TRUNCATE {table}")))
            }
            "AnalyzeStatement" => {
                let Some(target) = self.get_single_object(&node, &p("analyzeTarget")) else {
                    return Ok(None);
                };
                let table = self.ident(&target, &p("tableName"))?;
                Ok(Some(format!("ANALYZE {table}")))
            }
            "CommentStatement" => {
                let Some(comment) = self.get_single_object(&node, &p("comment")) else {
                    return Ok(None);
                };
                let object = self
                    .get_single_object(&comment, &p("commentObject"))
                    .ok_or_else(|| AstError::MissingProperty(p("commentObject"), comment.clone()))?;
                let object = match object.as_str() {
                    "http://graphlaw.org/sql#Table" => "TABLE",
                    other => {
                        return Err(AstError::UnsupportedTerm(other.to_string()));
                    }
                };
                let name = self.ident(&comment, &p("commentTargetName"))?;
                let text = self
                    .get_datatype_literal(&comment, &p("commentText"))
                    .ok_or_else(|| AstError::MissingProperty(p("commentText"), comment.clone()))?;
                Ok(Some(format!(
                    "COMMENT ON {object} {name} IS '{}'",
                    text.0.replace('\'', "''")
                )))
            }
            "CopyStatement" => {
                let Some(op) = self.get_single_object(&node, &p("copyOperation")) else {
                    return Ok(None);
                };
                let table = self.ident(&op, &p("tableName"))?;
                let (path, _) = self
                    .get_datatype_literal(&op, &p("filePath"))
                    .ok_or_else(|| AstError::MissingProperty(p("filePath"), op.clone()))?;
                let fmt = self
                    .get_datatype_literal(&op, &p("fileFormat"))
                    .map(|(f, _)| f)
                    .unwrap_or_else(|| "CSV".to_string());
                if !matches!(fmt.as_str(), "CSV" | "PARQUET" | "JSON") {
                    return Err(AstError::UnsupportedTerm(fmt));
                }
                Ok(Some(format!(
                    "COPY {table} TO '{}' (FORMAT {fmt})",
                    path.replace('\'', "''")
                )))
            }
            "ExportStatement" => {
                let Some((path, _)) = self.get_datatype_literal(&node, &p("exportPath")) else {
                    return Ok(None);
                };
                Ok(Some(format!(
                    "EXPORT DATABASE '{}'",
                    path.replace('\'', "''")
                )))
            }
            "ImportStatement" => {
                let Some((path, _)) = self.get_datatype_literal(&node, &p("importPath")) else {
                    return Ok(None);
                };
                Ok(Some(format!(
                    "IMPORT DATABASE '{}'",
                    path.replace('\'', "''")
                )))
            }
            "PrepareStatement" => {
                let Some(target) = self.get_single_object(&node, &p("prepareTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("statementName"))?;
                let query_node = self
                    .get_single_object(&target, &p("prepareQuery"))
                    .ok_or_else(|| AstError::MissingProperty(p("prepareQuery"), target.clone()))?;
                let query = self.inline_select(&query_node)?;
                Ok(Some(format!("PREPARE {name} AS {query}")))
            }
            "ExecuteStatement" => {
                let Some(target) = self.get_single_object(&node, &p("executeTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("statementName"))?;
                Ok(Some(format!("EXECUTE {name}")))
            }
            "DeallocateStatement" => {
                let Some(target) = self.get_single_object(&node, &p("deallocateTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("statementName"))?;
                Ok(Some(format!("DEALLOCATE {name}")))
            }
            "ExplainStatement" => {
                let Some(query_node) = self.get_single_object(&node, &p("explainQuery")) else {
                    return Ok(None);
                };
                let query = self.inline_select(&query_node)?;
                Ok(Some(format!("EXPLAIN {query}")))
            }
            "CallStatement" => {
                let Some(target) = self.get_single_object(&node, &p("callTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("routineName"))?;
                let args = match self.get_single_object(&target, &p("arguments")) {
                    Some(head) => {
                        let items = self.unroll_rdf_list(&head, 64)?;
                        let mut rendered = Vec::with_capacity(items.len());
                        for item in items {
                            rendered.push(self.inline_expr(&item)?);
                        }
                        rendered.join(", ")
                    }
                    None => String::new(),
                };
                Ok(Some(format!("CALL {name}({args})")))
            }
            "PragmaStatement" => {
                let Some(target) = self.get_single_object(&node, &p("pragmaTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("pragmaName"))?;
                Ok(Some(format!("PRAGMA {name}")))
            }
            "SetStatement" => {
                let Some(setting) = self.get_single_object(&node, &p("configSetting")) else {
                    return Ok(None);
                };
                let key = self.ident(&setting, &p("configKey"))?;
                let value_node = self
                    .get_single_object(&setting, &p("configValue"))
                    .ok_or_else(|| AstError::MissingProperty(p("configValue"), setting.clone()))?;
                let value = self.inline_expr(&value_node)?;
                Ok(Some(format!("SET {key} = {value}")))
            }
            "ResetStatement" => {
                let Some(target) = self.get_single_object(&node, &p("resetTarget")) else {
                    return Ok(None);
                };
                let key = self.ident(&target, &p("configKey"))?;
                Ok(Some(format!("RESET {key}")))
            }
            "TransactionStatement" => {
                // BEGIN/COMMIT/ROLLBACK compile to engine text; transaction
                // state is a connection-runtime property, so COMMIT/ROLLBACK
                // without an active BEGIN surfaces the engine's
                // TransactionContext error through the typed
                // EvaluatorError::DuckDb path at execution (documented
                // behavior, same class as a missing-table engine error).
                let action = self
                    .get_single_object(&node, &p("transactionAction"))
                    .unwrap_or_else(|| "http://graphlaw.org/sql#BeginTransaction".to_string());
                let sql = match action.as_str() {
                    "http://graphlaw.org/sql#BeginTransaction" => "BEGIN",
                    "http://graphlaw.org/sql#CommitTransaction" => "COMMIT",
                    "http://graphlaw.org/sql#RollbackTransaction" => "ROLLBACK",
                    other => return Err(AstError::UnsupportedTerm(other.to_string())),
                };
                Ok(Some(sql.to_string()))
            }
            "LoadStatement" => {
                let Some(target) = self.get_single_object(&node, &p("loadTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("extensionName"))?;
                Ok(Some(format!("LOAD {name}")))
            }
            "InstallStatement" => {
                let Some(target) = self.get_single_object(&node, &p("installTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("extensionName"))?;
                let from = match self.get_datatype_literal(&target, &p("repositoryPath")) {
                    Some((path, _)) => format!(" FROM '{}'", path.replace('\'', "''")),
                    None => String::new(),
                };
                Ok(Some(format!("INSTALL {name}{from}")))
            }
            "AttachStatement" => {
                let Some(target) = self.get_single_object(&node, &p("attachTarget")) else {
                    return Ok(None);
                };
                let (path, _) = self
                    .get_datatype_literal(&target, &p("databasePath"))
                    .ok_or_else(|| AstError::MissingProperty(p("databasePath"), target.clone()))?;
                let alias = self.ident(&target, &p("attachAlias"))?;
                Ok(Some(format!(
                    "ATTACH '{}' AS {alias}",
                    path.replace('\'', "''")
                )))
            }
            "DetachStatement" => {
                let Some(target) = self.get_single_object(&node, &p("detachTarget")) else {
                    return Ok(None);
                };
                let alias = self.ident(&target, &p("attachAlias"))?;
                Ok(Some(format!("DETACH {alias}")))
            }
            "UseStatement" => {
                let Some(target) = self.get_single_object(&node, &p("useTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("databaseName"))?;
                Ok(Some(format!("USE {name}")))
            }
            "ExtensionRepositoryStatement" => {
                let Some(target) = self.get_single_object(&node, &p("repositoryTarget")) else {
                    return Ok(None);
                };
                let name = self.ident(&target, &p("repositoryName"))?;
                let (prefix, _) = self
                    .get_datatype_literal(&target, &p("repositoryPrefix"))
                    .ok_or_else(|| {
                        AstError::MissingProperty(p("repositoryPrefix"), target.clone())
                    })?;
                let (key, _) = self
                    .get_datatype_literal(&target, &p("publicKey"))
                    .ok_or_else(|| AstError::MissingProperty(p("publicKey"), target.clone()))?;
                Ok(Some(format!(
                    "SET allow_extension_repositories='allowed'; CREATE OR REPLACE EXTENSION REPOSITORY {name} WITH PREFIX '{}' USING PUBLIC KEY '{}'",
                    prefix.replace('\'', "''"),
                    key.replace('\'', "''")
                )))
            }
            _ => Ok(None),
        }
    }

    /// Validates a column-type literal against the closed allowlist.
    fn column_type(&self, node: &str) -> Result<String, AstError> {
        let ty = "http://graphlaw.org/sql#columnType";
        let (lex, _) = self
            .get_datatype_literal(&node.to_string(), ty)
            .ok_or_else(|| AstError::MissingProperty(ty.to_string(), node.to_string()))?;
        let upper = lex.to_ascii_uppercase();
        match upper.as_str() {
            "INTEGER" | "BIGINT" | "VARCHAR" | "DOUBLE" | "BOOLEAN" | "DATE" => {
                Ok(upper)
            }
            other => Err(AstError::UnsupportedTerm(format!(
                "column type {other}"
            ))),
        }
    }

    /// Compiles a nested SELECT expression graph into inline SQL (literals
    /// rendered in place, not as bind parameters) for embedding in EXPLAIN /
    /// PREPARE.
    fn inline_select(&self, query_node: &str) -> Result<String, AstError> {
        let stmt = self.extract_select_statement(&query_node.to_string())?;
        validate_identifier(&stmt.from.name)?;
        let mut sql = String::from("SELECT ");
        if stmt.select_list.is_empty() {
            sql.push('*');
        } else {
            for (idx, node) in stmt.select_list.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(", ");
                }
                sql.push_str(&self.inline_expr_node(node)?);
            }
        }
        sql.push_str(" FROM ");
        sql.push_str(&stmt.from.name);
        if let Some(alias) = &stmt.from.alias {
            validate_identifier(alias)?;
            sql.push(' ');
            sql.push_str(alias);
        }
        for join in &stmt.joins {
            validate_identifier(&join.table.name)?;
            sql.push(' ');
            sql.push_str(join.join_type.as_sql());
            sql.push(' ');
            sql.push_str(&join.table.name);
            if let Some(alias) = &join.table.alias {
                validate_identifier(alias)?;
                sql.push(' ');
                sql.push_str(alias);
            }
            sql.push_str(" ON ");
            sql.push_str(&self.inline_expr_node(&join.condition)?);
        }
        if let Some(w) = &stmt.where_clause {
            sql.push_str(" WHERE ");
            sql.push_str(&self.inline_expr_node(w)?);
        }
        if !stmt.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            for (idx, g) in stmt.group_by.iter().enumerate() {
                if idx > 0 {
                    sql.push_str(", ");
                }
                sql.push_str(&self.inline_expr_node(g)?);
            }
        }
        if let Some(limit) = stmt.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        if let Some(offset) = stmt.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }
        Ok(sql)
    }

    /// Renders an already-extracted `SqlExpr` inline (literals in place).
    fn inline_expr_node(&self, expr: &SqlExpr) -> Result<String, AstError> {
        let mut sql = String::new();
        self.write_inline_expr(expr, &mut sql, 0)?;
        Ok(sql)
    }

    /// Extracts and renders an operand expression node from the graph.
    fn inline_expr(&self, node: &str) -> Result<String, AstError> {
        let expr = self.extract_expr(&node.to_string(), 0)?;
        self.inline_expr_node(&expr)
    }

    fn write_inline_expr(&self, expr: &SqlExpr, sql: &mut String, depth: usize) -> Result<(), AstError> {
        if depth > 32 {
            return Err(AstError::RecursionLimitExceeded(32));
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
            SqlExpr::Literal(lit) => match lit {
                SqlLiteral::Integer(i) => sql.push_str(&i.to_string()),
                SqlLiteral::Float(f) => sql.push_str(&f.to_string()),
                SqlLiteral::Boolean(b) => sql.push_str(if *b { "TRUE" } else { "FALSE" }),
                SqlLiteral::Null => sql.push_str("NULL"),
                SqlLiteral::String(s) => {
                    sql.push('\'');
                    sql.push_str(&s.replace('\'', "''"));
                    sql.push('\'');
                }
            },
            SqlExpr::Binary { op, left, right } => {
                sql.push('(');
                self.write_inline_expr(left, sql, depth + 1)?;
                sql.push(' ');
                sql.push_str(op.as_sql());
                sql.push(' ');
                self.write_inline_expr(right, sql, depth + 1)?;
                sql.push(')');
            }
            SqlExpr::FunctionCall { function, argument } => {
                sql.push_str(function.as_sql());
                sql.push('(');
                self.write_inline_expr(argument, sql, depth + 1)?;
                sql.push(')');
            }
            SqlExpr::Alias { expr, alias } => {
                validate_identifier(alias)?;
                self.write_inline_expr(expr, sql, depth + 1)?;
                sql.push_str(" AS ");
                sql.push_str(alias);
            }
        }
        Ok(())
    }

    fn compile_command(&self, node: &str, kind: &str) -> Result<CompiledQuery, AstError> {
        if let Some(sql) = self.compile_expression_form(node, kind)? {
            return Ok(CompiledQuery { sql, params: Vec::new(), ast_graph_turtle: None, canon_graph_bytes: None, receipt_state: None, cacheable: false });
        }
        let table = "http://graphlaw.org/sql#targetTable";
        let col = "http://graphlaw.org/sql#assignColumn";
        let val = "http://graphlaw.org/sql#assignValue";
        let wcol = "http://graphlaw.org/sql#whereColumn";
        let wval = "http://graphlaw.org/sql#whereValue";
        let sql = match kind {
            "InsertStatement" => format!(
                "INSERT INTO {} ({}) VALUES ({})",
                self.ident(node, table)?,
                self.ident(node, col)?,
                self.lit(node, val)?
            ),
            "UpdateStatement" => {
                let mut sql = format!(
                    "UPDATE {} SET {} = {}",
                    self.ident(node, table)?,
                    self.ident(node, col)?,
                    self.lit(node, val)?
                );
                if self.get_datatype_literal(&node.to_string(), wcol).is_some() {
                    sql.push_str(&format!(
                        " WHERE {} = {}",
                        self.ident(node, wcol)?,
                        self.lit(node, wval)?
                    ));
                }
                sql
            }
            "DeleteStatement" => {
                let t = self.ident(node, table)?;
                if self.get_datatype_literal(&node.to_string(), wcol).is_some() {
                    format!(
                        "DELETE FROM {t} WHERE {} = {}",
                        self.ident(node, wcol)?,
                        self.lit(node, wval)?
                    )
                } else {
                    format!("DELETE FROM {t}")
                }
            }
            "CreateStatement" => format!(
                "CREATE TABLE {} ({} INTEGER)",
                self.ident(node, table)?,
                self.ident(node, col)?
            ),
            "DropStatement" => format!("DROP TABLE IF EXISTS {}", self.ident(node, table)?),
            "AlterStatement" => format!(
                "ALTER TABLE {} ADD COLUMN {} INTEGER",
                self.ident(node, table)?,
                self.ident(node, col)?
            ),
            "TruncateStatement" => format!("TRUNCATE {}", self.ident(node, table)?),
            "AnalyzeStatement" => format!("ANALYZE {}", self.ident(node, table)?),
            "ExplainStatement" => format!(
                "EXPLAIN SELECT {} FROM {}",
                self.ident(node, col)?,
                self.ident(node, table)?
            ),
            "CopyStatement" => {
                let t = self.ident(node, table)?;
                format!("COPY {t} TO '/tmp/graphlaw-rdf-{t}.csv' (FORMAT CSV)")
            }
            "ExportStatement" => {
                let n = self.ident(node, table)?;
                format!("EXPORT DATABASE '/tmp/graphlaw-rdf-export-{n}'")
            }
            "ImportStatement" => {
                let n = self.ident(node, table)?;
                format!(
                    "ATTACH '/tmp/graphlaw-rdf-{n}.duckdb' AS {n}; USE {n}; CREATE OR REPLACE TABLE only_src(id INTEGER); EXPORT DATABASE '/tmp/graphlaw-rdf-export-{n}'; USE memory; DETACH {n}; IMPORT DATABASE '/tmp/graphlaw-rdf-export-{n}'"
                )
            }
            "MergeIntoStatement" => {
                let t = self.ident(node, table)?;
                let c = self.ident(node, col)?;
                let v = self.lit(node, val)?;
                format!("MERGE INTO {t} USING (SELECT {v} AS {c}) s ON {t}.{c} = s.{c} WHEN MATCHED THEN UPDATE SET {c} = s.{c}")
            }
            "VacuumStatement" => "VACUUM".into(),
            "CheckpointStatement" => "CHECKPOINT".into(),
            "TransactionStatement" => "BEGIN".into(),
            "ExternalResourceStatement" => "SHOW EXTERNAL RESOURCES".into(),
            "UpdateExtensionsStatement" => "UPDATE EXTENSIONS".into(),
            "SetStatement" => format!(
                "SET {} = {}",
                self.ident(node, table)?,
                self.lit(node, val)?
            ),
            "ResetStatement" => format!("RESET {}", self.ident(node, table)?),
            "PragmaStatement" => format!("PRAGMA {}", self.ident(node, table)?),
            "CallStatement" => format!("CALL {}()", self.ident(node, table)?),
            "PrepareStatement" => format!(
                "PREPARE {} AS SELECT {}",
                self.ident(node, table)?,
                self.lit(node, val)?
            ),
            "ExecuteStatement" => format!(
                "PREPARE {} AS SELECT {}; EXECUTE {}",
                self.ident(node, table)?,
                self.lit(node, val)?,
                self.ident(node, table)?
            ),
            "DeallocateStatement" => format!(
                "PREPARE {} AS SELECT 1; DEALLOCATE {}",
                self.ident(node, table)?,
                self.ident(node, table)?
            ),
            "CommentStatement" => format!(
                "COMMENT ON TABLE {} IS {}",
                self.ident(node, table)?,
                self.lit(node, val)?
            ),
            "LoadStatement" => format!("LOAD {}", self.ident(node, table)?),
            "InstallStatement" => format!(
                "INSTALL {} FROM {}",
                self.ident(node, table)?,
                self.lit(node, val)?
            ),
            "AttachStatement" => format!("ATTACH ':memory:' AS {}", self.ident(node, table)?),
            "UseStatement" => {
                let n = self.ident(node, table)?;
                format!("ATTACH ':memory:' AS {n}; USE {n}")
            }
            "DetachStatement" => {
                let n = self.ident(node, table)?;
                format!("ATTACH ':memory:' AS {n}; DETACH {n}")
            }
            "ExtensionRepositoryStatement" => format!(
                "SET allow_extension_repositories='allowed'; CREATE OR REPLACE EXTENSION REPOSITORY {} WITH PREFIX {} USING PUBLIC KEY {}",
                self.ident(node, table)?,
                self.lit(node, col)?,
                self.lit(node, val)?
            ),
            other => {
                return Err(AstError::UnsupportedTerm(format!(
                    "compile refusal: {other} has no SQL template"
                )))
            }
        };
        Ok(CompiledQuery { sql, params: Vec::new(), ast_graph_turtle: None, canon_graph_bytes: None, receipt_state: None, cacheable: false })
    }
}

impl RdfAstGraphReader for DatasetReader {
    type Node = String;

    fn query_document_turtle(&self) -> Option<String> {
        Some(self.turtle.clone())
    }

    fn object_count(&self, node: &String, predicate: &str) -> usize {
        self.outgoing
            .get(node)
            .and_then(|preds| preds.get(predicate))
            .map_or(0, Vec::len)
    }

    fn predicate_iris(&self, node: &String) -> Vec<String> {
        self.outgoing
            .get(node)
            .map(|preds| preds.keys().cloned().collect())
            .unwrap_or_default()
    }

    fn iri_object_count(&self, node: &String, predicate: &str) -> usize {
        self.outgoing
            .get(node)
            .and_then(|preds| preds.get(predicate))
            .map_or(0, |terms| {
                terms.iter().filter(|t| matches!(t, RdfTerm::Iri(_))).count()
            })
    }

    fn get_single_object(&self, subject: &Self::Node, predicate: &str) -> Option<Self::Node> {        self.outgoing.get(subject)?.get(predicate)?.first().and_then(|term| match term {
            RdfTerm::Iri(i) => Some(i.clone()),
            _ => None,
        })
    }

    fn get_datatype_literal(
        &self,
        subject: &Self::Node,
        predicate: &str,
    ) -> Option<(String, Option<String>)> {
        self.outgoing.get(subject)?.get(predicate)?.first().and_then(|term| match term {
            RdfTerm::Literal { lexical, datatype } => Some((lexical.clone(), datatype.clone())),
            _ => None,
        })
    }

    fn get_iri(&self, node: &Self::Node) -> Option<String> {
        Some(node.clone())
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
                schema: None,
            },
            joins: vec![],
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
            having: None,
            order_by: Vec::new(),
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
                schema: None,
            },
            joins: vec![],
            select_list: vec![],
            where_clause: None,
            group_by: vec![],
            having: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
        };

        let result = stmt.compile();
        assert!(matches!(result, Err(e) if matches!(e.code, RefusalCode::InvalidIdentifier)));
    }
}

#[cfg(test)]
mod r1_canon_once_tests {
    use super::*;
    fn count() -> usize { CANONICALIZATION_COUNT.get() }


    /// A minimal valid section-5 SELECT graph (real compile, real purrdf
    /// canonicalization — Chicago, no mocks). Single test owns the global
    /// CANONICALIZATION_COUNT to avoid cross-test parallel races.
    fn select_graph() -> String {
        "@prefix : <http://example.org/> .\n@prefix sql: <http://graphlaw.org/sql#> .\n:g a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"t\" ] ;\n    sql:selectList ( [ sql:columnName \"v\" ] ) .".to_string()
    }

    /// R1 non-vacuity: compile_turtle canonicalizes exactly ONCE per compile,
    /// carries the bytes, and the carried bytes are bit-identical to an
    /// independent RDFC-1.0 canonicalization.
    #[test]
    fn canon_happens_once_per_compile_and_bytes_are_bit_identical() {
        let before = count();
        let compiled = compile_turtle(&select_graph()).expect("graph must compile");
        let after_compile = count();
        assert_eq!(after_compile, before + 1, "canonicalization must run exactly once per compile_turtle");
        assert!(compiled.canon_graph_bytes.is_some(), "compile_turtle must carry Canon(G_AST) bytes");

        // Bit-identity: carried canon bytes equal an independent RDFC-1.0
        // canonical form computed outside the compile path.
        let dataset =
            purrdf::parse_dataset(select_graph().as_bytes(), "text/turtle", None).expect("parse");
        let independent = purrdf::try_canonicalize(&dataset).expect("canonicalize").nquads;
        assert_eq!(
            compiled.canon_graph_bytes.as_deref(),
            Some(independent.as_bytes()),
            "carried Canon(G_AST) bytes must be bit-identical to an independent canonicalization"
        );

        // A second compile canonicalizes again (once per compile, no global
        // memoization) — the counter is a true counter.
        compile_turtle(&select_graph()).expect("compile 2");
        compile_turtle(&select_graph()).expect("compile 3");
        assert_eq!(count(), after_compile + 2);
    }
}
