use crate::ast::*;
use crate::error::{EyeronError, Result};
use crate::lexer::TokenKind;
use crate::rdf_compat::RdfFormat;

use super::cursor::*;

fn named_graph_formula(graph: Term, triples: Vec<Triple>) -> Triple {
    Triple::new(graph, Term::iri(LOG_NAME_OF), Term::Formula(triples))
}

const RDF_REIFIES: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies";

impl super::Parser {
    pub(super) fn parse_document(mut self) -> Result<Document> {
        if self.profile.is_line_syntax() {
            self.parse_rdf_line_document()?;
            return Ok(self.doc);
        }

        while !self.check(&TokenKind::Eof) {
            if self.profile.is_rdf12() {
                self.parse_rdf12_statement()?;
                continue;
            }
            match self.peek_kind() {
                TokenKind::AtPrefix | TokenKind::Prefix => self.parse_prefix()?,
                TokenKind::AtBase | TokenKind::Base => self.parse_base()?,
                TokenKind::AtVersion | TokenKind::Version => self.parse_version()?,
                TokenKind::LBrace => self.parse_formula_statement()?,
                TokenKind::Boolean(true) => self.parse_true_formula_statement()?,
                TokenKind::Dot => {
                    self.advance();
                }
                _ => {
                    let source = self.source_ref_at(self.peek().offset);
                    let facts = self.parse_triples_sequence()?;
                    self.add_facts(facts, source);
                    self.expect_dot()?;
                }
            }
        }
        Ok(self.doc)
    }

    pub(super) fn parse_rdf12_statement(&mut self) -> Result<()> {
        let source = self.source_ref_at(self.peek().offset);
        match self.peek_kind() {
            TokenKind::AtPrefix | TokenKind::Prefix => return self.parse_prefix(),
            TokenKind::AtBase | TokenKind::Base => return self.parse_base(),
            TokenKind::AtVersion | TokenKind::Version => return self.parse_version(),
            TokenKind::Dot => {
                return Err(EyeronError::at("empty RDF statement", self.peek().offset));
            }
            TokenKind::LBrace if matches!(self.profile.rdf_format(), Some(RdfFormat::Trig)) => {
                let triples = self.parse_graph_block()?;
                self.add_facts(triples, source.clone());
                if self.check(&TokenKind::Dot) {
                    return Err(EyeronError::at(
                        "TriG graph blocks are not followed by '.'",
                        self.peek().offset,
                    ));
                }
                return Ok(());
            }
            _ => {}
        }

        if matches!(self.profile.rdf_format(), Some(RdfFormat::Trig)) && self.peek_is_bare("graph")
        {
            self.advance();
            let (graph, generated) = self.parse_term()?;
            self.ensure_rdf_graph_label(&graph)?;
            if !generated.is_empty() {
                return Err(EyeronError::new("graph label cannot generate triples"));
            }
            let triples = self.parse_graph_block()?;
            self.add_fact(named_graph_formula(graph, triples), source.clone());
            if self.check(&TokenKind::Dot) {
                return Err(EyeronError::at(
                    "TriG graph blocks are not followed by '.'",
                    self.peek().offset,
                ));
            }
            return Ok(());
        }

        if self.check(&TokenKind::A) {
            return Err(EyeronError::at(
                "'a' is only valid as an RDF predicate",
                self.peek().offset,
            ));
        }
        let (subject, generated) = self.parse_term()?;
        if self.check(&TokenKind::LBrace) {
            if !matches!(self.profile.rdf_format(), Some(RdfFormat::Trig)) {
                return Err(EyeronError::at(
                    "named graph blocks are only valid in TriG mode",
                    self.peek().offset,
                ));
            }
            self.ensure_rdf_graph_label(&subject)?;
            if !generated.is_empty() {
                return Err(EyeronError::new("graph label cannot generate triples"));
            }
            let triples = self.parse_graph_block()?;
            self.add_fact(named_graph_formula(subject, triples), source.clone());
            if self.check(&TokenKind::Dot) {
                return Err(EyeronError::at(
                    "TriG graph blocks are not followed by '.'",
                    self.peek().offset,
                ));
            }
            return Ok(());
        }

        self.ensure_rdf_subject(&subject)?;
        let facts = self.parse_triples_sequence_from_subject_with_generated(subject, generated)?;
        self.add_facts(facts, source.clone());
        self.expect_dot()?;
        Ok(())
    }

    pub(super) fn parse_rdf_line_document(&mut self) -> Result<()> {
        while !self.check(&TokenKind::Eof) {
            match self.peek_kind() {
                TokenKind::Dot => {
                    return Err(EyeronError::at(
                        "empty N-Triples/N-Quads statement",
                        self.peek().offset,
                    ));
                }
                TokenKind::AtPrefix
                | TokenKind::Prefix
                | TokenKind::AtBase
                | TokenKind::Base
                | TokenKind::AtVersion
                | TokenKind::Version => {
                    return Err(EyeronError::at(
                        "directives are not allowed in N-Triples/N-Quads",
                        self.peek().offset,
                    ));
                }
                _ => {}
            }

            let source = self.source_ref_at(self.peek().offset);
            if self.check(&TokenKind::A) {
                return Err(EyeronError::at(
                    "'a' is not N-Triples/N-Quads syntax",
                    self.peek().offset,
                ));
            }
            let (subject, mut generated) = self.parse_term()?;
            self.ensure_rdf_line_subject(&subject)?;
            if !generated.is_empty() {
                return Err(EyeronError::new(
                    "N-Triples/N-Quads subject cannot generate triples",
                ));
            }

            if self.check(&TokenKind::A) {
                return Err(EyeronError::at(
                    "'a' is not N-Triples/N-Quads syntax",
                    self.peek().offset,
                ));
            }
            let (predicate, mut pred_generated) = self.parse_term()?;
            self.ensure_rdf_predicate(&predicate)?;
            if !pred_generated.is_empty() {
                return Err(EyeronError::new(
                    "N-Triples/N-Quads predicate cannot generate triples",
                ));
            }

            if self.check(&TokenKind::A) {
                return Err(EyeronError::at(
                    "'a' is not N-Triples/N-Quads syntax",
                    self.peek().offset,
                ));
            }
            let (object, mut obj_generated) = self.parse_term()?;
            self.ensure_rdf_line_object(&object)?;
            if !obj_generated.is_empty() {
                return Err(EyeronError::new(
                    "N-Triples/N-Quads object cannot generate triples",
                ));
            }

            let asserted = Triple::new(subject, predicate, object);
            generated.append(&mut pred_generated);
            generated.append(&mut obj_generated);
            generated.push(asserted);

            if matches!(self.profile.rdf_format(), Some(RdfFormat::NQuads))
                && !self.check(&TokenKind::Dot)
            {
                let (graph, graph_generated) = self.parse_term()?;
                self.ensure_rdf_graph_label(&graph)?;
                if !graph_generated.is_empty() {
                    return Err(EyeronError::new("graph label cannot generate triples"));
                }
                self.add_fact(named_graph_formula(graph, generated), source.clone());
            } else if matches!(self.profile.rdf_format(), Some(RdfFormat::NTriples))
                && !self.check(&TokenKind::Dot)
            {
                return Err(EyeronError::at(
                    "N-Triples has too many terms before '.'",
                    self.peek().offset,
                ));
            } else {
                self.add_facts(generated, source.clone());
            }
            self.expect_dot()?;
        }
        Ok(())
    }

