//! Refusal-shape court: AstError size, truncation marking, observed/max limit
//! metrics, and typed (non-panicking) RdfTriple deserialization.

use graphlaw::duckdb_eval::RdfTriple;
use graphlaw::sql_ast::AstError;

#[test]
fn ast_error_is_small_by_value() {
    // Boxed context: code + pointer, not the ~400-byte inline payload.
    assert!(
        std::mem::size_of::<AstError>() <= 32,
        "AstError is {} bytes",
        std::mem::size_of::<AstError>()
    );
}

#[test]
fn oversized_identifier_truncation_is_marked() {
    let long = "x".repeat(500);
    let e = AstError::InvalidIdentifier(long);
    let msg = e.to_string();
    assert!(msg.contains("…[truncated 308 bytes]"), "{msg}");
    let short = AstError::InvalidIdentifier("ok".into()).to_string();
    assert_eq!(short, "Invalid SQL identifier: ok");
}

#[test]
fn truncation_respects_char_boundaries() {
    let long = "é".repeat(200); // 400 bytes, 2 per char
    let msg = AstError::InvalidIdentifier(long).to_string();
    assert!(msg.contains("…[truncated "), "{msg}");
}

#[test]
fn limit_refusal_carries_observed_and_max() {
    let e = AstError::RecursionLimitObserved(77, 64);
    assert_eq!(e.ctx.aux_num, 77);
    assert_eq!(e.ctx.num, 64);
    let msg = e.to_string();
    assert!(msg.contains("(64)") && msg.contains("observed 77, max 64"), "{msg}");
    // Legacy constructor text is byte-identical.
    assert_eq!(
        AstError::RecursionLimitExceeded(32).to_string(),
        "Recursion depth limit exceeded (32) while traversing AST"
    );
}

#[test]
fn malformed_direction_token_is_a_typed_deserialize_error() {
    let bad = r#"{"subject":"s","predicate":"p","object":{"Literal":{"lexical_form":"a","datatype":"http://www.w3.org/2001/XMLSchema#string","language":null,"direction":"sideways"}}}"#;
    let err = serde_json::from_str::<RdfTriple>(bad).expect_err("malformed direction must refuse");
    assert!(err.to_string().contains("[REFUSAL-01]"), "{err}");

    let nested = r#"{"subject":"s","predicate":"p","object":{"Triple":{"s":{"Iri":"a"},"p":{"Iri":"b"},"o":{"Literal":{"lexical_form":"a","datatype":"d","language":null,"direction":"up"}}}}}"#;
    assert!(serde_json::from_str::<RdfTriple>(nested).is_err());

    let good = r#"{"subject":"s","predicate":"p","object":{"Literal":{"lexical_form":"a","datatype":"d","language":"en","direction":"Rtl"}}}"#;
    assert!(serde_json::from_str::<RdfTriple>(good).is_ok());
}
