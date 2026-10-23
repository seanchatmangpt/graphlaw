#![allow(missing_docs)]
#![allow(clippy::result_large_err)]

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
    /// REFUSAL-01 arm: an sql:joinType IRI that is neither sql:InnerJoin nor
    /// sql:LeftJoin (unknown join-type IRIs refuse typed; a MISSING joinType
    /// predicate defaults to Inner and is not an arm here).
    pub const JOIN_TYPE_IRI: u16 = 37;
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
    /// Appended v26.10.22 (lane L4): an xsd:integer-typed `sql:literalValue`
    /// whose lexical form does not parse as an i64. Refuses typed; the value
    /// is never silently stringified or coerced to 0.
    LiteralNotInt = 36,
    /// Appended v26.10.22 (lane L4): an xsd:double-typed `sql:literalValue`
    /// whose lexical form does not parse as an f64. Refuses typed.
    LiteralNotFloat = 37,
}

/// Fixed-size refusal context: shape id, predicate index, numeric arguments,
/// and packed idents. `Copy`, no `String`, no heap — construction is a pair
/// of bounded memcpys (hot path budget <10ns).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
        Self {
            len: 0,
            bytes: [0u8; REFUSAL_IDENT_BYTES],
        }
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
        let mut w = FixedWriter {
            buf: &mut self.bytes,
            len: self.len as usize,
        };
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

