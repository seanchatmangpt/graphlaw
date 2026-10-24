//! Sniff-layer court for `docs/jira/v26.10.23/design-sniff-fixes.md` (G8 class).
//!
//! One `#[test]` per design row; real `sniff()` on real bytes; no mocks.
//! Flip discipline: FLIP rows must fail when their documented inversion is
//! applied to `src/dialect/sniff.rs` (operator applies the hand edit, reruns
//! this court, expects the named rows red); GUARD rows are negative courts
//! the design marks as non-flipping (4c/4d: the BareSql arm must not capture
//! SPARQL; 5b: law pin on the head budget).

use graphlaw::dialect::{Dialect, RefusalKind, sniff};

// ---- Fix 1: route_sql_rdf literal-strip (quote/triple/escape state machine) ----

/// Row 1 [FLIP: delete the literal-strip arm in route_sql_rdf's stripper].
/// Pre-9633c03 this fired `Ambiguous`; the literal-strip must keep it plain
/// Turtle.
#[test]
fn row_01_literal_mention_is_turtle() {
    let d = sniff(b"ex:s <p> \"predicate sql:foo\" .", None);
    assert_eq!(d.map_err(|r| r.kind), Ok(Dialect::Turtle));
}

/// Row 1b [FLIP: same inversion as row 1].
/// `sql:SelectStatement` inside a literal must not select SqlRdf.
#[test]
fn row_01b_literal_statement_class_is_turtle() {
    let d = sniff(b"ex:s <p> \"a sql:SelectStatement\" .", None);
    assert_eq!(d.map_err(|r| r.kind), Ok(Dialect::Turtle));
}

/// Row 1c [FLIP: same inversion as row 1].
/// Comment stripping still works after the literal-strip port: a `sql:`
/// mention in a comment never routes SqlRdf.
#[test]
fn row_01c_comment_mention_is_turtle() {
    let d = sniff(b"# comment mentioning sql:SelectStatement\nex:s <p> <o> .", None);
    assert_eq!(d.map_err(|r| r.kind), Ok(Dialect::Turtle));
}

// ---- Fix 2: skeleton IRI-aware `#` ----

/// Row 2 [FLIP: revert skeleton's `#` arm to the bare comment-skip shape].
/// A `#` inside `<...>` is a fragment separator, not a comment start.
#[test]
fn row_02_iri_with_fragment_is_ntriples() {
    let d = sniff(b"<http://e/s#t> <http://e/p> <http://e/o> .", None);
    assert_eq!(d.map_err(|r| r.kind), Ok(Dialect::NTriples));
}

/// Row 2b [FLIP: same inversion as row 2].
/// `#` in an IRI and `#` in a literal must both survive skeletonization.
#[test]
fn row_02b_iri_and_literal_hash_is_ntriples() {
    let d = sniff(b"<http://e/s#t> <p> \"lit with # hash\" .", None);
    assert_eq!(d.map_err(|r| r.kind), Ok(Dialect::NTriples));
}

// ---- Fix 3: escape-aware line_terms ----

/// Row 3 [FLIP: re-break line_terms to a bare `find('"')`].
/// `\"` inside a literal must not terminate the term scan: 3 terms, not 4.
#[test]
fn row_03_escaped_quote_counts_three_terms() {
    let d = sniff(b"<s> <p> \"escaped \\\" quote\" .", None);
    assert_eq!(d.map_err(|r| r.kind), Ok(Dialect::NTriples));
}

/// Row 3b [FLIP: same inversion as row 3].
/// Full-sniff duplicate of row 3 (end-to-end classification).
#[test]
fn row_03b_escaped_quote_full_sniff() {
    let d = sniff(b"<s> <p> \"tab\\there \\\" q\" .", None);
    assert_eq!(d.map_err(|r| r.kind), Ok(Dialect::NTriples));
}

// ---- Fix 4: BareSql refusal before the SPARQL keyword set, typed not generic ----

/// Row 4 [FLIP: delete the BareSql arm from sniff()].
/// Bare SQL keyword+clause shape with no SPARQL marker is a typed refusal,
/// not a misroute to Sparql followed by a generic parse error.
#[test]
fn row_04_bare_select_from_refused() {
    let d = sniff(b"SELECT a, b FROM t", None);
    assert!(matches!(d, Err(ref r) if r.kind == RefusalKind::BareSql));
}

/// Row 4b [FLIP: same inversion "delete the BareSql arm"].
#[test]
fn row_04b_insert_into_values_refused() {
    let d = sniff(b"INSERT INTO t VALUES (1)", None);
    assert!(matches!(d, Err(ref r) if r.kind == RefusalKind::BareSql));
}

/// Row 4c [GUARD — non-flip by design: the BareSql arm must not capture
/// SPARQL]. `WHERE` and the graph-pattern `{` are SPARQL markers.
#[test]
fn row_04c_sparql_select_where_is_sparql() {
    let d = sniff(b"SELECT * WHERE { ?s ?p ?o }", None);
    assert_eq!(d.map_err(|r| r.kind), Ok(Dialect::Sparql));
}

/// Row 4d [GUARD — non-flip by design]. SPARQL with PREFIX directives stays
/// Sparql under the BareSql arm.
#[test]
fn row_04d_prefix_select_where_is_sparql() {
    let d = sniff(b"PREFIX e: <https://e/> SELECT * WHERE { ?s ?p ?o }", None);
    assert_eq!(d.map_err(|r| r.kind), Ok(Dialect::Sparql));
}

// ---- Fix 5: head budget (8 KiB, charged after skeleton + collapse) ----

/// Row 5 [FLIP: hard-code the budget back to 600].
/// A `@prefix` header block pushes the first statement's routing token past
/// offset 600 (collapsed skeleton) but within the 8 KiB budget; routing must
/// still see it.
#[test]
fn row_05_routing_token_past_600_still_routes() {
    let mut doc = String::new();
    for n in 0..48 {
        doc.push_str(&format!(
            "@prefix p{n}: <http://example.org/vocab/p{n}#term> .\n"
        ));
    }
    doc.push_str("ex:s a sql:SelectStatement .\n");
    let d = sniff(doc.as_bytes(), None);
    assert_eq!(d.map_err(|r| r.kind), Ok(Dialect::SqlRdf));
}

/// Row 5b [LAW PIN — no flip by design]. A document with more than
/// `HEAD_BUDGET` characters of collapsed skeleton routes on the first
/// `HEAD_BUDGET` characters only: the `sql:SelectStatement` assertion placed
/// beyond the budget is invisible, so the document stays plain Turtle.
#[test]
fn row_05b_beyond_budget_is_invisible() {
    let mut doc = String::new();
    for n in 0..700 {
        doc.push_str(&format!("ex:s{n} <http://e/p> <http://e/o> .\n"));
    }
    doc.push_str("ex:s a sql:SelectStatement .\n");
    let d = sniff(doc.as_bytes(), None);
    assert_eq!(d.map_err(|r| r.kind), Ok(Dialect::Turtle));
}
