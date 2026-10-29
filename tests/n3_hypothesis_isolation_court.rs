//! N3 hypothesis quarantine court (WP P15, pillar 3 isolation).
//!
//! The `log:conclusion` builtin runs a quoted hypothesis through a nested
//! reasoner instance (purrdf fork crates/n3 `src/reasoner.rs`, `eval_log_conclusion`).
//! The quarantine contract under court here:
//!
//!   1. **baseline**: the asserting pack's derived surface, rendered through
//!      `Term::canonical_form`, is a stable snapshot (byte-identical reruns).
//!   2. **assert-route** (quoted `log:conclusion` premise): the finding lands
//!      in the outer derived set; the hypothesis internals (`:knows` family)
//!      appear nowhere in it; the rerun is byte-identical.
//!      **evaluate-route** (object variable): the bound formula value carries
//!      the hypothesis input + finding, never the outer contradiction.
//!   3. **contradicting hypothesis**: `Ok` with an EMPTY derived set — never
//!      an error, never a fabricated fact.
//!   4. **PreparedReasoner order-flip**: `reason(A)` then `reason(B)` vs
//!      `reason(B)` then `reason(A)` — per-batch derived sets are identical
//!      across the two orders and the batches' sets are disjoint (zero
//!      cross-batch residue).
//!   5. **re-baseline tripwire**: a `string:matches` premise populates the
//!      lazy-global `REGEX_CACHE` mid-court; the baseline re-run stays
//!      byte-identical.
//!   6. **law rung**: `Step::DeriveN3` receipt `added == 1` for the asserting
//!      pack, `added == 0` for the contradicting quarantine twin, and a
//!      data-join sub-twin (state fact + hypothesis result in one rule) at
//!      `added == 1`.
//!   7. **ABI rung**: `{"op":"n3"}` request order-flip is byte-identical per
//!      request.
//!   8. **negative twin**: a plain forward rule DOES write (`added == 1`,
//!      `child != parent`), proving the quarantine assertions detect a real
//!      write.
//!
//! All rungs hit the real engines (`reason_n3_bounded`, `LawState`,
//! `PreparedReasoner`, the ABI `call`) — no test doubles, no sleeps.

#![cfg(feature = "abi")]

use std::collections::BTreeSet;

use graphlaw::abi::call;
use graphlaw::law::{LawState, Step, reason_n3_bounded};
use graphlaw::tier::validate_n3_rules;
use purrdf::n3::{self, PreparedReasoner, ReasonerOptions};
use serde_json::{Value, json};

const NS: &str = "http://example.org/iso/";
const LOG_IMPLIES: &str = "http://www.w3.org/2000/10/swap/log#implies";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Hypothesis A, assert-route: the quoted premise carries the hypothesis (two
/// facts worth of content and its own internal rule); the quoted conclusion
/// pattern restates the full nested closure (input triples + the inner rule's
/// finding) because formula unification is cardinality-exact. The outer rule
/// asserts exactly one finding. A leak of hypothesis internals into the outer
/// derived set — the input facts, the inner finding, or any nested-scope
/// term — flips this fixture red.
fn asserting_pack() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         @prefix log: <http://www.w3.org/2000/10/swap/log#> .\n\
         {{ {{\n\
         \x20 :alice :knows :bob .\n\
         \x20 {{ :alice :knows ?x }} => {{ ?x :knownBy :alice }} .\n\
         }} log:conclusion {{\n\
         \x20 :alice :knows :bob .\n\
         \x20 {{ :alice :knows ?x }} => {{ ?x :knownBy :alice }} .\n\
         \x20 :bob :knownBy :alice .\n\
         }} }} => {{ :bob :witnessedBy :alice }} .\n"
    )
}

/// Hypothesis B, the contradicting twin: its inner rule's premise (`:carol
/// :hates ?x`) is refuted by the hypothesis content — nothing hates anyone —
/// so the expected finding `:carol :avoidedBy :alice` is never derivable and
/// the quoted conclusion pattern cannot unify (the nested closure holds two
/// triples, the pattern claims three). The engine must answer `Ok` with an
/// empty derived set, never an error and never a fabricated finding.
fn contradicting_pack() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         @prefix log: <http://www.w3.org/2000/10/swap/log#> .\n\
         {{ {{\n\
         \x20 :alice :knows :carol .\n\
         \x20 {{ :carol :hates ?x }} => {{ ?x :avoidedBy :alice }} .\n\
         }} log:conclusion {{\n\
         \x20 :alice :knows :carol .\n\
         \x20 {{ :carol :hates ?x }} => {{ ?x :avoidedBy :alice }} .\n\
         \x20 :carol :avoidedBy :alice .\n\
         }} }} => {{ :carol :witnessedBy :alice }} .\n"
    )
}

