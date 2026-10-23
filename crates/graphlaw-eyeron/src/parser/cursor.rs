use crate::ast::*;
use crate::error::{EyeronError, Result};
use crate::lexer::{Token, TokenKind};

use super::ParserProfile;

/// How deeply terms may nest, both in input and in anything reasoning derives.
///
///
/// Nested terms — `{ }`, `[ ]`, `( )`, `<< >>` — are parsed by recursion, and
/// running out of stack aborts the whole process instead of raising an error
/// a caller could handle.  Refusing absurd nesting while it is still an
/// ordinary parse error keeps a hostile input from taking the process down.
/// The deepest nesting in the example and conformance corpora is 14 levels,
/// and 64 stays well inside the 1 MiB stacks that debug and WebAssembly
/// builds run on.  `crate::reasoner` holds derived terms to the same
/// bound, so no term anywhere in the system can outgrow the stack.
pub(crate) const MAX_TERM_NESTING_DEPTH: usize = 64;

pub(super) struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) pos: usize,
    pub(super) doc: Document,
    pub(super) blank_counter: usize,
    pub(super) profile: ParserProfile,
    pub(super) source_label: Option<String>,
    pub(super) line_starts: Vec<usize>,
    /// Current term-nesting depth, checked against `MAX_TERM_NESTING_DEPTH`.
    pub(super) depth: usize,
}

impl Parser {
    pub(super) fn new(tokens: Vec<Token>, base_iri: Option<&str>) -> Self {
        Self::with_profile(tokens, base_iri, ParserProfile::n3())
    }

    pub(super) fn with_profile(
        tokens: Vec<Token>,
        base_iri: Option<&str>,
        profile: ParserProfile,
    ) -> Self {
        let mut doc = Document::new();
        doc.base_iri = base_iri.map(ToOwned::to_owned);
        // The notation3tests static corpus uses the standard N3/RDF prefixes
        // in a few files without redeclaring every one of them.  Seed the
        // conventional prefixes while still letting explicit declarations
        // override them.
        doc.prefixes.insert(
            "rdf".to_string(),
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
        );
        doc.prefixes.insert(
            "rdfs".to_string(),
            "http://www.w3.org/2000/01/rdf-schema#".to_string(),
        );
        doc.prefixes.insert(
            "xsd".to_string(),
            "http://www.w3.org/2001/XMLSchema#".to_string(),
        );
        doc.prefixes.insert(
            "log".to_string(),
            "http://www.w3.org/2000/10/swap/log#".to_string(),
        );
        doc.prefixes.insert(
            "list".to_string(),
            "http://www.w3.org/2000/10/swap/list#".to_string(),
        );
        doc.prefixes.insert(
            "math".to_string(),
            "http://www.w3.org/2000/10/swap/math#".to_string(),
        );
        doc.prefixes.insert(
            "string".to_string(),
            "http://www.w3.org/2000/10/swap/string#".to_string(),
        );
        doc.prefixes.insert(
            "time".to_string(),
            "http://www.w3.org/2000/10/swap/time#".to_string(),
        );
        Self {
            tokens,
            pos: 0,
            doc,
            blank_counter: 0,
            profile,
            source_label: None,
            line_starts: Vec::new(),
            depth: 0,
        }
    }

    pub(super) fn with_source(
        mut self,
        source_label: Option<&str>,
        line_starts: Vec<usize>,
    ) -> Self {
        self.source_label = source_label.map(ToOwned::to_owned);
        self.line_starts = line_starts;
        self
    }

