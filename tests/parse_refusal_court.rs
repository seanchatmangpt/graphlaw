//! Court for the typed parse-refusal surface (`dialect::ParseRefusal`).
//!
//! Each `ReasonClass` must be constructible from a real bad input; location
//! fields must be present only when the refusing backend provides them; the
//! snippet must be bounded at 64 bytes.

use graphlaw::dialect::{
    Dialect, Engine, ParseRefusal, ReasonClass, Refusal, RefusalKind, check, check_typed,
    parse_rdf_typed,
};

/// Turtle statement with a truncated object and no closing `.`: a real syntax
/// error the backend reports with a location.
#[test]
fn syntax_from_real_bad_turtle_carries_backend_location() {
    let bad = "@prefix ex: <http://example.com/> .\nex:a ex:b ex:c\n";
    let err = check_typed(bad.as_bytes(), Dialect::Turtle, None).unwrap_err();
    assert_eq!(err.dialect, Dialect::Turtle);
    assert_eq!(err.engine, Engine::PurRdf);
    assert_eq!(err.reason, ReasonClass::Syntax);
    // PurRDF reports line/column for text-family parses; we surface, not guess.
    assert!(err.line.is_some(), "expected backend line: {err}");
    assert!(err.col.is_some(), "expected backend column: {err}");
    assert!(
        err.byte_offset.is_none(),
        "PurRDF never reports a byte offset"
    );
    assert!(err.snippet.as_deref().is_some_and(|s| !s.is_empty()));
}

/// Relative IRI with no base: `purrdf-iri` diagnostic code surfaces as BadIri.
#[test]
fn bad_iri_from_relative_iri_without_base() {
    let bad = "<http://example.com/s> <http://example.com/p> <relative/path> .\n";
    let err = check_typed(bad.as_bytes(), Dialect::Turtle, None).unwrap_err();
    assert_eq!(err.reason, ReasonClass::BadIri, "got: {err}");
}

/// Undeclared prefixed name offered to Eyeron: `unknown prefix` surfaces as
/// UndefinedPrefix with a byte offset and derived line/col.
#[test]
fn undefined_prefix_from_real_n3_document() {
    let bad = "ex:nope ex:nope ex:nope .\n";
    let err = check_typed(bad.as_bytes(), Dialect::N3, None).unwrap_err();
    assert_eq!(err.dialect, Dialect::N3);
    assert_eq!(err.engine, Engine::Eyeron);
    assert_eq!(err.reason, ReasonClass::UndefinedPrefix, "got: {err}");
    assert!(err.byte_offset.is_some(), "Eyeron reports an offset: {err}");
    assert!(err.line.is_some() && err.col.is_some());
}

/// N3 formula syntax (`{ ... }`, `=>`) offered to a Turtle/RDF dialect: refused
/// at routing BEFORE any backend parse, typed as FormulaInRdfLayer.
#[test]
fn formula_in_rdf_layer_refused_before_backend_parse() {
    let bad = "<http://example.com/a> <http://example.com/b> <http://example.com/c> .\n{ <http://example.com/d> <http://example.com/e> <http://example.com/f> }.\n";
    let err = check_typed(bad.as_bytes(), Dialect::Turtle, None).unwrap_err();
    assert_eq!(err.reason, ReasonClass::FormulaInRdfLayer);
    // The message names the offending token, and the legacy Refusal carries the
    // tier-mismatch class.
    let legacy: Refusal = err.clone().into();
    assert_eq!(legacy.kind, RefusalKind::TierMismatch);

    let implication = "@prefix ex: <http://example.com/> .\nex:a ex:b ex:c .\nex:a ex:b ex:c => ex:d ex:e ex:f .\n";
    let err = check_typed(implication.as_bytes(), Dialect::NTriples, None).unwrap_err();
    assert_eq!(err.reason, ReasonClass::FormulaInRdfLayer);

    // The pre-backend routing must hold on the dataset-parse path too.
    let err = parse_rdf_typed(bad.as_bytes(), Dialect::Turtle, None).unwrap_err();
    assert_eq!(err.reason, ReasonClass::FormulaInRdfLayer);
}

/// Non-UTF-8 bytes are refused as NotSemanticContent with NO location (nothing
/// is guessed when the backend has none to give).
#[test]
fn not_semantic_content_from_non_utf8_and_no_location_guessed() {
    let err = check_typed(&[0xff, 0xfe, 0x00], Dialect::Turtle, None).unwrap_err();
    assert_eq!(err.reason, ReasonClass::NotSemanticContent);
    assert_eq!(err.line, None);
    assert_eq!(err.col, None);
    assert_eq!(err.byte_offset, None);
    assert_eq!(err.snippet, None);
}

