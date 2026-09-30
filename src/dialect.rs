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

/// Drop comments, `PREFIX`/`BASE` headers and blank lines.
fn body_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines().map(str::trim).filter(|l| {
        let up = l.to_ascii_uppercase();
        !(l.is_empty()
            || l.starts_with('#')
            || up.starts_with("PREFIX ")
            || up.starts_with("BASE ")
            || l.starts_with("@prefix")
            || l.starts_with("@base"))
    })
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
    let line_based = !body.is_empty() && body.iter().all(|l| line_terms(l).is_some());
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
    let text = || std::str::from_utf8(bytes).map_err(|e| Refusal::engine(dialect, e));
    match dialect {
        Dialect::N3 => eyeron::parse_n3(text()?, base)
            .map(|_| ())
            .map_err(|e| Refusal::engine(dialect, e)),
        Dialect::Sparql => purrdf::sparql::SparqlParser::new()
            .parse_query(text()?)
            .map(|_| ())
            .map_err(|e| Refusal::engine(dialect, e)),
        Dialect::ShExC => {
            let s = purrdf::shex::parse_shexc(text()?, base)
                .map_err(|e| Refusal::engine(dialect, e))?;
            purrdf::shex::check_structure(&s).map_err(|e| Refusal::engine(dialect, e))
        }
        Dialect::ShExJ => {
            let s = purrdf::shex::parse_shexj(text()?, base)
                .map_err(|e| Refusal::engine(dialect, e))?;
            purrdf::shex::check_structure(&s).map_err(|e| Refusal::engine(dialect, e))
        }
        rdf => parse_rdf(bytes, rdf, base).map(|_| ()),
    }
}

/// Parse an RDF dialect into a dataset with PurRDF.
pub fn parse_rdf(
    bytes: &[u8],
    dialect: Dialect,
    base: Option<&str>,
) -> Result<Arc<purrdf::RdfDataset>, Refusal> {
    let media = dialect.media_type().ok_or(Refusal {
        kind: RefusalKind::Unsupported,
        dialect: Some(dialect),
        engine: Some(dialect.engine()),
        message: "dialect does not denote an RDF dataset".into(),
    })?;
    purrdf::parse_dataset(bytes, media, base).map_err(|e| Refusal::engine(dialect, e))
}

/// Route by content, then parse into a [`LawState`].
pub fn admit_bytes(
    bytes: &[u8],
    hint: Option<&str>,
    base: Option<&str>,
) -> Result<LawState, Refusal> {
    let dialect = sniff(bytes, hint)?;
    let dataset = parse_rdf(bytes, dialect, base)?;
    LawState::from_dataset(dataset)
}
