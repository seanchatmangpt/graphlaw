//! Lane P16-transition — LawState transition isolation court (WP P16).
//!
//! Every leg proves the same law from a different refusal class: **a refused
//! transition leaves the parent state untouched and usable** — no child state,
//! no residue, byte-identical parent — and successful derivations are exact,
//! deterministic, and replayable. Real parsers, real reasoner, real canonical
//! ids; no mocks, no sleeps.
//!
//! ## Probe receipt (leg 3, run before pinning — three rounds, real output)
//!
//! The mission conditioned leg 3 on a probe: does the reasoner emit the parent
//! bound blank verbatim in derived output? Observed through the exact law-rung
//! flow (`reason_n3_bounded` over `base_nquads + "\n" + rules`, then purrdf
//! turtle -> nquads round trip):
//!
//! - Round 1 (rule-head blank): `{ ?x :s ?y . ?y :stop :yes } => { ?x :u _:h }`
//!   derived `:a :u _:h_11b7a63738bedd58` — rule-head fresh blanks are
//!   re-labeled `<written>_<16 hex>`.
//! - Round 2 (cross-run): identical input re-derived byte-identically
//!   (`_:h_11b7a63738bedd58` twice) — the re-label suffix is deterministic.
//! - Round 3 (premise-bound parent blank): parent `_:h :s :o .` with the echo
//!   rule emitted derived `_:h :u _:h_be35ba410f9c0a62 .` — the premise-bound
//!   **parent blank survives VERBATIM**; the turtle -> nquads round trip keeps
//!   the parsed label. Parent-side serialization writes parsed labels verbatim
//!   too (`_:h` in, `_:h` out).
//!
//! Verdict: the two label spaces CAN intersect (parent `_:h` vs derived
//! `_:h`), so the blank-collision refusal (`Unsupported`: "blank-node labels
//! collide between the state and derived triples") is **reachable** and is
//! pinned end-to-end in `leg3`. No known-limitation receipt is needed; the
//! adjacent contract (re-labeled head blanks keep ordinary states safe) is
//! pinned in the same test. The only honest boundary in this court is leg 7's:
//! receipts reconstruct the id chain, not the content bytes.
//!
//! Refusal classes referenced: `TierMismatch`, `Unsupported` (default-graph
//! and blank-collision rows), `ResourceLimit`, `UNKNOWN-STEP-CAPABILITY` —
//! stable codes in `docs/refusals.md`.

use graphlaw::dialect::{Dialect, Refusal, RefusalKind};
use graphlaw::law::{LawError, LawState, N3_MAX_ITERATIONS, Receipt, Step, reason_n3_bounded};
use graphlaw::tier::validate_n3_rules;

// ---------------------------------------------------------------------------
// Shared fixtures
// ---------------------------------------------------------------------------

/// Self-contained N3 rules deriving the inverse of every `:s` fact.
const GOOD_RULES: &str = "@prefix : <urn:n:> .\n{ ?x :s ?y } => { ?y :t ?x } .\n";

fn good_ruleset() -> graphlaw::tier::RuleSet {
    validate_n3_rules(GOOD_RULES.as_bytes(), None).expect("good rules validate")
}

/// Serialized N-Quads of a state (same flow as the law kernel's `nquads()`).
fn nquads_of(state: &LawState) -> String {
    let bytes = purrdf::serialize_dataset(
        state.dataset().as_ref(),
        "application/n-quads",
        purrdf::SerializeGraph::Dataset,
    )
    .expect("dataset serializes to n-quads");
    String::from_utf8(bytes).expect("n-quads is utf8")
}

/// Parent identity snapshot: canonical id, serialized bytes, quad count.
struct Snapshot {
    id: String,
    nquads: String,
    quads: usize,
}

fn snapshot(state: &LawState) -> Snapshot {
    Snapshot {
        id: state.id().to_string(),
        nquads: nquads_of(state),
        quads: state.quad_count(),
    }
}

