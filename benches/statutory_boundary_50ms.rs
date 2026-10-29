//! SB-50MS — Sub-50 ms statutory boundary benchmark (EU AI Act Art. 12/14/15,
//! 80/20 operational sweet spot).
//!
//! Three gates over real collaborators (Chicago: no mocks, no sleeps, assert
//! on real output state):
//!
//! - Art. 12 (traceable logging): BLAKE3 derivation-receipt issuance —
//!   `admitted_statement_hash` (full re-absorb baseline) and
//!   `fold_params_anchored` (anchored fold). Ceiling 500 µs; the in-tree
//!   TC-HOT-01 reference budget for the anchored leg is 250 ns p99.
//! - Art. 14 (fail-closed boundary): real SHACL structural validation over a
//!   candidate state (≤ 64 quads, degree ≤ 8, shape depth ≤ 2), plus the
//!   fail-closed halt leg (violating candidate → non-conformance → halt).
//!   Ceiling 5 ms.
//! - Art. 15 (deductive traceability): real bounded N3 derivation with proofs
//!   over quarantined `{ ... }` formulas via purrdf::n3, budget-capped
//!   (iterations/closure facts/match steps) exactly as `src/law.rs` wires
//!   `Step::DeriveN3`. Ceiling 30 ms. An incomplete fixpoint is a gate
//!   FAILURE, never a pass.
//!
//! End-to-end budget: sum of the gate p99s < 40 ms (strict ceiling 50 ms).
//! Gate verdicts are computed from THIS run's measurements — a gate that
//! cannot fail is not a gate.

use graphlaw::duckdb_eval::{admitted_statement_hash, fold_params_anchored};
use graphlaw::sql_ast::SqlLiteral;
use graphlaw::n3::{parse_n3, reason_document, CompletionStatus, ReasonerOptions, ReasonerResult};
use graphlaw::shacl::engine::{parse_shapes, validate_dataset};
use std::time::Instant;

/// Nearest-rank quantile over sorted samples.
fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((q * (sorted.len() as f64 - 1.0)) + 0.5).floor() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn p99_us(l: &Legs) -> f64 {
    quantile(&l.samples_ns, 0.99) / 1e3
}

struct Legs {
    label: &'static str,
    samples_ns: Vec<f64>,
}

impl Legs {
    fn new(label: &'static str) -> Self {
        Self { label, samples_ns: Vec::new() }
    }
    fn time<F: FnMut()>(&mut self, iters: usize, mut f: F) {
        for _ in 0..iters {
            let t0 = Instant::now();
            f();
            self.samples_ns.push(t0.elapsed().as_secs_f64() * 1e9);
        }
        self.samples_ns
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    }
    fn report(&self, ceiling_us: f64, out: &mut String) -> bool {
        let p50 = quantile(&self.samples_ns, 0.50);
        let p95 = quantile(&self.samples_ns, 0.95);
        let p99 = quantile(&self.samples_ns, 0.99);
        let ok = (p99 / 1e3) <= ceiling_us;
        out.push_str(&format!(
            "  {:<26} p50 {:>11.1} ns  p95 {:>11.1} ns  p99 {:>11.1} ns  ceiling {:>6.0} µs  {}\n",
            self.label,
            p50,
            p95,
            p99,
            ceiling_us,
            if ok { "GATE PASS" } else { "GATE FAIL" },
        ));
        ok
    }
}

const PROGRAM: &str = "\
@prefix : <http://example.org/sb#> .
@prefix math: <http://www.w3.org/2000/10/swap/math#> .
:trade_47 a :ProposedTrade .
:trade_47 :notional 25000000 ;
          :book :emea_deriv .
{ ?t a :ProposedTrade ;
     :notional ?n .
  ?n math:greaterThan 10000000 .
  ?t :book :emea_deriv }
  => { ?t :violates :Rule15c3_1 ; :requires :pretrade_review } .
{ ?t a :ProposedTrade ; :violates :Rule15c3_1 }
  => { :desk :halt :trade_47 } .
";

const SHAPES_TTL: &str = "\
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/sb#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
ex:TradeShape a sh:NodeShape ;
  sh:targetClass ex:Order ;
  sh:closed true ;
  sh:property [
    sh:path rdf:type ; sh:minCount 1 ; sh:maxCount 1 ; sh:hasValue ex:Order
  ] ;
  sh:property [
    sh:path ex:id ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] ;
  sh:property [
    sh:path ex:acct ; sh:minCount 1 ; sh:maxCount 2 ; sh:nodeKind sh:IRI
  ] ;
  sh:property [
    sh:path ex:qty ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer
  ] .
";

fn build_candidate(quads_target: usize) -> String {
    let mut s = String::from("@prefix ex: <http://example.org/sb#> .\n");
    let n = (quads_target / 4).max(8);
    for i in 0..n {
        s.push_str(&format!(
            "ex:o{} a ex:Order ; ex:id {} ; ex:acct ex:a{} ; ex:qty {} .\n",
            i, 90_000 + i, i % 8, 100 + i
        ));
    }
    s
}

