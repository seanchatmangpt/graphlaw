//! N3 quantifier execution court (closes audit F15 / C10).
//!
//! Audit C10 (`docs/jira/v26.10.23/n3-audit-100-report.md`): `docs/DIALECTS.md`
//! advertises N3 support "yes (`=>`, `@forAll`, `@forSome`)" while the
//! quantifier surface was never positively executed — no committed input
//! reached `parse_quantifier`, all four quantifier refusal arms
//! (purrdf fork crates/n3 `src/parser.rs`, fn `parse_quantifier`) were reachable
//! from no committed input, and zero tests exercised quantifiers. This court
//! positively executes the advertised surface on real engines (no mocks) at
//! three rungs:
//!
//!   1. pre-engine: `law::reason_n3_bounded` typed outcomes;
//!   2. law step: `Step::DeriveN3` admission + transition;
//!   3. ABI op: `{"op":"n3"}` wire envelopes.
//!
//! Positives assert on the derived SET (canonical subject/predicate/object
//! strings round-tripped through the N3 parser), not on substrings.
//!
//! Flip twins (the negative of each positive, in-court):
//!   - universal derivation with data VS the empty-universe twin (no data ->
//!     empty derivation, nothing fabricated);
//!   - existential bound through a rule VS the scope twin (a premise-scoped
//!     `@forSome` does not bind the conclusion);
//!   - skolemization visible VS distinct existentials skolemizing to distinct
//!     IRIs (a collapsing skolemizer flips this red);
//!   - well-formed quantifier documents admit VS all four misuse refusals
//!     (`quantifiers must precede the first triple of their graph`,
//!     `@forAll must precede @forSome in the same graph`, `quantifier
//!     requires an IRI or prefixed name`, `IRI is already quantified in this
//!     graph`) plus the empty quantifier list `@forAll .` / `@forSome .`;
//!   - `Step::DeriveN3` enforces the empty data universe for quantifier rules:
//!     empty state admits and derives; a non-empty state refuses typed
//!     (merged-document quantifier-order violation).

#![cfg(feature = "abi")]

use std::collections::BTreeSet;

use graphlaw::abi::call;
use graphlaw::dialect::{Dialect, RefusalKind};
use graphlaw::law::{LawState, N3Error, Step, reason_n3_bounded};
use graphlaw::tier::validate_n3_rules;
use purrdf::n3;
use serde_json::{Value, json};

const NS: &str = "http://example.org/q/";
const GENID: &str = "https://eyereasoner.github.io/.well-known/genid/";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// `@forAll` universal in a rule premise: the declaration is document-scoped,
/// the rule instantiates the universal over every matching datum.
fn universal_program() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         @forAll :u .\n\
         :alice :knows :bob .\n\
         :alice :knows :carol .\n\
         {{ :alice :knows :u }} => {{ :u :knownBy :alice }} .\n"
    )
}

/// The flip twin: the identical rule, the empty universe (no data).
fn universal_program_no_data() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         @forAll :u .\n\
         {{ :alice :knows :u }} => {{ :u :knownBy :alice }} .\n"
    )
}

/// `@forSome` existential bound through a rule conclusion: each firing
/// materializes the declared existential as a blank node in the closure.
fn existential_program() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         @forSome :w .\n\
         {{ ?p a :Person }} => {{ ?p :hasWitness :w }} .\n\
         :alice a :Person .\n\
         :carol a :Person .\n"
    )
}

/// The scope twin: the existential is declared inside the premise formula, so
/// the conclusion's `:w` is a plain IRI (the premise scope does not leak).
fn existential_scoped_premise_program() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         {{ @forSome :w . ?p a :Person }} => {{ ?p :hasWitness :w }} .\n\
         :alice a :Person .\n\
         :carol a :Person .\n"
    )
}

/// Skolemization visible in output: `log:skolem` maps the formula containing
/// the declared existential to its stable skolem IRI, and a rule derives that
/// IRI into the closure.
fn skolem_program() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         @prefix log: <http://www.w3.org/2000/10/swap/log#> .\n\
         @forSome :e .\n\
         {{ {{ :alice :accused :e }} log:skolem ?s }} => {{ :e :skolemIs ?s }} .\n\
         :alice :accused :e .\n"
    )
}