/// Evaluate-route: the `log:conclusion` object is a variable, so the rule
/// captures the whole nested closure formula. The outer graph also asserts a
/// contradiction (`:bob :refutedBy :mallory`) that the nested run can never
/// see; the captured value must carry input + finding and never the outer
/// refutation.
fn evaluate_pack() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         @prefix log: <http://www.w3.org/2000/10/swap/log#> .\n\
         :bob :refutedBy :mallory .\n\
         {{ {{\n\
         \x20   :alice :knows :bob .\n\
         \x20   {{ :alice :knows ?x }} => {{ ?x :knownBy :alice }} .\n\
         \x20 }} log:conclusion ?f }} => {{ :court :captured ?f }} .\n"
    )
}

/// Regex-arm pack: a `string:matches` premise forces the lazy-global
/// `REGEX_CACHE` to populate mid-court (leg 5 pollutant).
fn regex_pack() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         @prefix string: <http://www.w3.org/2000/10/swap/string#> .\n\
         :ticket :code \"ISO-77\" .\n\
         {{ :ticket :code ?c . ?c string:matches \"^ISO-[0-9]+$\" }} => {{ :ticket :validatedBy :regexArm }} .\n"
    )
}

/// Leg-4 program: hypotheses A and B as data-gated rules. `:court :hears :a`
/// opens only A's batch; `:court :hears :b` opens only B's (whose hypothesis
/// is the contradicting twin, so B's batch derives nothing).
fn prepared_program() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         @prefix log: <http://www.w3.org/2000/10/swap/log#> .\n\
         {{ :court :hears :a .\n\
         \x20 {{\n\
         \x20   :alice :knows :bob .\n\
         \x20   {{ :alice :knows ?x }} => {{ ?x :knownBy :alice }} .\n\
         \x20 }} log:conclusion {{\n\
         \x20   :alice :knows :bob .\n\
         \x20   {{ :alice :knows ?x }} => {{ ?x :knownBy :alice }} .\n\
         \x20   :bob :knownBy :alice .\n\
         \x20 }}\n\
         }} => {{ :bob :witnessedBy :alice }} .\n\
         {{ :court :hears :b .\n\
         \x20 {{\n\
         \x20   :alice :knows :carol .\n\
         \x20   {{ :carol :hates ?x }} => {{ ?x :avoidedBy :alice }} .\n\
         \x20 }} log:conclusion {{\n\
         \x20   :alice :knows :carol .\n\
         \x20   {{ :carol :hates ?x }} => {{ ?x :avoidedBy :alice }} .\n\
         \x20   :carol :avoidedBy :alice .\n\
         \x20 }}\n\
         }} => {{ :carol :witnessedBy :alice }} .\n"
    )
}

fn batch_a_text() -> String {
    format!("@prefix : <{NS}> .\n:court :hears :a .\n")
}

fn batch_b_text() -> String {
    format!("@prefix : <{NS}> .\n:court :hears :b .\n")
}

/// Leg-6 data-join rules: the state carries `:court :hears :a`; the rules
/// text carries only the gated hypothesis rule — state fact + hypothesis
/// result join in one rule.
fn data_join_rules() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         @prefix log: <http://www.w3.org/2000/10/swap/log#> .\n\
         {{ :court :hears :a .\n\
         \x20 {{\n\
         \x20   :alice :knows :bob .\n\
         \x20   {{ :alice :knows ?x }} => {{ ?x :knownBy :alice }} .\n\
         \x20 }} log:conclusion {{\n\
         \x20   :alice :knows :bob .\n\
         \x20   {{ :alice :knows ?x }} => {{ ?x :knownBy :alice }} .\n\
         \x20   :bob :knownBy :alice .\n\
         \x20 }}\n\
         }} => {{ :bob :witnessedBy :alice }} .\n"
    )
}

/// Leg-8 negative twin: a plain forward rule over a plain fact, no hypothesis
/// anywhere. It must genuinely write, proving the quarantine assertions
/// (added == 0 / child == parent above) can detect a real derivation.
fn plain_forward_rules() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         :alice :knows :bob .\n\
         {{ :alice :knows ?x }} => {{ ?x :knownBy :alice }} .\n"
    )
}

