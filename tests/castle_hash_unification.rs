//! L41 differential court: byte-identity of the two canonical-hash surfaces
//! for the same TTL input.
//!
//! Side A (castle): `castle-ggen-rdf validate` computes ggen-engine
//! `GraphLawStore::state_hash()` = BLAKE3 over the engine's canonical N-Quads
//! lines (bounded color-refinement, `c14n{i}` labels, NO trailing ` .`)
//! joined by `\n` (ggen-engine `src/graph.rs` `state_hash`, rev
//! 162e466d8f07d0a75a468b4441b4bc8b1aad369b — the exact rev castle pins).
//!
//! Side B (graphlaw): `op=canonical` (`src/abi.rs` `dispatch "canonical"`)
//! yields `LawState::id` = `sha256:` over purrdf RDFC-1.0 canonical N-Quads
//! WITH trailing ` .` (`src/law.rs` `from_dataset`).
//!
//! OBSERVED VERDICT (2026-10-13): byte-identity does NOT hold — 0/3 corpus
//! inputs match. The divergence is decomposed and pinned below:
//!
//! 1. Digest algorithm: castle = BLAKE3, graphlaw = SHA-256. Not a config
//!    delta; closed only by an algorithm change on one side.
//! 2. Serialization framing: even on identical canonical orderings, graphlaw
//!    hashes lines ending ` .` while castle hashes lines without it, so the
//!    hashed byte streams differ regardless of algorithm.
//! 3. Datatype normalization: castle (oxigraph canonical) normalizes the
//!    Turtle decimal `2.0` to `"2"^^xsd:decimal`; graphlaw/purrdf preserves
//!    the lexical form `"2.0"^^xsd:decimal`.
//! 4. Blank-node labeling: both emit `c14n{i}` labels but the index
//!    assignment differs (different refinement/ordering), so blank-node
//!    graphs diverge beyond framing.
//!
//! A minimal sort/flag alignment does NOT close the gap: layer 1 is an
//! algorithm change and layer 3 is a canonicalizer semantic difference, not
//! a sort order or flag. Unification requires either castle adopting
//! SHA-256 over RDFC-1.0 output (including ` .`) or graphlaw adopting the
//! ggen scheme — a real change on one side, not configuration.

use serde_json::json;

fn graphlaw_canonical(ttl: &str) -> (String, String) {
    let out = graphlaw::abi::call(
        serde_json::to_vec(&json!({
            "op": "canonical",
            "data": {"text": ttl, "dialect": "turtle"}
        }))
        .unwrap()
        .as_slice(),
    );
    let r: serde_json::Value = serde_json::from_slice(&out).expect("JSON response");
    assert_eq!(r["ok"], json!(true), "graphlaw canonical refused: {r}");
    let id = r["id"].as_str().expect("id").to_string();
    let nquads = r["nquads"].as_str().expect("nquads").to_string();
    (id, nquads)
}

fn castle_canonical(ttl: &str) -> (String, Vec<String>) {
    use ggen_engine::graph::{GraphEngine, GraphLawStore};
    let g = GraphLawStore::new().unwrap_or_else(|e| panic!("graph init: {e}"));
    g.insert_turtle(ttl)
        .unwrap_or_else(|e| panic!("turtle load: {e}"));
    let hex: String = GraphEngine::state_hash(&g)
        .unwrap_or_else(|e| panic!("state_hash: {e}"))
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let quads = GraphEngine::canonical_quads(&g)
        .unwrap_or_else(|e| panic!("canonical_quads: {e}"));
    (hex, quads)
}

const GL_TTL: &str = r#"@prefix ex: <http://example.org/> .
ex:alice ex:knows ex:bob .
ex:bob ex:knows ex:carol .
"#;

const CASTLE_TTL: &str = r#"@prefix ex: <http://example.org/deps#> .
ex:app ex:dependsOn ex:lib ;
    ex:version "1.2.3" .
"#;