/// Two distinct existentials in two distinct formulas: each gets its own
/// stable skolem IRI (the anti-collapse twin of `skolem_program`).
fn skolem_two_formula_program() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         @prefix log: <http://www.w3.org/2000/10/swap/log#> .\n\
         @forSome :e1, :e2 .\n\
         {{ {{ :alice :accused :e1 }} log:skolem ?s1 }} => {{ :e1 :skolemIs ?s1 }} .\n\
         {{ {{ :alice :accused :e2 }} log:skolem ?s2 }} => {{ :e2 :skolemIs ?s2 }} .\n\
         :alice :accused :e1, :e2 .\n"
    )
}

/// Quantifier rules + facts carried in one document, for the law-step rung.
fn quantifier_rules_document() -> String {
    universal_program()
}

// ---------------------------------------------------------------------------
// Runners
// ---------------------------------------------------------------------------

/// Run the bounded N3 path and return the derived closure as a set of
/// canonical `s p o` strings (round-tripped through the N3 parser, so the
/// assertion covers the written surface, not an internal structure).
fn derived_set(text: &str) -> BTreeSet<String> {
    match reason_n3_bounded(text) {
        Ok(derived) => set_of_derived(derived),
        Err(e) => panic!("quantifier program must derive, got {e:?}"),
    }
}

fn set_of_derived(derived: String) -> BTreeSet<String> {
    let doc = n3::parse_n3(&derived, None).expect("derived text re-parses as N3");
    doc.facts
        .iter()
        .map(|t| {
            format!(
                "{} {} {}",
                t.s.canonical_form(),
                t.p.canonical_form(),
                t.o.canonical_form()
            )
        })
        .collect()
}

fn run(req: Value) -> Value {
    serde_json::from_slice(&call(req.to_string().as_bytes())).expect("GraphLaw ABI returns JSON")
}

fn ok(req: Value) -> Value {
    let out = run(req.clone());
    assert_eq!(out["ok"], true, "request {req} was refused: {out}");
    out
}

fn derived_from_abi(text: &str) -> BTreeSet<String> {
    let out = ok(json!({"op": "n3", "text": text}));
    let derived = out["derived"].as_str().expect("N3 result text");
    set_of_derived(derived.to_string())
}

/// Assert a pre-engine quantifier misuse refuses with the typed phrase.
fn assert_refused_with(text: &str, phrase: &str) {
    match reason_n3_bounded(text) {
        Err(N3Error::Refused(refusal)) => {
            assert_eq!(refusal.kind, RefusalKind::EngineRejected, "got {refusal:?}");
            assert_eq!(refusal.dialect, Some(Dialect::N3), "got {refusal:?}");
            assert_eq!(refusal.engine, Some(Dialect::N3.engine()), "got {refusal:?}");
            let message = refusal.message.as_str();
            assert!(
                message.contains(phrase),
                "refusal message must carry the typed phrase `{phrase}`, got: {message}"
            );
        }
        Err(other) => panic!("expected N3Error::Refused, got {other:?}"),
        Ok(derived) => panic!("misuse must refuse, got silent derivation: {derived:?}"),
    }
}

// ---------------------------------------------------------------------------
// Positives
// ---------------------------------------------------------------------------

/// Canonical form of an IRI term (engine-escaped, single source of truth).
fn iri(value: &str) -> String {
    n3::Term::iri(value).canonical_form()
}

/// Canonical form of a blank-node term (engine-escaped).
fn blank(value: &str) -> String {
    n3::Term::blank(value).canonical_form()
}

#[test]
fn forall_universals_drive_a_real_derivation() {
    let expected: BTreeSet<String> = [
        format!("{} {} {}", iri(&format!("{NS}bob")), iri(&format!("{NS}knownBy")), iri(&format!("{NS}alice"))),
        format!("{} {} {}", iri(&format!("{NS}carol")), iri(&format!("{NS}knownBy")), iri(&format!("{NS}alice"))),
    ]
    .into_iter()
    .collect();

    // Rung 1: bounded pre-engine path, exact derived set.
    assert_eq!(derived_set(&universal_program()), expected);

    // Rung 3: the ABI op answers the identical derivation on the wire.
    assert_eq!(derived_from_abi(&universal_program()), expected);
}