    pub(super) fn source_ref_at(&self, offset: usize) -> Option<SourceRef> {
        let label = self.source_label.as_ref()?;
        let line = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx + 1,
            Err(idx) => idx,
        };
        Some(SourceRef {
            label: label.clone(),
            line: line.max(1),
        })
    }

    pub(super) fn add_fact(&mut self, fact: Triple, source: Option<SourceRef>) {
        if let Some(source) = source {
            self.doc.fact_sources.entry(fact.clone()).or_insert(source);
        }
        self.doc.facts.push(fact);
    }

    pub(super) fn add_facts<I>(&mut self, facts: I, source: Option<SourceRef>)
    where
        I: IntoIterator<Item = Triple>,
    {
        for fact in facts {
            self.add_fact(fact, source.clone());
        }
    }

    pub(super) fn peek_is_bare(&self, word: &str) -> bool {
        matches!(self.peek_kind(), TokenKind::PName(p) if p.eq_ignore_ascii_case(word))
    }

    pub(super) fn fresh_blank(&mut self, prefix: &str) -> Term {
        self.blank_counter += 1;
        Term::blank(format!("{}{}", prefix, self.blank_counter))
    }

    pub(super) fn expect_dot(&mut self) -> Result<()> {
        self.expect(TokenKind::Dot)
    }

    pub(super) fn expect(&mut self, expected: TokenKind) -> Result<()> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(EyeronError::at(
                format!("expected {:?}, got {:?}", expected, self.peek_kind()),
                self.peek().offset,
            ))
        }
    }

    pub(super) fn check(&self, expected: &TokenKind) -> bool {
        same_variant(self.peek_kind(), expected)
    }

    pub(super) fn advance(&mut self) -> &Token {
        // At Eof the cursor stays put and the Eof token itself is returned
        // (returning `tokens[pos - 1]` re-read the last real token forever,
        // so a truncated `[` or `<<` recursed until the stack overflowed).
        let at = self.pos;
        if at < self.tokens.len().saturating_sub(1) {
            self.pos += 1;
        }
        &self.tokens[at]
    }

    pub(super) fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    pub(super) fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }
}

pub(super) fn same_variant(a: &TokenKind, b: &TokenKind) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

pub(crate) fn number_literal(mut value: String) -> Term {
    if value.starts_with('+') {
        value.remove(0);
    }
    let datatype = if value.contains('e') || value.contains('E') {
        // Keep the lexical form as written. Rewriting `6.123233995736766e-17`
        // into its positional spelling loses the form the document used, and
        // makes eyeron's own printed output read back as a different literal
        // than the one it printed.
        "http://www.w3.org/2001/XMLSchema#double"
    } else if value.contains('.') {
        if value.starts_with('.') {
            value.insert(0, '0');
        }
        if value.starts_with("-.") {
            value.insert(1, '0');
        }
        if value.ends_with('.') {
            value.push('0');
        }
        "http://www.w3.org/2001/XMLSchema#decimal"
    } else {
        "http://www.w3.org/2001/XMLSchema#integer"
    };
    Term::Literal(Literal {
        value: value.into(),
        datatype: Some(datatype.into()),
        language: None,
    })
}

pub(crate) fn boolean_literal(value: bool) -> Term {
    Term::Literal(Literal {
        value: if value { "true" } else { "false" }.to_string().into(),
        datatype: Some(
            "http://www.w3.org/2001/XMLSchema#boolean"
                .to_string()
                .into(),
        ),
        language: None,
    })
}

pub(super) fn is_boolean_true_term(term: &Term) -> bool {
    match term {
        Term::Literal(lit) => {
            lit.value == "true"
                && lit.language.is_none()
                && lit.datatype.as_deref() == Some("http://www.w3.org/2001/XMLSchema#boolean")
        }
        _ => false,
    }
}