fn data_join_state_text() -> String {
    format!("@prefix : <{NS}> .\n:court :hears :a .\n")
}

// ---------------------------------------------------------------------------
// Runners
// ---------------------------------------------------------------------------

fn run(req: Value) -> Value {
    serde_json::from_slice(&call(req.to_string().as_bytes())).expect("GraphLaw ABI returns JSON")
}

fn ok(req: Value) -> Value {
    let out = run(req.clone());
    assert_eq!(out["ok"], true, "request {req} was refused: {out}");
    out
}

/// Canonical `s p o` lines of a derived N3 text, round-tripped through the N3
/// parser so every assertion covers the written surface.
fn canonical_lines(text: &str) -> BTreeSet<String> {
    let doc = n3::parse_n3(text, None).expect("derived text re-parses as N3");
    doc.facts
        .iter()
        .map(canonical_line)
        .collect()
}

fn canonical_line(t: &n3::Triple) -> String {
    format!(
        "{} {} {}",
        t.s.canonical_form(),
        t.p.canonical_form(),
        t.o.canonical_form()
    )
}

fn triples_set(triples: &[n3::Triple]) -> BTreeSet<String> {
    triples.iter().map(canonical_line).collect()
}

/// Canonical form of an IRI term (engine-escaped, single source of truth).
fn iri(value: &str) -> String {
    n3::Term::iri(value).canonical_form()
}

fn term_line(s: &str, p: &str, o: &str) -> String {
    format!("{s} {p} {o}")
}

/// The assert-route finding, in canonical form.
fn finding_line() -> String {
    term_line(
        &iri(&format!("{NS}bob")),
        &iri(&format!("{NS}witnessedBy")),
        &iri(&format!("{NS}alice")),
    )
}

/// Leg-1 snapshot: derived set + raw derived text of the asserting pack.
fn baseline_snapshot() -> (BTreeSet<String>, String) {
    let text = reason_n3_bounded(&asserting_pack()).expect("asserting pack derives");
    let set = canonical_lines(&text);
    (set, text)
}

// ---------------------------------------------------------------------------
// Leg 1 — baseline snapshot
// ---------------------------------------------------------------------------

#[test]
fn leg1_baseline_snapshot_is_canonical_and_stable() {
    let (set1, text1) = baseline_snapshot();
    let (set2, text2) = baseline_snapshot();

    assert_eq!(set1, set2, "canonical baseline is stable across reruns");
    assert_eq!(
        text1, text2,
        "baseline derived text is byte-identical across reruns"
    );
    assert_eq!(
        set1,
        [finding_line()].into_iter().collect::<BTreeSet<String>>(),
        "baseline derived set is exactly the assert-route finding, got {set1:?}"
    );

    // ABI rung of the same snapshot: identical set on the wire, stable bytes.
    let out1 = ok(json!({"op": "n3", "text": asserting_pack()}));
    let out2 = ok(json!({"op": "n3", "text": asserting_pack()}));
    assert_eq!(out1["derived"], out2["derived"], "ABI baseline is stable");
    let wire_text = out1["derived"].as_str().expect("ABI derived text");
    assert_eq!(
        canonical_lines(wire_text),
        set1,
        "ABI derived set matches the pre-engine baseline"
    );
}

// ---------------------------------------------------------------------------
// Leg 2 — assert-route quarantine + evaluate-route capture
// ---------------------------------------------------------------------------

#[test]
fn leg2_assert_route_quarantines_hypothesis_internals() {
    let (set, text) = baseline_snapshot();

    // The finding landed.
    assert!(set.contains(&finding_line()), "finding must land, got {set:?}");
    // Hypothesis internals appear nowhere in the outer derived set: neither
    // the hypothesis's own facts (knows) nor its internal rule's finding
    // (knownBy) may surface unquoted.
    assert!(
        !text.contains("knows"),
        "hypothesis internals (knows/knownBy family) must appear nowhere in the \
         outer derived text, got: {text}"
    );

    // Byte-identical rerun.
    let (set_rerun, text_rerun) = baseline_snapshot();
    assert_eq!(set_rerun, set, "rerun derived set is identical");
    assert_eq!(text_rerun, text, "rerun derived text is byte-identical");
}