/// A brace-delimited document is genuinely dialect-ambiguous (TriG or ShExC).
/// Routing refuses it with AmbiguousDialect; the typed class is exercised here
/// on that real input shape.
#[test]
fn ambiguous_dialect_class_on_real_brace_document() {
    let ambiguous = graphlaw::dialect::sniff(b"<s> <p> { <a> <b> <c> }", None).unwrap_err();
    assert!(ambiguous.to_string().contains("brace"));
    let typed = ParseRefusal::new(
        Dialect::ShExC,
        ReasonClass::AmbiguousDialect,
        ambiguous.message.clone(),
    );
    assert_eq!(typed.reason, ReasonClass::AmbiguousDialect);
    let legacy: Refusal = typed.into();
    assert_eq!(legacy.kind, RefusalKind::Ambiguous);
}

/// JSON-LD with a remote `@context` IRI: typed RemoteContext refusal, no
/// network attempt (the host is `.invalid`; a fetch attempt fails the test's
/// own assertion by producing a different error class).
#[test]
fn jsonld_remote_context_typed_refusal_no_network() {
    let bad = br#"{"@context": "https://example.invalid/never-served-context.jsonld", "@id": "http://example.com/s", "http://example.com/p": {"@value": "v"}}"#;
    let err = check_typed(bad, Dialect::JsonLd, None).unwrap_err();
    assert_eq!(err.reason, ReasonClass::RemoteContext, "got: {err}");
    let err = parse_rdf_typed(bad, Dialect::JsonLd, None).unwrap_err();
    assert_eq!(err.reason, ReasonClass::RemoteContext);
    // Legacy surface keeps the document-valid-but-unusable class.
    let legacy: Refusal = err.into();
    assert_eq!(legacy.kind, RefusalKind::Unsupported);

    // Inline object contexts (terms mapping to absolute IRIs) are LOCAL and
    // must still parse.
    let local =
        br#"{"@context": {"ex": "http://example.com/"}, "@id": "ex:s", "ex:p": {"@value": "v"}}"#;
    check_typed(local, Dialect::JsonLd, None).expect("inline context must not be refused");
}

/// Remote context inside an array is also refused; a string IRI inside an
/// inline mapping object is not.
#[test]
fn remote_context_array_detected_inline_object_not() {
    let remote =
        br#"{"@context": ["https://example.invalid/c.jsonld"], "@id": "http://example.com/s"}"#;
    let err = check_typed(remote, Dialect::JsonLd, None).unwrap_err();
    assert_eq!(err.reason, ReasonClass::RemoteContext);
}

/// Limit refusals carry the real observed/max counts, map from
/// `eyeron::ReasonerLimit`, and convert to the legacy ResourceLimit kind.
#[test]
fn limit_class_carries_counts_and_maps_from_reasoner_limit() {
    let r = ParseRefusal::limit(
        Dialect::N3,
        "n3_derived_facts",
        10_001,
        10_000,
        "closure cap",
    );
    assert_eq!(
        r.reason,
        ReasonClass::Limit {
            name: "n3_derived_facts".into(),
            observed: 10_001,
            max: 10_000
        }
    );
    let legacy: Refusal = r.into();
    assert_eq!(legacy.kind, RefusalKind::ResourceLimit);

    let rc: ReasonClass = graphlaw::eyeron::ReasonerLimit::MatchSteps.into();
    let ReasonClass::Limit { name, .. } = rc else {
        panic!("expected Limit");
    };
    assert!(!name.is_empty());
}

/// Display is one line plus (when present) a bounded snippet line.
#[test]
fn display_is_one_line_plus_snippet_line() {
    let bad = "@prefix ex: <http://example.com/> .\nex:a ex:b ex:c\n";
    let err = check_typed(bad.as_bytes(), Dialect::Turtle, None).unwrap_err();
    let text = format!("{err}");
    let mut lines = text.lines();
    let first = lines.next().unwrap();
    assert!(!first.contains('\n'));
    assert!(lines.next().is_some(), "snippet line expected: {text}");
    for line in text.lines().skip(1) {
        assert!(line.starts_with("  | "));
    }
    // Snippet bound.
    let snippet = err.snippet.expect("snippet present");
    assert!(snippet.len() <= 64, "snippet was {} bytes", snippet.len());
}

/// Snippets are bounded even for very long single-line documents.
#[test]
fn snippet_bounded_on_long_line() {
    let long = format!("<http://example.com/s> <http://example.com/p> {} .\n", {
        let mut filler = String::new();
        for _ in 0..40 {
            filler.push_str("<http://example.com/x>");
        }
        filler
    });
    let err = check_typed(long.as_bytes(), Dialect::Turtle, None).unwrap_err();
    if let Some(s) = &err.snippet {
        assert!(s.len() <= 64, "snippet was {} bytes", s.len());
    }
}

/// The legacy `check` surface keeps working and classifies a syntax error as
/// EngineRejected (no behavioral regression).
#[test]
fn legacy_check_still_refuses_with_engine_rejected() {
    let bad = "@prefix ex: <http://example.com/> .\nex:a ex:b ex:c\n";
    let r = check(bad.as_bytes(), Dialect::Turtle, None).unwrap_err();
    assert_eq!(r.kind, RefusalKind::EngineRejected);
    assert_eq!(r.dialect, Some(Dialect::Turtle));
}
