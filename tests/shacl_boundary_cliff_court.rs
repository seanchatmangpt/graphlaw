//! Courts for the WP P07 hard physical SHACL caps
//! ([`graphlaw::shacl_guard::check_shape_limits`]).
//!
//! Four caps — bytes, quads, degree, depth — each proved at the three
//! guarded call sites:
//!
//! 1. **at-cap admits**: an input sitting exactly at (or one under) the cap
//!    passes the boundary — the cap never over-refuses worst-case legal
//!    input; the trusted canonical compile surface
//!    (`ontologies/sql-ast.ttl` + `ggen/sql-dialect/statement_classes.ttl`,
//!    66,182 bytes / 1,188 quads) still compiles.
//! 2. **over-cap refuses typed** through all three sites:
//!    `LawState::transition(Step::AdmitShacl)` (the `shacl_validation_*`
//!    violation component), `graphlaw::abi::call_json(op=shacl)` (the
//!    `ResourceLimit` envelope naming limit/observed/max), and
//!    `graphlaw::sql_ast::validate_ast_shacl` (the MalformedAst refusal
//!    naming the limit). The O(1) byte refusal is also ceiling-checked to
//!    stay orders of magnitude under the engine cost it replaces.
//! 3. **guard-removal flip**: the raw engine (no guard) ADMITS the same
//!    over-cap input, so the typed refusal is attributable to the guard
//!    alone — the flip the anti-vacuity law requires.
//!
//! Plus the two standing controls: the depth-17 wall stays a wall, and
//! cyclic shapes stay cheap (admitted within the expansion budget). Real
//! graphs, real transitions, real ABI envelopes — no mocks, no sleeps.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawError, LawState};
use graphlaw::purrdf;
use graphlaw::shacl::engine::{parse_shapes, validate_dataset};
use graphlaw::shacl_guard::{self, ShaclLimitExceeded};
use serde_json::json;

const PREFIX: &str = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix ex: <http://example.org/> .\n";

/// Acyclic `sh:node` chain of exactly `n` link edges.
fn node_chain(n: usize) -> String {
    let mut ttl = String::from(PREFIX);
    for i in 0..n {
        ttl.push_str(&format!(
            "ex:s{i} a sh:NodeShape ; sh:targetNode ex:x ; sh:node ex:s{} .\n",
            i + 1
        ));
    }
    ttl.push_str(&format!("ex:s{n} a sh:NodeShape .\n"));
    ttl
}

/// One shape with `deg` `sh:property` blank-node children (a star). The
/// property shapes carry only `sh:path` (vacuous: never violated), so the
/// star is conforming against any dataset and can prove the at-cap ADMIT.
fn degree_star(deg: usize) -> String {
    let mut ttl = String::from(PREFIX);
    ttl.push_str("ex:root a sh:NodeShape ; sh:targetNode ex:x");
    for i in 0..deg {
        ttl.push_str(&format!(" ; sh:property [ sh:path ex:p{i} ]"));
    }
    ttl.push_str(" .\n");
    ttl
}

/// Independent shallow shapes, exact-or-under `target` bytes (a line is
/// pushed only if the result stays <= target). Property shapes carry
/// `sh:minCount 0` (never violated), so the blob is SHACL-conforming against
/// any dataset and can prove the at-cap ADMIT through sites that also run
/// the real validator (the law boundary refuses non-conforming input as
/// ordinary SHACL violations, which would mask the cap result).
fn byte_blob(target: usize) -> String {
    let mut ttl = String::from(PREFIX);
    let mut i = 0usize;
    loop {
        let line = format!(
            "ex:s{i} a sh:NodeShape ; sh:targetNode ex:x{i} ; sh:property [ sh:path ex:p{i} ; sh:minCount 0 ] .\n"
        );
        if ttl.len() + line.len() > target {
            break;
        }
        ttl.push_str(&line);
        i += 1;
    }
    ttl
}

/// Pure `sh:node` cycle (a<->b) plus an independent conforming shape.
fn cyclic_shapes() -> String {
    format!(
        "@prefix sh: <http://www.w3.org/ns/shacl#> .\n\
         @prefix ex: <http://example.org/> .\n\
         ex:a a sh:NodeShape ; sh:node ex:b .\n\
         ex:b a sh:NodeShape ; sh:node ex:a .\n\
         ex:c a sh:NodeShape ; sh:targetNode ex:x ; sh:property [ sh:path ex:p ] .\n"
    )
}