    pub(super) fn parse_graph_block(&mut self) -> Result<Vec<Triple>> {
        self.expect(TokenKind::LBrace)?;
        let mut triples = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            match self.peek_kind() {
                // The W3C TriG minimal-whitespace positive syntax case uses a
                // single leading statement separator in an anonymous graph
                // block.  Permit that compact form, but still reject the
                // negative "too many DOTs" cases such as `{ . . }`.
                TokenKind::Dot if triples.is_empty() => {
                    self.advance();
                    if self.check(&TokenKind::Dot)
                        || self.check(&TokenKind::RBrace)
                        || self.check(&TokenKind::Eof)
                    {
                        return Err(EyeronError::at(
                            "extra '.' in TriG graph block",
                            self.peek().offset,
                        ));
                    }
                    continue;
                }
                TokenKind::Dot => {
                    return Err(EyeronError::at(
                        "extra '.' in TriG graph block",
                        self.peek().offset,
                    ));
                }
                TokenKind::AtPrefix
                | TokenKind::Prefix
                | TokenKind::AtBase
                | TokenKind::Base
                | TokenKind::AtVersion
                | TokenKind::Version => {
                    return Err(EyeronError::at(
                        "directives are not allowed inside TriG graph blocks",
                        self.peek().offset,
                    ));
                }
                _ => {
                    let mut parsed = self.parse_triples_sequence()?;
                    triples.append(&mut parsed);
                    if self.check(&TokenKind::Dot) {
                        self.advance();
                    } else if !self.check(&TokenKind::RBrace) {
                        return Err(EyeronError::at(
                            "expected '.' or '}' in graph block",
                            self.peek().offset,
                        ));
                    }
                }
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(triples)
    }

    pub(super) fn parse_prefix(&mut self) -> Result<()> {
        let directive = self.advance().clone();
        let at_style = matches!(directive.kind, TokenKind::AtPrefix);
        let t = self.advance().clone();
        let name = match t.kind {
            TokenKind::PName(p) => {
                if self.profile.is_rdf12() && !p.ends_with(':') {
                    return Err(EyeronError::at(
                        "RDF prefix name must end with ':'",
                        t.offset,
                    ));
                }
                p.strip_suffix(':').unwrap_or(&p).to_string()
            }
            _ => return Err(EyeronError::at("expected prefix name", t.offset)),
        };
        let iri_tok = self.advance().clone();
        let iri = match iri_tok.kind {
            TokenKind::Iri(i) => self.resolve_iri(&i),
            _ => return Err(EyeronError::at("expected prefix IRI", iri_tok.offset)),
        };
        validate_prefix_label(&name, t.offset)?;
        self.doc.prefixes.insert(name, iri);
        if self.profile.is_rdf12() {
            if at_style {
                self.expect_dot()?;
            } else if self.check(&TokenKind::Dot) {
                return Err(EyeronError::at(
                    "SPARQL-style PREFIX must not end with '.'",
                    self.peek().offset,
                ));
            }
        } else if self.check(&TokenKind::Dot) {
            self.advance();
        }
        Ok(())
    }

    pub(super) fn parse_base(&mut self) -> Result<()> {
        let directive = self.advance().clone();
        let at_style = matches!(directive.kind, TokenKind::AtBase);
        let iri_tok = self.advance().clone();
        let iri = match iri_tok.kind {
            TokenKind::Iri(i) => self.resolve_iri(&i),
            _ => return Err(EyeronError::at("expected base IRI", iri_tok.offset)),
        };
        self.doc.base_iri = Some(iri);
        if self.profile.is_rdf12() {
            if at_style {
                self.expect_dot()?;
            } else if self.check(&TokenKind::Dot) {
                return Err(EyeronError::at(
                    "SPARQL-style BASE must not end with '.'",
                    self.peek().offset,
                ));
            }
        } else if self.check(&TokenKind::Dot) {
            self.advance();
        }
        Ok(())
    }

    pub(super) fn parse_version(&mut self) -> Result<()> {
        self.advance();
        let version_tok = self.advance().clone();
        if self.profile.is_rdf12() {
            match version_tok.kind {
                TokenKind::String(_) | TokenKind::StringSingle(_) => {}
                _ => {
                    return Err(EyeronError::at(
                        "RDF VERSION requires a short string",
                        version_tok.offset,
                    ))
                }
            }
            if self.check(&TokenKind::Dot) {
                self.advance();
            }
            return Ok(());
        }
        match version_tok.kind {
            TokenKind::String(_)
            | TokenKind::StringSingle(_)
            | TokenKind::StringLong(_)
            | TokenKind::StringLongSingle(_)
            | TokenKind::StringLongExtraQuote(_)
            | TokenKind::StringLongSingleExtraQuote(_)
            | TokenKind::PName(_)
            | TokenKind::Number(_) => {}
            _ => {
                return Err(EyeronError::at(
                    "expected version value",
                    version_tok.offset,
                ))
            }
        }
        if self.check(&TokenKind::Dot) {
            self.advance();
        }
        Ok(())
    }

    pub(super) fn parse_true_formula_statement(&mut self) -> Result<()> {
        let source = self.source_ref_at(self.peek().offset);
        self.advance();
        match self.peek_kind() {
            TokenKind::Arrow => {
                self.advance();
                let (rhs, fuse) = self.parse_forward_rule_rhs()?;
                self.doc.rules.push(
                    Rule::new(Vec::new(), rhs, true)
                        .with_fuse(fuse)
                        .with_source(source.clone()),
                );
            }
            TokenKind::BackArrow => {
                self.advance();
                let rhs = self.parse_formula_or_true()?;
                self.doc
                    .rules
                    .push(Rule::new(rhs, Vec::new(), false).with_source(source.clone()));
            }
            _ => {
                // `true false true .` is a valid N3 triple with boolean terms.
                let facts = self.parse_triples_sequence_from_subject(boolean_literal(true))?;
                self.add_facts(facts, source.clone());
            }
        }
        self.expect_dot()?;
        Ok(())
    }

    pub(super) fn parse_formula_statement(&mut self) -> Result<()> {
        let source = self.source_ref_at(self.peek().offset);
        let lhs = self.parse_formula()?;
        match self.peek_kind() {
            TokenKind::Arrow => {
                self.advance();
                let (rhs, fuse) = self.parse_forward_rule_rhs()?;
                self.doc.rules.push(
                    Rule::new(lhs, rhs, true)
                        .with_fuse(fuse)
                        .with_source(source.clone()),
                );
            }
            TokenKind::BackArrow => {
                self.advance();
                let rhs = self.parse_formula_or_true()?;
                // `{ head } <= { body }` is a backward rule: use it
                // goal-directed when a forward premise asks for `head`.
                self.doc
                    .rules
                    .push(Rule::new(rhs, lhs, false).with_source(source.clone()));
            }
            _ => {
                let predicate = self.parse_verb()?;
                match predicate {
                    Term::Iri(ref iri) if iri == LOG_QUERY => {
                        let rhs = self.parse_formula_or_true()?;
                        self.doc.rules.push(
                            Rule::new(lhs, rhs, true)
                                .with_source(source.clone())
                                .with_query(true),
                        );
                    }
                    other => {
                        // Otherwise the leading formula is an ordinary term subject, e.g.
                        // `{ :A :B :C } a :Statement .` or a formula-valued predicate.
                        let subject = Term::formula(lhs);
                        let mut triples = Vec::new();
                        loop {
                            let (object, mut generated) = self.parse_term()?;
                            triples.push(Triple::new(subject.clone(), other.clone(), object));
                            triples.append(&mut generated);
                            if self.check(&TokenKind::Comma) {
                                self.advance();
                                continue;
                            }
                            break;
                        }
                        // The remaining `; <verb> <objects>` groups are an
                        // ordinary predicate-object list about the same
                        // subject, however many of them there are.
                        if self.check(&TokenKind::Semicolon) {
                            while self.check(&TokenKind::Semicolon) {
                                self.advance();
                            }
                            triples.extend(self.parse_predicate_object_list(subject.clone())?);
                        }
                        self.add_facts(triples, source.clone());
                    }
                }
            }
        }
        self.expect_dot()?;
        Ok(())
    }

    pub(super) fn parse_formula_or_true(&mut self) -> Result<Vec<Triple>> {
        if matches!(self.peek_kind(), TokenKind::Boolean(true)) {
            self.advance();
            return Ok(Vec::new());
        }
        if matches!(self.peek_kind(), TokenKind::Boolean(false)) {
            return Err(EyeronError::at(
                "false rule conclusions are not supported",
                self.peek().offset,
            ));
        }
        self.parse_formula()
    }

    /// The right-hand side of `=>`, and whether it was `false` -- an
    /// inference fuse rather than something to conclude.
    pub(super) fn parse_forward_rule_rhs(&mut self) -> Result<(Vec<Triple>, bool)> {
        if matches!(self.peek_kind(), TokenKind::Boolean(true)) {
            self.advance();
            return Ok((Vec::new(), false));
        }
        if matches!(self.peek_kind(), TokenKind::Boolean(false)) {
            self.advance();
            return Ok((Vec::new(), true));
        }
        if matches!(self.peek_kind(), TokenKind::LBrace) {
            return Ok((self.parse_formula()?, false));
        }

        // N3 allows a forward-rule RHS to be a term that resolves to a quoted
        // formula, e.g. `{ :a :b ?F } => ?F .`.  Represent that as an internal
        // unquote instruction; the reasoner expands the formula contents when
        // the rule fires.
        let (term, generated) = self.parse_term()?;
        if !generated.is_empty() {
            return Err(EyeronError::new(
                "generated triples cannot appear around an unquoted RHS term",
            ));
        }
        Ok((
            vec![Triple::new(
                Term::iri(EYERON_UNQUOTE),
                Term::iri(EYERON_UNQUOTE),
                term,
            )],
            false,
        ))
    }

    pub(super) fn parse_formula(&mut self) -> Result<Vec<Triple>> {
        self.expect(TokenKind::LBrace)?;
        let mut triples = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            if self.check(&TokenKind::Dot) {
                self.advance();
                continue;
            }
            if matches!(self.peek_kind(), TokenKind::AtPrefix | TokenKind::Prefix) {
                self.parse_prefix()?;
                continue;
            }
            if matches!(self.peek_kind(), TokenKind::AtBase | TokenKind::Base) {
                self.parse_base()?;
                continue;
            }
            if matches!(self.peek_kind(), TokenKind::AtVersion | TokenKind::Version) {
                self.parse_version()?;
                continue;
            }
            triples.extend(self.parse_triples_sequence()?);
            if self.check(&TokenKind::Dot) {
                self.advance();
            } else if !self.check(&TokenKind::RBrace) {
                return Err(EyeronError::at(
                    "expected '.' or '}' in formula",
                    self.peek().offset,
                ));
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(triples)
    }

    pub(super) fn parse_triples_sequence_from_subject(
        &mut self,
        subject: Term,
    ) -> Result<Vec<Triple>> {
        self.parse_triples_sequence_from_subject_with_generated(subject, Vec::new())
    }

    pub(super) fn parse_triples_sequence_from_subject_with_generated(
        &mut self,
        subject: Term,
        mut generated: Vec<Triple>,
    ) -> Result<Vec<Triple>> {
        let generated_standalone_ok = self.profile.is_rdf12()
            && !generated.is_empty()
            && (matches!(&subject, Term::Blank(_)) || generated_are_rdf12_reifications(&generated));
        let mut triples = Vec::new();
        triples.append(&mut generated);

        // N3 implication can appear as a statement inside a quoted formula,
        // for example `{ { ?x a :Cat } => { ?x a :Animal } . }`.
        // Store it as a first-class triple whose subject/object are formula terms;
        // the reasoner promotes derived implication triples to active rules.
        if self.check(&TokenKind::Arrow) || self.check(&TokenKind::BackArrow) {
            if self.profile.is_rdf12() {
                return Err(EyeronError::at(
                    "N3 implication is not RDF syntax",
                    self.peek().offset,
                ));
            }
            let backward = self.check(&TokenKind::BackArrow);
            self.advance();
            let (object, mut object_generated) = self.parse_term()?;
            if matches!((&subject, &object), (Term::Formula(_), Term::Var(_))) {
                return Err(EyeronError::at(
                    "formula-to-variable implication is not supported",
                    self.peek().offset,
                ));
            }
            let object = if is_boolean_true_term(&object) {
                Term::Formula(Vec::new())
            } else {
                object
            };
            triples.append(&mut object_generated);
            if backward {
                // Preserve `<=` polarity in quoted rule terms.  These are
                // promoted as backward rules and printed again as `<=`.
                triples.push(Triple::new(subject, Term::iri(LOG_IMPLIED_BY), object));
            } else {
                triples.push(Triple::new(subject, Term::iri(LOG_IMPLIES), object));
            }
            return Ok(triples);
        }

        let predicate_object_triples = self.parse_predicate_object_list(subject.clone())?;
        if self.profile.is_rdf12()
            && predicate_object_triples.is_empty()
            && !generated_standalone_ok
        {
            return Err(EyeronError::at(
                "expected RDF predicate-object list",
                self.peek().offset,
            ));
        }
        triples.extend(predicate_object_triples);
        Ok(triples)
    }

    pub(super) fn parse_triples_sequence(&mut self) -> Result<Vec<Triple>> {
        if self.profile.is_rdf12() && self.check(&TokenKind::A) {
            return Err(EyeronError::at(
                "'a' is only valid as an RDF predicate",
                self.peek().offset,
            ));
        }
        let (subject, generated) = self.parse_term()?;
        if self.profile.is_rdf12() {
            self.ensure_rdf_subject(&subject)?;
        }
        self.parse_triples_sequence_from_subject_with_generated(subject, generated)
    }

    pub(super) fn parse_predicate_object_list(&mut self, subject: Term) -> Result<Vec<Triple>> {
        let mut triples = Vec::new();
        loop {
            if matches!(
                self.peek_kind(),
                TokenKind::Dot | TokenKind::RBrace | TokenKind::RBracket | TokenKind::RAnnotation
            ) {
                break;
            }

            // N3 reverse-property path: `S <- :p O` means `O :p S`.
            if !self.profile.is_rdf12() && self.check(&TokenKind::Reverse) {
                self.advance();
                let predicate = self.parse_verb()?;
                loop {
                    let (object, mut generated) = self.parse_term()?;
                    let asserted = Triple::new(object, predicate.clone(), subject.clone());
                    triples.push(asserted.clone());
                    triples.append(&mut generated);
                    if self.profile.is_rdf12() {
                        self.parse_rdf12_reifier_suffixes(&asserted, &mut triples)?;
                    }
                    if self.check(&TokenKind::Comma) {
                        self.advance();
                        continue;
                    }
                    break;
                }
            }
            // N3 alternate spelling: `S is :p of O` means `O :p S`.
            else if !self.profile.is_rdf12() && self.peek_is_bare("is") {
                self.advance();
                let predicate = self.parse_verb()?;
                if !self.peek_is_bare("of") {
                    return Err(EyeronError::at(
                        "expected 'of' after 'is <predicate>'",
                        self.peek().offset,
                    ));
                }
                self.advance();
                loop {
                    let (object, mut generated) = self.parse_term()?;
                    let asserted = Triple::new(object, predicate.clone(), subject.clone());
                    triples.push(asserted.clone());
                    triples.append(&mut generated);
                    if self.profile.is_rdf12() {
                        self.parse_rdf12_reifier_suffixes(&asserted, &mut triples)?;
                    }
                    if self.check(&TokenKind::Comma) {
                        self.advance();
                        continue;
                    }
                    break;
                }
            }
            // N3 alternate spelling: `S has :p O` means `S :p O`.
            else {
                if !self.profile.is_rdf12() && self.peek_is_bare("has") {
                    self.advance();
                }
                let predicate = self.parse_verb()?;
                loop {
                    if self.profile.is_rdf12() && self.check(&TokenKind::A) {
                        return Err(EyeronError::at(
                            "'a' is only valid as an RDF predicate",
                            self.peek().offset,
                        ));
                    }
                    let (object, mut generated) = self.parse_term()?;
                    let asserted = Triple::new(subject.clone(), predicate.clone(), object);
                    triples.push(asserted.clone());
                    triples.append(&mut generated);
                    if self.profile.is_rdf12() {
                        self.parse_rdf12_reifier_suffixes(&asserted, &mut triples)?;
                    }
                    if self.check(&TokenKind::Comma) {
                        self.advance();
                        continue;
                    }
                    break;
                }
            }

            if self.check(&TokenKind::Semicolon) {
                while self.check(&TokenKind::Semicolon) {
                    self.advance();
                }
                if matches!(
                    self.peek_kind(),
                    TokenKind::Dot
                        | TokenKind::RBrace
                        | TokenKind::RBracket
                        | TokenKind::RAnnotation
                ) {
                    break;
                }
                continue;
            }
            break;
        }
        Ok(triples)
    }

    pub(super) fn parse_rdf12_reifier_suffixes(
        &mut self,
        asserted: &Triple,
        triples: &mut Vec<Triple>,
    ) -> Result<()> {
        let mut pending_reifier: Option<Term> = None;
        loop {
            if self.peek_is_reifier_token() {
                let reifier = self.parse_reifier_token()?;
                triples.push(Triple::new(
                    reifier.clone(),
                    Term::iri(RDF_REIFIES),
                    Term::formula(vec![asserted.clone()]),
                ));
                pending_reifier = Some(reifier);
                continue;
            }
            if self.check(&TokenKind::LAnnotation) {
                let reifier = if let Some(reifier) = pending_reifier.take() {
                    reifier
                } else {
                    let reifier = self.fresh_blank("reif");
                    triples.push(Triple::new(
                        reifier.clone(),
                        Term::iri(RDF_REIFIES),
                        Term::formula(vec![asserted.clone()]),
                    ));
                    reifier
                };
                self.parse_annotation_block(reifier, triples)?;
                continue;
            }
            break;
        }
        Ok(())
    }

    pub(super) fn parse_annotation_block(
        &mut self,
        reifier: Term,
        triples: &mut Vec<Triple>,
    ) -> Result<()> {
        self.expect(TokenKind::LAnnotation)?;
        if self.check(&TokenKind::RAnnotation) {
            return Err(EyeronError::at(
                "empty RDF 1.2 annotation block",
                self.peek().offset,
            ));
        }
        triples.extend(self.parse_predicate_object_list(reifier)?);
        self.expect(TokenKind::RAnnotation)?;
        Ok(())
    }

    pub(super) fn peek_is_reifier_token(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::PName(p) if p.starts_with('~'))
    }

    pub(super) fn parse_reifier_token(&mut self) -> Result<Term> {
        let tok = self.advance().clone();
        let (suffix, offset) = match tok.kind {
            TokenKind::PName(p) if p.starts_with('~') => (p[1..].to_string(), tok.offset),
            _ => {
                return Err(EyeronError::at(
                    "expected RDF 1.2 reifier token",
                    tok.offset,
                ))
            }
        };
        if suffix.is_empty() {
            match self.peek_kind() {
                TokenKind::Iri(_) | TokenKind::PName(_) | TokenKind::Blank(_) | TokenKind::A => {
                    let (term, generated) = self.parse_term()?;
                    if !generated.is_empty() {
                        return Err(EyeronError::new("reifier term cannot generate triples"));
                    }
                    match term {
                        Term::Iri(_) | Term::Blank(_) => Ok(term),
                        _ => Err(EyeronError::at(
                            "reifier must be an IRI or blank node",
                            self.peek().offset,
                        )),
                    }
                }
                _ => Ok(self.fresh_blank("reif")),
            }
        } else if let Some(label) = suffix.strip_prefix("_:") {
            if label.is_empty() {
                return Err(EyeronError::at("empty blank-node reifier label", offset));
            }
            Ok(Term::blank(label.to_string()))
        } else if suffix.contains(':') {
            Ok(Term::iri(self.expand_pname(&suffix, offset)?))
        } else {
            Err(EyeronError::at(
                "reifier suffix must be an IRI, prefixed name, or blank node",
                offset,
            ))
        }
    }

    pub(super) fn parse_verb(&mut self) -> Result<Term> {
        if self.check(&TokenKind::A) {
            self.advance();
            return Ok(Term::iri(RDF_TYPE));
        }
        if self.profile.is_rdf12() {
            let (term, generated) = self.parse_term()?;
            if !generated.is_empty() {
                return Err(EyeronError::new("RDF predicate cannot generate triples"));
            }
            self.ensure_rdf_predicate(&term)?;
            return Ok(term);
        }
        if self.check(&TokenKind::Equals) {
            self.advance();
            return Ok(Term::iri(OWL_SAME_AS));
        }
        if self.check(&TokenKind::Arrow) {
            self.advance();
            return Ok(Term::iri(LOG_IMPLIES));
        }
        if self.check(&TokenKind::BackArrow) {
            self.advance();
            return Ok(Term::iri(LOG_IMPLIED_BY));
        }
        let (term, _generated) = self.parse_term()?;
        // Predicate terms may themselves be lists or formulas in N3.  Generated
        // list-support triples are not needed to match such predicate terms, so
        // keep the predicate and drop the auxiliary triples here.
        Ok(term)
    }

    /// Every nested term form routes through here, so this is the one place
    /// the nesting depth has to be counted.
    pub(super) fn parse_term(&mut self) -> Result<(Term, Vec<Triple>)> {
        if self.depth >= MAX_TERM_NESTING_DEPTH {
            let offset = self.peek().offset;
            return Err(EyeronError::at(
                format!(
                    "terms nested more than {} levels deep",
                    MAX_TERM_NESTING_DEPTH
                ),
                offset,
            ));
        }
        self.depth += 1;
        let parsed = self.parse_term_inner();
        self.depth -= 1;
        parsed
    }

    pub(super) fn parse_term_inner(&mut self) -> Result<(Term, Vec<Triple>)> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::Iri(i) => {
                let iri = if self.profile.is_line_syntax() {
                    i
                } else {
                    self.resolve_iri(&i)
                };
                Ok((Term::iri(iri), Vec::new()))
            }
            TokenKind::PName(p) => self.parse_pname_or_path(&p, tok.offset),
            TokenKind::Var(v) => {
                if self.profile.is_rdf12() {
                    Err(EyeronError::at(
                        "variables are not allowed in RDF syntax",
                        tok.offset,
                    ))
                } else {
                    Ok((Term::var(v), Vec::new()))
                }
            }
            TokenKind::Blank(b) => {
                if self.profile.is_rdf12() {
                    validate_blank_label(&b, tok.offset)?;
                }
                Ok((Term::blank(b), Vec::new()))
            }
            TokenKind::String(value) => self.finish_literal(value),
            TokenKind::StringSingle(value)
            | TokenKind::StringLong(value)
            | TokenKind::StringLongSingle(value) => {
                if self.profile.is_line_syntax() {
                    Err(EyeronError::at(
                        "N-Triples/N-Quads only allow short double-quoted string literals",
                        tok.offset,
                    ))
                } else {
                    self.finish_literal(value)
                }
            }
            TokenKind::StringLongExtraQuote(value)
            | TokenKind::StringLongSingleExtraQuote(value) => {
                if self.profile.is_rdf12() {
                    Err(EyeronError::at(
                        "extra quote before long string terminator is not RDF syntax",
                        tok.offset,
                    ))
                } else {
                    self.finish_literal(value)
                }
            }
            TokenKind::Number(value) => {
                if self.profile.is_line_syntax() {
                    Err(EyeronError::at(
                        "N-Triples/N-Quads do not allow bare numeric literals",
                        tok.offset,
                    ))
                } else if self.profile.is_rdf12() {
                    rdf_number_literal(value, tok.offset).map(|t| (t, Vec::new()))
                } else {
                    Ok((number_literal(value), Vec::new()))
                }
            }
            TokenKind::Boolean(value) => Ok((boolean_literal(value), Vec::new())),
            TokenKind::A => Ok((Term::iri(RDF_TYPE), Vec::new())),
            TokenKind::LBrace => {
                if self.profile.is_rdf12() {
                    return Err(EyeronError::at(
                        "formula terms are not allowed in RDF syntax",
                        tok.offset,
                    ));
                }
                self.pos -= 1;
                let triples = self.parse_formula()?;
                Ok((Term::formula(triples), Vec::new()))
            }
            TokenKind::LBracket => self.parse_blank_node_property_list(),
            TokenKind::LParen => self.parse_list(),
            TokenKind::LTriple => self.parse_triple_term(),
            other => Err(EyeronError::at(
                format!("expected term, got {:?}", other),
                tok.offset,
            )),
        }
    }

    pub(super) fn parse_triple_term(&mut self) -> Result<(Term, Vec<Triple>)> {
        let parenthesized = self.check(&TokenKind::LParen);
        if parenthesized {
            self.advance();
        }

        if self.profile.is_line_syntax() && !parenthesized {
            return Err(EyeronError::at(
                "N-Triples/N-Quads use parenthesized triple terms",
                self.peek().offset,
            ));
        }

        let (subject, mut generated) = self.parse_term()?;
        if self.profile.is_rdf12() {
            self.ensure_rdf_triple_subject(&subject)?;
            if !generated.is_empty() && !generated_are_rdf12_reifications(&generated) {
                return Err(EyeronError::new(
                    "compound subject is not allowed inside an RDF triple term",
                ));
            }
        }
        let predicate = self.parse_verb()?;
        if self.profile.is_rdf12() {
            self.ensure_rdf_predicate(&predicate)?;
        }
        let (object, mut object_generated) = self.parse_term()?;
        if self.profile.is_rdf12() {
            self.ensure_rdf_triple_object(&object)?;
            if !object_generated.is_empty() && !generated_are_rdf12_reifications(&object_generated)
            {
                return Err(EyeronError::new(
                    "compound object is not allowed inside an RDF triple term",
                ));
            }
        }
        generated.append(&mut object_generated);
        if parenthesized {
            self.expect(TokenKind::RParen)?;
        }
        let triple = Triple::new(subject, predicate, object);

        if self.profile.is_rdf12() && !parenthesized {
            let reifier = if self.peek_is_reifier_token() {
                self.parse_reifier_token()?
            } else {
                self.fresh_blank("reif")
            };
            self.expect(TokenKind::RTriple)?;
            generated.push(Triple::new(
                reifier.clone(),
                Term::iri(RDF_REIFIES),
                Term::formula(vec![triple]),
            ));
            Ok((reifier, generated))
        } else {
            self.expect(TokenKind::RTriple)?;
            Ok((Term::formula(vec![triple]), generated))
        }
    }

    pub(super) fn finish_literal(&mut self, value: String) -> Result<(Term, Vec<Triple>)> {
        if !self.profile.is_rdf12() {
            validate_n3_literal_value(&value)?;
        }
        let mut lit = Literal::plain(value);
        if self.check(&TokenKind::HatHat) {
            self.advance();
            let (dt, generated) = self.parse_term()?;
            if !generated.is_empty() {
                return Err(EyeronError::new("datatype cannot generate triples"));
            }
            match dt {
                Term::Iri(i) => {
                    if self.profile.is_line_syntax() {
                        self.ensure_absolute_iri(&i, "N-Triples/N-Quads datatype IRI")?;
                    }
                    if self.profile.is_rdf12() && (i == RDF_LANG_STRING || i == RDF_DIR_LANG_STRING)
                    {
                        return Err(EyeronError::new(
                            "rdf:langString and rdf:dirLangString require language-tag syntax",
                        ));
                    }
                    lit.datatype = Some(i)
                }
                _ => return Err(EyeronError::new("datatype must be an IRI")),
            }
        } else if let TokenKind::Lang(lang) = self.peek_kind() {
            let lang = lang.clone();
            self.advance();
            if self.profile.is_rdf12() {
                let (clean, dir) = validate_lang_or_lang_dir(&lang, self.peek().offset)?;
                let clean = clean.to_ascii_lowercase();
                lit.language = Some(match dir {
                    Some(dir) => format!("{}--{}", clean, dir).into(),
                    None => clean.into(),
                });
            } else {
                lit.language = Some(lang.into());
            }
        }
        if self.profile.is_rdf12()
            && matches!(self.peek_kind(), TokenKind::PName(p) if p == "--ltr" || p == "--rtl")
        {
            return Err(EyeronError::at(
                "base direction requires a language tag",
                self.peek().offset,
            ));
        }
        Ok((Term::Literal(lit), Vec::new()))
    }

    pub(super) fn parse_blank_node_property_list(&mut self) -> Result<(Term, Vec<Triple>)> {
        let blank = self.fresh_blank("b");
        if self.check(&TokenKind::RBracket) {
            self.advance();
            return Ok((blank, Vec::new()));
        }
        // N3's `id` form lets a property list name its subject:
        // `[ id :c :d :e ]` is parsed as triples about `:c`, not about a
        // fresh blank node.  This is used in the notation3 static grammar
        // tests and is a compact way to nest property lists.
        if self.peek_is_bare("id") {
            self.advance();
            let (subject, mut generated) = self.parse_term()?;
            let mut triples = Vec::new();
            triples.append(&mut generated);
            triples.extend(self.parse_predicate_object_list(subject.clone())?);
            self.expect(TokenKind::RBracket)?;
            return Ok((subject, triples));
        }
        let triples = self.parse_predicate_object_list(blank.clone())?;
        self.expect(TokenKind::RBracket)?;
        Ok((blank, triples))
    }

    pub(super) fn parse_list(&mut self) -> Result<(Term, Vec<Triple>)> {
        let mut items = Vec::new();
        let mut triples = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
            let (item, mut generated) = self.parse_term()?;
            items.push(item);
            triples.append(&mut generated);
        }
        self.expect(TokenKind::RParen)?;
        Ok((Term::list(items), triples))
    }

    pub(super) fn parse_pname_or_path(
        &mut self,
        pname: &str,
        offset: usize,
    ) -> Result<(Term, Vec<Triple>)> {
        if self.profile.is_rdf12() {
            if contains_unescaped_path_operator(pname) {
                return Err(EyeronError::at("property paths are not RDF syntax", offset));
            }
            if !pname.contains(':') {
                return Err(EyeronError::at(
                    format!("expected IRI or prefixed name, got {}", pname),
                    offset,
                ));
            }
        } else if contains_unescaped_path_operator(pname) {
            return self.parse_path_pname(pname, offset);
        }
        Ok((Term::iri(self.expand_pname(pname, offset)?), Vec::new()))
    }

    pub(super) fn parse_path_pname(
        &mut self,
        pname: &str,
        offset: usize,
    ) -> Result<(Term, Vec<Triple>)> {
        let parts = split_unescaped_path(pname, offset)?;
        let mut current = Term::iri(self.expand_pname(&parts[0].1, offset)?);
        let mut generated = Vec::new();
        for (op, raw_pred) in parts.into_iter().skip(1) {
            let pred = Term::iri(self.expand_pname(&raw_pred, offset)?);
            let next = self.fresh_blank("path");
            if op == '!' {
                generated.push(Triple::new(current, pred, next.clone()));
            } else {
                generated.push(Triple::new(next.clone(), pred, current));
            }
            current = next;
        }
        Ok((current, generated))
    }

    pub(super) fn expand_pname(&self, pname: &str, offset: usize) -> Result<String> {
        if let Some((prefix, local)) = pname.split_once(':') {
            let Some(base) = self.doc.prefixes.get(prefix) else {
                return Err(EyeronError::at(
                    format!("unknown prefix '{}:'", prefix),
                    offset,
                ));
            };
            let local = if self.profile.is_rdf12() {
                decode_and_validate_pname_local(local, offset)?
            } else {
                local.to_string()
            };
            return Ok(format!("{}{}", base, local));
        }
        if let Some(base) = self.doc.base_iri.as_deref() {
            return Ok(self.resolve_iri_against_base(base, pname));
        }
        if let Some(base) = self.doc.prefixes.get("") {
            return Ok(format!("{}{}", base, pname));
        }
        Ok(format!("http://example.org/{}", pname))
    }

    pub(super) fn resolve_iri(&self, iri: &str) -> String {
        if has_uri_scheme(iri) {
            return iri.to_string();
        }
        let Some(base) = &self.doc.base_iri else {
            return iri.to_string();
        };
        self.resolve_iri_against_base(base, iri)
    }

    pub(super) fn resolve_iri_against_base(&self, base: &str, reference: &str) -> String {
        if has_uri_scheme(reference) {
            return reference.to_string();
        }
        let (base_scheme, base_authority, base_path, base_query, _) = split_uri(base);
        let (ref_path, ref_query, ref_fragment) = split_reference(reference);

        let (authority, path, query) = if let Some(rest) = ref_path.strip_prefix("//") {
            let (auth, p) = split_authority_path(rest);
            (Some(auth), remove_dot_segments(&p), ref_query)
        } else if ref_path.is_empty() {
            (
                base_authority.clone(),
                base_path.clone(),
                ref_query.or(base_query),
            )
        } else if ref_path.starts_with('/') {
            (
                base_authority.clone(),
                remove_dot_segments(&ref_path),
                ref_query,
            )
        } else {
            let merged = merge_paths(&base_path, &ref_path, base_authority.is_some());
            (
                base_authority.clone(),
                remove_dot_segments(&merged),
                ref_query,
            )
        };

        let mut out = String::new();
        if let Some(scheme) = base_scheme {
            out.push_str(&scheme);
            out.push(':');
        }
        if let Some(auth) = authority {
            out.push_str("//");
            out.push_str(&auth);
        }
        out.push_str(&path);
        if let Some(q) = query {
            out.push('?');
            out.push_str(&q);
        }
        if let Some(f) = ref_fragment {
            out.push('#');
            out.push_str(&f);
        }
        out
    }

    pub(super) fn ensure_rdf_subject(&self, term: &Term) -> Result<()> {
        match term {
            Term::Iri(_) | Term::Blank(_) | Term::List(_) => Ok(()),
            Term::Literal(_) => Err(EyeronError::new("RDF subject cannot be a literal")),
            Term::Formula(_) => Err(EyeronError::new(
                "RDF triple terms cannot be used as subjects",
            )),
            Term::Var(_) => Err(EyeronError::new("variables are not allowed in RDF syntax")),
        }
    }

    pub(super) fn ensure_rdf_line_subject(&self, term: &Term) -> Result<()> {
        match term {
            Term::Iri(iri) => self.ensure_absolute_iri(iri, "N-Triples/N-Quads subject"),
            Term::Blank(_) => Ok(()),
            _ => Err(EyeronError::new(
                "N-Triples/N-Quads subject must be an IRI or blank node",
            )),
        }
    }

    pub(super) fn ensure_rdf_line_object(&self, term: &Term) -> Result<()> {
        match term {
                Term::Iri(iri) => self.ensure_absolute_iri(iri, "N-Triples/N-Quads object"),
                Term::Blank(_) | Term::Literal(_) => Ok(()),
                Term::Formula(triples) if triples.len() == 1 => Ok(()),
                _ => Err(EyeronError::new("N-Triples/N-Quads object must be an IRI, blank node, literal, or parenthesized triple term")),
            }
    }

    pub(super) fn ensure_rdf_graph_label(&self, term: &Term) -> Result<()> {
        match term {
            Term::Iri(iri) => self.ensure_absolute_iri(iri, "graph label"),
            Term::Blank(_) => Ok(()),
            _ => Err(EyeronError::new("graph label must be an IRI or blank node")),
        }
    }

    pub(super) fn ensure_rdf_predicate(&self, term: &Term) -> Result<()> {
        match term {
            Term::Iri(iri) => self.ensure_absolute_iri(iri, "RDF predicate"),
            _ => Err(EyeronError::new("RDF predicate must be an IRI")),
        }
    }

    pub(super) fn ensure_rdf_triple_subject(&self, term: &Term) -> Result<()> {
        match term {
            Term::Iri(_) | Term::Blank(_) => Ok(()),
            Term::Formula(triples) if triples.len() == 1 => Ok(()),
            _ => Err(EyeronError::new(
                "RDF triple-term subject must be an IRI, blank node, or triple term",
            )),
        }
    }

    pub(super) fn ensure_rdf_triple_object(&self, term: &Term) -> Result<()> {
        match term {
            Term::Iri(_) | Term::Blank(_) | Term::Literal(_) => Ok(()),
            Term::Formula(triples) if triples.len() == 1 => Ok(()),
            _ => Err(EyeronError::new(
                "RDF triple-term object must be an IRI, blank node, literal, or triple term",
            )),
        }
    }

    pub(super) fn ensure_absolute_iri(&self, iri: &str, position: &str) -> Result<()> {
        if self.profile.is_line_syntax() && !has_uri_scheme(iri) {
            Err(EyeronError::new(format!("{} must be absolute", position)))
        } else {
            Ok(())
        }
    }
}

