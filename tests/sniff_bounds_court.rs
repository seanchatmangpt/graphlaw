//! Sniff bounds court: routing must not build a full-document `Vec<char>`.
use graphlaw::dialect::{sniff, Dialect};

const BIG: usize = 16 * 1024 * 1024;

fn padded(head: &str, filler: &str) -> Vec<u8> {
    let mut v = head.as_bytes().to_vec();
    while v.len() < BIG {
        v.extend_from_slice(filler.as_bytes());
    }
    v
}

/// Behavior parity: a 16 MiB document routes identically to its small head.
#[test]
fn big_input_routes_like_small_head() {
    let head = "@prefix ex: <http://e/> .\nex:s ex:p ex:o .\n";
    let big = padded(head, "ex:s ex:p \"caf\u{e9} \u{1f600}\" .\n");
    assert_eq!(sniff(&big, None).map_err(|r| r.kind), Ok(Dialect::Turtle));
    assert_eq!(sniff(head.as_bytes(), None).map_err(|r| r.kind), Ok(Dialect::Turtle));
    let sql = "ex:q a sql:SelectStatement .\n";
    let big = padded(sql, "# comment line\n");
    assert_eq!(sniff(&big, None).map_err(|r| r.kind), Ok(Dialect::SqlRdf));
    let nt = "<http://a> <http://b> <http://c> .\n";
    let big = padded(nt, nt);
    // The routing window is HEAD_BUDGET characters of *skeleton* (`<> <> <> .` per line here),
    // far more than 8192 source bytes, so parity is against a line-aligned prefix that covers
    // the window: every line is `nt`, so a multiple of `nt.len()` ends on a line boundary.
    let prefix = &big[..nt.len() * 4000];
    assert_eq!(
        sniff(&big, None).map_err(|r| r.kind),
        sniff(prefix, None).map_err(|r| r.kind)
    );
    assert_eq!(sniff(&big, None).map_err(|r| r.kind), Ok(Dialect::NTriples));
}

/// Source law: no `Vec<char>` collection over the full input remains.
#[test]
fn no_full_document_char_vec() {
    let src = include_str!("../src/dialect/sniff.rs");
    for (n, line) in src.lines().enumerate() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        assert!(
            !line.contains("Vec<char>"),
            "sniff.rs:{} collects a char vector: {line}",
            n + 1
        );
    }
}
