use crate::ast::*;
use crate::error::{EyeronError, Result};
use crate::lexer::lex;
use crate::rdf_compat::RdfFormat;

mod cursor;
mod grammar;

pub(crate) use cursor::MAX_TERM_NESTING_DEPTH;

use cursor::{number_literal, Parser};

pub fn parse_n3(input: &str, base_iri: Option<&str>) -> Result<Document> {
    parse_n3_with_source(input, base_iri, None)
}

pub fn parse_n3_with_source(
    input: &str,
    base_iri: Option<&str>,
    source_label: Option<&str>,
) -> Result<Document> {
    let tokens = lex(input)?;
    let line_starts = line_starts(input);
    Parser::new(tokens, base_iri)
        .with_source(source_label, line_starts)
        .parse_document()
}

pub(crate) fn parse_rdf12_compat(
    input: &str,
    base_iri: Option<&str>,
    format: RdfFormat,
) -> Result<Document> {
    let tokens = lex(input)?;
    Parser::with_profile(tokens, base_iri, ParserProfile::rdf12(format)).parse_document()
}

fn line_starts(input: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (idx, ch) in input.char_indices() {
        if ch == '\n' {
            starts.push(idx + ch.len_utf8());
        }
    }
    starts
}

/// Parse an RDF Message Log using the draft `VERSION "1.2-messages"` /
/// `MESSAGE` delimiter syntax and materialize Eyeron's internal replay view.
///
/// The replay view uses the `eymsg:` vocabulary: one stream resource, ordered
/// message envelopes, next-envelope links, payload kind, and one payload graph
/// resource per non-empty message.  Payload graph resources are connected to a
/// quoted formula with `log:nameOf`, so rules can inspect each message
/// atomically through `log:includes`.
pub fn parse_rdf_message_log(input: &str, base_iri: Option<&str>) -> Result<Document> {
    let (prefixes, messages) = split_rdf_message_log(input)?;
    let mut doc = Document::new();
    doc.base_iri = base_iri.map(ToOwned::to_owned);

    // Seed prefixes even if all messages are empty.
    if !prefixes.trim().is_empty() {
        let seed_text = format!(
            "{}\n<urn:eyeron:prefix-seed> <urn:eyeron:prefix-seed> <urn:eyeron:prefix-seed> .\n",
            prefixes
        );
        let mut seed = parse_n3(&seed_text, base_iri)?;
        seed.facts.clear();
        doc.merge(seed);
    }

    let stream = Term::iri("urn:eyeron:rdf-message-stream:1");
    let envelope_terms: Vec<Term> = (0..messages.len())
        .map(|i| {
            Term::iri(format!(
                "urn:eyeron:rdf-message-stream:1:envelope:{}",
                i + 1
            ))
        })
        .collect();

    let ey = |local: &str| {
        Term::iri(format!(
            "https://eyereasoner.github.io/eyeling/vocab/message#{}",
            local
        ))
    };
    let log = |local: &str| Term::iri(format!("http://www.w3.org/2000/10/swap/log#{}", local));

    doc.facts.push(Triple::new(
        stream.clone(),
        Term::iri(RDF_TYPE),
        ey("RDFMessageStream"),
    ));
    doc.facts.push(Triple::new(
        stream.clone(),
        ey("orderedEnvelopes"),
        Term::List(envelope_terms.clone()),
    ));
    if let Some(first) = envelope_terms.first() {
        doc.facts.push(Triple::new(
            stream.clone(),
            ey("firstEnvelope"),
            first.clone(),
        ));
    }

    for (idx, message) in messages.iter().enumerate() {
        let envelope = envelope_terms[idx].clone();
        doc.facts.push(Triple::new(
            stream.clone(),
            ey("envelope"),
            envelope.clone(),
        ));
        doc.facts.push(Triple::new(
            envelope.clone(),
            Term::iri(RDF_TYPE),
            ey("MessageEnvelope"),
        ));
        doc.facts.push(Triple::new(
            envelope.clone(),
            ey("offset"),
            number_literal(idx.to_string()),
        ));
        if idx + 1 < envelope_terms.len() {
            doc.facts.push(Triple::new(
                envelope.clone(),
                ey("nextEnvelope"),
                envelope_terms[idx + 1].clone(),
            ));
        }

        if message_is_empty(message) {
            doc.facts
                .push(Triple::new(envelope, ey("payloadKind"), ey("empty")));
            continue;
        }

        let rewritten = rewrite_message_blank_labels(message, idx + 1);
        let msg_text = format!("{}\n{}\n", prefixes, rewritten);
        let msg_doc = parse_n3(&msg_text, base_iri)?;
        for (k, v) in &msg_doc.prefixes {
            doc.prefixes.insert(k.clone(), v.clone());
        }

        let payload = Term::iri(format!(
            "urn:eyeron:rdf-message-stream:1:payload:{}",
            idx + 1
        ));
        doc.facts.push(Triple::new(
            envelope.clone(),
            ey("payloadKind"),
            ey("nonEmpty"),
        ));
        doc.facts
            .push(Triple::new(envelope, ey("payloadGraph"), payload.clone()));
        doc.facts.push(Triple::new(
            payload,
            log("nameOf"),
            Term::Formula(msg_doc.facts),
        ));
    }

    Ok(doc)
}