#[test]
fn forsome_existentials_bind_through_a_rule() {
    // The declared existential materializes as a blank node (`__q1`) in the
    // derived closure for every rule firing.
    let expected: BTreeSet<String> = [
        format!("{} {} {}", iri(&format!("{NS}alice")), iri(&format!("{NS}hasWitness")), blank("__q1")),
        format!("{} {} {}", iri(&format!("{NS}carol")), iri(&format!("{NS}hasWitness")), blank("__q1")),
    ]
    .into_iter()
    .collect();

    assert_eq!(derived_set(&existential_program()), expected);
    assert_eq!(derived_from_abi(&existential_program()), expected);
}

#[test]
fn skolemization_of_existentials_is_visible_in_output() {
    // The derived closure carries the skolem IRI: genid namespace + the
    // stable term hash of the existential's formula. Golden hash pinned: a
    // change here is an identity-substrate change and flips this court.
    let expected: BTreeSet<String> = [format!(
        "{} {} {}",
        blank("__q1"),
        iri(&format!("{NS}skolemIs")),
        iri(&format!("{GENID}7274e90ced1e4a2a"))
    )]
    .into_iter()
    .collect();

    assert_eq!(derived_set(&skolem_program()), expected);

    // Determinism on the wire: the same program twice answers byte-identical
    // derived text (the skolem hash is a stable function of the term, never
    // of a run counter or clock).
    let first = ok(json!({"op": "n3", "text": skolem_program()}));
    let second = ok(json!({"op": "n3", "text": skolem_program()}));
    assert_eq!(
        first["derived"], second["derived"],
        "skolem derivation must be byte-identical across runs"
    );
    assert!(
        first["derived"]
            .as_str()
            .expect("derived text")
            .contains(GENID),
        "skolem namespace must be visible in the derived output"
    );
}

/// The object IRI of a canonical `s p o` line, unescaped (the canonical form
/// escapes `:` as `\:`; the genid namespace carries only `:` as a structural
/// character, and the `starts_with(GENID)` asserts below fail loudly if that
/// ever stops holding).
fn object_iri(line: &str) -> String {
    let object = line.split(' ').nth(2).expect("canonical `s p o` shape");
    let inner = object
        .strip_prefix('<')
        .and_then(|rest| rest.strip_suffix('>'))
        .expect("object is an IRI term");
    inner.replace("\\:", ":")
}

#[test]
fn distinct_existentials_skolemize_to_distinct_iris() {
    // Anti-collapse twin: two distinct formulas (differing existentials) mint
    // two distinct skolem IRIs, each bound to its own existential blank.
    let set = derived_set(&skolem_two_formula_program());
    assert_eq!(set.len(), 2, "two formulas, two derivations, got {set:?}");

    let q1 = format!("{} ", blank("__q1"));
    let q2 = format!("{} ", blank("__q2"));
    let q1_line = set.iter().find(|l| l.starts_with(&q1)).expect("__q1 bound");
    let q2_line = set.iter().find(|l| l.starts_with(&q2)).expect("__q2 bound");

    let genid = |line: &str| -> String {
        let iri_text = object_iri(line);
        let tail = iri_text
            .strip_prefix(GENID)
            .unwrap_or_else(|| panic!("skolem IRI carries the genid namespace, got {line}"))
            .to_string();
        assert_eq!(tail.len(), 16, "skolem tail is the 16-hex term hash, got {line}");
        tail
    };
    assert_ne!(
        genid(q1_line),
        genid(q2_line),
        "distinct existentials must skolemize to distinct IRIs, got {set:?}"
    );
}

// ---------------------------------------------------------------------------
// Flip twins
// ---------------------------------------------------------------------------

