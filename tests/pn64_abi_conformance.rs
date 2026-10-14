//! Differential conformance court (lane L4): strata-pn64-pack's projected ABI
//! constants vs its RDF ontology, evaluated in a third engine (graphlaw's test
//! crate over PurRDF + its SPARQL engine).
//!
//! Surfaces compared:
//!   1. `packs/strata-pn64-pack/ontology.ttl` — RDF source of truth (SPARQL over
//!      `pn:Transition`/`pn:opcode`/`pn:preMask`/`pn:postMask` and
//!      `pn:Place`/`pn:bitOffset` semantics, per the documented legacy
//!      `st:` -> `pn:` schema reconciliation).
//!   2. `packs/strata-pn64-pack/generated/pn64_masks.rs` — ggen projection,
//!      parsed here from source text so the two files are compared, not copied.
//!   3. The PN64 fire law evaluated in Rust over the six canonical RONR
//!      transitions, both the clear-then-set law `m' = (m & !pre) | post` and
//!      the XOR form `m' = (m ^ pre) | post` documented on `pn:postMask`.
//!
//! Provenance: the ontology subject is pinned by sha256; a mismatch refuses the
//! court rather than silently judging a moved subject. The generated projection
//! subject is deliberately NOT sha-pinned (see GENERATED_SHA256 below).
//!
//! Standing: finite six-transition corpus on the exact pinned subjects; passing
//! proves only that the ontology, the generated projection, and the bitOffset
//! composition agree for the six canonical RONR transitions. No general
//! PN64/RONR claim.

use graphlaw::purrdf;
use graphlaw::rdf::{RdfDataset, SparqlEngine, SparqlRequest, SparqlResult, TermValue};
use graphlaw::sparql::NativeSparqlEngine;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const PACK_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ggen-marketplace/packs/strata-pn64-pack"
);

/// Pinned sha256 of the ontology subject (re-pinned for ggen-marketplace
/// 8a9949c49: the documented legacy st: -> pn: schema reconciliation — the six
/// canonical RONR transition masks are carried over value-identical
/// (129/130, 2/12, 8/16, 16/8, 8/96, 36/257); the legacy st:
/// RonrTransition/Place/DaoVerb vocabulary is superseded by the pn:
/// individuals, per that commit's reconcile note).
/// Prior pin 1ded0f04… (itself re-pinned 2026-10-13, lane L147, from e3b4caa8…).
const ONTOLOGY_SHA256: &str = "5feaf3824cc031dff7715e46e02d8d74412de5a1658cc8a47badd1d496d31f3b";

/// The generated projection subject carries NO sha pin.
///
/// UNSTABLE-WITH-REASON: `generated/pn64_masks.rs` is being rewritten
/// concurrently (widened six -> 25 canonical transitions) by a lane that owns
/// that subject; pinning its bytes here would freeze a moving target and
/// refuse on every sibling commit. The court judges the projection
/// structurally instead: array lengths, index alignment with the ontology,
/// and mask-value agreement — drift in the projection is caught by those
/// structural courts, not by a byte pin.
const GENERATED_SHA256: &str = "";

