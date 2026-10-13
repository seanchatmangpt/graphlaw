//! Differential conformance court (lane L4): strata-pn64-pack's projected ABI
//! constants vs its RDF ontology, evaluated in a third engine (graphlaw's test
//! crate over PurRDF + its SPARQL engine).
//!
//! Surfaces compared:
//!   1. `packs/strata-pn64-pack/ontology.ttl` — RDF source of truth (SPARQL over
//!      `st:RonrTransition` pre/post masks and `st:Place` bitOffset semantics).
//!   2. `packs/strata-pn64-pack/generated/pn64_masks.rs` — ggen projection,
//!      parsed here from source text so the two files are compared, not copied.
//!   3. The PN64 fire law evaluated in Rust over the same six RONR transitions,
//!      both the clear-then-set law `m' = (m & !pre) | post` and the XOR form
//!      `m' = (m ^ pre) | post` documented on `st:transitionPost`.
//!
//! Provenance: both subjects are pinned by sha256; a mismatch refuses the court
//! rather than silently judging a moved subject.
//!
//! Standing: finite six-transition corpus on the exact pinned subjects; passing
//! proves only that the ontology, the generated projection, and the bitOffset
//! composition agree for these six transitions. No general PN64/RONR claim.

use graphlaw::purrdf;
use graphlaw::rdf::{RdfDataset, SparqlEngine, SparqlRequest, SparqlResult, TermValue};
use graphlaw::sparql::NativeSparqlEngine;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const PACK_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ggen-marketplace/packs/strata-pn64-pack"
);

/// Pinned sha256 of the ontology subject (re-pinned 2026-10-13, lane L147;
/// prior pin e3b4caa8… drifted by comment-only edits — L38/L86/L103 — no
/// mask/ordinal semantics changed, verified by the court's own cross-checks).
const ONTOLOGY_SHA256: &str = "1ded0f0421c32a28f387fde2ae8a9c6acbdd433408b8adc1269741012c692c65";
/// Pinned sha256 of the generated projection subject.
const GENERATED_SHA256: &str = "0c9a641f11b792051a23e11d11ad3bebe473ca5667a30cdce3a811bd21457b32";

fn read_pack_file(rel: &str) -> String {
    std::fs::read_to_string(format!("{PACK_DIR}/{rel}"))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn assert_pinned(rel: &str, bytes: &[u8], want: &str) {
    let got = hex(&Sha256::digest(bytes));
    assert_eq!(got, want, "{rel} drifted from its pinned sha256; re-pin the court");
}

fn hex(digest: &[u8]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Extract hex-literal values between `start_marker` and the closing `];`.
fn hex_block(src: &str, start_marker: &str) -> Vec<u16> {
    let body = src
        .split(start_marker)
        .nth(1)
        .unwrap_or_else(|| panic!("generated file lacks {start_marker}"));
    let body = body.split("];").next().expect("non-empty body");
    body
        .lines()
        // Only the code part of each line, never the trailing comment
        // (comments carry opcode hex like `0x01` that is not a mask).
        .flat_map(|line| line.split("//").next().unwrap_or("").split(','))
        .filter_map(|piece| {
            let idx = piece.find("0x")?;
            let rest = &piece[idx + 2..];
            let end = rest
                .find(|c: char| !c.is_ascii_hexdigit())
                .unwrap_or(rest.len());
            Some(u16::from_str_radix(&rest[..end], 16).expect("valid hex literal"))
        })
        .collect()
}

/// Loads the ontology dataset after checking the sha256 pin.
fn ontology_dataset() -> Arc<RdfDataset> {
    let bytes = std::fs::read(format!("{PACK_DIR}/ontology.ttl")).expect("ontology.ttl readable");
    assert_pinned("ontology.ttl", &bytes, ONTOLOGY_SHA256);
    purrdf::parse_dataset(&bytes, "text/turtle", None).expect("ontology.ttl parses as Turtle")
}

fn select(ds: &Arc<RdfDataset>, query: &str) -> Vec<Vec<TermValue>> {
    match NativeSparqlEngine::new().query(
        ds,
        SparqlRequest {
            query,
            base_iri: None,
            substitutions: &[],
        },
    ) {
        Ok(SparqlResult::Solutions { rows, .. }) => rows
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|cell| cell.expect("bound solution variable"))
                    .collect()
            })
            .collect(),
        Ok(other) => panic!("expected SPARQL solutions, got {other:?}"),
        Err(e) => panic!("SPARQL failed: {e:?}"),
    }
}

