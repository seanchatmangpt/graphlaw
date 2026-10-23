use super::sniff::skeleton;
use super::{Dialect, Engine, ParseRefusal};

/// Why a document was refused.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::dialect::RefusalKind;
///
/// let k = RefusalKind::ResourceLimit;
/// let retryable = match k {
///     RefusalKind::ResourceLimit => false,
///     _ => true, // required: new kinds may be added
/// };
/// assert!(!retryable);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RefusalKind {
    /// Empty input, an HTML page, or an HTTP error body.
    NotSemanticContent,
    /// Routing cannot tell two dialects apart without a hint.
    Ambiguous,
    /// The owning engine rejected the document.
    EngineRejected,
    /// The document is valid but cannot be used in this operation.
    Unsupported,
    /// The request exceeded a documented resource cap (see `abi::MAX_*`).
    ResourceLimit,
    /// The document belongs to the other semantic tier (R04): e.g. N3 logic
    /// given to an RDF parser, or N3 logic sniffed with no tier declaration.
    /// See `crate::tier`.
    TierMismatch,
    /// The embedded DuckDB execution substrate (ABI op `duckdb`, linked into
    /// the wasm module) failed or the request misused it. Never traps:
    /// the caller gets this typed refusal instead. Absence from the hashed
    /// 14-op list is not this refusal and is not REFUSAL-04.
    DuckDb,
}

/// Diagnostic text on a [`Refusal`]. Literals stay a `'static` borrow so the
/// hot path does not allocate; engine and `format!` diagnostics own a `String`.
#[derive(Debug, Clone)]
pub enum RefusalMessage {
    /// A literal diagnostic. No heap allocation.
    Static(&'static str),
    /// A diagnostic built at runtime (`format!`, engine errors).
    Dynamic(String),
}

impl RefusalMessage {
    /// The diagnostic text, whether static or owned.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(s) => s,
            Self::Dynamic(s) => s,
        }
    }
}

impl std::ops::Deref for RefusalMessage {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl From<&'static str> for RefusalMessage {
    fn from(s: &'static str) -> Self {
        Self::Static(s)
    }
}

impl From<String> for RefusalMessage {
    fn from(s: String) -> Self {
        Self::Dynamic(s)
    }
}

impl From<RefusalMessage> for String {
    fn from(m: RefusalMessage) -> Self {
        match m {
            RefusalMessage::Static(s) => s.to_owned(),
            RefusalMessage::Dynamic(s) => s,
        }
    }
}

impl std::fmt::Display for RefusalMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A typed refusal naming the engine and dialect that refused.
#[derive(Debug, Clone)]
pub struct Refusal {
    /// Machine-readable refusal class.
    pub kind: RefusalKind,
    /// Dialect that was refused, when known.
    pub dialect: Option<Dialect>,
    /// Engine that refused, when known.
    pub engine: Option<Engine>,
    /// Diagnostic text from the refusing engine or router.
    pub message: RefusalMessage,
}

impl Refusal {
    pub(crate) fn engine(dialect: Dialect, message: impl std::fmt::Debug) -> Self {
        Refusal {
            kind: RefusalKind::EngineRejected,
            dialect: Some(dialect),
            engine: Some(dialect.engine()),
            message: RefusalMessage::Dynamic(format!("{message:?}")),
        }
    }

    pub(super) fn routing(kind: RefusalKind, message: &'static str) -> Self {
        Refusal {
            kind,
            dialect: None,
            engine: None,
            message: RefusalMessage::Static(message),
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} refusal", self.kind)?;
        if let (Some(e), Some(d)) = (self.engine, self.dialect) {
            write!(f, " by {e:?} ({d:?})")?;
        }
        write!(f, ": {}", self.message.as_str())
    }
}

impl std::error::Error for Refusal {}