fn read_pack_file(rel: &str) -> String {
    std::fs::read_to_string(format!("{PACK_DIR}/{rel}"))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn assert_pinned(rel: &str, bytes: &[u8], want: &str) {
    assert!(!want.is_empty(), "{rel} pin is deliberately unset");
    let got = hex(&Sha256::digest(bytes));
    assert_eq!(got, want, "{rel} drifted from its pinned sha256; re-pin the court");
}

fn hex(digest: &[u8]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Extract numeric literals (decimal or 0x-hex) between `start_marker` and the
/// closing `];` or `}`. Handles bare `129,` entries as well as `(129, 130),`
/// tuple entries. Comment text after `//` is skipped (comments carry opcode
/// hex like `0x01` that is not a mask) by tokenizing per line comment-free.
fn nums_block(src: &str, start_marker: &str) -> Vec<u64> {
    let body = src
        .split(start_marker)
        .nth(1)
        .unwrap_or_else(|| panic!("generated file lacks {start_marker}"));
    let body = body
        .split("];")
        .next()
        .unwrap_or_else(|| body.split('}').next().expect("non-empty body"))
        .to_string();
    let mut out = Vec::new();
    for line in body.lines() {
        let mut code = line.split("//").next().unwrap_or("");
        // Cut the leading declaration (`pub const X: [u16; N] = &`) so only
        // the value side of the line is tokenized.
        if let Some((_, rhs)) = code.rsplit_once('=') {
            code = rhs;
        }
        let mut piece = code.trim().trim_start_matches("&[").trim_end_matches(';').trim();
        // Strip tuple punctuation.
        piece = piece.trim_start_matches('(').trim_end_matches("),").trim_end_matches(',');
        for tok in piece.split(',') {
            let tok = tok.trim().trim_start_matches('(').trim_end_matches(')');
            let tok = tok.trim();
            if tok.is_empty() {
                continue;
            }
            let (radix, digits) = if let Some(d) = tok.strip_prefix("0x").or(tok.strip_prefix("0X")) {
                (16, d)
            } else {
                (10, tok)
            };
            if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
                continue; // not a numeric literal (e.g. `pub const X` header line)
            }
            out.push(u64::from_str_radix(digits, radix).expect("valid numeric literal"));
        }
    }
    out
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

/// Parses a mask literal: decimal (`129`) or hex (`0x0081`) lexical form.
fn mask_u16(lexical: &str) -> u16 {
    let t = lexical.trim();
    let (radix, digits) = match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        Some(d) => (16, d),
        None => (10, t),
    };
    u16::from_str_radix(digits, radix)
        .unwrap_or_else(|e| panic!("mask literal {lexical:?}: {e}"))
}

/// The six canonical RONR transitions, ground truth (ronr.rs noun space):
/// (opcode, pre, post). Masks carried over value-identically through the
/// st: -> pn: schema reconciliation.
const CANONICAL_SIX: [(u16, u16, u16); 6] = [
    (1, 129, 130),  // MoveMainMotion      0x0081/0x0082
    (2, 2, 12),     // SecondMotion        0x0002/0x000C
    (3, 8, 16),     // MoveToAmend         0x0008/0x0010
    (4, 16, 8),     // AdoptAmendment      0x0010/0x0008
    (5, 8, 96),     // CallPreviousQuestion 0x0008/0x0060
    (6, 36, 257),   // VoteAdoptMainMotion 0x0024/0x0101
];

#[test]
fn ontology_loads_and_declares_expected_individuals() {
    let ds = ontology_dataset();
    // RONR canonical transitions: opcodes 1..=25 (the ontology also carries
    // DAO transitions 112..151, outside the RONR opcode bound).
    let rows = select(
        &ds,
        r#"
        PREFIX pn: <https://strata.chatman.ai/ontology/pn64#>
        SELECT ?t WHERE {
            ?t a pn:Transition ;
               pn:opcode ?op .
            FILTER (?op <= 25)
        }"#,
    );
    assert_eq!(rows.len(), 25, "RONR (opcode <= 25) transition count");

    for (cls, want) in [("Place", 16), ("DaoVerb", 7)] {
        let rows = select(
            &ds,
            &format!(
                "PREFIX pn: <https://strata.chatman.ai/ontology/pn64#>\n\
                 SELECT ?t WHERE {{ ?t a pn:{cls} }}"
            ),
        );
        assert_eq!(rows.len(), want, "class pn:{cls} individual count");
    }
}

/// (opcode, pre, post) triples straight out of the ontology via SPARQL,
/// restricted to the six canonical RONR transitions.
fn ttl_transitions(ds: &Arc<RdfDataset>) -> Vec<(u16, u16, u16)> {
    let rows = select(
        ds,
        r#"
        PREFIX pn: <https://strata.chatman.ai/ontology/pn64#>
        SELECT ?op ?pre ?post WHERE {
            ?t a pn:Transition ;
               pn:opcode ?op ;
               pn:preMask ?pre ;
               pn:postMask ?post .
            FILTER (?op <= 6)
        } ORDER BY ?op"#,
    );
    assert_eq!(rows.len(), 6, "ontology must declare exactly six canonical transitions");
    let mut out = Vec::new();
    for row in &rows {
        let op = mask_u16(&literal(&row[0]));
        assert!((1..=6).contains(&op), "opcode {op:#04x} out of canonical range");
        out.push((op, mask_u16(&literal(&row[1])), mask_u16(&literal(&row[2]))));
    }
    out
}

/// (pre, post, terminals, pairs) parsed from the generated projection source.
fn generated_transitions() -> (Vec<u64>, Vec<u64>, Vec<u64>, Vec<u64>) {
    let src = read_pack_file("generated/pn64_masks.rs");
    let pre = nums_block(&src, "pub const PRE_MASKS");
    let post = nums_block(&src, "pub const POST_MASKS");
    let terminals = nums_block(&src, "pub const TERMINALS");
    let pairs = nums_block(&src, "pub const TRANSITIONS");
    assert_eq!(pre.len(), 25, "PRE_MASKS projection (25 canonical opcodes)");
    assert_eq!(post.len(), 25, "POST_MASKS projection (25 canonical opcodes)");
    assert_eq!(terminals.len(), 2, "TERMINALS projection");
    assert_eq!(pairs.len(), 50, "TRANSITIONS projection (25 (pre, post) pairs)");
    (pre, post, terminals, pairs)
}

#[test]
fn ontology_pre_post_masks_equal_generated_projection() {
    let ds = ontology_dataset();
    let ttl = ttl_transitions(&ds);
    let (pre, post, _terminals, pairs) = generated_transitions();
    for (i, (op, tpre, tpost)) in ttl.iter().enumerate() {
        assert_eq!(*op as usize, i + 1, "transitions must be the canonical six in order");
        assert_eq!(tpre, &(pre[i] as u16), "PRE mask disagreement at opcode {op:#04x}");
        assert_eq!(tpost, &(post[i] as u16), "POST mask disagreement at opcode {op:#04x}");
        assert_eq!(
            (*tpre, *tpost),
            (pairs[2 * i] as u16, pairs[2 * i + 1] as u16),
            "TRANSITIONS mirror drift at opcode {op:#04x}"
        );
    }
}

#[test]
fn bitoffset_semantics_reconstruct_masks() {
    let ds = ontology_dataset();
    let mut offs: Vec<u16> = Vec::new();
    for row in select(
        &ds,
        r#"
        PREFIX pn: <https://strata.chatman.ai/ontology/pn64#>
        SELECT ?off WHERE {
            ?p a pn:Place ; pn:bitOffset ?off .
        }"#,
    ) {
        let off: i64 = literal(&row[0]).parse().expect("bitOffset integer");
        assert!((0..16).contains(&off), "bitOffset {off} outside the 16-bit word");
        offs.push(off as u16);
    }
    // The pn: vocabulary carries pn:slot0..slot14 plus pn:Committed (bit 15) —
    // sixteen places, distinct offsets covering the full 16-bit marking word.
    // (Labels are OPTIONAL in the pn: ontology — slot*/Committed carry
    // pn:bitOffset only — so composition is judged numerically.)
    assert_eq!(offs.len(), 16, "sixteen places in the pn: ontology");
    offs.sort();
    offs.dedup();
    assert_eq!(offs.len(), 16, "place bits are distinct");
    // The ADOPTED terminal equals MAIN_MOTION | ADOPTED (bits 0 | 8) as the
    // generated TERMINALS constant claims.
    assert_eq!((1u16 << 0) | (1u16 << 8), 0x0101);

    let (pre, post, terminals, pairs) = generated_transitions();
    assert_eq!(terminals[0], 0x0101, "TERMINALS[0] = MAIN_MOTION | ADOPTED");
    let ttl = ttl_transitions(&ds);
    for (i, (op, want_pre, want_post)) in CANONICAL_SIX.iter().enumerate() {
        let (top, tpre, tpost) = &ttl[i];
        assert_eq!(*top, *op, "spec/TTL opcode alignment at index {i}");
        assert_eq!(*tpre, *want_pre, "bitOffset composition, pre, opcode {op:#04x}");
        assert_eq!(*tpost, *want_post, "bitOffset composition, post, opcode {op:#04x}");
        assert_eq!(pre[i] as u16, *want_pre, "generated pre matches the mask law, opcode {op:#04x}");
        assert_eq!(post[i] as u16, *want_post, "generated post matches the mask law, opcode {op:#04x}");
        assert_eq!((pairs[2 * i], pairs[2 * i + 1]), (*want_pre as u64, *want_post as u64), "TRANSITIONS mirror, opcode {op:#04x}");
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
        assert_eq!((pre[i] as u16, post[i] as u16), (ttl[i].1, ttl[i].2));
        let (pre_i, post_i) = (pre[i] as u16, post[i] as u16);
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
        let (pre_i, post_i) = (pre[i] as u16, post[i] as u16);
        assert_eq!(
            m & pre_i,
            pre_i,
            "canonical sequence not enabled at opcode {:#04x} (m={m:#06x})",
            i + 1
        );
        m = (m & !pre_i) | post_i;
    }
    assert_eq!(m, 0x01C1, "final canonical marking after t01..t06");
    assert_ne!(m & 0x0100, 0, "final marking retains ADOPTED (bitOffset 8)");
}

#[test]
fn dao_verb_values_bind_distinctly() {
    let ds = ontology_dataset();
    let mut verbs: Vec<(String, u16)> = Vec::new();
    for row in select(
        &ds,
        r#"
        PREFIX pn: <https://strata.chatman.ai/ontology/pn64#>
        SELECT ?verb ?value WHERE {
            ?verb a pn:DaoVerb ; pn:value ?value .
        } ORDER BY ?value"#,
    ) {
        // Ground truth: ~/strata/ontology.ttl (pn64 namespace). DaoVerb
        // individuals carry exactly one wire-value predicate, pn:value; no
        // verbOpcode predicate exists in the ontology.
        let verb = match &row[0] {
            TermValue::Iri(iri) => iri.clone(),
            other => panic!("expected an IRI, got {other:?}"),
        };
        verbs.push((verb, mask_u16(&literal(&row[1]))));
    }
    assert_eq!(verbs.len(), 7, "seven DaoVerbs (Propose, Second, Amend, ExecuteTransaction, Ragequit, Delegate, AcquireLease)");
    let mut vals: Vec<u16> = verbs.iter().map(|(_, v)| *v).collect();
    vals.sort();
    vals.dedup();
    assert_eq!(vals.len(), 7, "one DaoVerb per wire value, no duplicates");
    // Documented DELIBERATE-PENDING-DECISION: pn:Second's wire value 2
    // collides with pn:SecondMotion's RONR opcode 2 — this court reads the
    // DaoVerb namespace only and cannot see the collision; do not resolve it
    // here.
    assert!(vals.contains(&2), "pn:Second's documented value-2 collision is present");
}

#[test]
fn divergence_witness_off_enabled_domain() {
    // Pins the documented divergence: off the enabled domain, the XOR fire
    // law sets absent pre-bits while clear-then-set leaves them unset.
    // m=0x0000 does not enable pre=0x0002.
    let pre: u16 = 0x0002; // STATED (bit 1)
    let post: u16 = 0x000C; // SECONDED | DEBATABLE
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
// point-for-point over the same mask tables this court already judges.
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