fn literal(term: &TermValue) -> String {
    match term {
        TermValue::Literal { lexical_form, .. } => lexical_form.clone(),
        other => panic!("expected a literal, got {other:?}"),
    }
}

fn hex_u16(lexical: &str) -> u16 {
    let t = lexical.trim();
    u16::from_str_radix(t.strip_prefix("0x").unwrap_or(t), 16)
        .unwrap_or_else(|e| panic!("hex literal {lexical:?}: {e}"))
}

/// Ground-truth place composition per transition (L07 ronr.rs noun space),
/// used to judge bitOffset semantics: (opcode, pre labels, post labels).
const TRANSITION_SPEC: [(&str, [&str; 2], [&str; 2]); 6] = [
    ("0x01", ["N_MAIN_MOTION", "N_FLOOR"], ["N_FLOOR", "N_STATED"]),
    ("0x02", ["N_STATED", ""], ["N_SECONDED", "N_DEBATABLE"]),
    ("0x03", ["N_DEBATABLE", ""], ["N_AMENDMENT_PENDING", ""]),
    ("0x04", ["N_AMENDMENT_PENDING", ""], ["N_DEBATABLE", ""]),
    ("0x05", ["N_DEBATABLE", ""], ["N_PREV_QUESTION_MOVED", "N_PREV_QUESTION_ORDERED"]),
    ("0x06", ["N_SECONDED", "N_PREV_QUESTION_MOVED"], ["N_MAIN_MOTION", "N_ADOPTED"]),
];

#[test]
fn ontology_loads_and_declares_expected_individuals() {
    let ds = ontology_dataset();
    for (cls, want) in [("RonrTransition", 6), ("Place", 9), ("DaoVerb", 6)] {
        let rows = select(
            &ds,
            &format!(
                "PREFIX st: <https://strata.chatman.ai/ontology#>\n\
                 SELECT ?t WHERE {{ ?t a st:{cls} }}"
            ),
        );
        assert_eq!(rows.len(), want, "class st:{cls} individual count");
    }
}

/// (opcode, pre, post) triples straight out of the ontology via SPARQL.
fn ttl_transitions(ds: &Arc<RdfDataset>) -> Vec<(u16, u16, u16)> {
    let rows = select(
        ds,
        r#"
        PREFIX st: <https://strata.chatman.ai/ontology#>
        SELECT ?op ?pre ?post WHERE {
            ?t a st:RonrTransition ;
               st:transitionOpcode ?op ;
               st:transitionPre ?pre ;
               st:transitionPost ?post .
        } ORDER BY ?op"#,
    );
    assert_eq!(rows.len(), 6, "ontology must declare exactly six transitions");
    let mut out = Vec::new();
    for row in &rows {
        let op = hex_u16(&literal(&row[0]));
        assert!((1..=6).contains(&op), "opcode {op:#04x} out of canonical range");
        out.push((op, hex_u16(&literal(&row[1])), hex_u16(&literal(&row[2]))));
    }
    out
}

/// (opcode, pre, post) parsed from the generated projection source.
fn generated_transitions() -> (Vec<u16>, Vec<u16>, Vec<u16>, Vec<u16>) {
    let src = read_pack_file("generated/pn64_masks.rs");
    assert_pinned("generated/pn64_masks.rs", src.as_bytes(), GENERATED_SHA256);
    let pre = hex_block(&src, "pub const PRE_MASKS");
    let post = hex_block(&src, "pub const POST_MASKS");
    let terminals = hex_block(&src, "pub const TERMINALS");
    let pairs = hex_block(&src, "pub const TRANSITIONS");
    assert_eq!(pre.len(), 6, "PRE_MASKS projection");
    assert_eq!(post.len(), 6, "POST_MASKS projection");
    assert_eq!(terminals.len(), 2, "TERMINALS projection");
    assert_eq!(pairs.len(), 12, "TRANSITIONS projection");
    (pre, post, terminals, pairs)
}

#[test]
fn ontology_pre_post_masks_equal_generated_projection() {
    let ds = ontology_dataset();
    let ttl = ttl_transitions(&ds);
    let (pre, post, _terminals, pairs) = generated_transitions();
    for (i, (op, tpre, tpost)) in ttl.iter().enumerate() {
        assert_eq!(*op as usize, i + 1, "transitions must be the canonical six in order");
        assert_eq!(tpre, &pre[i], "PRE mask disagreement at opcode {op:#04x}");
        assert_eq!(tpost, &post[i], "POST mask disagreement at opcode {op:#04x}");
        assert_eq!(
            (*tpre, *tpost),
            (pairs[2 * i], pairs[2 * i + 1]),
            "TRANSITIONS mirror drift at opcode {op:#04x}"
        );
    }
}

