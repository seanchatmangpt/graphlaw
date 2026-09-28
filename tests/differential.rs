//! Cross-engine differential oracle.
//!
//! N3 (Eyeron), Datalog (PurRDF) and OWL 2 RL (PurRDF) each own a rule
//! semantics, and no single upstream tests the overlap. The transitive
//! closure of an edge relation is inside all three; any disagreement is a
//! defect in an engine or in a pin, and fails here.

use std::collections::BTreeSet;

use graphlaw::datalog::{
    clause::{ClauseAtom, ClauseTerm, DlClause},
    seminaive::{compile, evaluate},
    store::RelationStore,
};
use graphlaw::entailment::Materialization;

const NS: &str = "https://example.org/";

fn iri(n: usize) -> String {
    format!("{NS}n{n}")
}

/// Deterministic edge sets, cycles and self-loops included.
fn edges(seed: u64, nodes: usize, count: usize) -> BTreeSet<(usize, usize)> {
    let mut x = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let mut out = BTreeSet::new();
    for _ in 0..count {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let a = (x >> 33) as usize % nodes;
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let b = (x >> 33) as usize % nodes;
        out.insert((a, b));
    }
    out
}

fn nt_edges(es: &BTreeSet<(usize, usize)>) -> String {
    es.iter()
        .map(|(a, b)| format!("<{}> <{NS}parent> <{}> .\n", iri(*a), iri(*b)))
        .collect()
}

/// Pairs related by `ex:ancestor` in an RDF dataset, via PurRDF N-Triples output.
fn ancestor_pairs(ds: &graphlaw::rdf::RdfDataset) -> BTreeSet<(String, String)> {
    let bytes = graphlaw::rdf::serialize_dataset(
        ds,
        "application/n-triples",
        graphlaw::rdf::SerializeGraph::Dataset,
    )
    .unwrap();
    String::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter_map(|l| {
            let mut t = l.split_whitespace();
            let (s, p, o) = (t.next()?, t.next()?, t.next()?);
            (p == format!("<{NS}ancestor>")).then(|| (s.to_string(), o.to_string()))
        })
        .collect()
}

fn n3_closure(es: &BTreeSet<(usize, usize)>) -> BTreeSet<(String, String)> {
    let doc = format!(
        "{}\n{{ ?x <{NS}parent> ?y }} => {{ ?x <{NS}ancestor> ?y }} .\n\
         {{ ?x <{NS}ancestor> ?y . ?y <{NS}parent> ?z }} => {{ ?x <{NS}ancestor> ?z }} .\n",
        nt_edges(es)
    );
    let out = graphlaw::n3::reason(&doc).expect("eyeron closure");
    let ds = graphlaw::rdf::parse_dataset(out.as_bytes(), "text/turtle", None).unwrap();
    ancestor_pairs(&ds)
}

fn owlrl_closure(es: &BTreeSet<(usize, usize)>) -> BTreeSet<(String, String)> {
    let doc = format!(
        "{}<{NS}parent> <http://www.w3.org/2000/01/rdf-schema#subPropertyOf> <{NS}ancestor> .\n\
         <{NS}ancestor> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#TransitiveProperty> .\n",
        nt_edges(es)
    );
    let ds = graphlaw::rdf::parse_dataset(doc.as_bytes(), "application/n-triples", None).unwrap();
    let (closure, _) = graphlaw::entailment::materialize(ds.as_ref(), Materialization::OwlRl)
        .expect("owl-rl closure");
    ancestor_pairs(closure.as_ref())
}

fn datalog_closure(es: &BTreeSet<(usize, usize)>, nodes: usize) -> BTreeSet<(String, String)> {
    let anc = format!("{NS}ancestor");
    let par = format!("{NS}parent");
    let rules = vec![
        DlClause::datalog(
            ClauseAtom::positive(ClauseTerm::var("x"), &anc, ClauseTerm::var("y")),
            vec![ClauseAtom::positive(
                ClauseTerm::var("x"),
                &par,
                ClauseTerm::var("y"),
            )],
        ),
        DlClause::datalog(
            ClauseAtom::positive(ClauseTerm::var("x"), &anc, ClauseTerm::var("z")),
            vec![
                ClauseAtom::positive(ClauseTerm::var("x"), &anc, ClauseTerm::var("y")),
                ClauseAtom::positive(ClauseTerm::var("y"), &par, ClauseTerm::var("z")),
            ],
        ),
    ];
    let exe = compile(rules).expect("safe stratified");
    let mut facts = RelationStore::new();
    for (a, b) in es {
        facts.insert(
            &format!("<{}>", iri(*a)),
            &format!("<{par}>"),
            &format!("<{}>", iri(*b)),
            RelationStore::DEFAULT_GRAPH,
        );
    }
    let ev = evaluate(&exe, facts).expect("fixpoint");
    let mut out = BTreeSet::new();
    for a in 0..nodes {
        for b in 0..nodes {
            let (s, o) = (format!("<{}>", iri(a)), format!("<{}>", iri(b)));
            if ev
                .facts()
                .contains(&s, &format!("<{anc}>"), &o, RelationStore::DEFAULT_GRAPH)
            {
                out.insert((s, o));
            }
        }
    }
    out
}

/// Ground truth by Warshall, independent of every engine under test.
fn warshall(es: &BTreeSet<(usize, usize)>, nodes: usize) -> BTreeSet<(String, String)> {
    let mut r = vec![vec![false; nodes]; nodes];
    for (a, b) in es {
        r[*a][*b] = true;
    }
    for k in 0..nodes {
        for i in 0..nodes {
            for j in 0..nodes {
                if r[i][k] && r[k][j] {
                    r[i][j] = true;
                }
            }
        }
    }
    (0..nodes)
        .flat_map(|i| (0..nodes).map(move |j| (i, j)))
        .filter(|(i, j)| r[*i][*j])
        .map(|(i, j)| (format!("<{}>", iri(i)), format!("<{}>", iri(j))))
        .collect()
}

#[test]
fn n3_datalog_and_owlrl_agree_on_transitive_closure() {
    for seed in 0..12 {
        let nodes = 6;
        let es = edges(seed, nodes, 3 + (seed as usize % 7));
        let truth = warshall(&es, nodes);
        assert!(!truth.is_empty());
        assert_eq!(
            datalog_closure(&es, nodes),
            truth,
            "datalog, seed {seed}, edges {es:?}"
        );
        assert_eq!(n3_closure(&es), truth, "eyeron, seed {seed}, edges {es:?}");
        assert_eq!(
            owlrl_closure(&es),
            truth,
            "owl-rl, seed {seed}, edges {es:?}"
        );
    }
}
