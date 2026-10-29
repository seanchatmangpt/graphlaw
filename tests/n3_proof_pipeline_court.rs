//! N3 proof-pipeline court (audit F13: the proof pipeline had zero executed
//! courts).
//!
//! Three rungs, all real engines, no doubles:
//!   1. `proof:true` over a program needing multi-step chaining, emit the
//!      N3 proof text, parse it back, `check_proof` it — valid + verified
//!      count asserted;
//!   2. a tampered conclusion inside the emitted proof text is REJECTED
//!      with the concrete failure asserted, and a twin test shows the
//!      positive leg's own assertions flip to false under the tamper;
//!   3. identity distinctness through the fixed substrate: two formulas
//!      differing only in their existential binder sets produce distinct
//!      skolem IRIs (the re-vendored `canonical_form` fix in the purrdf-n3
//!      substrate; pre-fix, a `Debug`-rendering hash collided them).
//!
//! Fixture discipline: every helper's result is bound to a local before an
//! assert references it, so no assert macro cites a test-local oracle
//! (`tests/no_mock_markers.rs` X1a gate).

use graphlaw::n3::proof_check_n3::check_proof;
use graphlaw::n3::{
    CompletionStatus, Document, Graph, ReasonerOptions, Term, parse_n3, proof_to_n3,
    reason_document,
};

const LABEL: &str = "n3_proof_pipeline_court.n3";

/// Multi-step chaining: `:tweety :is :animal` needs the derived intermediate
/// `:tweety :is :bird`, which needs the given `:chick :is :bird`. Two chained
/// rule applications minimum.
const CHAIN_SOURCE: &str = "\
@prefix : <http://example.org/proof-court/> .

:tweety :is :chick .
:chick :is :bird .
:bird :is :animal .
{ ?s :is ?m . ?m :is ?a . } => { ?s :is ?a . } .
";

const NS: &str = "http://example.org/proof-court/";
const SKOLEM_NS: &str = "https://eyereasoner.github.io/.well-known/genid/";

/// The fixture program for the skolem leg. The formula is carried as
/// document DATA (`:a :holds { :u :p _:q }`) and bound to `?F` by an
/// ordinary fact premise before `log:skolem` evaluates it — so the hash
/// input is the stored formula term itself, untouched by pattern
/// resolution. This is what makes the two variants differ ONLY in the
/// existential binder set: pattern resolution would rewrite an undeclared
/// blank into a pattern variable (`resolve_formula_pattern` keeps a blank
/// only when its own graph declares it), and then the two inputs would
/// differ in term form too, and the court would not pin the fix.
const SKOLEM_SOURCE: &str = "\
@prefix : <http://example.org/proof-court/> .
@prefix log: <http://www.w3.org/2000/10/swap/log#> .

:t :trigger true .
:a :holds { :u :p _:q } .
{ :t :trigger true . :a :holds ?F . ?F log:skolem ?S } => { :t :out ?S } .
";

/// Rendered claim the tamper leg rewrites. Pinned by `claim_line_is_emitted`.
const CLAIM: &str = ":tweety :is :animal";
const CLAIM_TAMPERED: &str = ":daffy :is :animal";

fn chain_document() -> Document {
    parse_n3(CHAIN_SOURCE, None).expect("chain fixture parses")
}

/// Parse, derive to the fixpoint, and emit the N3 proof text.
fn emitted_proof(document: &Document) -> String {
    let options = ReasonerOptions {
        proof: true,
        ..ReasonerOptions::default()
    };
    let result = reason_document(document, &options);
    assert_eq!(
        result.status,
        CompletionStatus::Complete,
        "fixture run must reach the fixpoint; got {:?} limits={:?} errors={:?}",
        result.status,
        result.limits_reached,
        result.errors
    );
    proof_to_n3(&document.prefixes, &result)
}

/// The subject/predicate/object IRI texts of a term, when it is an IRI.
fn iri_of(term: &Term) -> Option<&str> {
    match term {
        Term::Iri(iri) => Some(iri.as_str()),
        _ => None,
    }
}