#[test]
fn bitoffset_semantics_reconstruct_masks() {
    let ds = ontology_dataset();
    let mut bits: Vec<(String, u16)> = Vec::new();
    for row in select(
        &ds,
        r#"
        PREFIX st: <https://strata.chatman.ai/ontology#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        SELECT ?label ?off WHERE {
            ?p a st:Place ; rdfs:label ?label ; st:bitOffset ?off .
        }"#,
    ) {
        let off: i64 = literal(&row[1]).parse().expect("bitOffset integer");
        assert!((0..16).contains(&off), "bitOffset {off} outside the 16-bit word");
        bits.push((literal(&row[0]), 1u16 << off));
    }
    assert_eq!(bits.len(), 9, "nine RONR places");
    let bit_of = |label: &str| -> u16 {
        bits.iter()
            .find(|(l, _)| l == label)
            .map(|(_, b)| *b)
            .unwrap_or_else(|| panic!("place {label} absent from ontology"))
    };
    // Offsets 0..8 are distinct powers of two, and the ADOPTED terminal equals
    // MAIN_MOTION | ADOPTED as the generated TERMINALS constant claims.
    let mut offs: Vec<u16> = bits.iter().map(|(_, b)| *b).collect();
    offs.sort();
    offs.dedup();
    assert_eq!(offs.len(), 9, "place bits are distinct");
    assert_eq!(bit_of("N_MAIN_MOTION") | bit_of("N_ADOPTED"), 0x0101);

    let (pre, post, _terminals, _pairs) = generated_transitions();
    let ttl = ttl_transitions(&ds);
    for (i, spec) in TRANSITION_SPEC.iter().enumerate() {
        let (op, tpre, tpost) = &ttl[i];
        assert_eq!(format!("{op:#04x}"), spec.0, "spec/TTL opcode alignment at index {i}");
        let want_pre = if spec.1[1].is_empty() {
            bit_of(spec.1[0])
        } else {
            bit_of(spec.1[0]) | bit_of(spec.1[1])
        };
        let want_post = if spec.2[1].is_empty() {
            bit_of(spec.2[0])
        } else {
            bit_of(spec.2[0]) | bit_of(spec.2[1])
        };
        assert_eq!(*tpre, want_pre, "bitOffset composition, pre, opcode {op:#04x}");
        assert_eq!(*tpost, want_post, "bitOffset composition, post, opcode {op:#04x}");
        assert_eq!(pre[i], want_pre, "generated pre matches bitOffset law, opcode {op:#04x}");
        assert_eq!(post[i], want_post, "generated post matches bitOffset law, opcode {op:#04x}");
    }
}

#[test]
fn fire_law_over_six_transitions_both_forms_agree_on_enabled_domain() {
    let ds = ontology_dataset();
    let ttl = ttl_transitions(&ds);
    let (pre, post, _terminals, _pairs) = generated_transitions();

    // The ontology masks must first agree with the projection (court above);
    // here evaluate the fire law over the projection tables in Rust.
    for i in 0..6 {
        assert_eq!((pre[i], post[i]), (ttl[i].1, ttl[i].2));
        let (pre_i, post_i) = (pre[i], post[i]);
        let mut enabled_count = 0u32;
        for m in 0..=u16::MAX {
            if (m & pre_i) == pre_i {
                enabled_count += 1;
                assert_eq!(
                    (m & !pre_i) | post_i,
                    ((m ^ pre_i) | post_i) & 0xFFFF,
                    "fire-law divergence on the enabled domain, opcode {:#04x}, m={m:#06x}",
                    i + 1
                );
            }
        }
        assert!(enabled_count > 0, "transition {:#04x} never enabled", i + 1);
    }

    // Canonical firing sequence from quorate start MAIN|STATED|FLOOR (0x0083),
    // clear-then-set law, every step must be enabled.
    let mut m: u16 = 0x0083;
    for i in 0..6 {
        let (pre_i, post_i) = (pre[i], post[i]);
        assert_eq!(
            m & pre_i,
            pre_i,
            "canonical sequence not enabled at opcode {:#04x} (m={m:#06x})",
            i + 1
        );
        m = (m & !pre_i) | post_i;
    }
    assert_eq!(m, 0x01C1, "final canonical marking after t01..t06");
    assert_ne!(m & 0x0100, 0, "final marking retains N_ADOPTED (bitOffset 8)");
}