pub fn is_rdf_message_log(input: &str) -> bool {
    input
        .lines()
        .any(|line| is_message_version_directive(line.trim()))
        || input.lines().any(|line| is_message_delimiter(line.trim()))
}

fn split_rdf_message_log(input: &str) -> Result<(String, Vec<String>)> {
    let mut prefixes = String::new();
    let mut current = String::new();
    let mut messages = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            current.push_str(line);
            current.push('\n');
            continue;
        }
        if is_message_version_directive(trimmed) {
            continue;
        }
        if is_message_delimiter(trimmed) {
            messages.push(current.clone());
            current.clear();
            continue;
        }
        if trimmed.starts_with("PREFIX ") || trimmed.starts_with("prefix ") {
            prefixes.push_str(&normalize_turtle_directive(trimmed, "PREFIX", "@prefix")?);
            prefixes.push('\n');
            continue;
        }
        if trimmed.starts_with("BASE ") || trimmed.starts_with("base ") {
            prefixes.push_str(&normalize_turtle_directive(trimmed, "BASE", "@base")?);
            prefixes.push('\n');
            continue;
        }
        current.push_str(line);
        current.push('\n');
    }
    messages.push(current);
    Ok((prefixes, messages))
}

fn is_message_version_directive(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    (lower.starts_with("version ") || lower.starts_with("@version ")) && lower.contains("-messages")
}

fn is_message_delimiter(line: &str) -> bool {
    line.eq_ignore_ascii_case("MESSAGE") || line.eq_ignore_ascii_case("@message .")
}

fn normalize_turtle_directive(line: &str, upper: &str, n3: &str) -> Result<String> {
    let rest = if line.len() >= upper.len() && line[..upper.len()].eq_ignore_ascii_case(upper) {
        line[upper.len()..].trim()
    } else {
        return Err(EyeronError::new(format!("expected {} directive", upper)));
    };
    let without_dot = rest.strip_suffix('.').unwrap_or(rest).trim();
    Ok(format!("{} {} .", n3, without_dot))
}

fn message_is_empty(message: &str) -> bool {
    message.lines().all(|line| {
        let trimmed = line.trim();
        trimmed.is_empty() || trimmed.starts_with('#')
    })
}

fn rewrite_message_blank_labels(message: &str, message_index: usize) -> String {
    let mut out = String::with_capacity(message.len() + 16);
    let mut chars = message.char_indices().peekable();
    let mut in_string: Option<char> = None;
    let mut escaped = false;

    while let Some((_idx, ch)) = chars.next() {
        if let Some(quote) = in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                in_string = None;
            }
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = Some(ch);
            out.push(ch);
            continue;
        }

        if ch == '#' {
            out.push(ch);
            for (_j, c) in chars.by_ref() {
                out.push(c);
                if c == '\n' {
                    break;
                }
            }
            continue;
        }

        if ch == '_' {
            if let Some(&(_, ':')) = chars.peek() {
                chars.next(); // consume ':'
                let mut label = String::new();
                while let Some(&(_, c)) = chars.peek() {
                    if c.is_ascii_alphanumeric() || matches!(c, '_' | '-') {
                        label.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if !label.is_empty() {
                    out.push_str(&format!("_:m{}_{}", message_index, label));
                    continue;
                }
                out.push('_');
                out.push(':');
                continue;
            }
        }

        out.push(ch);
    }

    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParserProfile {
    syntax: SyntaxProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyntaxProfile {
    N3,
    Rdf12(RdfFormat),
}

impl ParserProfile {
    fn n3() -> Self {
        Self {
            syntax: SyntaxProfile::N3,
        }
    }

    fn rdf12(format: RdfFormat) -> Self {
        Self {
            syntax: SyntaxProfile::Rdf12(format),
        }
    }

    fn rdf_format(&self) -> Option<RdfFormat> {
        match self.syntax {
            SyntaxProfile::N3 => None,
            SyntaxProfile::Rdf12(format) => Some(format),
        }
    }

    fn is_rdf12(&self) -> bool {
        self.rdf_format().is_some()
    }

    fn is_line_syntax(&self) -> bool {
        matches!(
            self.rdf_format(),
            Some(RdfFormat::NTriples | RdfFormat::NQuads)
        )
    }
}
