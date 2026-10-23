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

/// The document with comments, string literals and IRI references blanked, so
/// that structural tokens (`{`, `=>`, `GRAPH`) are only seen where they are
/// syntax and not where they are data.
pub(super) fn skeleton(text: &str) -> String {
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
    // Turtle comments are stripped first (`#` to end of line, but never inside
    // a `<...>` IRI, where `#` is a fragment separator).
    let mut stripped = String::with_capacity(head.len());
    let mut in_iri = false;
    let mut in_comment = false;
    for c in head.chars() {
        if in_comment {
            if c == '\n' {
                in_comment = false;
                stripped.push(c);
            }
            continue;
        }
        match c {
            '<' => {
                in_iri = true;
                stripped.push(c);
            }
            '>' => {
                in_iri = false;
                stripped.push(c);
            }
            '#' if !in_iri => in_comment = true,
            '\n' => {
                in_iri = false;
                stripped.push(c);
            }
            _ => stripped.push(c),
        }
    }
    let toks: Vec<&str> = stripped.split_whitespace().collect();

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
