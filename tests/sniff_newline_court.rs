//! Newline-preservation court: sniff's collapse keeps line structure, so
//! line-based routing sees real lines.

use graphlaw::dialect::{Dialect, sniff};

#[test]
fn multiline_ntriples_routes_ntriples() {
    let doc = b"<http://e/a> <http://e/b> <http://e/c> .\n<http://e/d> <http://e/e> \"x\" .\n";
    assert_eq!(sniff(doc, None).map_err(|r| r.kind), Ok(Dialect::NTriples));
}

#[test]
fn multiline_nquads_routes_nquads() {
    let doc = b"<http://e/a> <http://e/b> <http://e/c> <http://e/g> .\n<http://e/d>   <http://e/e>\t<http://e/f> <http://e/g> .\n";
    assert_eq!(sniff(doc, None).map_err(|r| r.kind), Ok(Dialect::NQuads));
}

#[test]
fn prefix_directive_line_routes_turtle() {
    let doc = b"@prefix e: <http://e/> .\n<http://e/a> <http://e/b> <http://e/c> .\n";
    assert_eq!(sniff(doc, None).map_err(|r| r.kind), Ok(Dialect::Turtle));
}

#[test]
fn large_ntriples_truncates_at_line_boundary() {
    let mut doc = String::new();
    for i in 0..2000 {
        doc.push_str(&format!("<http://e/s{i}> <http://e/p> <http://e/o{i}> .\n"));
    }
    assert_eq!(
        sniff(doc.as_bytes(), None).map_err(|r| r.kind),
        Ok(Dialect::NTriples)
    );
}
