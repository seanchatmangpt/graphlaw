use super::{Dialect, Refusal, RefusalKind};

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

/// Routing head budget (`design-sniff-fixes.md` fix 5, court
/// `tests/sniff_court.rs`): routing never sees more than this many characters
/// of collapsed skeleton per document — charged after comment-stripping,
/// literal-blanking and whitespace-collapse. Documents whose routing-relevant
/// tokens land beyond this budget require an explicit `hint`. Named constant,
/// asserted by court; never a magic literal.
const HEAD_BUDGET: usize = 8192;

/// SNIFF-01 (design-sniff-fixes fix 4): true when the collapsed skeleton has
/// a leading SQL statement shape (`SELECT`/`INSERT`/`UPDATE`, `DELETE FROM`,
/// `CREATE TABLE`) plus a SQL clause keyword (`FROM`/`INTO`/`SET`/`VALUES`)
/// and none of the SPARQL markers — `WHERE`, `FILTER`, a graph-pattern `{`,
/// an `a` type token, `sql:` vocabulary. SPARQL text carrying any marker
/// still routes [`Dialect::Sparql`].
fn bare_sql(skel: &str, first_keyword: &str) -> bool {
    let lower = skel.to_ascii_lowercase();
    let toks: Vec<&str> = lower.split_whitespace().collect();
    let marker = toks.iter().any(|t| matches!(*t, "where" | "filter" | "a"))
        || toks.iter().any(|t| t.starts_with("sql:"))
        || skel.contains('{');
    if marker {
        return false;
    }
    let leading = match first_keyword {
        "SELECT" | "INSERT" | "UPDATE" => true,
        "DELETE" => toks.get(1) == Some(&"from"),
        "CREATE" => toks.get(1) == Some(&"table"),
        _ => false,
    };
    if !leading {
        return false;
    }
    toks.iter()
        .any(|t| matches!(*t, "from" | "into" | "set" | "values"))
}