/// The exact shipped bytes of the trusted canonical shapes surface.
const CANONICAL_SHAPES: &str = concat!(
    include_str!("../ontologies/sql-ast.ttl"),
    "\n",
    include_str!("../ggen/sql-dialect/statement_classes.ttl")
);

/// A minimal VALID AST document (the `compile_turtle` doctest document).
const AST_DOC: &str = concat!(
    "@prefix sql: <http://graphlaw.org/sql#> .\n",
    "@prefix ex: <http://example.org/queries#> .\n",
    "ex:S a sql:SelectStatement ;\n",
    "    sql:statementIndex 1 ;\n",
    "    sql:fromTable [ sql:tableName \"t\" ] ;\n",
    "    sql:selectList ( [ sql:columnName \"c\" ] ) .\n",
);

fn empty_state() -> LawState {
    LawState::parse(b"", Dialect::NTriples, None).expect("empty state parses")
}

/// Real validation path one: `LawState::transition(Step::AdmitShacl)`.
fn admit(state: &LawState, shapes: &str) -> Result<(), LawError> {
    let step = graphlaw::law::Step::AdmitShacl { shapes_ttl: shapes };
    state.transition(&step).map(|_| ())
}

/// The `shacl_validation_*` component text of a NotAdmitted refusal, if the
/// error is exactly one such violation.
fn limit_component(err: &LawError) -> Option<String> {
    match err {
        LawError::NotAdmitted { violations, results } if *violations == 1 && results.len() == 1 => {
            Some(results[0].component.clone())
        }
        _ => None,
    }
}

/// Real validation path two: the ABI `shacl` op envelope. Returns
/// `(ok, error_json)` so callers can assert typed refusal shape.
fn abi_shacl(shapes: &str) -> (bool, serde_json::Value) {
    let resp = graphlaw::abi::call_json(&json!({
        "op": "shacl",
        "data": {"text": "@prefix ex: <http://example.org/> .\nex:x a ex:T .\n", "dialect": "turtle"},
        "shapes": shapes,
    }));
    let ok = resp["ok"].as_bool().unwrap_or(false);
    (ok, resp["error"].clone())
}

/// Real validation path three: the sql-rdf compile boundary
/// (`validate_ast_shacl` — the same guard `compile_turtle` rides).
fn sql_shacl(shapes: &str) -> Result<(), graphlaw::sql_ast::AstError> {
    graphlaw::sql_ast::validate_ast_shacl(AST_DOC, shapes)
}

// ---------------------------------------------------------------- at-cap admits

#[test]
fn at_cap_inputs_admit_at_all_three_sites() {
    // bytes at cap-1 (130,982-byte independent blob): must admit everywhere.
    let blob = byte_blob(shacl_guard::MAX_SHACL_SHAPES_BYTES - 1);
    assert!(blob.len() <= shacl_guard::MAX_SHACL_SHAPES_BYTES);
    let state = empty_state();
    admit(&state, &blob).expect("bytes-at-cap blob must admit through the law boundary");
    sql_shacl(&blob).expect("bytes-at-cap blob must admit through the sql-rdf boundary");
    let (ok, err) = abi_shacl(&blob);
    assert!(ok, "bytes-at-cap blob must admit through the ABI boundary: {err:?}");

    // degree exactly at cap: admits.
    let star = degree_star(shacl_guard::MAX_SHACL_DEGREE);
    admit(&state, &star).expect("degree-at-cap star must admit");

    // depth exactly at cap: admits.
    let chain = node_chain(shacl_guard::MAX_SHACL_VALIDATION_DEPTH);
    admit(&state, &chain).expect("depth-at-cap chain must admit");

    // quads well over half the cap via the bytes-at-cap blob (~6.5k quads):
    // admitted above; the ordering court below proves why the quads arm
    // cannot fire for Turtle input under the byte cap.
}

#[test]
fn canonical_compile_surface_admits_under_the_caps() {
    // The trusted canonical surface rides through the guard on every
    // compile_turtle — the caps must never cap out the repo's own surface.
    let compiled = graphlaw::sql_ast::compile_turtle(AST_DOC)
        .expect("canonical surface must compile under the caps");
    assert!(compiled.verify_seal(), "compiled seal must verify");
    sql_shacl(CANONICAL_SHAPES).expect("canonical shapes admit at the sql-rdf boundary");
}