fn validate_blank_label(value: &str, offset: usize) -> Result<()> {
    if value.is_empty() || value.ends_with('.') || value.contains("..") {
        return Err(EyeronError::at("invalid blank node label", offset));
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(EyeronError::at("invalid blank node label", offset));
    };
    if matches!(first, '-' | '.') || !(first == '_' || first.is_alphanumeric()) {
        return Err(EyeronError::at("invalid blank node label", offset));
    }
    if value.chars().any(|c| {
        c.is_whitespace()
            || matches!(
                c,
                '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\' | ':'
            )
    }) {
        return Err(EyeronError::at("invalid blank node label", offset));
    }
    Ok(())
}

fn validate_prefix_label(value: &str, offset: usize) -> Result<()> {
    if value.ends_with('.') || value.contains("..") {
        return Err(EyeronError::at("invalid prefix label", offset));
    }
    Ok(())
}

fn validate_lang_or_lang_dir(raw: &str, offset: usize) -> Result<(String, Option<&'static str>)> {
    let (lang, dir) = if let Some(lang) = raw.strip_suffix("--ltr") {
        (lang, Some("ltr"))
    } else if let Some(lang) = raw.strip_suffix("--rtl") {
        (lang, Some("rtl"))
    } else {
        (raw, None)
    };
    if lang.is_empty() || lang.contains("--") {
        return Err(EyeronError::at("invalid language tag", offset));
    }
    let mut parts = lang.split('-');
    let Some(first) = parts.next() else {
        return Err(EyeronError::at("invalid language tag", offset));
    };
    if first.is_empty() || first.len() > 8 || !first.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(EyeronError::at("invalid language tag", offset));
    }
    for part in parts {
        if part.is_empty() || part.len() > 8 || !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(EyeronError::at("invalid language tag", offset));
        }
    }
    Ok((lang.to_string(), dir))
}

