//! Dialect routing and typed refusals.
//!
//! Routing decides *which upstream engine owns a document*; it never parses
//! one. The owning engine is always the final authority, and a document that
//! routing cannot place is refused rather than tried against every engine.

use std::sync::Arc;

use crate::LawState;

/// A syntax GraphLaw can hand to an upstream engine.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::dialect::Dialect;
///
/// let d = Dialect::Turtle;
/// let name = match d {
///     Dialect::Turtle => "turtle",
///     _ => "other", // required: new dialects may be added
/// };
/// assert_eq!(name, "turtle");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Dialect {
    /// Turtle.
    Turtle,
    /// TriG.
    TriG,
    /// N-Triples.
    NTriples,
    /// N-Quads.
    NQuads,
    /// RDF/XML.
    RdfXml,
    /// JSON-LD.
    JsonLd,
    /// YAML-LD.
    YamlLd,
    /// TriX.
    TriX,
    /// HexTuples.
    HexTuples,
    /// Notation3 (owned by Eyeron).
    N3,
    /// ShEx compact syntax.
    ShExC,
    /// ShEx JSON syntax.
    ShExJ,
    /// SPARQL query or update text.
    Sparql,
    /// SQL-in-RDF: a Turtle document whose query is a `sql:SelectStatement`.
    SqlRdf,
}

/// The upstream crate that owns a dialect.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::dialect::{Dialect, Engine};
///
/// let e = Dialect::N3.engine();
/// let owner = match e {
///     Engine::Eyeron => "eyeron",
///     _ => "other", // required: new engines may be added
/// };
/// assert_eq!(owner, "eyeron");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Engine {
    /// PurRDF owns the dialect.
    PurRdf,
    /// Eyeron owns the dialect.
    Eyeron,
    /// frozen-duckdb owns the dialect.
    FrozenDuckdb,
}

impl Dialect {
    /// The engine that owns this dialect's semantics.
    pub const fn engine(self) -> Engine {
        match self {
            Dialect::N3 => Engine::Eyeron,
            Dialect::SqlRdf => Engine::FrozenDuckdb,
            _ => Engine::PurRdf,
        }
    }

    /// RDF media type, for dialects that denote an RDF dataset.
    pub const fn media_type(self) -> Option<&'static str> {
        match self {
            Dialect::Turtle => Some("text/turtle"),
            Dialect::TriG => Some("application/trig"),
            Dialect::NTriples => Some("application/n-triples"),
            Dialect::NQuads => Some("application/n-quads"),
            Dialect::RdfXml => Some("application/rdf+xml"),
            Dialect::JsonLd => Some("application/ld+json"),
            Dialect::YamlLd => Some("application/ld+yaml"),
            Dialect::TriX => Some("application/trix"),
            Dialect::HexTuples => Some("application/x-hextuples"),
            // A SQL-in-RDF document is a Turtle dataset. The SQL shape check
            // still runs; the media type is what lets shacl, law, and
            // canonical see the same graph as the other RDF dialects.
            Dialect::SqlRdf => Some("text/turtle"),
            _ => None,
        }
    }
}

mod refusal;
mod sniff;

pub use refusal::{ReasonClass, Refusal, RefusalKind, RefusalMessage};
pub use sniff::sniff;

/// Maximum snippet length in bytes.
const SNIPPET_MAX: usize = 64;

/// Truncate to at most [`SNIPPET_MAX`] bytes without splitting a UTF-8 scalar.
fn bound_snippet(s: &str) -> String {
    let mut end = SNIPPET_MAX.min(s.len());
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

/// A typed parse refusal: which engine, which dialect, why, and where.
///
/// Location fields are present only when the refusing backend supplies them;
/// they are never guessed. The snippet is bounded at 64 bytes.
///
/// This struct is `#[non_exhaustive]`: fields may be added in a minor release.
///
/// ```
/// use graphlaw::dialect::{Dialect, ParseRefusal, ReasonClass};
///
/// let r = ParseRefusal::new(Dialect::Turtle, ReasonClass::Syntax, "expected '.'");
/// let one_line = format!("{r}").lines().next().unwrap().to_string();
/// assert!(one_line.contains("syntax error"));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ParseRefusal {
    /// Dialect that was refused.
    pub dialect: Dialect,
    /// Engine that refused.
    pub engine: Engine,
    /// Machine-readable refusal class.
    pub reason: ReasonClass,
    /// 1-based line, when the backend provided one.
    pub line: Option<u32>,
    /// 1-based column, when the backend provided one.
    pub col: Option<u32>,
    /// Byte offset into the input, when the backend provided one.
    pub byte_offset: Option<usize>,
    /// Bounded (≤ 64 bytes) excerpt of the offending region, when locatable.
    pub snippet: Option<String>,
    /// Human-readable diagnostic from the refusing backend or router.
    pub message: String,
}