const BNODE_TTL: &str = r#"@prefix ex: <http://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
[] ex:name "root" ;
   ex:child [ ex:weight 2.0 ; ex:name "leaf" ] .
ex:doc ex:cites [ ex:name "cited" ] .
"#;

/// Same core graph content, different surface syntax: the second document is
/// the first with reordered statements, renamed prefixes, and the decimal
/// written `2.00` instead of `2.0`.
const BNODE_TTL_SYNTAX_VARIANT: &str = r#"@prefix p: <http://example.org/> .
p:doc p:cites [ p:name "cited" ] .
[] p:name "root" ;
   p:child [ p:name "leaf" ; p:weight 2.00 ] .
"#;

fn corpus() -> [(&'static str, &'static str); 3] {
    [
        ("graphlaw-corpus alice-bob-carol", GL_TTL),
        ("castle-corpus dependency-system", CASTLE_TTL),
        ("adversarial blank-nodes+typed-literals", BNODE_TTL),
    ]
}

/// COURT: the digests must stay DIVERGENT in the observed way — if either
/// surface changes (a convergence PR lands, or a drift re-key), this fails
/// and the unification question must be re-decided deliberately.
#[test]
fn court_digests_remain_divergent_until_unification_is_admitted() {
    for (name, ttl) in corpus() {
        let (gl_id, _) = graphlaw_canonical(ttl);
        let (castle_hex, _) = castle_canonical(ttl);
        let gl_hex = gl_id.trim_start_matches("sha256:");
        assert_ne!(
            gl_hex, castle_hex,
            "{name}: digests now match — the hash-unification seam has moved; \
             re-run the identity probe and re-decide this court"
        );
        // Digest namespaces stay disjoint by algorithm prefix on the wire.
        assert!(gl_id.starts_with("sha256:"), "{name}: graphlaw id lost sha256: prefix");
        assert_eq!(castle_hex.len(), 64, "{name}: castle state_hash no longer 32 bytes");
    }
}

/// Layer 1+2 receipts: even where the canonical ORDER agrees exactly, the
/// hashed byte streams differ (` .` framing) and the algorithms differ
/// (SHA-256 vs BLAKE3).
#[test]
fn divergence_layer_framing_and_algorithm() {
    for (name, ttl) in corpus() {
        let (gl_id, gl_nq) = graphlaw_canonical(ttl);
        let (castle_hex, castle_quads) = castle_canonical(ttl);

        // Framing: graphlaw lines carry the N-Quads terminator, castle's do not.
        for line in gl_nq.lines() {
            assert!(
                line.ends_with(" ."),
                "{name}: graphlaw canonical line lost ` .` framing: {line}"
            );
        }
        for line in &castle_quads {
            assert!(
                !line.ends_with(" ."),
                "{name}: castle canonical line gained ` .` framing: {line}"
            );
        }

        // Algorithm: SHA-256 of graphlaw's own canonical form equals
        // graphlaw's id (self-consistent), and never equals castle's digest.
        use sha2::{Digest, Sha256};
        let sha_of_gl: String = Sha256::digest(gl_nq.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            sha_of_gl,
            gl_id.trim_start_matches("sha256:"),
            "{name}: graphlaw id is not sha256 over its canonical form"
        );
        assert_ne!(
            sha_of_gl, castle_hex,
            "{name}: castle hash now equals sha256 over graphlaw's canonical form"
        );
    }
}

/// Layer 3 receipt: on bnode-free inputs the canonical ORDERINGS agree
/// line-for-line (modulo the ` .` framing) — ordering is NOT the cause.
#[test]
fn bnode_free_orderings_agree_modulo_framing() {
    for (name, ttl) in [("graphlaw-corpus", GL_TTL), ("castle-corpus", CASTLE_TTL)] {
        let (_, gl_nq) = graphlaw_canonical(ttl);
        let (_, castle_quads) = castle_canonical(ttl);
        let gl: Vec<String> = gl_nq
            .lines()
            .map(|l| l.trim_end_matches(" .").trim().to_string())
            .collect();
        let c: Vec<String> = castle_quads
            .iter()
            .map(|l| l.trim_end().to_string())
            .collect();
        assert_eq!(
            gl, c,
            "{name}: bnode-free canonical orderings diverged — \
             the localization receipt above is stale"
        );
    }
}