impl AstError {
    #[inline]
    pub(super) fn new(code: RefusalCode, shape_id: u16) -> Self {
        Self {
            code,
            ctx: RefusalContext {
                shape_id,
                ..RefusalContext::default()
            },
        }
    }
    #[inline]
    pub(super) fn with_num(mut self, num: usize) -> Self {
        self.ctx.num = num as u32;
        self
    }
    #[inline]
    pub(super) fn with_aux_num(mut self, aux_num: usize) -> Self {
        self.ctx.aux_num = aux_num as u32;
        self
    }
    #[inline]
    pub(super) fn with_ident(mut self, ident: impl fmt::Display) -> Self {
        self.ctx.ident.push(ident);
        self
    }
    #[inline]
    pub(super) fn with_aux_ident(mut self, aux: impl fmt::Display) -> Self {
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
        AstError::new(RefusalCode::MissingProperty, 0)
            .with_ident(a)
            .with_aux_ident(b)
    }
    #[allow(non_snake_case)]
    pub fn InvalidIdentifier(a: String) -> Self {
        AstError::new(RefusalCode::InvalidIdentifier, 0).with_ident(a)
    }
    #[allow(non_snake_case)]
    pub fn UnsupportedTerm(a: String) -> Self {
        AstError::new(
            RefusalCode::UnsupportedTerm,
            refusal_arm::UNSUPPORTED_GENERIC,
        )
        .with_ident(a)
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
            RefusalCode::MalformedList => {
                write!(f, "Malformed RDF list: circular reference or unclosed node")
            }
            RefusalCode::MalformedNode => {
                write!(f, "Unexpected terminal node: {id}")
            }
            RefusalCode::LiteralNotInt => write!(
                f,
                "SHACL AST structural violation: {REFUSAL_01_MALFORMED_AST}xsd:integer literal '{id}' on {aux} does not parse as an integer"
            ),
            RefusalCode::LiteralNotFloat => write!(
                f,
                "SHACL AST structural violation: {REFUSAL_01_MALFORMED_AST}xsd:double literal '{id}' on {aux} does not parse as a float"
            ),
            RefusalCode::UnsupportedTerm | RefusalCode::UnimplementedStatement => {
                match self.ctx.shape_id {
                    arm::UNIMPLEMENTED_STATEMENT => write!(
                        f,
                        "compile refusal: http://graphlaw.org/sql#{id} {REFUSAL_02_UNIMPLEMENTED} not implemented by this DuckDB build"
                    ),
                    arm::NO_SQL_TEMPLATE => write!(
                        f,
                        "Unsupported operator or function IRI: compile refusal: {id} has no SQL template"
                    ),
                    _ => write!(f, "Unsupported operator or function IRI: {id}"),
                }
            }
            RefusalCode::MalformedAst => {
                // ShaclViolation carriers render the structural prefix, the
                // stable marker, then the arm template.
                let marker = self.code.marker().unwrap_or("");
                match self.ctx.shape_id {
                    arm::OPERATOR_TOKEN => write!(
                        f,
                        "SHACL AST structural violation: {marker}operator token '{id}' is not in the section 4 enum (= != < <= > >= AND OR + - * / % LIKE ILIKE)"
                    ),
                    arm::SCOPE_INDIVIDUAL => write!(
                        f,
                        "SHACL AST structural violation: {marker}'{id}' is not a sql:Scope individual (sql:ViewScope | sql:ProcessScope)"
                    ),
                    arm::CANONICALIZATION => write!(
                        f,
                        "SHACL AST structural violation: {marker} admitted AST graph canonicalization refused"
                    ),
                    arm::DUP_CLAUSE_PREDICATE => write!(
                        f,
                        "SHACL AST structural violation: {marker}{id} on {aux} appears more than once; clause predicates are single-cardinality"
                    ),
                    arm::SCOPE_CARDINALITY => write!(
                        f,
                        "SHACL AST structural violation: {marker}node {id} carries {} sql:scope values; scope is single-cardinality",
                        self.ctx.num
                    ),
                    arm::SCOPE_WRONG_NODE => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:scope on {id} is only defined on column or output-expression nodes, not on literals or operator nodes"
                    ),
                    arm::DISCRIMINATORS => write!(
                        f,
                        "SHACL AST structural violation: {marker}node {id} carries {} expression discriminators (columnName/literalValue/value/aliasName/function/functionName/operator); exactly one is allowed",
                        self.ctx.num
                    ),
                    arm::EXPRESSION_WRAPPER => write!(
                        f,
                        "SHACL AST structural violation: {marker}node {id} carries sql:expression alongside another expression discriminator; a bare sql:expression wrapper must be the node's only term"
                    ),
                    arm::ALIAS_IN_AGG_ARG => write!(
                        f,
                        "SHACL AST structural violation: {marker}aliased expression {id} inside an aggregate argument is not serializable SQL (fuzz 3: SUM(a AS col))"
                    ),
                    arm::DUAL_LEFT_OPERAND => write!(
                        f,
                        "SHACL AST structural violation: {marker}binary expression {id} carries both sql:leftOperand and sql:left; mixed operand names launder a term past review"
                    ),
                    arm::DUAL_RIGHT_OPERAND => write!(
                        f,
                        "SHACL AST structural violation: {marker}binary expression {id} carries both sql:rightOperand and sql:right; mixed operand names launder a term past review"
                    ),
                    arm::DUAL_OPERATOR_FORMS => write!(
                        f,
                        "SHACL AST structural violation: {marker}binary expression {id} carries sql:operator as both a string token and an IRI; exactly one operator form is allowed"
                    ),
                    arm::SQL_ALIAS_IN_AGG_ARG => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:alias on {id} inside an aggregate argument is not serializable SQL"
                    ),
                    arm::SCOPE_ON_TABLE_REF => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:scope on table-reference node {id} is a term out of place; scope is defined on column or output-expression nodes, never on sql:fromTable/sql:joinTable table references"
                    ),
                    arm::DUAL_FROM => write!(
                        f,
                        "SHACL AST structural violation: {marker}statement {id} carries both sql:from and sql:fromTable; exactly one data source clause is allowed"
                    ),
                    arm::EMPTY_FROM => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:from list on {id} is empty"
                    ),
                    arm::NO_FROM => write!(
                        f,
                        "SHACL AST structural violation: {marker}statement {id} has no sql:from (or legacy sql:fromTable) clause"
                    ),
                    arm::DUAL_SELECT => write!(
                        f,
                        "SHACL AST structural violation: {marker}SelectStatement {id} carries both sql:selectList and sql:select; exactly one projection list is allowed"
                    ),
                    arm::NO_PROJECTION => write!(
                        f,
                        "SHACL AST structural violation: {marker}SelectStatement {id} has no projection list (sql:select or sql:selectList)"
                    ),
                    arm::EMPTY_PROJECTION => write!(
                        f,
                        "SHACL AST structural violation: {marker}SelectStatement {id} has an empty projection list; projection is never zero-column (SELECT * is not implied by rdf:nil)"
                    ),
                    arm::DUAL_WHERE => write!(
                        f,
                        "SHACL AST structural violation: {marker}SelectStatement {id} carries both sql:whereClause and sql:where; exactly one filter clause is allowed"
                    ),
                    arm::DUAL_HAVING => write!(
                        f,
                        "SHACL AST structural violation: {marker}SelectStatement {id} carries both sql:having and sql:havingClause; exactly one filter clause is allowed"
                    ),
                    arm::DIRECTION_LEX => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:direction '{id}' on {aux} is not ASC or DESC"
                    ),
                    arm::DIRECTION_IRI => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:direction on {id} must be an ASC or DESC string literal, not an IRI"
                    ),
                    arm::LIMIT_MALFORMED | arm::OFFSET_MALFORMED => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:{} '{id}' on {aux} is not a valid non-negative integer",
                        if self.ctx.shape_id == arm::LIMIT_MALFORMED {
                            "limit"
                        } else {
                            "offset"
                        }
                    ),
                    arm::LIMIT_DUP | arm::OFFSET_DUP => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:{} on {id} appears more than once; clause predicates are single-cardinality",
                        if self.ctx.shape_id == arm::LIMIT_DUP {
                            "limit"
                        } else {
                            "offset"
                        }
                    ),
                    arm::LIMIT_NOT_INT | arm::OFFSET_NOT_INT => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:{} on {id} must be an xsd:integer literal",
                        if self.ctx.shape_id == arm::LIMIT_NOT_INT {
                            "limit"
                        } else {
                            "offset"
                        }
                    ),
                    arm::INDEX_MISSING => write!(
                        f,
                        "SHACL AST structural violation: {marker}statement {id} is missing required sql:statementIndex (section 4 StatementBaseShape)"
                    ),
                    arm::INDEX_DUP => write!(
                        f,
                        "SHACL AST structural violation: {marker}statement {id} carries {} sql:statementIndex values, maxCount is 1",
                        self.ctx.num
                    ),
                    arm::INDEX_NOT_INT => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:statementIndex on {id} is not an xsd:integer literal (section 4 datatype)"
                    ),
                    arm::INDEX_BAD_DTYPE => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:statementIndex on {id} has a non-integer datatype, expected xsd:integer"
                    ),
                    arm::INDEX_NEGATIVE => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:statementIndex '{id}' on {aux} is not a non-negative xsd:integer"
                    ),
                    arm::UNKNOWN_STATEMENT_PREDICATE => write!(
                        f,
                        "SHACL AST structural violation: {marker}statement {id} carries unknown sql: predicate '{aux}'; term out of place — the sql: predicate set is a closed enum, a rename is not a silent-ignore escape hatch"
                    ),
                    arm::JOIN_TYPE_IRI => write!(
                        f,
                        "SHACL AST structural violation: {marker}sql:joinType IRI '{id}' is not sql:InnerJoin or sql:LeftJoin; an unknown join-type IRI refuses typed, it never silently downgrades to Inner"
                    ),
                    arm::SHACL_DEPTH => write!(
                        f,
                        "SHACL AST structural violation: SHACL nesting depth exceeded: observed {}, max {}",
                        self.ctx.num, self.ctx.aux_num
                    ),
                    arm::PARSE_DATASET => write!(
                        f,
                        "SHACL AST structural violation: Failed to parse AST dataset"
                    ),
                    arm::PARSE_SHAPES => write!(
                        f,
                        "SHACL AST structural violation: Failed to parse SHACL shapes"
                    ),
                    arm::SHACL_ENGINE => write!(
                        f,
                        "SHACL AST structural violation: Validation engine failure"
                    ),
                    arm::LEGACY_MESSAGE => {
                        write!(f, "SHACL AST structural violation: {marker}{id}")
                    }
                    arm::SHACL_VIOLATIONS => {
                        if self.ctx.num > 1 {
                            write!(
                                f,
                                "SHACL AST structural violation: Node <{id}>: {aux} (+{} more violations)",
                                self.ctx.num - 1
                            )
                        } else {
                            write!(f, "SHACL AST structural violation: Node <{id}>: {aux}")
                        }
                    }
                    _ => write!(f, "SHACL AST structural violation: {marker}node {id}"),
                }
            }
            RefusalCode::ScopeIntegrity => {
                // REFUSAL-03 arms: per-code arm namespace (same numeric ids
                // as the MalformedAst namespace, different code).
                let marker = self.code.marker().unwrap_or("");
                match self.ctx.shape_id {
                    arm::AGG_IN_POSITION => write!(
                        f,
                        "SHACL AST structural violation: {marker}aggregate {} is not allowed in this syntactic position (section 1.3/5: WHERE, JOIN, GROUP BY are process-only)",
                        aggregate_debug_name(self.ctx.num)
                    ),
                    arm::ALIASED_AGG_ARG => write!(
                        f,
                        "SHACL AST structural violation: {marker}aliased aggregate argument is not serializable SQL"
                    ),
                    arm::AGG_IN_PREDICATE => write!(
                        f,
                        "SHACL AST structural violation: {marker}aggregate in a WHERE/JOIN/GROUP BY position is refused (section 1.3/5); re-scope the expression"
                    ),
                    arm::PROCESS_SCOPE_IN_SELECT => write!(
                        f,
                        "SHACL AST structural violation: {marker}column '{id}' is sql:ProcessScope but sits in the projection list (section 1.3: a selected column crosses into the consumer's view)"
                    ),
                    arm::VIEW_SCOPE_IN_PREDICATE => write!(
                        f,
                        "SHACL AST structural violation: {marker}column '{id}' is sql:ViewScope but sits inside a predicate (section 1.3: WHERE/JOIN/HAVING columns are processed, not viewed)"
                    ),
                    arm::SCOPE_IN_PRED_AGG_ARG => write!(
                        f,
                        "SHACL AST structural violation: {marker}column '{id}' is {aux} inside an aggregate argument in a predicate position (section 1.3: the predicate-side argument is process-only by construction; an explicit sql:scope marker re-scopes the aggregate past review)"
                    ),
                    arm::LEGACY_MESSAGE => {
                        write!(f, "SHACL AST structural violation: {marker}{id}")
                    }
                    _ => write!(f, "SHACL AST structural violation: {marker}node {id}"),
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