#[test]
fn leg2_evaluate_route_captures_input_plus_finding_never_contradiction() {
    let text1 = reason_n3_bounded(&evaluate_pack()).expect("evaluate route derives");
    let set1 = canonical_lines(&text1);

    // Exactly one derived triple: the capture.
    assert_eq!(set1.len(), 1, "only the capture derives, got {set1:?}");
    let capture = set1.iter().next().expect("capture line");
    assert!(
        capture.contains(&iri(&format!("{NS}captured"))),
        "the derived triple is the capture, got {capture}"
    );

    // Byte-identical rerun.
    let text2 = reason_n3_bounded(&evaluate_pack()).expect("evaluate route re-derives");
    assert_eq!(text2, text1, "evaluate route is byte-identical on rerun");

    // Inspect the captured formula value through a real re-parse.
    let doc = n3::parse_n3(&text1, None).expect("derived text re-parses");
    let captured = doc
        .facts
        .iter()
        .find(|t| {
            matches!(&t.p, n3::Term::Iri(p) if p.as_str() == format!("{NS}captured"))
        })
        .expect("capture triple present in derived text");
    let n3::Term::Formula(closure) = &captured.o else {
        panic!("captured object must be the quoted closure formula, got {:?}", captured.o);
    };

    let value = triples_set(&closure.triples);
    assert_eq!(
        closure.triples.len(),
        3,
        "captured value = hypothesis input (fact + rule) + finding, got {value:?}"
    );
    assert!(
        value.contains(&term_line(
            &iri(&format!("{NS}alice")),
            &iri(&format!("{NS}knows")),
            &iri(&format!("{NS}bob")),
        )),
        "captured value carries the hypothesis input fact, got {value:?}"
    );
    assert!(
        value.contains(&term_line(
            &iri(&format!("{NS}bob")),
            &iri(&format!("{NS}knownBy")),
            &iri(&format!("{NS}alice")),
        )),
        "captured value carries the hypothesis finding, got {value:?}"
    );
    assert!(
        closure.triples.iter().any(|t| {
            matches!(&t.p, n3::Term::Iri(p) if p.as_str() == LOG_IMPLIES)
        }),
        "captured value carries the hypothesis's internal rule (log:implies), got {value:?}"
    );

    // The outer contradiction never enters the quoted closure, and neither
    // does the outer capture machinery.
    assert!(
        !value.iter().any(|line| line.contains("refutedBy")),
        "the outer refutation must stay outside the hypothesis closure, got {value:?}"
    );
    assert!(
        !value.iter().any(|line| line.contains("court") || line.contains("captured")),
        "outer capture machinery stays outside the hypothesis closure, got {value:?}"
    );
}

// ---------------------------------------------------------------------------
// Leg 3 — contradicting hypothesis: Ok + empty derived
// ---------------------------------------------------------------------------

#[test]
fn leg3_contradicting_hypothesis_answers_ok_with_empty_derived() {
    // Pre-engine rung: a successful run with nothing derived — never an
    // error, never a fabricated finding.
    let derived = reason_n3_bounded(&contradicting_pack())
        .expect("a contradicted hypothesis is a successful run, not an error");
    assert!(
        derived.trim().is_empty(),
        "contradicted hypothesis must derive nothing, got: {derived:?}"
    );
    assert_eq!(
        canonical_lines(&derived),
        BTreeSet::new(),
        "contradicted hypothesis fabricates no facts"
    );

    // ABI rung: ok:true with an empty derived text.
    let out = ok(json!({"op": "n3", "text": contradicting_pack()}));
    assert!(
        out["derived"]
            .as_str()
            .expect("derived text")
            .trim()
            .is_empty(),
        "ABI contradicted hypothesis derives nothing, got {out}"
    );
}

// ---------------------------------------------------------------------------
// Leg 4 — PreparedReasoner order-flip: per-batch purity
// ---------------------------------------------------------------------------