fn decode_and_validate_pname_local(raw: &str, offset: usize) -> Result<String> {
    if raw.is_empty() {
        return Ok(String::new());
    }
    if raw.starts_with('-') || raw.starts_with("\\-") || raw.starts_with('.') {
        return Err(EyeronError::at("invalid prefixed-name local part", offset));
    }
    let mut out = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let Some(esc) = chars.next() else {
                return Err(EyeronError::at("trailing prefixed-name escape", offset));
            };
            if !"_~.-!$&'()*+,;=/?#@%".contains(esc) {
                return Err(EyeronError::at("invalid prefixed-name escape", offset));
            }
            out.push(esc);
        } else if ch == '%' {
            let a = chars.next();
            let b = chars.next();
            match (a, b) {
                (Some(a), Some(b)) if a.is_ascii_hexdigit() && b.is_ascii_hexdigit() => {
                    out.push('%');
                    out.push(a);
                    out.push(b);
                }
                _ => {
                    return Err(EyeronError::at(
                        "invalid percent escape in prefixed-name local part",
                        offset,
                    ))
                }
            }
        } else {
            if matches!(ch, '~' | '^') {
                return Err(EyeronError::at("invalid prefixed-name local part", offset));
            }
            out.push(ch);
        }
    }
    Ok(out)
}