fn main() {
    let quick = std::env::args().any(|a| a == "--quick");
    let n_receipt = if quick { 20_000 } else { 100_000 };
    let n_shacl = if quick { 400 } else { 2_000 };
    let n_n3 = if quick { 100 } else { 400 };

    let mut report = String::new();
    report.push_str("# SB-50MS statutory boundary run\n");
    report.push_str(&format!(
        "# iters: receipt={} shacl={} n3={} | os={} arch={}\n",
        n_receipt, n_shacl, n_n3,
        std::env::consts::OS, std::env::consts::ARCH,
    ));

    // ---- Art. 12: receipt issuance -------------------------------------
    let canon: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
    let params: Vec<SqlLiteral> = vec![
        SqlLiteral::Integer(90_001),
        SqlLiteral::String("a7".into()),
        SqlLiteral::Boolean(true),
    ];
    let h_ast: [u8; 32] = std::array::from_fn(|i| (i as u8) ^ 0x5a);
    let mut sink: u64 = 0;
    let mut receipt_baseline = Legs::new("art12 admitted_statement_hash");
    receipt_baseline.time(n_receipt, || {
        let h = admitted_statement_hash(&canon, &params);
        sink = sink.wrapping_add(h[0] as u64);
    });
    let mut receipt_anchored = Legs::new("art12 fold_params_anchored");
    receipt_anchored.time(n_receipt, || {
        let h = fold_params_anchored(&h_ast, &params);
        sink = sink.wrapping_add(h[0] as u64);
    });
    std::hint::black_box(sink);

    // ---- Art. 14: SHACL boundary ---------------------------------------
    let shapes = parse_shapes(SHAPES_TTL, None).expect("SB-50MS: shapes parse");
    let candidate = build_candidate(64);
    let ds = graphlaw::purrdf::parse_dataset(candidate.as_bytes(), "text/turtle", None)
        .expect("SB-50MS: candidate parse");
    let mut shacl_ok = Legs::new("art14 validate_candidate");
    let mut conformed = 0usize;
    shacl_ok.time(n_shacl, || {
        let report = validate_dataset(&ds, &shapes).expect("SB-50MS: validate");
        if report.conforms {
            conformed += 1;
        }
    });
    if std::env::var_os("SB_DEBUG").is_some() {
        let rep = validate_dataset(&ds, &shapes).expect("dbg");
        for r in rep.results.iter().take(5) {
            println!(
                "DBG focus={} path={} msg={}",
                r.focus_node,
                format!("{:?}", r.source_constraint_component),
                r.messages.first().map(|l| l.value().to_string()).unwrap_or_default(),
            );
        }
    }
    assert_eq!(conformed, n_shacl, "SB-50MS: conforming candidate must conform");

    let mut violating = candidate.clone();
    violating.push_str("ex:o999 a ex:Order ; ex:id 1 ; ex:acct ex:a1 ; ex:ghost 9 .\n");
    let ds_bad = graphlaw::purrdf::parse_dataset(violating.as_bytes(), "text/turtle", None)
        .expect("SB-50MS: violating parse");
    let mut shacl_halt = Legs::new("art14 failclosed_halt");
    let mut halted = 0usize;
    shacl_halt.time(n_shacl, || {
        let report = validate_dataset(&ds_bad, &shapes).expect("SB-50MS: validate bad");
        if !report.conforms {
            halted += 1;
        }
    });
    assert_eq!(halted, n_shacl, "SB-50MS: violating candidate MUST be halted");
    std::hint::black_box((conformed, halted));

    // ---- Art. 15: bounded N3 derivation with proofs --------------------
    let doc = parse_n3(PROGRAM, None).expect("SB-50MS: program parse");
    let options = ReasonerOptions {
        proof: true,
        ..ReasonerOptions::default()
    };
    let mut n3_leg = Legs::new("art15 n3_derive_proof");
    let mut derived_total = 0usize;
    n3_leg.time(n_n3, || {
        let result: ReasonerResult = reason_document(&doc, &options);
        assert!(
            result.status == CompletionStatus::Complete,
            "SB-50MS: incomplete fixpoint is a gate FAILURE (limits: {:?})",
            result.limits_reached,
        );
        derived_total += result.derived.len();
    });
    assert!(derived_total > 0, "SB-50MS: derivation must derive");
    std::hint::black_box(derived_total);

    // ---- Gate verdicts --------------------------------------------------
    let mut ok_all = true;
    report.push_str("\n== Art. 12 receipt issuance (ceiling 500 us) ==\n");
    ok_all &= receipt_baseline.report(500.0, &mut report);
    ok_all &= receipt_anchored.report(500.0, &mut report);
    report.push_str("\n== Art. 14 SHACL boundary (ceiling 5000 us) ==\n");
    ok_all &= shacl_ok.report(5_000.0, &mut report);
    ok_all &= shacl_halt.report(5_000.0, &mut report);
    report.push_str("\n== Art. 15 bounded N3 derivation (ceiling 30000 us) ==\n");
    ok_all &= n3_leg.report(30_000.0, &mut report);

    let e2e = p99_us(&receipt_anchored) + p99_us(&shacl_ok) + p99_us(&shacl_halt) + p99_us(&n3_leg);
    report.push_str(&format!(
        "\n== END-TO-END (sum of gate p99s): {:.1} us | budget 40000 us (strict 50000 us) | {}\n",
        e2e,
        if e2e < 40_000.0 { "PASS" } else { "FAIL" }
    ));
    report.push_str(&format!(
        "overall: {}\n",
        if ok_all && e2e < 40_000.0 { "ALL GATES PASS" } else { "GATE FAILURE" }
    ));
    println!("{}", report);
}