#[test]
fn leg4_prepared_reasoner_order_flip_is_batch_pure() {
    let prepared = PreparedReasoner::new(
        n3::parse_n3(&prepared_program(), None).expect("prepared program parses"),
    );
    let batch_a = n3::parse_n3(&batch_a_text(), None).expect("batch a parses");
    let batch_b = n3::parse_n3(&batch_b_text(), None).expect("batch b parses");
    let options = ReasonerOptions::default();

    // Order 1: reason(A's batch) then reason(B's batch).
    let ab_a = prepared.reason(&batch_a, &options);
    let ab_b = prepared.reason(&batch_b, &options);
    assert!(ab_a.is_complete() && ab_b.is_complete(), "both runs complete");
    let set_a_in_ab = triples_set(&ab_a.derived);
    let set_b_in_ab = triples_set(&ab_b.derived);

    // Order 2: reason(B's batch) then reason(A's batch).
    let ba_b = prepared.reason(&batch_b, &options);
    let ba_a = prepared.reason(&batch_a, &options);
    assert!(ba_a.is_complete() && ba_b.is_complete(), "both runs complete");
    let set_a_in_ba = triples_set(&ba_a.derived);
    let set_b_in_ba = triples_set(&ba_b.derived);

    // Per-batch sets identical across the two orders.
    assert_eq!(
        set_a_in_ab, set_a_in_ba,
        "batch A's derived set is identical in both orders"
    );
    assert_eq!(
        set_b_in_ab, set_b_in_ba,
        "batch B's derived set is identical in both orders"
    );

    // Batch A derives exactly its finding; batch B (the contradicting twin's
    // batch) derives nothing — before OR after A ran.
    assert_eq!(
        set_a_in_ab,
        [finding_line()].into_iter().collect::<BTreeSet<String>>(),
        "batch A derives exactly the assert-route finding, got {set_a_in_ab:?}"
    );
    assert!(
        set_b_in_ab.is_empty(),
        "batch B derives nothing (quarantined), got {set_b_in_ab:?}"
    );

    // Zero cross-batch residue: the two batches' derived sets are disjoint —
    // A's finding never bleeds into B's batch result or vice versa.
    assert!(
        set_a_in_ab.is_disjoint(&set_b_in_ab),
        "zero cross-batch residue, got A={set_a_in_ab:?} B={set_b_in_ab:?}"
    );
}

// ---------------------------------------------------------------------------
// Leg 5 — re-baseline tripwire over the lazy-global REGEX_CACHE
// ---------------------------------------------------------------------------

#[test]
fn leg5_regex_pollution_midcourt_rebaseline_is_byte_identical() {
    let (baseline_set, baseline_text) = baseline_snapshot();

    // Populate the lazy-global REGEX_CACHE mid-court.
    let regex_text = reason_n3_bounded(&regex_pack())
        .expect("regex pack derives through string:matches");
    assert!(
        regex_text.contains("validatedBy"),
        "string:matches premise genuinely fired, got: {regex_text}"
    );

    // Re-baseline: byte-identical despite the populated cache.
    let (rebase_set, rebase_text) = baseline_snapshot();
    assert_eq!(
        rebase_text, baseline_text,
        "re-baseline derived text is byte-identical after REGEX_CACHE population"
    );
    assert_eq!(rebase_set, baseline_set, "re-baseline derived set is identical");

    // ABI rung tripwire: baseline, pollutant, baseline.
    let first = ok(json!({"op": "n3", "text": asserting_pack()}));
    let polluted = ok(json!({"op": "n3", "text": regex_pack()}));
    let second = ok(json!({"op": "n3", "text": asserting_pack()}));
    assert!(
        polluted["derived"]
            .as_str()
            .expect("derived text")
            .contains("validatedBy"),
        "regex pack fires on the wire too, got {polluted}"
    );
    assert_eq!(
        second["derived"], first["derived"],
        "ABI re-baseline is byte-identical after the regex pollutant"
    );
}

// ---------------------------------------------------------------------------
// Leg 6 — law rung: DeriveN3 receipts, quarantine twin, data-join sub-twin
// ---------------------------------------------------------------------------

#[test]
fn leg6_derive_n3_asserting_pack_adds_exactly_the_finding() {
    let rules = asserting_pack();
    let ruleset = validate_n3_rules(rules.as_bytes(), None).expect("asserting pack admits as N3");
    let state = LawState::parse(b"", graphlaw::dialect::Dialect::Turtle, None)
        .expect("empty state admits");

    let (child, receipt) = state
        .transition(&Step::DeriveN3 {
            rules: &rules,
            ruleset: &ruleset,
        })
        .expect("asserting pack derives at the law step");

    assert_eq!(receipt.added, 1, "exactly the finding is written, got {receipt:?}");
    assert_eq!(child.quad_count(), 1, "child state = parent (0) + finding");
    assert_ne!(receipt.child, receipt.parent, "the write is real");
}

