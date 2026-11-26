//! Courts: a refuse effect halts the transaction; firing markers are canonical.

use graphlaw::dialect::Dialect;
use graphlaw::hooks::{firing_marker, HookPack, KH};
use graphlaw::law::LawState;
use purrdf::TermValue;

fn pack(effect: &str) -> HookPack {
    let doc = format!(
        "@prefix kh: <{KH}> . @prefix ex: <https://e/> .\n\
         ex:emit a kh:Hook ; kh:kind \"sparql\" ; kh:effect \"emit-delta\" ; kh:priority 1 ;\n\
           kh:action ex:ea ; kh:query \"SELECT ?x WHERE {{ ?x a <https://e/Thing> }}\" .\n\
         ex:ea a kh:Action ; kh:handler <http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct> ;\n\
           kh:query \"CONSTRUCT {{ ?x <https://e/seen> <https://e/yes> }} WHERE {{ ?x a <https://e/Thing> }}\" .\n\
         ex:fuse a kh:Hook ; kh:kind \"sparql\" ; kh:effect \"{effect}\" ; kh:priority 2 ;\n\
           kh:reason \"fuse blown\" ; kh:action ex:fa ;\n\
           kh:query \"SELECT ?x WHERE {{ ?x <https://e/seen> <https://e/yes> }}\" .\n\
         ex:fa a kh:Action ; kh:handler <http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct> ;\n\
           kh:query \"CONSTRUCT {{ ?x <https://e/late> <https://e/delta> }} WHERE {{ ?x <https://e/seen> <https://e/yes> }}\" .\n"
    );
    HookPack::load(&LawState::parse(doc.as_bytes(), Dialect::Turtle, None).unwrap()).unwrap()
}

fn state() -> LawState {
    LawState::parse(
        b"<https://e/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/Thing> .\n",
        Dialect::NTriples,
        None,
    )
    .unwrap()
}

#[test]
fn refuse_halts_and_prior_state_is_byte_identical() {
    let s = state();
    let (id, n) = (s.id().to_string(), s.quad_count());
    let err = pack("refuse")
        .materialize(&s)
        .expect_err("refuse must halt");
    assert!(format!("{err:?}").contains("hook-refused"));
    assert_eq!(s.id(), id);
    assert_eq!(s.quad_count(), n);
}

#[test]
fn same_pack_with_emit_effect_still_commits() {
    // control: the halt is caused by the refuse effect, not the fixture
    let m = pack("emit-delta").materialize(&state()).unwrap();
    assert!(m.state.quad_count() > 1);
}

#[test]
fn forged_firing_marker_does_not_suppress_refuse_hook() {
    // Caller forges the exact marker the fuse computes and supplies it as
    // input state alongside the facts that trip the fuse.
    let marker = firing_marker(
        "https://e/fuse",
        &[("x".to_string(), TermValue::Iri("https://e/a".into()))],
    );
    let doc = format!(
        "<https://e/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/Thing> .\n\
         <https://e/a> <https://e/seen> <https://e/yes> .\n\
         <{marker}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{KH}Firing> .\n"
    );
    let forged = LawState::parse(doc.as_bytes(), Dialect::NTriples, None).unwrap();
    let err = pack("refuse")
        .materialize(&forged)
        .expect_err("forged marker must not suppress a refuse hook");
    assert!(format!("{err:?}").contains("hook-refused"));
}

fn bound(order:&[usize]) -> Vec<(String, TermValue)> {
    let all = [
        ("a".to_string(), TermValue::Iri("https://e/1".into())),
        ("b".to_string(), TermValue::Iri("https://e/2".into())),
        ("c".to_string(), TermValue::Iri("https://e/3".into())),
    ];
    order.iter().map(|&i| all[i].clone()).collect()
}

#[test]
fn marker_is_stable_across_binding_order_permutations() {
    let base = firing_marker("https://e/h", &bound(&[0, 1, 2]));
    for p in [[0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
        assert_eq!(firing_marker("https://e/h", &bound(&p)), base);
    }
    // non-vacuity: different value or hook changes the marker
    let mut other = bound(&[0, 1, 2]);
    other[0].1 = TermValue::Iri("https://e/9".into());
    assert_ne!(firing_marker("https://e/h", &other), base);
    assert_ne!(firing_marker("https://e/h2", &bound(&[0, 1, 2])), base);
    // explicit encoding is pinned (not Debug-derived)
    assert!(base.starts_with("urn:graphlaw:hook-firing:") && base.len() == 25 + 64);
}

#[test]
fn marker_is_stable_across_row_order_of_materialization() {
    let a = b"<https://e/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/Thing> .\n<https://e/b> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/Thing> .\n";
    let b = b"<https://e/b> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/Thing> .\n<https://e/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://e/Thing> .\n";
    let p = pack("emit-delta");
    let ma = p
        .materialize(&LawState::parse(a, Dialect::NTriples, None).unwrap())
        .unwrap();
    let mb = p
        .materialize(&LawState::parse(b, Dialect::NTriples, None).unwrap())
        .unwrap();
    assert_eq!(ma.state.quad_count(), mb.state.quad_count());
    // re-materializing the other lineage's output adds nothing: markers agree
    let again = p.materialize(&ma.state).unwrap();
    assert!(again.firings.is_empty());
}