fn validate_n3_literal_value(value: &str) -> Result<()> {
    if value
        .chars()
        .any(|c| matches!(c, '\u{0000}' | '\u{FFFE}' | '\u{FFFF}'))
    {
        return Err(EyeronError::new("forbidden character in N3 string literal"));
    }
    Ok(())
}

fn contains_unescaped_path_operator(value: &str) -> bool {
    let mut escaped = false;
    for ch in value.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '!' || ch == '^' {
            return true;
        }
    }
    false
}

fn split_unescaped_path(value: &str, offset: usize) -> Result<Vec<(char, String)>> {
    let mut parts = Vec::<(char, String)>::new();
    let mut buf = String::new();
    let mut op = '!';
    let mut escaped = false;
    for ch in value.chars() {
        if escaped {
            buf.push('\\');
            buf.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '!' || ch == '^' {
            if buf.is_empty() {
                return Err(EyeronError::at("empty path component", offset));
            }
            parts.push((op, buf.clone()));
            buf.clear();
            op = ch;
        } else {
            buf.push(ch);
        }
    }
    if escaped {
        return Err(EyeronError::at("trailing path escape", offset));
    }
    if buf.is_empty() {
        return Err(EyeronError::at("empty path component", offset));
    }
    parts.push((op, buf));
    Ok(parts)
}

fn generated_are_rdf12_reifications(generated: &[Triple]) -> bool {
    generated
        .iter()
        .all(|triple| matches!(&triple.p, Term::Iri(iri) if iri == RDF_REIFIES))
}