#[test]
fn dao_verb_opcodes_bind_to_canonical_transitions() {
    let ds = ontology_dataset();
    let mut verbs: Vec<(String, u16)> = Vec::new();
    for row in select(
        &ds,
        r#"
        PREFIX st: <https://strata.chatman.ai/ontology#>
        SELECT ?value ?op WHERE {
            ?v a st:DaoVerb ; st:value ?value ; st:verbOpcode ?op .
        } ORDER BY ?op"#,
    ) {
        let value = literal(&row[0]);
        let op = hex_u16(&literal(&row[1]));
        assert!((1..=6).contains(&op), "DaoVerb opcode {op:#04x} out of range");
        verbs.push((value, op));
    }
    assert_eq!(verbs.len(), 6, "six DaoVerbs, one per canonical opcode");
    let mut ops: Vec<u16> = verbs.iter().map(|(_, op)| *op).collect();
    ops.sort();
    ops.dedup();
    assert_eq!(ops.len(), 6, "one DaoVerb per opcode, no duplicates");
}

#[test]
fn divergence_witness_off_enabled_domain() {
    // Pins the documented (generated/pn64_masks.rs doc comment) divergence:
    // off the enabled domain, the XOR fire law sets absent pre-bits while
    // clear-then-set leaves them unset. m=0x0000 does not enable pre=0x0002.
    let pre: u16 = 0x0002; // N_STATED
    let post: u16 = 0x000C; // N_SECONDED | N_DEBATABLE
    assert_eq!((0x0000u16 & !pre) | post, 0x000C);
    assert_eq!(((0x0000u16 ^ pre) | post) & 0xFFFF, 0x000E);
    assert_ne!((0x0000u16 & !pre) | post, ((0x0000u16 ^ pre) | post) & 0xFFFF);
    // On the enabled domain (m=0x0002 enables pre=0x0002) they agree.
    assert_eq!((0x0002u16 & !pre) | post, ((0x0002u16 ^ pre) | post) & 0xFFFF);
}

// ---------------------------------------------------------------------------
// Fire-law equivalence property court (L-wave, PN64 fire law)
//
// Lean witness cross-anchor: `ProcInt.PN64.fire_law_equivalence_on_enabled`
// and `ProcInt.PN64.fire_law_diverges_off_domain`, machine-checked in
// /Users/sac/mfact/procint/ProcInt/Petri/PN64Conservation.lean
// (sha256 3601e1bb7e6ff6e89e2ac60eb8b8d57cd72b2a4bd94d2f5b9773f778d7ac0e88,
// mfact HEAD 92b08da5c8, toolchain leanprover/lean4:v4.31.0), receipted in
// /Users/sac/mfact/witnesses.jsonl (lean-witness/v1 schema, axioms
// propext/Quot.sound). The Rust property tests below mirror those theorems
// point-for-point over the same mask tables this court already pins:
//
//   Rust `fire_xor`  <-> Lean `Fire pre post m = (m ^^^ pre) ||| post`
//   Rust `fire_cts`  <-> Lean `ClearThenSet = (m &&& pre ^^^ m) ||| post`
//   Rust `kernel_fire` (gated) <-> Lean `Enabled pre m` hypothesis
//
// This is Rust-side corroboration of the Lean witness, not a re-proof: the
// finite sweep samples the same law over the pinned six-transition corpus.
// ---------------------------------------------------------------------------

/// XOR fire law (Lean `Fire`): `m' = (m ^ pre) | post`.
fn fire_xor(pre: u16, post: u16, m: u16) -> u16 {
    (m ^ pre) | post
}

/// Clear-then-set fire law: `m' = (m & !pre) | post`
/// (Lean `ClearThenSet`, expressed there as `m ^ (m & pre) | post`).
fn fire_cts(pre: u16, post: u16, m: u16) -> u16 {
    (m & !pre) | post
}

/// The kernel's gated fire: enabled iff `(m & pre) == pre`; an off-domain
/// request is refused and leaves the marking unchanged (Lean's `Enabled`
/// hypothesis, enforced rather than assumed).
fn kernel_fire(pre: u16, post: u16, m: u16) -> Option<u16> {
    if m & pre == pre {
        Some(fire_cts(pre, post, m))
    } else {
        None
    }
}

/// Deterministic LCG so the sampled mask corpus is reproducible.
fn lcg(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *state >> 16
}