#[test]
fn forall_flip_empty_universe_derives_nothing() {
    // The identical rule over the empty universe derives the empty set —
    // `Ok("")`, a successful run with nothing fabricated. Flip of
    // `forall_universals_drive_a_real_derivation`: adding the two data facts
    // flips the derived set from empty to the two knownBy facts.
    let derived = reason_n3_bounded(&universal_program_no_data())
        .expect("the empty universe is a successful run, not an error");
    assert!(
        derived.trim().is_empty(),
        "an empty universe must derive nothing, got: {derived:?}"
    );
    assert_eq!(derived_set(&universal_program_no_data()), BTreeSet::new());

    // ABI rung: ok:true with an empty derived text (never a fabricated fact).
    let out = ok(json!({"op": "n3", "text": universal_program_no_data()}));
    assert!(
        out["derived"].as_str().expect("derived text").trim().is_empty(),
        "ABI empty universe derives nothing, got {out}"
    );
}

#[test]
fn forsome_scope_flip_premise_scope_does_not_bind_the_conclusion() {
    // Flip of `forsome_existentials_bind_through_a_rule`: the same rule shape
    // with the existential declared inside the premise formula. The premise
    // scope does not leak, so the conclusion's `:w` is a plain ground IRI in
    // the derived facts — not the `_:__q1` blank of the document-scoped twin.
    let expected: BTreeSet<String> = [
        format!("{} {} {}", iri(&format!("{NS}alice")), iri(&format!("{NS}hasWitness")), iri(&format!("{NS}w"))),
        format!("{} {} {}", iri(&format!("{NS}carol")), iri(&format!("{NS}hasWitness")), iri(&format!("{NS}w"))),
    ]
    .into_iter()
    .collect();
    let set = derived_set(&existential_scoped_premise_program());
    assert_eq!(set, expected, "conclusion :w is ground when the existential is premise-scoped");
    assert!(
        !set.iter().any(|line| line.contains("__q")),
        "no quantifier-minted blank may reach the conclusion, got {set:?}"
    );
}

// ---------------------------------------------------------------------------
// Typed refusals (misuse)
// ---------------------------------------------------------------------------

#[test]
fn quantifier_misuse_refuses_typed() {
    // The four `parse_quantifier` refusal arms, each pinned by its stable
    // phrase, plus the empty quantifier list (a `@forAll`/`@forSome` naming
    // no variable at all).
    let misuses: &[(&str, &str, &str)] = &[
        (
            "quantifier after the first triple of its graph",
            "@prefix : <http://example.org/q/> .\n:s :p :o .\n@forAll :x .\n",
            "quantifiers must precede the first triple of their graph",
        ),
        (
            "@forAll after @forSome in one graph",
            "@prefix : <http://example.org/q/> .\n@forSome :e .\n@forAll :u .\n{s :p :o} => {s :q :o} .\n",
            "@forAll must precede @forSome in the same graph",
        ),
        (
            "quantifier with a non-IRI operand",
            "@prefix : <http://example.org/q/> .\n@forAll 5 .\n",
            "quantifier requires an IRI or prefixed name",
        ),
        (
            "empty quantifier list (@forAll)",
            "@prefix : <http://example.org/q/> .\n@forAll .\n",
            "quantifier requires an IRI or prefixed name",
        ),
        (
            "empty quantifier list (@forSome)",
            "@prefix : <http://example.org/q/> .\n@forSome .\n",
            "quantifier requires an IRI or prefixed name",
        ),
        (
            "the same IRI quantified twice",
            "@prefix : <http://example.org/q/> .\n@forAll :x .\n@forAll :x .\n",
            "IRI is already quantified in this graph",
        ),
    ];
    for (label, text, phrase) in misuses {
        assert_refused_with(text, phrase);
        // ABI rung: the refusal crosses the wire with its typed envelope.
        let out = run(json!({"op": "n3", "text": text}));
        assert_eq!(out["ok"], false, "{label} must refuse, got {out}");
        assert_eq!(out["error"]["kind"], "EngineRejected", "{label}: {out}");
        assert_eq!(out["error"]["dialect"], "N3", "{label}: {out}");
        assert_eq!(out["error"]["engine"], "PurRdf", "{label}: {out}");
        let message = out["error"]["message"].as_str().expect("refusal message");
        assert!(
            message.contains(*phrase),
            "{label}: message must carry `{phrase}`, got: {message}"
        );
    }
}