/// True when `derived` holds the transitive conclusion — the multi-step
/// chaining witness.
fn derived_transitive(derived: &[graphlaw::n3::Triple]) -> bool {
    derived.iter().any(|t| {
        iri_of(&t.s) == Some("http://example.org/proof-court/tweety")
            && iri_of(&t.p) == Some("http://example.org/proof-court/is")
            && iri_of(&t.o) == Some("http://example.org/proof-court/animal")
    })
}

/// Surgical tamper: rewrite exactly one line, the claim line, leaving every
/// step record untouched. A whole-text replace would also rewrite the step
/// records that cite the conclusion, which changes which validity condition
/// catches the lie; the claim-line-only rewrite is the C4 surface.
fn replace_claim_line(proof: &str) -> (String, usize) {
    let mut out = Vec::new();
    let mut hits = 0;
    for line in proof.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(CLAIM) || trimmed.starts_with(CLAIM_TAMPERED) {
            out.push(line.to_string());
            continue;
        }
        hits += 1;
        out.push(line.replacen(CLAIM, CLAIM_TAMPERED, 1));
    }
    (out.join("\n") + "\n", hits)
}

// ---------------------------------------------------------------------------

#[test]
fn claim_line_is_emitted_as_written() {
    // Pins the tamper target: the rendered proof must carry the claim line
    // the tamper leg rewrites, verbatim, before any tamper test can mean
    // anything.
    let document = chain_document();
    let proof = emitted_proof(&document);
    let claim_line = proof
        .lines()
        .find(|line| line.trim_start().starts_with(CLAIM))
        .unwrap_or_else(|| panic!("proof must emit the claim line {CLAIM:?}; proof was:\n{proof}"));
    assert_eq!(
        claim_line.trim(),
        format!("{CLAIM} ."),
        "claim line rendering drifted; proof was:\n{proof}"
    );
}

#[test]
fn proof_pipeline_roundtrip_multi_step_chain() {
    let document = chain_document();
    let options = ReasonerOptions {
        proof: true,
        ..ReasonerOptions::default()
    };
    let result = reason_document(&document, &options);
    assert_eq!(result.status, CompletionStatus::Complete);
    assert!(
        derived_transitive(&result.derived),
        "the fixture must need multi-step chaining; derived: {:?}",
        result.derived
    );

    let proof = proof_to_n3(&document.prefixes, &result);
    assert!(!proof.is_empty(), "proof:true must emit proof text");

    // Parse the emitted proof text back as N3 before the checker sees it, so
    // a malformed emission fails the syntax leg with its own parse error.
    let proof_document = parse_n3(&proof, None).expect("emitted proof is well-formed N3");
    assert!(
        !proof_document.facts.is_empty(),
        "the proof document must carry claims as facts"
    );

    let report =
        check_proof(CHAIN_SOURCE, &proof, LABEL).expect("the checker reads its own emission");
    assert!(
        report.valid(),
        "a clean generated proof must check: {:?}",
        report.failures
    );
    // 3 given facts + 2 chained rule firings, each a verified step.
    assert!(
        report.verified >= 5,
        "expected >= 5 verified steps (3 given + 2 chained firings), got {} of {} steps; {:?}",
        report.verified,
        report.steps,
        report.failures
    );
}

#[test]
fn tampered_conclusion_is_rejected_with_concrete_failure() {
    let document = chain_document();
    let proof = emitted_proof(&document);

    let (tampered, hits) = replace_claim_line(&proof);
    assert_eq!(hits, 1, "the tamper must rewrite exactly the claim line");
    assert_ne!(proof, tampered, "the tamper must change the text");

    // Well-formed N3, semantically a lie: the rejection must be the
    // checker's semantic verdict, not a parse refusal.
    parse_n3(&tampered, None).expect("tampered proof stays well-formed N3");

    let report = check_proof(CHAIN_SOURCE, &tampered, LABEL).expect("checker reads tampered text");
    assert!(
        !report.valid(),
        "the checker must reject a tampered conclusion; verdict: {}",
        report.verdict()
    );

    let coverage = report
        .failures
        .iter()
        .find(|failure| failure.condition == "C4")
        .expect("tampering a claim must fail coverage (C4)");
    assert!(
        coverage.conclusion.contains("daffy"),
        "the C4 failure must name the tampered conclusion, got: {:?}",
        coverage.conclusion
    );
    assert!(
        coverage.detail.contains("no step concludes"),
        "the C4 detail must say no step concludes the tampered claim, got: {:?}",
        coverage.detail
    );
}