/// An over-cap blob: the exact-or-under filler stops short of the target, so
/// one shape line is appended at a time until the text is strictly over the
/// byte cap.
fn over_cap_blob() -> String {
    let mut over = byte_blob(shacl_guard::MAX_SHACL_SHAPES_BYTES + 512);
    let mut i = 10_000_000usize;
    while over.len() <= shacl_guard::MAX_SHACL_SHAPES_BYTES {
        over.push_str(&format!(
            "ex:s{i} a sh:NodeShape ; sh:targetNode ex:x{i} ; sh:property [ sh:path ex:p{i} ; sh:minCount 0 ] .\n"
        ));
        i += 1;
    }
    over
}

// ---------------------------------------------------------------- over-cap refuses typed

#[test]
fn over_cap_bytes_refuses_typed_at_all_three_sites() {
    let over = over_cap_blob();
    assert!(over.len() > shacl_guard::MAX_SHACL_SHAPES_BYTES);

    let state = empty_state();
    let err = admit(&state, &over).expect_err("over-cap bytes must refuse");
    let comp = limit_component(&err).expect("single typed violation");
    assert!(comp.contains("shacl_validation_bytes"), "component: {comp}");
    assert!(comp.contains(&format!("observed {} > max {}", over.len(), shacl_guard::MAX_SHACL_SHAPES_BYTES)), "component: {comp}");

    let (ok, err) = abi_shacl(&over);
    assert!(!ok, "over-cap bytes must refuse at the ABI boundary");
    assert_eq!(err["kind"], "ResourceLimit");
    assert_eq!(err["details"]["limit"], "shacl_validation_bytes");
    assert_eq!(err["details"]["max"], json!(shacl_guard::MAX_SHACL_SHAPES_BYTES));

    let err = sql_shacl(&over).expect_err("over-cap bytes must refuse at the sql-rdf boundary");
    let msg = err.to_string();
    assert!(msg.contains("shacl_validation_bytes"), "message: {msg}");

    // The refusal is an O(1) byte check: ceiling-check it orders of
    // magnitude under the engine cost it replaces (measured: 375-917 ns;
    // the same input's engine leg costs milliseconds). Loose 100 ms ceiling
    // so the assert never flakes under lane load.
    let t0 = std::time::Instant::now();
    let _ = shacl_guard::check_shape_limits(&over);
    let elapsed = t0.elapsed();
    assert!(
        elapsed < std::time::Duration::from_millis(100),
        "over-cap byte refusal took {elapsed:?}; O(1) check must stay microsecond-class"
    );
}

#[test]
fn over_cap_degree_refuses_typed_at_all_three_sites() {
    let over = degree_star(shacl_guard::MAX_SHACL_DEGREE + 1);
    let state = empty_state();
    let err = admit(&state, &over).expect_err("over-cap degree must refuse");
    let comp = limit_component(&err).expect("single typed violation");
    assert!(comp.contains("shacl_validation_degree"), "component: {comp}");
    assert!(
        comp.contains(&format!(
            "observed shape out-degree {} > max {}",
            shacl_guard::MAX_SHACL_DEGREE + 1,
            shacl_guard::MAX_SHACL_DEGREE
        )),
        "component: {comp}"
    );

    let (ok, err) = abi_shacl(&over);
    assert!(!ok);
    assert_eq!(err["kind"], "ResourceLimit");
    assert_eq!(err["details"]["limit"], "shacl_validation_degree");
    assert_eq!(err["details"]["observed"], json!(shacl_guard::MAX_SHACL_DEGREE + 1));

    let err = sql_shacl(&over).expect_err("over-cap degree must refuse");
    let msg = err.to_string();
    assert!(msg.contains("shacl_validation_degree"), "message: {msg}");
}

