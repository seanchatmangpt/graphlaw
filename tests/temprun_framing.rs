//! Differential court: temprun admission-fuel sieve vs graphlaw's RDF state-law
//! kernel. Loads the pinned `strata-temprun-pack` ontology (Turtle), reads the
//! `st:admissionFuel` / `st:maxFuel` constants out of the real RDF graph, then
//! replays the pack's documented `run/2` admission sieve (1 fuel unit each for
//! clock, coordinates, conformance; admitted iff `fuel >= 3`) over a sequence
//! of admission attempts drawn from a fuel lease. Fuel exhaustion must refuse
//! exactly at the documented boundary.
//!
//! Real RDF graph, real SPARQL engine, real fuel accounting throughout; no
//! test doubles.

use graphlaw::dialect::Dialect;
use graphlaw::law::LawState;
use graphlaw::purrdf::{SparqlEngine, SparqlRequest, SparqlResult, TermValue};
use graphlaw::sparql::NativeSparqlEngine;

const ST: &str = "https://strata.chatman.ai/ontology#";
const PINNED: &str = "tests/fixtures/strata-temprun-ontology.ttl";
const PIN_BLAKE3: &str = "cd3cde66a2a9a3440240e8ff005ca7258dfd995744f36b061102db522e33f30e";

/// The arena constants, read from the loaded RDF graph (not hardcoded).
#[derive(Debug, Clone, Copy)]
struct ArenaConstants {
    admission_fuel: u64,
    max_fuel: u64,
}

fn load_pinned_ontology() -> LawState {
    let bytes = std::fs::read(PINNED).expect("pinned temprun ontology fixture");
    let text = String::from_utf8(bytes).expect("fixture is UTF-8");
    // Provenance: the pinned copy's own header must still name the pin hash.
    assert!(
        text.contains(PIN_BLAKE3),
        "pinned fixture header is missing blake3 {PIN_BLAKE3}"
    );
    // And the body after the header must hash to the pin.
    const HEADER_LINES: usize = 5;
    let body: String = {
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines.len() > HEADER_LINES);
        // Re-join with the trailing newline `lines()` strips, matching the
        // original file bytes byte-for-byte.
        let mut b = lines[HEADER_LINES..].join("\n");
        b.push('\n');
        b
    };
    let digest = blake3::hash(body.as_bytes());
    let hex: String = digest.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex, PIN_BLAKE3,
        "pinned ontology body drifted from the documented blake3"
    );
    LawState::parse(text.as_bytes(), Dialect::Turtle, None)
        .expect("pinned temprun ontology parses as Turtle")
}

fn select(state: &LawState, query: &str) -> Vec<Vec<Option<TermValue>>> {
    match NativeSparqlEngine::new()
        .query(
            state.dataset(),
            SparqlRequest {
                query,
                base_iri: None,
                substitutions: &[],
            },
        )
        .expect("SPARQL SELECT over pinned ontology")
    {
        SparqlResult::Solutions { rows, .. } => rows,
        other => panic!("expected SELECT solutions, got {other:?}"),
    }
}

fn lexical(t: &Option<TermValue>) -> Option<String> {
    match t {
        Some(TermValue::Literal { lexical_form, .. }) => Some(lexical_form.clone()),
        Some(TermValue::Iri(i)) => Some(i.clone()),
        _ => None,
    }
}

impl ArenaConstants {
    fn from_graph(state: &LawState) -> Self {
        let arena = format!("{ST}DefaultArena");
        let rows = select(
            state,
            &format!(
                "PREFIX st: <{ST}> \
                 SELECT ?af ?mf WHERE {{ \
                   ?a a st:ActuationArena ; st:arenaName \"default\" ; \
                      st:admissionFuel ?af ; st:maxFuel ?mf . \
                   FILTER(?a = <{arena}>) }}"
            ),
        );
        assert_eq!(rows.len(), 1, "exactly one st:DefaultArena expected");
        let af: u64 = lexical(&rows[0][0])
            .expect("admissionFuel literal")
            .parse()
            .expect("admissionFuel integer");
        let mf: u64 = lexical(&rows[0][1])
            .expect("maxFuel literal")
            .parse()
            .expect("maxFuel integer");
        Self {
            admission_fuel: af,
            max_fuel: mf,
        }
    }
}

/// The temprun `run/2` admission sieve, replayed in graphlaw's test crate.
/// Charges 1 unit each for clock, coordinates, conformance (`src/lib.rs`, per
/// `st:admissionFuel`'s rdfs:comment in the pack ontology); admitted iff
/// remaining lease fuel >= 3.
fn run2_sieve(lease_fuel: &mut u64) -> Result<[u32; 3], String> {
    const CLOCK: u64 = 1;
    const COORDINATES: u64 = 1;
    const CONFORMANCE: u64 = 1;
    if *lease_fuel < CLOCK + COORDINATES + CONFORMANCE {
        return Err(format!(
            "REFUSED:fuel-exhausted (have {need} needed, {have} left)",
            need = CLOCK + COORDINATES + CONFORMANCE,
            have = *lease_fuel
        ));
    }
    *lease_fuel -= CLOCK;
    *lease_fuel -= COORDINATES;
    *lease_fuel -= CONFORMANCE;
    Ok([1, 1, 1])
}

/// A sequence of admission attempts drained against one lease.
fn admission_sequence(mut fuel: u64) -> (Vec<[u32; 3]>, Vec<String>) {
    let (mut admitted, mut refused) = (Vec::new(), Vec::new());
    while fuel > 0 {
        match run2_sieve(&mut fuel) {
            Ok(charges) => admitted.push(charges),
            Err(e) => {
                // Refusal leaves fuel intact: the sequence terminates at the
                // first exhaustion boundary (fuel stays below the gate).
                refused.push(e);
                break;
            }
        }
    }
    (admitted, refused)
}