pub(super) fn has_uri_scheme(s: &str) -> bool {
    let Some(idx) = s.find(':') else {
        return false;
    };
    let prefix = &s[..idx];
    !prefix.is_empty()
        && prefix
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
        && prefix
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

pub(super) fn split_uri(
    uri: &str,
) -> (
    Option<String>,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
) {
    let (scheme, rest) = if let Some(idx) = uri.find(':') {
        (Some(uri[..idx].to_string()), &uri[idx + 1..])
    } else {
        (None, uri)
    };
    let (authority, rest) = if let Some(after) = rest.strip_prefix("//") {
        let (auth, path) = split_authority_path(after);
        (Some(auth), path)
    } else {
        (None, rest.to_string())
    };
    let (path_q, fragment) = match rest.split_once('#') {
        Some((a, b)) => (a, Some(b.to_string())),
        None => (rest.as_str(), None),
    };
    let (path, query) = match path_q.split_once('?') {
        Some((a, b)) => (a.to_string(), Some(b.to_string())),
        None => (path_q.to_string(), None),
    };
    (scheme, authority, path, query, fragment)
}

pub(super) fn split_reference(reference: &str) -> (String, Option<String>, Option<String>) {
    let (path_q, fragment) = match reference.split_once('#') {
        Some((a, b)) => (a, Some(b.to_string())),
        None => (reference, None),
    };
    let (path, query) = match path_q.split_once('?') {
        Some((a, b)) => (a.to_string(), Some(b.to_string())),
        None => (path_q.to_string(), None),
    };
    (path, query, fragment)
}

pub(super) fn split_authority_path(s: &str) -> (String, String) {
    if let Some(idx) = s.find('/') {
        (s[..idx].to_string(), s[idx..].to_string())
    } else {
        (s.to_string(), String::new())
    }
}

pub(super) fn merge_paths(base_path: &str, rel_path: &str, has_authority: bool) -> String {
    if has_authority && base_path.is_empty() {
        return format!("/{}", rel_path);
    }
    match base_path.rfind('/') {
        Some(idx) => format!("{}{}", &base_path[..idx + 1], rel_path),
        None => rel_path.to_string(),
    }
}

pub(super) fn remove_dot_segments(path: &str) -> String {
    // RFC 3986 dot-segment removal must preserve significant empty path
    // segments.  Splitting on '/' and dropping empty segments incorrectly turns
    // paths such as //de//xyz into /de/xyz, which the W3C IRI-resolution tests
    // catch.
    let mut input = path.to_string();
    let mut output = String::new();
    while !input.is_empty() {
        if input.starts_with("../") {
            input.drain(..3);
        } else if input.starts_with("./") {
            input.drain(..2);
        } else if input.starts_with("/./") {
            input.replace_range(..3, "/");
        } else if input == "/." {
            input.replace_range(..2, "/");
        } else if input.starts_with("/../") {
            input.replace_range(..4, "/");
            remove_last_path_segment(&mut output);
        } else if input == "/.." {
            input.replace_range(..3, "/");
            remove_last_path_segment(&mut output);
        } else if input == "." || input == ".." {
            input.clear();
        } else {
            let n = if let Some(stripped) = input.strip_prefix('/') {
                stripped.find('/').map(|idx| idx + 1).unwrap_or(input.len())
            } else {
                input.find('/').unwrap_or(input.len())
            };
            output.push_str(&input[..n]);
            input.drain(..n);
        }
    }
    output
}

pub(super) fn remove_last_path_segment(output: &mut String) {
    if output.is_empty() {
        return;
    }
    if let Some(idx) = output.rfind('/') {
        output.truncate(idx);
    } else {
        output.clear();
    }
}

pub(super) fn rdf_number_literal(value: String, offset: usize) -> Result<Term> {
    let datatype = if is_integer_lexical(&value) {
        "http://www.w3.org/2001/XMLSchema#integer"
    } else if is_decimal_lexical_rdf(&value) {
        "http://www.w3.org/2001/XMLSchema#decimal"
    } else if is_double_lexical(&value) {
        "http://www.w3.org/2001/XMLSchema#double"
    } else {
        return Err(EyeronError::at("invalid numeric literal", offset));
    };
    Ok(Term::Literal(Literal {
        value: value.into(),
        datatype: Some(datatype.into()),
        language: None,
    }))
}

pub(super) fn is_integer_lexical(s: &str) -> bool {
    let s = s
        .strip_prefix('+')
        .or_else(|| s.strip_prefix('-'))
        .unwrap_or(s);
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

pub(super) fn is_decimal_lexical_rdf(s: &str) -> bool {
    let s = s
        .strip_prefix('+')
        .or_else(|| s.strip_prefix('-'))
        .unwrap_or(s);
    if s.contains('e') || s.contains('E') {
        return false;
    }
    match s.split_once('.') {
        Some((a, b)) => {
            (!a.is_empty() || !b.is_empty())
                && a.chars().all(|c| c.is_ascii_digit())
                && b.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

pub(super) fn is_double_lexical(s: &str) -> bool {
    let s2 = s
        .strip_prefix('+')
        .or_else(|| s.strip_prefix('-'))
        .unwrap_or(s);
    let Some(pos) = s2.find('e').or_else(|| s2.find('E')) else {
        return false;
    };
    let mant = &s2[..pos];
    let exp = &s2[pos + 1..];
    if exp.is_empty() {
        return false;
    }
    let exp = exp
        .strip_prefix('+')
        .or_else(|| exp.strip_prefix('-'))
        .unwrap_or(exp);
    if exp.is_empty() || !exp.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if mant.is_empty() {
        return false;
    }
    if let Some((a, b)) = mant.split_once('.') {
        (!a.is_empty() || !b.is_empty())
            && a.chars().all(|c| c.is_ascii_digit())
            && b.chars().all(|c| c.is_ascii_digit())
    } else {
        mant.chars().all(|c| c.is_ascii_digit())
    }
}

pub(super) const RDF_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
pub(super) const RDF_DIR_LANG_STRING: &str =
    "http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString";