/// The isolation invariant: refused transitions change NOTHING on the parent.
fn assert_untouched(parent: &LawState, before: &Snapshot, context: &str) {
    assert_eq!(parent.id(), before.id, "{context}: id changed");
    assert_eq!(parent.quad_count(), before.quads, "{context}: quad count changed");
    assert_eq!(
        nquads_of(parent),
        before.nquads,
        "{context}: serialized N-Quads not byte-identical"
    );
}

fn blank_labels(nquads: &str) -> std::collections::BTreeSet<String> {
    nquads
        .split_whitespace()
        .filter(|t| t.starts_with("_:"))
        .map(str::to_string)
        .collect()
}

/// A re-labeled derived blank: `<written>_<16 lowercase hex>`.
fn assert_relabeled(label: &str) {
    let Some((written, suffix)) = label.rsplit_once('_') else {
        panic!("probe: blank `{label}` carries no re-label suffix");
    };
    assert!(
        !written.is_empty() && suffix.len() == 16,
        "probe: `{label}` must be `<written>_<16 hex>`, got written=`{written}` suffix=`{suffix}`"
    );
    assert!(
        suffix.bytes().all(|b| b.is_ascii_hexdigit()),
        "probe: re-label suffix must be hex: {label}"
    );
}

fn expect_refused(outcome: Result<(LawState, Receipt), LawError>, context: &str) -> Refusal {
    match outcome {
        Err(LawError::Refused(r)) => r,
        other => panic!("{context}: expected typed refusal, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Leg 1: gate refusals leave the parent byte-identical
// ---------------------------------------------------------------------------

/// Leg 1a: rules text that fails N3 admission — TierMismatch child refusal.
#[test]
fn leg1a_parse_fail_rules_refuse_tier_mismatch_parent_byte_identical() {
    let parent = LawState::parse(
        b"<urn:n:a> <urn:n:s> <urn:n:b> .\n",
        Dialect::NTriples,
        None,
    )
    .expect("parent parses");
    let before = snapshot(&parent);
    // No @prefix declaration: validate_n3_rules refuses, transition maps it to
    // TierMismatch before any derivation runs.
    let bad_rules = "{ ?x :s ?y } => { ?y :t ?x } .\n";
    let refusal = expect_refused(
        parent.transition(&Step::DeriveN3 {
            rules: bad_rules,
            ruleset: &good_ruleset(),
        }),
        "leg1a",
    );
    assert_eq!(refusal.kind, RefusalKind::TierMismatch, "{refusal:?}");
    assert_eq!(refusal.dialect, Some(Dialect::N3));
    assert!(
        refusal
            .message
            .as_str()
            .contains("DeriveN3 rules failed N3 admission"),
        "leg1a: message must name the admission failure: {}",
        refusal.message.as_str()
    );
    assert_untouched(&parent, &before, "leg1a");
}

/// Leg 1b: carried RuleSet disagreeing with the rules text — TierMismatch.
#[test]
fn leg1b_ruleset_count_disagreement_refuses_parent_byte_identical() {
    let parent = LawState::parse(
        b"<urn:n:a> <urn:n:s> <urn:n:b> .\n",
        Dialect::NTriples,
        None,
    )
    .expect("parent parses");
    let before = snapshot(&parent);
    // Rules text carries 1 rule; the carried (validated) ruleset carries 0.
    let empty_ruleset = validate_n3_rules(b"", None).expect("empty rules validate");
    assert_eq!(empty_ruleset.rule_count(), 0);
    let refusal = expect_refused(
        parent.transition(&Step::DeriveN3 {
            rules: GOOD_RULES,
            ruleset: &empty_ruleset,
        }),
        "leg1b",
    );
    assert_eq!(refusal.kind, RefusalKind::TierMismatch, "{refusal:?}");
    let msg = refusal.message.as_str();
    assert!(
        msg.contains("does not match the rules text") && msg.contains('0') && msg.contains('1'),
        "leg1b: message must name both counts: {msg}"
    );
    assert_untouched(&parent, &before, "leg1b");
}

/// Leg 1c: named-graph state — Unsupported (default-graph-only) refusal.
#[test]
fn leg1c_named_graph_state_refuses_unsupported_parent_byte_identical() {
    let parent = LawState::parse(
        b"<urn:n:s> <urn:n:p> <urn:n:o> <urn:n:g> .\n",
        Dialect::NQuads,
        None,
    )
    .expect("named-graph parent parses");
    assert_eq!(parent.quad_count(), 1);
    let before = snapshot(&parent);
    let refusal = expect_refused(
        parent.transition(&Step::DeriveN3 {
            rules: GOOD_RULES,
            ruleset: &good_ruleset(),
        }),
        "leg1c",
    );
    assert_eq!(refusal.kind, RefusalKind::Unsupported, "{refusal:?}");
    assert_eq!(refusal.dialect, Some(Dialect::N3));
    assert!(
        refusal
            .message
            .as_str()
            .contains("N3 derivation needs a default-graph-only state"),
        "leg1c: message must name the default-graph-only constraint: {}",
        refusal.message.as_str()
    );
    assert_untouched(&parent, &before, "leg1c");
}

// ---------------------------------------------------------------------------
// Leg 2: budget-limit child isolated (limit fixture at the law rung)
// ---------------------------------------------------------------------------

/// Leg 2: a derivation that exceeds `N3_MAX_ITERATIONS` at the law rung (the
/// resource_limits.rs fixture: every derived triple mints a fresh blank that
/// re-triggers the rule) refuses typed `ResourceLimit` and leaves the parent
/// byte-identical AND still usable for a subsequent good transition.
#[test]
fn leg2_budget_limit_refusal_isolates_parent_keeps_it_usable() {
    let parent = LawState::parse(
        b"@prefix : <urn:n:> .\n:a :s :b .\n",
        Dialect::Turtle,
        None,
    )
    .expect("parent parses");
    let before = snapshot(&parent);
    let rules = "@prefix : <urn:n:> .\n{ ?x :s ?y } => { ?y :s _:n } .\n";
    let ruleset = validate_n3_rules(rules.as_bytes(), None).expect("divergent rules validate");
    let refusal = expect_refused(
        parent.transition(&Step::DeriveN3 { rules, ruleset: &ruleset }),
        "leg2",
    );
    assert_eq!(refusal.kind, RefusalKind::ResourceLimit, "{refusal:?}");
    let msg = refusal.message.as_str();
    assert!(
        msg.contains("n3_iterations"),
        "leg2: message must name the hit limit: {msg}"
    );
    assert!(
        msg.contains(&N3_MAX_ITERATIONS.to_string()),
        "leg2: message must carry the configured max {N3_MAX_ITERATIONS}: {msg}"
    );
    assert_untouched(&parent, &before, "leg2");
    // Zero residue: the same parent still completes a good derivation and the
    // resulting child matches a fresh-parse replay exactly.
    let (forked, _) = parent
        .transition(&Step::DeriveN3 {
            rules: GOOD_RULES,
            ruleset: &good_ruleset(),
        })
        .expect("post-refusal good derivation");
    let reparsed = LawState::parse(before.nquads.as_bytes(), Dialect::NQuads, None)
        .expect("reparse parent");
    let (fresh, _) = reparsed
        .transition(&Step::DeriveN3 {
            rules: GOOD_RULES,
            ruleset: &good_ruleset(),
        })
        .expect("fresh-parse good derivation");
    assert_eq!(forked.id(), fresh.id(), "leg2: post-refusal fork must equal fresh replay");
}

// ---------------------------------------------------------------------------
// Leg 3: blank-collision refusal isolated (probe-receipted, reachable)
// ---------------------------------------------------------------------------

/// Leg 3: the collision refusal fires when the parent's blank label survives
/// verbatim into derived output (probe round 3) and is isolated to the refused
/// child. The adjacent reachable contract — re-labeled rule-head blanks keep
/// blank-free parents safe and deterministic — is pinned in the same test so
/// the pin can never be faked by deleting the refusal arm.
#[test]
fn leg3_blank_collision_refusal_reachable_and_isolated() {
    // Probe assertions first (the receipt, executed not narrated):
    // (i) a premise-bound parent blank survives VERBATIM into derived output.
    let parent_blank_text = "@prefix : <urn:n:> .\n_:h :s :o .\n:o :stop :yes .\n";
    let probe_parent = LawState::parse(parent_blank_text.as_bytes(), Dialect::Turtle, None)
        .expect("probe parent parses");
    let base = nquads_of(&probe_parent);
    assert_eq!(
        blank_labels(&base),
        ["_:h".to_string()].into_iter().collect(),
        "probe: parent serialization writes the parsed blank label verbatim"
    );
    let echo_rules =
        "@prefix : <urn:n:> .\n{ ?x :s ?y . ?y :stop :yes } => { ?x :u _:h } .\n";
    let derived =
        reason_n3_bounded(&format!("{base}\n{echo_rules}")).expect("probe derivation");
    let derived_labels = blank_labels(&purrdf_serialized(&derived));
    assert!(
        derived_labels.contains("_:h"),
        "probe: premise-bound parent blank must survive verbatim, got {derived_labels:?}"
    );
    // (ii) rule-head fresh blanks are re-labeled `<written>_<16 hex>` — they
    // never equal the written head label, so blank-free parents stay safe.
    let head_fresh: Vec<&String> =
        derived_labels.iter().filter(|l| *l != "_:h").collect();
    assert!(
        !head_fresh.is_empty(),
        "probe: the head blank must appear re-labeled, got {derived_labels:?}"
    );
    for label in head_fresh {
        assert_relabeled(label);
    }
    // (iii) the re-label suffix is deterministic in-process.
    let again =
        reason_n3_bounded(&format!("{base}\n{echo_rules}")).expect("probe re-derivation");
    assert_eq!(derived, again, "probe: re-labeling must be deterministic");

    // The leg itself: parent `_:h` + echo rule -> collision refusal.
    let before = snapshot(&probe_parent);
    let ruleset = validate_n3_rules(echo_rules.as_bytes(), None).expect("echo rules validate");
    let refusal = expect_refused(
        probe_parent.transition(&Step::DeriveN3 {
            rules: echo_rules,
            ruleset: &ruleset,
        }),
        "leg3",
    );
    assert_eq!(refusal.kind, RefusalKind::Unsupported, "{refusal:?}");
    assert!(
        refusal
            .message
            .as_str()
            .contains("blank-node labels collide between the state and derived triples"),
        "leg3: message must name the collision: {}",
        refusal.message.as_str()
    );
    assert_untouched(&probe_parent, &before, "leg3");

    // Adjacent reachable contract: a blank-free parent with the same
    // head-blank rule merges cleanly (head blanks re-label away from the
    // parent's empty label space) and replays deterministically.
    let clean_text = "@prefix : <urn:n:> .\n:a :s :o .\n:o :stop :yes .\n";
    let clean = LawState::parse(clean_text.as_bytes(), Dialect::Turtle, None).expect("clean");
    let (child, r) = clean
        .transition(&Step::DeriveN3 {
            rules: echo_rules,
            ruleset: &ruleset,
        })
        .expect("blank-free parent must derive");
    assert_eq!(r.added, 1, "exactly one head-blank fact derived");
    let parent_labels = blank_labels(&nquads_of(&clean));
    let child_labels = blank_labels(&nquads_of(&child));
    assert_eq!(parent_labels.len(), 0, "clean parent is blank-free");
    assert_eq!(
        child_labels.intersection(&parent_labels).count(),
        0,
        "derived labels must not collide with the parent's label set"
    );
    for label in &child_labels {
        assert_relabeled(label);
    }
    let (child_again, _) = clean
        .transition(&Step::DeriveN3 {
            rules: echo_rules,
            ruleset: &ruleset,
        })
        .expect("replay");
    assert_eq!(child.id(), child_again.id(), "blank-bearing child id deterministic");
}

fn purrdf_serialized(derived_turtle: &str) -> String {
    let ds = purrdf::parse_dataset(derived_turtle.as_bytes(), "text/turtle", None)
        .expect("derived turtle parses");
    String::from_utf8(
        purrdf::serialize_dataset(
            ds.as_ref(),
            "application/n-quads",
            purrdf::SerializeGraph::Dataset,
        )
        .expect("derived n-quads"),
    )
    .expect("utf8")
}

// ---------------------------------------------------------------------------
// Leg 4: unknown-capability refusal isolated + zero residue
// ---------------------------------------------------------------------------

/// Leg 4: a step whose capability has no `BACKEND_AUTHORITIES` entry refuses
/// typed (`UNKNOWN-STEP-CAPABILITY`), mints no child, leaves zero residue —
/// the subsequent good transition yields the same child id as a fresh parse.
#[test]
fn leg4_unknown_capability_refusal_isolated_zero_residue() {
    let parent = LawState::parse(
        b"<urn:n:a> <urn:n:s> <urn:n:b> .\n",
        Dialect::NTriples,
        None,
    )
    .expect("parent parses");
    let before = snapshot(&parent);
    let refusal = expect_refused(
        parent.transition(&Step::TestCapabilityProbe("court-p16-unknown-capability")),
        "leg4",
    );
    assert_eq!(refusal.kind, RefusalKind::Unsupported, "{refusal:?}");
    let msg = refusal.message.as_str();
    assert!(
        msg.contains("[UNKNOWN-STEP-CAPABILITY]"),
        "leg4: stable marker missing: {msg}"
    );
    assert!(
        msg.contains("court-p16-unknown-capability"),
        "leg4: message must name the capability: {msg}"
    );
    assert_untouched(&parent, &before, "leg4");

    // Zero residue: fork after the refusal == fork from a fresh parse.
    let step = Step::DeriveN3 {
        rules: GOOD_RULES,
        ruleset: &good_ruleset(),
    };
    let (forked, r_forked) = parent.transition(&step).expect("fork after refusal");
    let fresh_parent =
        LawState::parse(before.nquads.as_bytes(), Dialect::NQuads, None).expect("reparse");
    let (fresh, r_fresh) = fresh_parent.transition(&step).expect("fresh fork");
    assert_eq!(forked.id(), fresh.id(), "leg4: refused-then-fork id must equal fresh");
    assert_eq!(r_forked, r_fresh, "leg4: refused-then-fork receipt must equal fresh");
}

// ---------------------------------------------------------------------------
// Leg 5: fork exactness — child quads == parent union independently derived
// ---------------------------------------------------------------------------

/// Leg 5: the DeriveN3 child is EXACTLY the parent union the public
/// `reason_n3_bounded` independently computes, and the receipt carries the
/// full field contract.
#[test]
fn leg5_fork_exactness_and_receipt_fields() {
    let parent = LawState::parse(
        b"@prefix : <urn:n:> .\n:a :s :b .\n:b :s :c .\n",
        Dialect::Turtle,
        None,
    )
    .expect("parent parses");
    let base = nquads_of(&parent);

    // Independent expectation: the public reasoner over the same doc.
    let derived = reason_n3_bounded(&format!("{base}\n{GOOD_RULES}")).expect("public derivation");
    let derived_nq = purrdf_serialized(&derived);
    let expected =
        LawState::parse(format!("{base}{derived_nq}").as_bytes(), Dialect::NQuads, None)
            .expect("expected union parses");

    let (child, r) = parent
        .transition(&Step::DeriveN3 {
            rules: GOOD_RULES,
            ruleset: &good_ruleset(),
        })
        .expect("fork succeeds");

    // Exactness: same canonical id and same N-Quads line set as the union.
    assert_eq!(child.id(), expected.id(), "child id must equal the independent union id");
    let child_nq = nquads_of(&child);
    let expected_nq = nquads_of(&expected);
    let child_lines: std::collections::BTreeSet<&str> = child_nq.lines().collect();
    let expected_lines: std::collections::BTreeSet<&str> = expected_nq.lines().collect();
    assert_eq!(child_lines, expected_lines, "child quads must equal the union");
    assert_eq!(child.quad_count(), expected.quad_count());
    assert_eq!(r.added, expected.quad_count() - parent.quad_count());

    // Receipt field contract.
    assert_eq!(r.parent, parent.id());
    assert_eq!(r.child, child.id());
    assert_eq!(r.step, "derive:n3");
    assert_eq!(r.authority.capability, "Notation3");
    assert_eq!(r.authority.authority, "purrdf::n3");
    assert_eq!(r.lease_id, None);
    assert_eq!(r.plan_blake3, None);
    assert_eq!(r.subject_blake3, None);
}

// ---------------------------------------------------------------------------
// Leg 6: replay determinism — order noise and refused-then-fork
// ---------------------------------------------------------------------------

/// Leg 6: statement-order noise produces identical parent ids, identical
/// child ids, and identical receipts; a refused attempt on the path changes
/// nothing: refused-then-fork is byte-equivalent (ids + receipts) to fork
/// alone.
#[test]
fn leg6_replay_determinism_refused_then_fork_equals_fork_alone() {
    let order_a = b"@prefix : <urn:n:> .\n:a :s :b .\n:b :s :c .\n";
    let order_b = b"@prefix : <urn:n:> .\n\n:b   :s :c .\n:a :s :b .\n"; // reordered + noise
    let pa = LawState::parse(order_a, Dialect::Turtle, None).expect("parse a");
    let pb = LawState::parse(order_b, Dialect::Turtle, None).expect("parse b");
    assert_eq!(pa.id(), pb.id(), "order noise must not change the id");

    let step = Step::DeriveN3 {
        rules: GOOD_RULES,
        ruleset: &good_ruleset(),
    };
    let (ca, ra) = pa.transition(&step).expect("fork a");
    let (cb, rb) = pb.transition(&step).expect("fork b");
    assert_eq!(ca.id(), cb.id(), "order noise must not change the child id");
    assert_eq!(ra, rb, "order noise must not change the receipt");

    // Refused-then-fork == fork alone.
    let bad_rules = "{ ?x :s ?y } => "; // fails N3 admission
    let empty_ruleset = validate_n3_rules(b"", None).expect("empty rules validate");
    let before = snapshot(&pa);
    expect_refused(pa.transition(&Step::DeriveN3 {
        rules: bad_rules,
        ruleset: &empty_ruleset,
    }), "leg6 refusal");
    assert_untouched(&pa, &before, "leg6 after refusal");
    let (c_after, r_after) = pa.transition(&step).expect("fork after refusal");
    assert_eq!(c_after.id(), ca.id(), "refused-then-fork must equal fork alone");
    assert_eq!(r_after, ra, "refused-then-fork receipt must equal fork alone");
}

// ---------------------------------------------------------------------------
// Leg 7: 5-step receipt chain, offline auditor
// ---------------------------------------------------------------------------

/// Offline auditor: reconstruct the id chain from a root id plus receipts.
/// Returns the final child id, or a typed error naming the broken link.
///
/// Honest boundary: this reconstructs IDS only. Receipts carry no state
/// content bytes, so the states themselves are not reconstructable from
/// receipts — a content-reconstruction API (e.g. receipts carrying the
/// canonical delta) is an extension, out of scope for this court.
fn audit(root_id: &str, receipts: &[Receipt]) -> Result<String, String> {
    let mut current = root_id.to_string();
    for (i, r) in receipts.iter().enumerate() {
        if r.parent != current {
            return Err(format!(
                "receipt {i} (step `{}`) breaks the chain: parent `{}` != expected `{}`",
                r.step, r.parent, current
            ));
        }
        current = r.child.clone();
    }
    Ok(current)
}

/// Leg 7: five receipted transitions from one root; the offline auditor
/// reconstructs every intermediate id and the final id; a tampered chain
/// refuses (the auditor is not vacuous).
#[test]
fn leg7_receipt_chain_offline_auditor_reconstructs_ids() {
    let root = LawState::parse(
        b"@prefix : <urn:n:> .\n:a :s :b .\n",
        Dialect::Turtle,
        None,
    )
    .expect("root parses");

    // Step 1: derive the inverse fact (`:b :t :a`).
    let rules_a = "@prefix : <urn:n:> .\n{ ?x :s ?y } => { ?y :t ?x } .\n";
    let ruleset_a = validate_n3_rules(rules_a.as_bytes(), None).expect("rules A validate");
    // Step 2: type the objects of `:s` as `:Linked` (`:b a :Linked`).
    let rules_b = "@prefix : <urn:n:> .\n{ ?x :s ?y } => { ?y a :Linked } .\n";
    let ruleset_b = validate_n3_rules(rules_b.as_bytes(), None).expect("rules B validate");
    // Step 3: SHACL shapes that the state satisfies (`:b` has a `:t`).
    let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n\
                  @prefix : <urn:n:> .\n\
                  :LinkedShape a sh:NodeShape ;\n\
                      sh:targetClass :Linked ;\n\
                      sh:property [ sh:path :t ; sh:minCount 1 ] .\n";

    let (s1, r1) = root
        .transition(&Step::DeriveN3 { rules: rules_a, ruleset: &ruleset_a })
        .expect("step 1");
    assert!(r1.added >= 1, "step 1 must add the inverse fact");
    let (s2, r2) = s1
        .transition(&Step::DeriveN3 { rules: rules_b, ruleset: &ruleset_b })
        .expect("step 2");
    assert!(r2.added >= 1, "step 2 must add the type fact");
    let (s3, r3) = s2
        .transition(&Step::AdmitShacl { shapes_ttl: shapes })
        .expect("step 3 conforms");
    assert_eq!(r3.added, 0, "admission gates never change the state");
    assert_eq!(s3.id(), s2.id());
    let (s4, r4) = s3.transition(&Step::EntailRdfs).expect("step 4");
    assert!(r4.added >= 1, "RDFS closure must add facts");
    let (s5, r5) = s4.transition(&Step::EntailOwlRl).expect("step 5");

    let receipts = [r1.clone(), r2.clone(), r3.clone(), r4.clone(), r5.clone()];
    assert_eq!(receipts.len(), 5);

    // The auditor reconstructs the full chain and every prefix.
    let final_id = audit(root.id(), &receipts).expect("chain must audit clean");
    assert_eq!(final_id, s5.id(), "auditor final id must equal the terminal child");
    for k in 1..=receipts.len() {
        let prefix = audit(root.id(), &receipts[..k]).expect("prefix must audit clean");
        let expected = [s1.id(), s2.id(), s3.id(), s4.id(), s5.id()][k - 1];
        assert_eq!(prefix, expected, "auditor prefix {k} must equal the k-th child id");
    }

    // Non-vacuity: a tampered link refuses with a typed error.
    let mut tampered = receipts.clone();
    tampered[2].parent = "blake3:deadbeef".to_string();
    assert!(
        audit(root.id(), &tampered).is_err(),
        "a broken chain must refuse to audit"
    );
    // And the honest boundary: ids only, no content bytes.
    assert!(
        final_id.starts_with("blake3:"),
        "the auditor reconstructs ids, not content: {final_id}"
    );
}