impl ParseRefusal {
    /// Build a refusal with no location information (used by the router for
    /// classes it detects before any backend parse provides positions).
    pub fn new(dialect: Dialect, reason: ReasonClass, message: impl Into<String>) -> Self {
        ParseRefusal {
            dialect,
            engine: dialect.engine(),
            reason,
            line: None,
            col: None,
            byte_offset: None,
            snippet: None,
            message: message.into(),
        }
    }

    /// Line/col derived from a backend-supplied byte offset (Eyeron's own
    /// `with_source_location` derives these the same way; the offset itself is
    /// the backend-provided fact).
    fn from_offset(source: &str, offset: usize) -> (Option<u32>, Option<u32>) {
        let mut line = 1u32;
        let mut col = 1u32;
        for (i, ch) in source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (Some(line), Some(col))
    }

    /// Excerpt around a byte offset, bounded to [`SNIPPET_MAX`] bytes.
    fn snippet_at(source: &str, offset: usize) -> Option<String> {
        let start = source[..offset.min(source.len())]
            .char_indices()
            .rev()
            .find(|(i, _)| offset - *i >= SNIPPET_MAX / 2 || *i == 0)
            .map(|(i, _)| i)
            .unwrap_or(0);
        Some(bound_snippet(&source[start..]))
    }

    /// Excerpt of a 1-based line number, bounded to [`SNIPPET_MAX`] bytes.
    fn snippet_line(source: &str, line: u32) -> Option<String> {
        source
            .lines()
            .nth(line.saturating_sub(1) as usize)
            .map(|l| bound_snippet(l.trim()))
    }

    /// Build a refusal from a PurRDF [`purrdf::RdfDiagnostic`].
    pub(crate) fn from_purrdf(
        dialect: Dialect,
        source: &str,
        diag: &purrdf::RdfDiagnostic,
    ) -> Self {
        let reason = if diag.code.starts_with("iri-") {
            ReasonClass::BadIri
        } else {
            ReasonClass::Syntax
        };
        let (line, col, snippet) = match diag.location.as_ref() {
            Some(loc) => match (loc.line, loc.column) {
                (Some(l), _) => (Some(l), loc.column, ParseRefusal::snippet_line(source, l)),
                (None, _) => (None, None, None),
            },
            None => (None, None, None),
        };
        ParseRefusal {
            dialect,
            engine: dialect.engine(),
            reason,
            line,
            col,
            byte_offset: None, // PurRDF reports line/column, never a byte offset
            snippet,
            message: diag.to_string(),
        }
    }

    /// Build a refusal from an Eyeron [`eyeron::EyeronError`].
    pub(crate) fn from_eyeron(dialect: Dialect, source: &str, err: &eyeron::EyeronError) -> Self {
        let msg = err.message.to_ascii_lowercase();
        let reason = if msg.contains("unknown prefix") || msg.contains("undefined prefix") {
            ReasonClass::UndefinedPrefix
        } else {
            ReasonClass::Syntax
        };
        let (line, col, snippet, byte_offset) = match err.offset {
            Some(off) => {
                let (l, c) = ParseRefusal::from_offset(source, off);
                (l, c, ParseRefusal::snippet_at(source, off), Some(off))
            }
            None => (None, None, None, None),
        };
        ParseRefusal {
            dialect,
            engine: dialect.engine(),
            reason,
            line,
            col,
            byte_offset,
            snippet,
            message: err.message.clone(),
        }
    }