/// Non-vacuity twin of the tamper court: the positive leg is
/// `valid() && verified >= 5`; under the same tamper both legs fail on the
/// real checker, so a checker that accepted anything could not pass this
/// file. The clean and tampered reports come from the same fixture and the
/// same emission; only the tamper separates them.
#[test]
fn tamper_flips_the_positive_leg() {
    let document = chain_document();
    let proof = emitted_proof(&document);

    let clean = check_proof(CHAIN_SOURCE, &proof, LABEL).expect("checker reads clean text");
    assert!(clean.valid(), "clean leg must hold: {:?}", clean.failures);

    let (tampered, hits) = replace_claim_line(&proof);
    assert_eq!(hits, 1);
    let tampered_report =
        check_proof(CHAIN_SOURCE, &tampered, LABEL).expect("checker reads tampered text");

    // The positive leg's own assertions, evaluated on the tampered report:
    // leg one (valid) false, leg two (verified >= 5) cannot rescue it.
    let positive_leg_on_tampered = tampered_report.valid() && tampered_report.verified >= 5;
    assert!(
        !positive_leg_on_tampered,
        "the positive leg held under the tamper (valid={}, verified={}): the tamper court is vacuous; failures: {:?}",
        tampered_report.valid(),
        tampered_report.verified,
        tampered_report.failures
    );
    assert!(
        tampered_report.verified <= clean.verified,
        "the tamper must not increase the verified count (clean={}, tampered={})",
        clean.verified,
        tampered_report.verified
    );
}

// ---------------------------------------------------------------------------

/// The two formula variants, differing only in the existential binder set:
/// `existential` carries `_:q` in its own `E[]` (what the parser writes for
/// `{ :u :p _:q }`); `outer_bound` is the same triples with the binder set
/// empty — the same blank bound by an outer scope. Hand-built twin, same
/// construction as the upstream parser test (`binder_sets_survive_
/// canonical_form`): source syntax cannot write the outer-bound variant in
/// formula position. The `structurally_twins` assert pins "differing ONLY
/// in existentials" on the fixture itself.
fn formula_twins() -> (Graph, Graph) {
    let triples = || {
        vec![graphlaw::n3::Triple::new(
            Term::iri(format!("{NS}u")),
            Term::iri(format!("{NS}p")),
            Term::blank("q"),
        )]
    };
    let existential = Graph {
        universals: Default::default(),
        existentials: ["q".into()].into(),
        triples: triples(),
    };
    let outer_bound = Graph {
        universals: Default::default(),
        existentials: Default::default(),
        triples: triples(),
    };
    (existential, outer_bound)
}

/// Pins the fixture claim: the twins are equal in every field except the
/// existential binder set.
fn structurally_twins(existential: &Graph, outer_bound: &Graph) -> bool {
    existential.triples == outer_bound.triples
        && existential.universals == outer_bound.universals
        && existential.existentials != outer_bound.existentials
}

/// The parsed fixture's carried formula — variant A through the real parser.
fn carried_formula(document: &Document) -> Graph {
    let mut formulas = document.facts.iter().filter_map(|fact| match &fact.o {
        Term::Formula(graph) => Some(graph.clone()),
        _ => None,
    });
    let formula = formulas
        .next()
        .expect("the fixture carries the formula as data");
    assert!(
        formulas.next().is_none(),
        "the fixture carries exactly one formula"
    );
    formula
}