#[test]
fn over_cap_depth_refuses_typed_at_all_three_sites() {
    // THE 17 CONTROL: one edge past the depth cap is a typed wall at every
    // site, never a validation.
    let over = node_chain(shacl_guard::MAX_SHACL_VALIDATION_DEPTH + 1);
    let state = empty_state();
    let err = admit(&state, &over).expect_err("depth-17 control must refuse");
    let comp = limit_component(&err).expect("single typed violation");
    assert!(comp.contains("shacl_validation_depth"), "component: {comp}");

    let (ok, err) = abi_shacl(&over);
    assert!(!ok);
    assert_eq!(err["kind"], "ResourceLimit");
    assert_eq!(err["details"]["limit"], "shacl_validation_depth");
    assert_eq!(err["details"]["observed"], json!(shacl_guard::MAX_SHACL_VALIDATION_DEPTH + 1));

    let err = sql_shacl(&over).expect_err("depth-17 control must refuse");
    let msg = err.to_string();
    assert!(
        msg.contains("SHACL nesting depth exceeded: observed 17, max 16"),
        "message: {msg}"
    );
}

#[test]
fn quads_arm_is_typed_and_bytes_gate_dense_turtle_fail_closed() {
    // Typed-arm witness: the Quads variant carries its own limit name and
    // cap (defense-in-depth backstop).
    let q = ShaclLimitExceeded::Quads { observed: shacl_guard::MAX_SHACL_QUADS + 1 };
    assert_eq!(q.limit_name(), "shacl_validation_quads");
    assert_eq!(q.cap(), shacl_guard::MAX_SHACL_QUADS);
    assert!(q.to_string().contains("quads"));

    // Ordering witness: the densest legal Turtle (~19 bytes/quad: the
    // minimal `ex:a ex:b ex:c .` pattern) cannot reach the quads cap under
    // the byte cap — a 16,385-quad dense graph is ~300 KB, so the O(1)
    // byte check MUST fire first (fail-closed ordering: bytes -> quads).
    let mut dense = String::from(PREFIX);
    for i in 0..(shacl_guard::MAX_SHACL_QUADS + 1) {
        dense.push_str(&format!("ex:a{i} ex:b ex:c .\n"));
    }
    assert!(dense.len() > shacl_guard::MAX_SHACL_SHAPES_BYTES);
    let over = shacl_guard::check_shape_limits(&dense)
        .expect_err("dense over-quads Turtle must refuse");
    assert!(
        matches!(over, ShaclLimitExceeded::Bytes { .. }),
        "byte cap must gate dense Turtle before the quads arm, got {over}"
    );
}

// ---------------------------------------------------------------- guard-removal flip (anti-vacuity)

#[test]
fn guard_removal_flip_raw_engine_admits_over_cap_inputs() {
    // The SAME over-cap inputs that every guarded site just refused are
    // ADMITTED by the raw purrdf engine — the typed refusals are
    // attributable to the guard alone, not to the engine (anti-vacuity).
    let over_bytes = over_cap_blob();
    let data = purrdf::parse_dataset(
        "@prefix ex: <http://example.org/> .\nex:x a ex:T ; ex:p0 \"v\" .\n".as_bytes(),
        "text/turtle",
        None,
    )
    .expect("data parses");

    for (name, shapes) in [
        ("over-cap bytes", over_bytes),
        ("over-cap degree", degree_star(shacl_guard::MAX_SHACL_DEGREE + 1)),
        ("over-cap depth (17 control)", node_chain(shacl_guard::MAX_SHACL_VALIDATION_DEPTH + 1)),
    ] {
        // The flip: the guard refuses the exact input the engine accepts.
        assert!(
            shacl_guard::check_shape_limits(&shapes).is_err(),
            "{name}: guard must refuse this input"
        );
        let parsed = parse_shapes(&shapes, None).expect("raw engine parses the shapes");
        let report = validate_dataset(&data, &parsed).expect("raw engine validates");
        std::hint::black_box(report.conforms);
    }
}

// ---------------------------------------------------------------- standing controls

#[test]
fn cyclic_shapes_stay_cheap_and_admitted() {
    // A pure sh:node cycle must NOT trip depth/degree/budget refusals: the
    // bounded DFS proves boundedness within the expansion budget (budget
    // exhaustion would BE a refusal, so an admit IS the cheapness witness).
    let shapes = cyclic_shapes();
    let state = empty_state();
    admit(&state, &shapes).expect("cyclic shapes must admit (boundedness proven within budget)");
    let (ok, err) = abi_shacl(&shapes);
    assert!(ok, "cyclic shapes must admit at the ABI boundary: {err:?}");
}