    /// Build a limit refusal with the real observed/max counts.
    pub fn limit(
        dialect: Dialect,
        name: impl Into<String>,
        observed: u64,
        max: u64,
        message: impl Into<String>,
    ) -> Self {
        ParseRefusal {
            dialect,
            engine: dialect.engine(),
            reason: ReasonClass::Limit {
                name: name.into(),
                observed,
                max,
            },
            line: None,
            col: None,
            byte_offset: None,
            snippet: None,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ParseRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} in {:?} ({:?})",
            self.reason, self.dialect, self.engine
        )?;
        if let Some(l) = self.line {
            write!(f, " at line {l}")?;
            if let Some(c) = self.col {
                write!(f, ":{c}")?;
            }
        }
        if let Some(off) = self.byte_offset {
            write!(f, " at byte {off}")?;
        }
        write!(f, ": {}", self.message)?;
        if let Some(snippet) = &self.snippet {
            write!(f, "\n  | {snippet}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseRefusal {}

impl From<Box<ParseRefusal>> for Refusal {
    fn from(r: Box<ParseRefusal>) -> Self {
        Refusal::from(*r)
    }
}

impl From<ParseRefusal> for Refusal {
    fn from(r: ParseRefusal) -> Self {
        let kind = match &r.reason {
            ReasonClass::NotSemanticContent => RefusalKind::NotSemanticContent,
            ReasonClass::AmbiguousDialect => RefusalKind::Ambiguous,
            ReasonClass::Limit { .. } => RefusalKind::ResourceLimit,
            // An N3 formula offered to an RDF dialect is the R04 tier mismatch.
            ReasonClass::FormulaInRdfLayer => RefusalKind::TierMismatch,
            // RemoteContext is a valid-document refusal: the content is fine,
            // this operation cannot fetch it.
            ReasonClass::RemoteContext => RefusalKind::Unsupported,
            ReasonClass::Syntax | ReasonClass::UndefinedPrefix | ReasonClass::BadIri => {
                RefusalKind::EngineRejected
            }
        };
        Refusal {
            kind,
            dialect: Some(r.dialect),
            engine: Some(r.engine),
            message: RefusalMessage::Dynamic(r.to_string()),
        }
    }
}

/// Ask the owning engine to accept `bytes` as `dialect`, discarding the result.
pub fn check(bytes: &[u8], dialect: Dialect, base: Option<&str>) -> Result<(), Refusal> {
    check_typed(bytes, dialect, base).map_err(|b| Refusal::from(*b))
}

/// [`check`] with the full typed refusal (location, snippet, reason class).
pub fn check_typed(
    bytes: &[u8],
    dialect: Dialect,
    base: Option<&str>,
) -> Result<(), Box<ParseRefusal>> {
    // Routing-level guards run BEFORE any backend parse: a formula in an RDF
    // layer or a remote @context is refused without per-engine fallback.
    if let Some(r) = refusal::formula_in_rdf_layer(bytes, dialect) {
        return Err(Box::new(r));
    }
    if dialect == Dialect::JsonLd && refusal::jsonld_remote_context(bytes) {
        return Err(Box::new(ParseRefusal {
            dialect,
            engine: dialect.engine(),
            reason: ReasonClass::RemoteContext,
            line: None,
            col: None,
            byte_offset: None,
            snippet: None,
            message: "JSON-LD @context references a remote IRI; remote fetch is forbidden".into(),
        }));
    }
    let text = || {
        std::str::from_utf8(bytes).map_err(|e| {
            Box::new(ParseRefusal {
                dialect,
                engine: dialect.engine(),
                reason: ReasonClass::NotSemanticContent,
                line: None,
                col: None,
                byte_offset: None,
                snippet: None,
                message: format!("input is not UTF-8 text: {e}"),
            })
        })
    };
    match dialect {
        Dialect::N3 => {
            let src = text()?;
            eyeron::parse_n3(src, base)
                .map(|_| ())
                .map_err(|e| Box::new(ParseRefusal::from_eyeron(dialect, src, &e)))
        }
        Dialect::Sparql => purrdf::sparql::SparqlParser::new()
            .parse_query(text()?)
            .map(|_| ())
            .map_err(|e| {
                Box::new(ParseRefusal {
                    dialect,
                    engine: dialect.engine(),
                    reason: ReasonClass::Syntax,
                    line: None,
                    col: None,
                    byte_offset: None,
                    snippet: None,
                    message: format!("{e:?}"),
                })
            }),
        Dialect::ShExC => {
            let s = purrdf::shex::parse_shexc(text()?, base).map_err(|e| {
                Box::new(ParseRefusal {
                    dialect,
                    engine: dialect.engine(),
                    reason: ReasonClass::Syntax,
                    line: None,
                    col: None,
                    byte_offset: None,
                    snippet: None,
                    message: format!("{e:?}"),
                })
            })?;
            purrdf::shex::check_structure(&s).map_err(|e| {
                Box::new(ParseRefusal {
                    dialect,
                    engine: dialect.engine(),
                    reason: ReasonClass::Syntax,
                    line: None,
                    col: None,
                    byte_offset: None,
                    snippet: None,
                    message: format!("{e:?}"),
                })
            })
        }
        Dialect::SqlRdf => crate::sql_ast::compile_turtle(text()?)
            .map(|_| ())
            .map_err(|e| {
                Box::new(ParseRefusal {
                    dialect,
                    engine: dialect.engine(),
                    reason: ReasonClass::Syntax,
                    line: None,
                    col: None,
                    byte_offset: None,
                    snippet: None,
                    message: e.to_string(),
                })
            }),
        Dialect::ShExJ => {
            let s = purrdf::shex::parse_shexj(text()?, base).map_err(|e| {
                Box::new(ParseRefusal {
                    dialect,
                    engine: dialect.engine(),
                    reason: ReasonClass::Syntax,
                    line: None,
                    col: None,
                    byte_offset: None,
                    snippet: None,
                    message: format!("{e:?}"),
                })
            })?;
            purrdf::shex::check_structure(&s).map_err(|e| {
                Box::new(ParseRefusal {
                    dialect,
                    engine: dialect.engine(),
                    reason: ReasonClass::Syntax,
                    line: None,
                    col: None,
                    byte_offset: None,
                    snippet: None,
                    message: format!("{e:?}"),
                })
            })
        }
        rdf => parse_rdf_typed(bytes, rdf, base).map(|_| ()),
    }
}

/// Parse an RDF dialect into a dataset with PurRDF.
pub fn parse_rdf(
    bytes: &[u8],
    dialect: Dialect,
    base: Option<&str>,
) -> Result<Arc<purrdf::RdfDataset>, Refusal> {
    // Preserve the historical classification exactly: a non-RDF dialect is
    // `Unsupported` on the legacy surface.
    if dialect.media_type().is_none() {
        return Err(Refusal {
            kind: RefusalKind::Unsupported,
            dialect: Some(dialect),
            engine: Some(dialect.engine()),
            message: "dialect does not denote an RDF dataset".into(),
        });
    }
    parse_rdf_typed(bytes, dialect, base).map_err(|b| Refusal::from(*b))
}

/// [`parse_rdf`] with the full typed refusal.
pub fn parse_rdf_typed(
    bytes: &[u8],
    dialect: Dialect,
    base: Option<&str>,
) -> Result<Arc<purrdf::RdfDataset>, Box<ParseRefusal>> {
    let unsupported = || ParseRefusal {
        dialect,
        engine: dialect.engine(),
        reason: ReasonClass::NotSemanticContent,
        line: None,
        col: None,
        byte_offset: None,
        snippet: None,
        message: "dialect does not denote an RDF dataset".into(),
    };
    let media = dialect.media_type().ok_or_else(unsupported)?;
    if std::str::from_utf8(bytes).is_err() {
        return Err(Box::new(ParseRefusal {
            dialect,
            engine: dialect.engine(),
            reason: ReasonClass::NotSemanticContent,
            line: None,
            col: None,
            byte_offset: None,
            snippet: None,
            message: "input is not UTF-8 text".into(),
        }));
    }
    if dialect == Dialect::JsonLd && refusal::jsonld_remote_context(bytes) {
        return Err(Box::new(ParseRefusal {
            dialect,
            engine: dialect.engine(),
            reason: ReasonClass::RemoteContext,
            line: None,
            col: None,
            byte_offset: None,
            snippet: None,
            message: "JSON-LD @context references a remote IRI; remote fetch is forbidden".into(),
        }));
    }
    if let Some(r) = refusal::formula_in_rdf_layer(bytes, dialect) {
        return Err(Box::new(r));
    }
    let text = std::str::from_utf8(bytes).unwrap_or("");
    purrdf::parse_dataset(bytes, media, base)
        .map_err(|e| Box::new(ParseRefusal::from_purrdf(dialect, text, &e)))
}

/// Route by content, then parse into a [`LawState`].
///
/// Failures whose real cause is a tier mismatch (the document is N3 logic) are
/// refused as [`RefusalKind::TierMismatch`] instead of a dataset-parse error;
/// see [`crate::tier::admit_bytes_tiered`] for the declared-tier dispatch.
pub fn admit_bytes(
    bytes: &[u8],
    hint: Option<&str>,
    base: Option<&str>,
) -> Result<LawState, Refusal> {
    match crate::tier::admit_bytes_tiered(bytes, hint, base, None)? {
        crate::tier::Admitted::Dataset(dataset) => LawState::from_dataset(dataset),
        // Unreachable in the undeclared path: N3 logic there is refused as
        // `TierMismatch`, never returned as `Rules`.
        crate::tier::Admitted::Rules(_) => Err(Refusal::routing(
            RefusalKind::Unsupported,
            "N3 rules cannot become a LawState; use tier::admit_declared",
        )),
        crate::tier::Admitted::Query(_) => Err(Refusal::routing(
            RefusalKind::Unsupported,
            "SQL-in-RDF cannot become a LawState; use tier::admit_bytes_tiered",
        )),
    }
}
