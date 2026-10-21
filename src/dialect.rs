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
}

impl Dialect {
    /// The engine that owns this dialect's semantics.
    pub const fn engine(self) -> Engine {
        match self {
            Dialect::N3 => Engine::Eyeron,
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
            _ => None,
        }
    }
}

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
    pub message: String,
}

impl Refusal {
    pub(crate) fn engine(dialect: Dialect, message: impl std::fmt::Debug) -> Self {
        Refusal {
            kind: RefusalKind::EngineRejected,
            dialect: Some(dialect),
            engine: Some(dialect.engine()),
            message: format!("{message:?}"),
        }
    }

    fn routing(kind: RefusalKind, message: &str) -> Self {
        Refusal {
            kind,
            dialect: None,
            engine: None,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} refusal", self.kind)?;
        if let (Some(e), Some(d)) = (self.engine, self.dialect) {
            write!(f, " by {e:?} ({d:?})")?;
        }
        write!(f, ": {}", self.message)
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
    pub fn new(
        dialect: Dialect,
        reason: ReasonClass,
        message: impl Into<String>,
    ) -> Self {
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
        write!(f, "{} in {:?} ({:?})", self.reason, self.dialect, self.engine)?;
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
            message: r.to_string(),
        }
    }
}

/// N3 formula syntax seen in the structural skeleton of an RDF-dialect document.
fn formula_in_rdf_layer(bytes: &[u8], dialect: Dialect) -> Option<ParseRefusal> {
    if !matches!(dialect, Dialect::Turtle | Dialect::NTriples | Dialect::NQuads) {
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
fn jsonld_remote_context(bytes: &[u8]) -> bool {
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

/// True for a `PREFIX`/`BASE` (SPARQL style) or `@prefix`/`@base` (Turtle style) header line.
fn is_directive_line(line: &str) -> bool {
    let up = line.to_ascii_uppercase();
    up.starts_with("PREFIX ")
        || up.starts_with("BASE ")
        || line.starts_with("@prefix")
        || line.starts_with("@base")
}

/// Drop comments, `PREFIX`/`BASE` headers and blank lines.
fn body_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !(l.is_empty() || l.starts_with('#') || is_directive_line(l)))
}

/// The document with comments, string literals and IRI references blanked, so
/// that structural tokens (`{`, `=>`, `GRAPH`) are only seen where they are
/// syntax and not where they are data.
fn skeleton(text: &str) -> String {
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < c.len() {
        match c[i] {
            '#' => {
                while i < c.len() && c[i] != '\n' {
                    i += 1;
                }
            }
            q @ ('"' | '\'') => {
                let triple = c.get(i + 1) == Some(&q) && c.get(i + 2) == Some(&q);
                i += if triple { 3 } else { 1 };
                while i < c.len() {
                    if c[i] == '\\' {
                        i += 2;
                        continue;
                    }
                    if c[i] == q {
                        if !triple {
                            i += 1;
                            break;
                        }
                        if c.get(i + 1) == Some(&q) && c.get(i + 2) == Some(&q) {
                            i += 3;
                            break;
                        }
                    } else if !triple && c[i] == '\n' {
                        break;
                    }
                    i += 1;
                }
                out.push_str("\"\"");
            }
            '<' => {
                let end = c[i + 1..].iter().position(|ch| {
                    matches!(ch, '>' | '<' | '"' | '{' | '}' | '|' | '^' | '`' | '\\')
                        || ch.is_whitespace()
                });
                match end {
                    Some(n) if c[i + 1 + n] == '>' => {
                        out.push_str("<>");
                        i += n + 2;
                    }
                    _ => {
                        out.push('<');
                        i += 1;
                    }
                }
            }
            ch => {
                out.push(ch);
                i += 1;
            }
        }
    }
    out
}

/// First token that is not a `PREFIX`/`BASE` declaration.
fn first_keyword(skel: &str) -> String {
    let mut toks = skel.split_whitespace();
    while let Some(t) = toks.next() {
        match t.to_ascii_uppercase().as_str() {
            "PREFIX" => {
                toks.next();
                toks.next();
            }
            "BASE" => {
                toks.next();
            }
            up => return up.to_string(),
        }
    }
    String::new()
}

/// Number of RDF terms on a line-oriented (N-Triples/N-Quads) statement.
fn line_terms(line: &str) -> Option<usize> {
    let mut rest = line.trim_end().strip_suffix('.')?.trim();
    let mut n = 0;
    while !rest.is_empty() {
        let end = if rest.starts_with('<') {
            rest.find('>')? + 1
        } else if rest.starts_with("_:") {
            rest.find(char::is_whitespace).unwrap_or(rest.len())
        } else if let Some(tail) = rest.strip_prefix('"') {
            let close = tail.find('"')? + 2;
            close
                + rest[close..]
                    .find(char::is_whitespace)
                    .unwrap_or(rest.len() - close)
        } else {
            return None;
        };
        n += 1;
        rest = rest[end..].trim_start();
    }
    Some(n)
}

/// Decide which dialect `bytes` is written in.
///
/// `hint` is a file extension (without the dot) or media subtype used only to
/// break ties routing cannot resolve from content.
pub fn sniff(bytes: &[u8], hint: Option<&str>) -> Result<Dialect, Refusal> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        Refusal::routing(RefusalKind::NotSemanticContent, "input is not UTF-8 text")
    })?;
    let text = text.trim_start_matches('\u{feff}');
    let head = text.trim_start();
    if head.is_empty() {
        return Err(Refusal::routing(
            RefusalKind::NotSemanticContent,
            "empty document",
        ));
    }
    let lower: String = head
        .chars()
        .take(600)
        .collect::<String>()
        .to_ascii_lowercase();
    if lower.starts_with("<!doctype html")
        || lower.starts_with("<html")
        || lower.starts_with("404:")
        || lower.starts_with("<!doctype html public")
    {
        return Err(Refusal::routing(
            RefusalKind::NotSemanticContent,
            "HTML page or HTTP error body, not a semantic document",
        ));
    }
    let hint = hint.map(str::to_ascii_lowercase);
    match hint.as_deref() {
        Some("shex" | "shexc") => return Ok(Dialect::ShExC),
        Some("trig") => return Ok(Dialect::TriG),
        _ => {}
    }
    if head.contains("<TriX") {
        return Ok(Dialect::TriX);
    }
    if head.starts_with("[\"") {
        return Ok(Dialect::HexTuples);
    }
    if head.starts_with("<?xml") || head.contains("<rdf:RDF") || lower.starts_with("<rdf:rdf") {
        return Ok(Dialect::RdfXml);
    }
    let compact: String = head
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(4)
        .collect();
    if compact.starts_with("{\"") || compact.starts_with("[{") || compact.starts_with("{}") {
        let compact: String = head.chars().filter(|c| !c.is_whitespace()).collect();
        return Ok(if compact.contains("\"type\":\"Schema\"") {
            Dialect::ShExJ
        } else {
            Dialect::JsonLd
        });
    }
    let skel = skeleton(head);
    let body: Vec<&str> = body_lines(&skel).collect();
    let first_keyword = first_keyword(&skel);
    if matches!(
        first_keyword.as_str(),
        "SELECT"
            | "CONSTRUCT"
            | "ASK"
            | "DESCRIBE"
            | "INSERT"
            | "DELETE"
            | "WITH"
            | "LOAD"
            | "CLEAR"
            | "DROP"
            | "CREATE"
            | "ADD"
            | "MOVE"
            | "COPY"
    ) {
        return Ok(Dialect::Sparql);
    }
    if skel.contains("=>")
        || skel.contains("<=")
        || skel.contains("@forAll")
        || skel.contains("@forSome")
    {
        return Ok(Dialect::N3);
    }
    let opens_block = skel.contains('{');
    if opens_block {
        return Err(Refusal::routing(
            RefusalKind::Ambiguous,
            "brace-delimited blocks are TriG or ShExC; supply a `trig` or `shex` hint",
        ));
    }
    // N-Triples/N-Quads have no directives: a document that declares prefixes or a base
    // is Turtle even when every statement is written with absolute IRIs (the shape
    // `convert ... to turtle` emits).
    let has_directive = skel.lines().map(str::trim).any(is_directive_line);
    let line_based =
        !has_directive && !body.is_empty() && body.iter().all(|l| line_terms(l).is_some());
    if line_based {
        return Ok(match line_terms(body[0]) {
            Some(4) => Dialect::NQuads,
            _ => Dialect::NTriples,
        });
    }
    Ok(Dialect::Turtle)
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
    if let Some(r) = formula_in_rdf_layer(bytes, dialect) {
        return Err(Box::new(r));
    }
    if dialect == Dialect::JsonLd && jsonld_remote_context(bytes) {
        return Err(Box::new(ParseRefusal {
            dialect,
            engine: dialect.engine(),
            reason: ReasonClass::RemoteContext,
            line: None,
            col: None,
            byte_offset: None,
            snippet: None,
            message: "JSON-LD @context references a remote IRI; remote fetch is forbidden"
                .into(),
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
    if dialect == Dialect::JsonLd && jsonld_remote_context(bytes) {
        return Err(Box::new(ParseRefusal {
            dialect,
            engine: dialect.engine(),
            reason: ReasonClass::RemoteContext,
            line: None,
            col: None,
            byte_offset: None,
            snippet: None,
            message: "JSON-LD @context references a remote IRI; remote fetch is forbidden"
                .into(),
        }));
    }
    if let Some(r) = formula_in_rdf_layer(bytes, dialect) {
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
    }
}