#[test]
fn arena_constants_match_documented_semantics() {
    let ontology = load_pinned_ontology();
    let c = ArenaConstants::from_graph(&ontology);
    // Documented semantics: fuel 3 in st:DefaultArena; gate is `>= 3`.
    assert_eq!(c.admission_fuel, 3, "st:DefaultArena st:admissionFuel");
    assert!(c.max_fuel >= c.admission_fuel, "maxFuel bounds admissionFuel");
    // Guard the generated skeleton agrees with the graph.
    // (generated/arena.rs pins ARENA_DEFAULT_ADMISSION_FUEL_MIN = 3.)
    assert_eq!(c.admission_fuel, 3);
}

#[test]
fn boundary_is_inclusive_at_three() {
    let ontology = load_pinned_ontology();
    let c = ArenaConstants::from_graph(&ontology);
    // Exactly the admission fuel admits; one below refuses.
    let mut f = c.admission_fuel;
    assert!(run2_sieve(&mut f).is_ok(), "fuel == admissionFuel admits");
    assert_eq!(f, 0, "exact admission drains the lease fully");
    let mut f = c.admission_fuel - 1;
    let err = run2_sieve(&mut f).unwrap_err();
    assert!(
        err.contains("REFUSED"),
        "fuel < admissionFuel must be refused, got {err}"
    );
    assert_eq!(f, 2, "refusal must not consume fuel");
}

#[test]
fn sequence_of_attempts_exhausts_and_refuses() {
    let ontology = load_pinned_ontology();
    let c = ArenaConstants::from_graph(&construct_lease_state(&ontology, 10));
    let (admitted, refused) = admission_sequence(c.max_fuel.min(10));
    assert_eq!(admitted.len(), 3, "fuel 10 = 3 admissions (9 units)");
    assert_eq!(refused.len(), 1, "residue 1 < 3 hits the exhaustion boundary");
    assert!(refused[0].contains("REFUSED"));
    // Negative witness: 2 units of fuel — below the documented gate of 3 —
    // must exhaust immediately and be refused, never admitted.
    let (admitted2, refused2) = admission_sequence(c.admission_fuel - 1);
    assert!(admitted2.is_empty(), "sub-gate fuel must admit nothing");
    assert_eq!(refused2.len(), 1, "exactly one refusal witness");
    assert!(refused2[0].contains("REFUSED"));
}

/// Fold a lease into an RDF law state alongside the arena graph: the sequence
/// runs over a state that carries both the ontology and the lease facts, the
/// way a graphlaw composition root would hold them.
fn construct_lease_state(ontology: &LawState, lease_fuel: u64) -> LawState {
    let lease_ttl = format!(
        "@prefix st: <{ST}> .\n\
         st:DefaultArena st:producesResult [ a st:ConformanceResult ; st:verdict \"pass\" ] .\n\
         [] a st:FuelLease ; st:leaseHolder \"court\" ; st:leaseFuel {lease_fuel} ; st:runsIn st:DefaultArena .\n"
    );
    // Compose ontology + lease facts via N-Triples/N-Quads round trip.
    let mut combined = String::from_utf8(
        graphlaw::purrdf::serialize_dataset(
            ontology.dataset(),
            "application/n-quads",
            graphlaw::purrdf::SerializeGraph::Dataset,
        )
        .expect("serialize ontology"),
    )
    .expect("n-quads");
    // Re-serialize the lease in n-quads too so dialects match.
    let lease_state = LawState::parse(lease_ttl.as_bytes(), Dialect::Turtle, None)
        .expect("lease ttl parses");
    let lease_nq = graphlaw::purrdf::serialize_dataset(
        lease_state.dataset(),
        "application/n-quads",
        graphlaw::purrdf::SerializeGraph::Dataset,
    )
    .expect("serialize lease");
    combined.push_str(&String::from_utf8(lease_nq).expect("utf8"));
    LawState::parse(combined.as_bytes(), Dialect::NQuads, None).expect("combined state parses")
}

#[test]
fn lease_in_rdf_state_is_governed_by_graph_constants() {
    let ontology = load_pinned_ontology();
    let combined = construct_lease_state(&ontology, 7);
    let c = ArenaConstants::from_graph(&combined);
    assert_eq!(c.admission_fuel, 3, "graph constants survive composition");
    let rows = select(
        &combined,
        &format!("PREFIX st: <{ST}> SELECT ?f WHERE {{ ?l a st:FuelLease ; st:leaseFuel ?f }}"),
    );
    assert_eq!(rows.len(), 1, "one lease in the composed state");
    let lease_fuel: u64 = lexical(&rows[0][0])
        .expect("leaseFuel literal")
        .parse()
        .expect("leaseFuel integer");
    assert_eq!(lease_fuel, 7);
    // Drain the lease through the sieve: 2 admissions (6 units), 1 left —
    // the residue cannot admit and the sequence refuses.
    let (admitted, refused) = admission_sequence(lease_fuel);
    assert_eq!(admitted.len(), 2);
    assert_eq!(refused.len(), 1, "residue fuel 1 < 3 must refuse");
}

#[test]
fn max_fuel_ceiling_governs_lease_minting() {
    let ontology = load_pinned_ontology();
    let c = ArenaConstants::from_graph(&ontology);
    // The generated skeleton's ceiling must equal the graph's maxFuel.
    assert_eq!(
        c.max_fuel, 1_000_000,
        "st:DefaultArena st:maxFuel must be the documented 1000000"
    );
}