#[test]
fn fire_law_equivalence_512_marking_sweep_sampled_masks() {
    let (_pre, _post, _terminals, _pairs) = generated_transitions();
    let ttl = ttl_transitions(&ontology_dataset());
    assert_eq!(ttl.len(), 6);

    // Sampled pre/post mask corpus: the six pinned transitions plus 42
    // deterministic pseudo-random pairs spanning the 16-bit word.
    let mut cases: Vec<(u16, u16)> = ttl.iter().map(|&(_, p, q)| (p, q)).collect();
    let mut rng: u64 = 0x504E_3634_2026_1013; // "PN64 2026-10-13"
    for _ in 0..42 {
        cases.push((lcg(&mut rng) as u16, (lcg(&mut rng) >> 16) as u16));
    }

    for (pre_i, post_i) in &cases {
        // 512-marking sweep over the nine-place RONR bit space (bits 0..8).
        for m in 0u16..512 {
            let enabled = m & pre_i == *pre_i;
            if enabled {
                // All three agree on the enabled domain.
                assert_eq!(
                    fire_cts(*pre_i, *post_i, m),
                    fire_xor(*pre_i, *post_i, m),
                    "cts != xor on enabled domain, pre={pre_i:#06x} post={post_i:#06x} m={m:#06x}"
                );
                assert_eq!(
                    kernel_fire(*pre_i, *post_i, m),
                    Some(fire_xor(*pre_i, *post_i, m)),
                    "kernel gated fire disagrees, pre={pre_i:#06x} post={post_i:#06x} m={m:#06x}"
                );
            } else {
                // Off-domain: gated fire refuses; marking unchanged.
                assert_eq!(
                    kernel_fire(*pre_i, *post_i, m),
                    None,
                    "kernel fired while disabled, pre={pre_i:#06x} post={post_i:#06x} m={m:#06x}"
                );
            }
        }
    }
    assert_eq!(cases.len(), 48, "6 pinned + 42 sampled mask pairs");
}

#[test]
fn off_domain_divergence_witness_m0_pre1_post0() {
    // Mirrors Lean `fire_law_diverges_off_domain` exactly: at m=0, pre=1,
    // post=0 the transition is NOT enabled, the raw xor form fabricates a
    // token in the pre place (m'=1), while the gated kernel fire refuses and
    // the clear-then-set form would leave the marking unchanged.
    let (pre, post, m) = (1u16, 0u16, 0u16);
    assert_ne!(m & pre, pre, "witness case must be off the enabled domain");
    assert_eq!(fire_xor(pre, post, m), 1, "xor form fabricates a token");
    assert_eq!(fire_cts(pre, post, m), 0, "clear-then-set leaves m unchanged");
    assert_ne!(fire_xor(pre, post, m), fire_cts(pre, post, m));
    assert_eq!(kernel_fire(pre, post, m), None, "gated kernel fire refuses");
    // The masked bit-level xnor-free statement: Fire 1 0 0 = 1 ∧
    // ClearThenSet 1 0 0 = 0, exactly the Lean conjunction's last three parts.
}

#[test]
fn sabotage_twin_flipped_equivalence_fails() {
    // Sabotage twin: a deliberately mutated equivalence comparator (one bit
    // flipped in the checked value) must be caught by the same property the
    // honest comparator passes. If this test ever fails, the property above
    // is vacuous — it no longer distinguishes a broken fire law.
    let ttl = ttl_transitions(&ontology_dataset());
    let mut cases: Vec<(u16, u16)> = ttl.iter().map(|&(_, p, q)| (p, q)).collect();
    let mut rng: u64 = 0x504E_3634_2026_1013;
    for _ in 0..8 {
        cases.push((lcg(&mut rng) as u16, (lcg(&mut rng) >> 16) as u16));
    }

    // Sabotaged comparator: xor law with one bit of the result flipped.
    let sabotaged = |pre: u16, post: u16, m: u16| fire_xor(pre, post, m) ^ 0x0001;

    let mut caught = 0usize;
    for (pre_i, post_i) in &cases {
        for m in 0u16..512 {
            if m & pre_i == *pre_i {
                assert_ne!(
                    fire_cts(*pre_i, *post_i, m),
                    sabotaged(*pre_i, *post_i, m),
                    "sabotaged comparator escaped detection \
                     (pre={pre_i:#06x} post={post_i:#06x} m={m:#06x})"
                );
                caught += 1;
            }
        }
    }
    assert!(caught > 0, "sabotage court never executed the comparator");
}