#[test]
fn leg6_derive_n3_contradicting_pack_is_the_quarantine_twin() {
    let rules = contradicting_pack();
    let ruleset =
        validate_n3_rules(rules.as_bytes(), None).expect("contradicting pack admits as N3");
    let state = LawState::parse(b"", graphlaw::dialect::Dialect::Turtle, None)
        .expect("empty state admits");

    let (child, receipt) = state
        .transition(&Step::DeriveN3 {
            rules: &rules,
            ruleset: &ruleset,
        })
        .expect("contradicting pack is Ok at the law step, never an error");

    assert_eq!(receipt.added, 0, "quarantine twin writes nothing, got {receipt:?}");
    assert_eq!(child.quad_count(), 0, "child state carries only the empty parent");
    assert_eq!(
        receipt.child, receipt.parent,
        "a quarantined derivation leaves the state identity untouched"
    );
}

#[test]
fn leg6_derive_n3_data_join_sub_twin() {
    // State fact + hypothesis result join in one rule: the state carries the
    // gate fact, the rules text carries the gated hypothesis rule.
    let rules = data_join_rules();
    let ruleset = validate_n3_rules(rules.as_bytes(), None).expect("data-join rules admit as N3");
    let state = LawState::parse(
        data_join_state_text().as_bytes(),
        graphlaw::dialect::Dialect::Turtle,
        None,
    )
    .expect("gate fact admits");

    assert_eq!(state.quad_count(), 1, "state carries exactly the gate fact");

    let (child, receipt) = state
        .transition(&Step::DeriveN3 {
            rules: &rules,
            ruleset: &ruleset,
        })
        .expect("data-join derives at the law step");

    assert_eq!(receipt.added, 1, "exactly the joined finding is written, got {receipt:?}");
    assert_eq!(child.quad_count(), 2, "child = gate fact + finding");
}

// ---------------------------------------------------------------------------
// Leg 7 — ABI rung: op n3 order-flip, byte-identical per request
// ---------------------------------------------------------------------------

#[test]
fn leg7_abi_order_flip_is_byte_identical_per_request() {
    // Order 1: asserting request, then contradicting request.
    let a1 = ok(json!({"op": "n3", "text": asserting_pack()}));
    let b1 = ok(json!({"op": "n3", "text": contradicting_pack()}));

    // Order 2 (flip): contradicting again, then asserting again.
    let b2 = ok(json!({"op": "n3", "text": contradicting_pack()}));
    let a2 = ok(json!({"op": "n3", "text": asserting_pack()}));

    assert_eq!(a1["ok"], true);
    assert_eq!(b1["ok"], true, "contradicting pack is never an error, got {b1}");

    // Byte-identical derived per request across the flip.
    assert_eq!(
        a1["derived"], a2["derived"],
        "asserting request is byte-identical across the order flip"
    );
    assert_eq!(
        b1["derived"], b2["derived"],
        "contradicting request is byte-identical across the order flip"
    );

    // The contradicting request answers empty on the wire (quarantined).
    assert!(
        b1["derived"]
            .as_str()
            .expect("derived text")
            .trim()
            .is_empty(),
        "contradicting request derives nothing, got {b1}"
    );

    // Each request's derived set matches its pre-engine twin.
    assert_eq!(
        canonical_lines(a1["derived"].as_str().expect("derived text")),
        [finding_line()].into_iter().collect::<BTreeSet<String>>(),
    );
}

// ---------------------------------------------------------------------------
// Leg 8 — negative twin: a plain forward rule genuinely writes
// ---------------------------------------------------------------------------

#[test]
fn leg8_negative_twin_plain_forward_rule_writes() {
    let rules = plain_forward_rules();
    let ruleset = validate_n3_rules(rules.as_bytes(), None).expect("plain rules admit as N3");
    let state = LawState::parse(b"", graphlaw::dialect::Dialect::Turtle, None)
        .expect("empty state admits");

    let (child, receipt) = state
        .transition(&Step::DeriveN3 {
            rules: &rules,
            ruleset: &ruleset,
        })
        .expect("plain forward rule derives");

    assert_eq!(receipt.added, 1, "the plain rule writes one fact, got {receipt:?}");
    assert_ne!(
        receipt.child, receipt.parent,
        "the plain rule moves the state identity — the quarantine assertions \
         above (added == 0, child == parent) can detect a real write"
    );
    assert_eq!(child.quad_count(), 1, "child state carries the derived fact");
}
