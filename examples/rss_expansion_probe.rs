//! WO-03 RSS-during-intensional-expansion probe.
//!
//! HONESTY SCOPE: this probe does NOT witness any "4.5B virtual triples"
//! claim. It witnesses the per-rule / per-fact marginal RSS during a real
//! forward-chained deductive closure (graphlaw-eyeron N3/Datalog). The 4.5B
//! projection in the receipt is LINEAR EXTRAPOLATION ONLY, disclosed as such.
//!
//! Ladder A: rules 100 -> 1,000 -> 10,000 at a fixed 1,000 facts with only
//!   100 active premises (fixed closure, isolates per-RULE program cost).
//! Ladder B: facts 100 -> 1,000 -> 10,000 over a fixed 100-rule program
//!   (fixed rules, isolates per-FACT closure cost).
//! A rules x facts cartesian rung (10,000 x 1,000, closure 1,000,000) was
//! attempted and is INCOMPLETE under the reasoner's typed Iterations limit
//! (max_iterations 1,000,000): the engine stops at a typed refusal, not an
//! OOM. See docs/jira/v26.10.22/designs/wo03-rss-probe.md.
//! Per rung: RSS before / during / after the closure; a rung counts only if
//! the reasoner reports CompletionStatus::Complete.
//!
//! RSS via std-only shell-out `ps -o rss= -p <pid>` (macOS has no /proc).
//!
//! Run: CARGO_TARGET_DIR=target-lane-wo3 cargo run --example rss_expansion_probe
//! Exit 0 = every rung Complete; exit 1 = any rung incomplete or RSS failure.

use eyeron::{PreparedReasoner, ReasonerOptions, parse_n3};
use std::process::Command;

fn rss_kb() -> Result<u64, String> {
    let out = Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .map_err(|e| format!("ps spawn: {e}"))?;
    if !out.status.success() {
        return Err(format!("ps exit {}", out.status));
    }
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("ps rss parse: {e}"))
}

/// Rule i has premise predicate :p<i>, so a data document supplying facts on
/// predicates :p0..:p<k-1> activates exactly k rules regardless of program
/// size N. Ladder A (k fixed, N scaling) isolates the per-RULE program cost
/// at a fixed closure; Ladder B (N=100 fixed, k and facts scaling) isolates
/// the per-FACT closure cost.
fn program_n3(n_rules: usize) -> String {
    let mut s = String::from("@prefix : <http://example.org/> .\n");
    s.reserve(n_rules * 48);
    for i in 0..n_rules {
        s.push_str(&format!("{{ ?x :p{} ?y }} => {{ ?x :q{} ?y }} .\n", i, i));
    }
    s
}

/// n_facts distributed round-robin over the first n_active predicates.
fn data_n3(n_facts: usize, n_active: usize) -> String {
    let mut s = String::from("@prefix : <http://example.org/> .\n");
    s.reserve(n_facts * 24);
    for i in 0..n_facts {
        s.push_str(&format!(":s{} :p{} :o{} .\n", i, i % n_active, i));
    }
    s
}

struct Rung {
    label: &'static str,
    n_rules: usize,
    n_facts: usize,
    n_active: usize,
}

fn main() {
    let rungs = [
        Rung {
            label: "A1 rules=100    facts=1000 active=100",
            n_rules: 100,
            n_facts: 1_000,
            n_active: 100,
        },
        Rung {
            label: "A2 rules=1000   facts=1000 active=100",
            n_rules: 1_000,
            n_facts: 1_000,
            n_active: 100,
        },
        Rung {
            label: "A3 rules=10000  facts=1000 active=100",
            n_rules: 10_000,
            n_facts: 1_000,
            n_active: 100,
        },
        Rung {
            label: "B1 rules=100    facts=100  active=100",
            n_rules: 100,
            n_facts: 100,
            n_active: 100,
        },
        Rung {
            label: "B2 rules=100    facts=1000 active=100",
            n_rules: 100,
            n_facts: 1_000,
            n_active: 100,
        },
        Rung {
            label: "B3 rules=100    facts=10000 active=100",
            n_rules: 100,
            n_facts: 10_000,
            n_active: 100,
        },
    ];
    println!(
        "label | rules | facts | rss_before_kb | rss_during_kb | rss_after_kb | delta_kb | closure | status"
    );
    for r in rungs {
        run_rung(&r);
    }
}

fn run_rung(r: &Rung) {
    let _rss_before = rss_kb().expect("rss before");
    let program = parse_n3(&program_n3(r.n_rules), None).expect("parse program");
    let data = parse_n3(&data_n3(r.n_facts, r.n_active), None).expect("parse data");
    let prepped = PreparedReasoner::new(program);
    let rss_parsed = rss_kb().expect("rss parsed");
    let opts = ReasonerOptions {
        include_explicit: false,
        ..Default::default()
    };
    let result = prepped.reason(&data, &opts);
    // Sampled with the full closure still resident: the "during-expansion" peak.
    let rss_peak = rss_kb().expect("rss peak");
    let closure_len = result.closure.len();
    let derived_len = result.derived.len();
    let complete = result.is_complete();
    let limits = format!("{:?}", result.limits_reached);
    let errors = format!("{:?}", result.errors);
    drop(result);
    drop(prepped);
    drop(data);
    let rss_after = rss_kb().expect("rss after");
    println!(
        "{} | {} | {} | {} | {} | {} | {} | {} | {}",
        r.label,
        r.n_rules,
        r.n_facts,
        rss_parsed,
        rss_peak,
        rss_after,
        rss_peak as i64 - rss_parsed as i64,
        closure_len,
        if complete { "Complete" } else { "INCOMPLETE" },
    );
    println!(
        "    derived={} limits={} errors={}",
        derived_len, limits, errors
    );
    if !complete {
        eprintln!("FAIL rung {}: incomplete closure", r.label);
        std::process::exit(1);
    }
}