/// The document with comments, string literals and IRI references blanked, so
/// that structural tokens (`{`, `=>`, `GRAPH`) are only seen where they are
/// syntax and not where they are data.
///
/// `#` handling is IRI-aware (design-sniff-fixes fix 2): `in_iri` is tracked
/// across `<`/`>` exactly as [`route_sql_rdf`]'s stripper does — a `<` that
/// never closes falls through to per-char processing with `in_iri` set, `>`
/// and `\n` clear it, and a `#` inside an unclosed IRI span is pushed as data
/// (a fragment separator), not treated as a comment start.
pub(super) fn skeleton(text: &str) -> String {
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut in_iri = false;
    while i < c.len() {
        match c[i] {
            '#' if !in_iri => {
                while i < c.len() && c[i] != '\n' {
                    i += 1;
                }
            }
            // Inside an unclosed `<...>` span: data, not a comment.
            '#' => {
                out.push('#');
                i += 1;
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
                        in_iri = false;
                    }
                    _ => {
                        out.push('<');
                        i += 1;
                        // Unclosed `<`: track the span like the stripper does
                        // so a same-line `#` is data, not a comment.
                        in_iri = true;
                    }
                }
            }
            '>' => {
                in_iri = false;
                out.push('>');
                i += 1;
            }
            '\n' => {
                in_iri = false;
                out.push('\n');
                i += 1;
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
        } else if rest.starts_with('"') {
            // Escape-aware close (design-sniff-fixes fix 3): on `\` skip the
            // escape and its escaped char, so `\"` never terminates the
            // literal scan early. Byte-scan is UTF-8-safe (`\` and `"` are
            // ASCII; continuation bytes are >= 0x80).
            let b = rest.as_bytes();
            let mut j = 1; // rest[0] is the opening quote
            let close = loop {
                match b.get(j) {
                    None => return None, // unterminated literal
                    Some(b'\\') => j += 2,
                    Some(b'"') => break j + 1,
                    Some(_) => j += 1,
                }
            };
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
        .take(HEAD_BUDGET)
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
        Some("sql") => return Ok(Dialect::SqlRdf),
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
    // Fix 5 (design-sniff-fixes.md): the routing view of the document is its
    // skeleton with whitespace collapsed, charged against [`HEAD_BUDGET`] —
    // routing never sees more than 8 KiB of skeleton per document. The window
    // ends at the last complete line within the budget: a partial trailing
    // line is an artifact of truncation, not document content, and counting
    // it as a body line would break the line-based classification (measured:
    // `ontologies/schema-org/schema_all_https.nt`, 2.37 MB of N-Triples,
    // classified Turtle when the cut landed mid-line).
    let skel = {
        let full = skeleton(head);
        let collapsed = full.split_whitespace().collect::<Vec<_>>().join(" ");
        let window: String = collapsed.chars().take(HEAD_BUDGET).collect();
        match window.rfind('\n') {
            Some(n) => window[..=n].to_string(),
            None => window,
        }
    };
    let body: Vec<&str> = body_lines(&skel).collect();
    let first_keyword = first_keyword(&skel);
    // Fix 4 / SNIFF-01: bare SQL keyword+clause shape with none of the
    // SPARQL markers present is a typed refusal at routing — before the
    // SPARQL keyword set below, which would otherwise admit it as
    // [`Dialect::Sparql`] and fail SPARQL parse with a generic engine error.
    if bare_sql(&skel, &first_keyword) {
        return Err(Refusal::routing(
            RefusalKind::BareSql,
            "bare SQL statement, not SPARQL; no SPARQL markers (WHERE, \
             FILTER, graph pattern) are present",
        ));
    }
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
    if let Some(routed) = route_sql_rdf(head) {
        return routed;
    }
    Ok(Dialect::Turtle)
}

/// SQL-in-RDF is Turtle that carries the `sql:` vocabulary. A type assertion
/// (`a sql:SelectStatement`) selects [`Dialect::SqlRdf`]. A `sql:` term used
/// in predicate position (term followed by an object) without any statement
/// class is [`RefusalKind::Ambiguous`] and demands the `sql` hint. Mere
/// vocabulary declarations (`sql:X a owl:Class`) and object-position mentions
/// (`sh:targetClass sql:SelectStatement`) are plain Turtle — the sql-rdf
/// ontology and SHACL shapes themselves must keep routing as Turtle.
fn route_sql_rdf(head: &str) -> Option<Result<Dialect, Refusal>> {
    const SQL: &str = "sql:";
    const SQL_IRI: &str = "http://graphlaw.org/sql#";

    // Whitespace-collapsed token stream: assertions and usages may span lines.
    // Comments (`#` to end of line, never inside a `<...>` IRI, where `#` is a
    // fragment separator) are stripped. String literals are blanked through
    // the same quote/triple-quote/backslash-escape state machine as
    // [`skeleton`] (design-sniff-fixes fix 1: a port of existing machinery),
    // replacing each literal span with a `""` placeholder — so a `sql:` token
    // inside `"..."`, `'...'` or `"""..."""` is data, never usage, and a `#`
    // inside a literal is data, not a comment.
    let c: Vec<char> = head.chars().collect();
    let mut stripped = String::with_capacity(head.len());
    let mut i = 0;
    let mut in_iri = false;
    let mut in_comment = false;
    while i < c.len() {
        if in_comment {
            if c[i] == '\n' {
                in_comment = false;
                stripped.push('\n');
            }
            i += 1;
            continue;
        }
        match c[i] {
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
                stripped.push_str("\"\"");
            }
            '<' => {
                in_iri = true;
                stripped.push('<');
                i += 1;
            }
            '>' => {
                in_iri = false;
                stripped.push('>');
                i += 1;
            }
            '#' if !in_iri => {
                in_comment = true;
                i += 1;
            }
            '\n' => {
                in_iri = false;
                stripped.push('\n');
                i += 1;
            }
            ch => {
                stripped.push(ch);
                i += 1;
            }
        }
    }
    // Fix 5: the token view is charged against [`HEAD_BUDGET`] after
    // comment-stripping, literal-blanking and whitespace-collapse.
    let view: String = {
        let collapsed = stripped.split_whitespace().collect::<Vec<_>>().join(" ");
        collapsed.chars().take(HEAD_BUDGET).collect()
    };
    let toks: Vec<&str> = view.split_whitespace().collect();

    let ends_with_statement = |tok: &str| -> bool {
        let rest = tok.strip_prefix(SQL).or_else(|| tok.strip_prefix(SQL_IRI));
        match rest {
            Some(r) => !r.is_empty() && r.ends_with("Statement"),
            None => false,
        }
    };

    // Type assertion: `a sql:*Statement` (or `rdf:type sql:*Statement`).
    // Trailing punctuation (`;` `,` `.`) is trimmed from the token first.
    let has_select = toks.iter().zip(toks.iter().skip(1)).any(|(prev, tok)| {
        let class = tok.trim_end_matches([';', ',', '.']);
        (*prev == "a" || *prev == "rdf:type") && ends_with_statement(class)
    });
    if has_select {
        return Some(Ok(Dialect::SqlRdf));
    }

    // Predicate-position usage: a sql: term token followed by an object token
    // (literal, IRI, number, or blank-node/collection open). Prefix
    // declarations, subject-position declarations (`sql:X a owl:Class`), and
    // object-position mentions (`sh:path sql:statementIndex ;`) are not usage.
    let is_sql_tok = |t: &str| {
        (t.starts_with(SQL) && t.len() > SQL.len())
            || (t.starts_with(SQL_IRI) && t.len() > SQL_IRI.len())
    };
    let object_start = |b: &[u8]| matches!(b.first(), Some(c) if c == &b'"' || c == &b'<' || c == &b'[' || c == &b'(' || c.is_ascii_digit());
    let used = toks
        .windows(2)
        .any(|w| is_sql_tok(w[0]) && object_start(w[1].as_bytes()));
    if used {
        return Some(Err(Refusal::routing(
            RefusalKind::Ambiguous,
            "sql: vocabulary without sql:SelectStatement; supply a `sql` hint",
        )));
    }
    None
}