// ---------------------------------------------------------------------------
// Law-step rung: Step::DeriveN3
// ---------------------------------------------------------------------------

#[test]
fn derive_n3_admits_quantifier_rules_on_an_empty_data_universe() {
    let rules = quantifier_rules_document();
    let ruleset = validate_n3_rules(rules.as_bytes(), None).expect("quantifier rules admit as N3");
    let state = LawState::parse(b"", Dialect::Turtle, None).expect("empty state admits");

    let (_next, receipt) = state
        .transition(&Step::DeriveN3 {
            rules: &rules,
            ruleset: &ruleset,
        })
        .expect("quantifier derivation admits at the law step");
    assert_eq!(
        receipt.added, 2,
        "both universal instantiations land in the new state"
    );
}

#[test]
fn derive_n3_refuses_quantifier_rules_over_a_non_empty_data_universe() {
    // The law step merges state (N-Quads, first) with the rules text: a
    // quantifier declaration in the rules then follows the first triple of
    // the merged graph, which N3 refuses. The empty data universe is
    // therefore an admission precondition of this path, and violating it is
    // a typed EngineRejected — never a silent mis-scope.
    let rules = quantifier_rules_document();
    let ruleset = validate_n3_rules(rules.as_bytes(), None).expect("quantifier rules admit as N3");
    let data = format!("@prefix : <{NS}> .\n:alice :knows :bob .\n");
    let state = LawState::parse(data.as_bytes(), Dialect::Turtle, None).expect("data admits");

    match state.transition(&Step::DeriveN3 {
        rules: &rules,
        ruleset: &ruleset,
    }) {
        Err(graphlaw::law::LawError::Refused(refusal)) => {
            assert_eq!(refusal.kind, RefusalKind::EngineRejected, "got {refusal:?}");
            assert_eq!(refusal.dialect, Some(Dialect::N3), "got {refusal:?}");
            assert!(
                refusal.message.contains("quantifiers must precede the first triple of their graph"),
                "merge-order violation carries the quantifier refusal, got: {refusal:?}"
            );
        }
        Err(other) => panic!("expected LawError::Refused, got {other:?}"),
        Ok(_) => panic!("quantifier rules over a non-empty state must refuse the DeriveN3 step"),
    }
}

#[test]
fn derive_n3_wraps_a_quantifier_admission_refusal_as_tier_mismatch() {
    // A rules text that fails N3 admission is refused before any derivation
    // runs, with the typed TierMismatch wrap naming the underlying phrase.
    // (The carried RuleSet only feeds the rule-count gate, so any admitting
    // document with the same rule count satisfies it — the wrap here comes
    // from the text's own admission failure.)
    let bad = "@prefix : <http://example.org/q/> .\n:s :p :o .\n@forAll :x .\n".to_string();
    assert!(
        validate_n3_rules(bad.as_bytes(), None).is_err(),
        "quantifier-after-triple must refuse at N3 admission"
    );
    let ruleset = validate_n3_rules(b"@prefix : <http://example.org/q/> .\n", None)
        .expect("a bare-prefix document admits");
    assert_eq!(ruleset.rule_count(), 0, "same rule count as the misusing text");
    let state = LawState::parse(b"", Dialect::Turtle, None).expect("empty state admits");

    match state.transition(&Step::DeriveN3 {
        rules: &bad,
        ruleset: &ruleset,
    }) {
        Err(graphlaw::law::LawError::Refused(refusal)) => {
            assert_eq!(refusal.kind, RefusalKind::TierMismatch, "got {refusal:?}");
            let message = refusal.message.as_str();
            assert!(
                message.contains("DeriveN3 rules failed N3 admission")
                    && message.contains("quantifiers must precede the first triple of their graph"),
                "wrap names the admission failure and the phrase, got: {message}"
            );
        }
        Err(other) => panic!("expected LawError::Refused(TierMismatch), got {other:?}"),
        Ok(_) => panic!("an admission-failing rules text must refuse the DeriveN3 step"),
    }
}