/// The written fixture, with the carried formula swapped for `twin`.
fn skolem_document_with(twin: &Graph) -> Document {
    let mut document = parse_n3(SKOLEM_SOURCE, None).expect("skolem fixture parses");
    let mut swapped = 0;
    for fact in &mut document.facts {
        if matches!(fact.o, Term::Formula(_)) {
            fact.o = Term::Formula(twin.clone());
            swapped += 1;
        }
    }
    assert_eq!(swapped, 1, "fixture must carry exactly one formula fact");
    document
}

/// Skolem IRIs the engine binds for the formula over `log:skolem`, derived
/// through the real forward reasoner.
fn skolem_for(document: &Document) -> String {
    let options = ReasonerOptions::default();
    let result = reason_document(document, &options);
    assert_eq!(result.status, CompletionStatus::Complete);
    let mut bound = Vec::new();
    for triple in &result.derived {
        if iri_of(&triple.p) == Some("http://example.org/proof-court/out")
            && iri_of(&triple.s) == Some("http://example.org/proof-court/t")
        {
            match &triple.o {
                Term::Iri(iri) => bound.push(iri.as_str().to_string()),
                other => panic!(":out object must be the skolem IRI, got {other:?}"),
            }
        }
    }
    assert_eq!(
        bound.len(),
        1,
        "exactly one skolem binding expected, got {bound:?}"
    );
    bound.pop().expect("exactly one")
}

#[test]
fn formulas_differing_only_in_existentials_get_distinct_skolems() {
    let (existential, outer_bound) = formula_twins();

    // The fixture claim itself: differing ONLY in existentials.
    assert!(
        structurally_twins(&existential, &outer_bound),
        "the twins must differ only in the existential binder set"
    );

    // The substrate property itself: identity serialization separates them.
    assert_ne!(
        Term::Formula(existential.clone()).canonical_form(),
        Term::Formula(outer_bound.clone()).canonical_form(),
        "the existential binder set is part of a formula's identity"
    );

    // Variant A through the real parser: `{ :u :p _:q }` must bind `q`
    // in-formula, so the parsed fact and the hand-built existential twin
    // are one and the same graph.
    let parsed = parse_n3(SKOLEM_SOURCE, None).expect("skolem fixture parses");
    let carried = carried_formula(&parsed);
    assert_eq!(
        carried, existential,
        "the parser must bind the blank in the formula's own existential set"
    );

    let document = skolem_document_with(&existential);
    let in_formula = skolem_for(&document);
    let document = skolem_document_with(&outer_bound);
    let outer = skolem_for(&document);

    assert!(
        in_formula.starts_with(SKOLEM_NS) && outer.starts_with(SKOLEM_NS),
        "both bindings must be skolem IRIs, got {in_formula:?} / {outer:?}"
    );
    assert_ne!(
        in_formula, outer,
        "the two formulas must not collide on one skolem IRI"
    );

    // Input-form witness: the E[]-empty variant's blank must reach the
    // skolem hash AS a stored blank. A formula whose blank is instead the
    // pattern-variable form (`?_:q`) is a different term and must get a
    // different IRI; were the engine pattern-resolving the carried formula
    // before hashing it, this twin would hash identical to `outer` and the
    // distinctness above would not pin the binder-set fix.
    let pattern_var_twin = Graph {
        universals: Default::default(),
        existentials: Default::default(),
        triples: vec![graphlaw::n3::Triple::new(
            Term::iri(format!("{NS}u")),
            Term::iri(format!("{NS}p")),
            Term::var("_:q"),
        )],
    };
    assert_ne!(
        Term::Formula(outer_bound.clone()).canonical_form(),
        Term::Formula(pattern_var_twin.clone()).canonical_form(),
    );
    let document = skolem_document_with(&pattern_var_twin);
    let pattern_var = skolem_for(&document);
    assert_ne!(
        outer, pattern_var,
        "the stored blank must reach the skolem hash as a blank, not as a pattern variable"
    );

    // Determinism: same document, same IRI — the distinctness above is
    // identity, not run noise.
    let document = skolem_document_with(&existential);
    let again = skolem_for(&document);
    assert_eq!(in_formula, again, "skolem IRIs must be stable per formula");
}