/// Why a backend or router refused to parse a document, at parse granularity.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::dialect::ReasonClass;
///
/// let r = ReasonClass::Syntax;
/// let is_limit = match r {
///     ReasonClass::Limit { .. } => true,
///     _ => false, // required: new classes may be added
/// };
/// assert!(!is_limit);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReasonClass {
    /// The document violates the dialect's grammar.
    Syntax,
    /// A prefixed name uses a prefix the document never declared.
    UndefinedPrefix,
    /// An IRI reference is malformed or not absolute where required.
    BadIri,
    /// N3 formula syntax (`{ ... }`, `=>`) was offered to an RDF dialect:
    /// the RDF layer cannot represent quoted graphs. Refused at routing, before
    /// any engine sees the document.
    FormulaInRdfLayer,
    /// The input is not a semantic document at all (HTML, empty, error body).
    NotSemanticContent,
    /// Routing cannot tell two dialects apart without a hint.
    AmbiguousDialect,
    /// A JSON-LD `@context` references a remote IRI; fetching is forbidden, so
    /// the document is refused without any network attempt.
    RemoteContext,
    /// A documented resource cap was hit.
    Limit {
        /// Machine name of the cap (`n3_match_steps`, `n3_derived_facts`, ...).
        name: String,
        /// What was measured against the cap.
        observed: u64,
        /// The configured ceiling.
        max: u64,
    },
}

impl std::fmt::Display for ReasonClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasonClass::Syntax => write!(f, "syntax error"),
            ReasonClass::UndefinedPrefix => write!(f, "undefined prefix"),
            ReasonClass::BadIri => write!(f, "bad IRI"),
            ReasonClass::FormulaInRdfLayer => write!(f, "N3 formula in RDF layer"),
            ReasonClass::NotSemanticContent => write!(f, "not semantic content"),
            ReasonClass::AmbiguousDialect => write!(f, "ambiguous dialect"),
            ReasonClass::RemoteContext => write!(f, "remote @context forbidden"),
            ReasonClass::Limit { name, .. } => write!(f, "limit `{name}` reached"),
        }
    }
}

impl From<eyeron::ReasonerLimit> for ReasonClass {
    fn from(limit: eyeron::ReasonerLimit) -> Self {
        ReasonClass::Limit {
            name: limit.to_string(),
            observed: 0,
            max: 0,
        }
    }
}

/// N3 formula syntax seen in the structural skeleton of an RDF-dialect document.
pub(super) fn formula_in_rdf_layer(bytes: &[u8], dialect: Dialect) -> Option<ParseRefusal> {
    if !matches!(
        dialect,
        Dialect::Turtle | Dialect::NTriples | Dialect::NQuads
    ) {
        return None;
    }
    let text = std::str::from_utf8(bytes).ok()?;
    let skel = skeleton(text.trim_start_matches('\u{feff}'));
    let token = if skel.contains("=>") {
        "=>"
    } else if skel.contains('{') {
        "{"
    } else {
        return None;
    };
    Some(ParseRefusal {
        dialect,
        engine: dialect.engine(),
        reason: ReasonClass::FormulaInRdfLayer,
        line: None,
        col: None,
        byte_offset: None,
        snippet: None,
        message: format!(
            "N3 formula syntax (`{token}`) cannot be parsed as {:?}; \
             re-route as N3 or expand the formula",
            dialect
        ),
    })
}

/// True when the JSON-LD document's `@context` references a remote IRI.
///
/// Only string contexts (and arrays containing string contexts) that look like
/// IRIs count; inline object mappings are local by definition.
pub(super) fn jsonld_remote_context(bytes: &[u8]) -> bool {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return false; // not decodable JSON: leave classification to the backend
    };
    fn is_remote(ctx: &serde_json::Value) -> bool {
        match ctx {
            serde_json::Value::String(s) => s.contains("://"),
            serde_json::Value::Array(items) => items.iter().any(is_remote),
            // Inline mapping objects are local; do not recurse into their values
            // (a term may legitimately map to an absolute IRI in-document).
            _ => false,
        }
    }
    v.get("@context").is_some_and(is_remote)
}