/// Layer 4 receipt: datatype lexical normalization differs (castle
/// normalizes `2.0` -> `"2"^^xsd:decimal`; graphlaw preserves `"2.0"`), and
/// blank-node `c14n{{i}}` index assignment differs. Both survive a
/// syntax-variant rewrite of the same graph.
#[test]
fn datatype_and_bnode_labeling_deltas() {
    let (_, gl_nq_a) = graphlaw_canonical(BNODE_TTL);
    let (_, castle_quads_a) = castle_canonical(BNODE_TTL);
    assert!(
        gl_nq_a.contains(r#""2.0"^^<http://www.w3.org/2001/XMLSchema#decimal>"#),
        "graphlaw stopped preserving the decimal lexical form: {gl_nq_a}"
    );
    assert!(
        castle_quads_a
            .iter()
            .any(|l| l.contains(r#""2"^^<http://www.w3.org/2001/XMLSchema#decimal>"#)),
        "castle stopped normalizing the decimal lexical form: {castle_quads_a:?}"
    );

    // Blank-node labeling: both use c14n labels but the index assignment
    // differs between engines for the same graph.
    fn bnode_labels(s: &str) -> Vec<String> {
        s.split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .filter(|t| t.starts_with("c14n"))
            .map(str::to_string)
            .collect()
    }
    let gl_labels = bnode_labels(&gl_nq_a);
    let c_labels = bnode_labels(&castle_quads_a.join("\n"));
    assert!(
        gl_labels.iter().any(|l| l.starts_with("c14n")),
        "graphlaw lost c14n labeling"
    );
    assert!(
        c_labels.iter().any(|l| l.starts_with("c14n")),
        "castle lost c14n labeling"
    );
    // The doc:cites object is c14n2 on the graphlaw side and c14n1 on the
    // castle side for this graph — the assignment schemes are not aligned.
    let cites_object = |lines: &[String]| -> String {
        lines
            .iter()
            .find(|l| l.contains("<http://example.org/doc> <http://example.org/cites>"))
            .and_then(|l| {
                l.split_whitespace()
                    .find(|t| t.starts_with("_:c14n"))
                    .map(str::to_string)
            })
            .unwrap_or_default()
    };
    let gl_cites = cites_object(&gl_nq_a.lines().map(str::to_string).collect::<Vec<_>>());
    let c_cites = cites_object(&castle_quads_a);
    assert!(
        gl_cites.starts_with("_:c14n") && c_cites.starts_with("_:c14n"),
        "cites object lost c14n labeling: graphlaw={gl_cites} castle={c_cites}"
    );
    assert_ne!(
        gl_cites, c_cites,
        "blank-node label assignment for doc:cites converged — re-run the identity probe"
    );

    // Both sides are internally deterministic under the syntax-variant
    // rewrite; the divergence is engine-level. graphlaw preserves the
    // lexical decimal, so 2.0 vs 2.00 re-keys its id; castle normalizes
    // both to "2", so its hash is invariant.
    let (gl_id_variant, _) = graphlaw_canonical(BNODE_TTL_SYNTAX_VARIANT);
    let (gl_id_orig, _) = graphlaw_canonical(BNODE_TTL);
    assert_ne!(
        gl_id_variant, gl_id_orig,
        "graphlaw identity unexpectedly INVARIANT under 2.0 -> 2.00 — \
         lexical-form preservation changed"
    );
    let (c_variant, _) = castle_canonical(BNODE_TTL_SYNTAX_VARIANT);
    let (c_orig, _) = castle_canonical(BNODE_TTL);
    assert_eq!(
        c_variant, c_orig,
        "castle state_hash drifted under 2.0 -> 2.00 — \
         lexical-form normalization changed"
    );
}
