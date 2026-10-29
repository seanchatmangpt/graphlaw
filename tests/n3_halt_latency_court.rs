//! Halt-latency honesty court (WP P14).
//!
//! Every halt of a bounded N3 run — parse refusal, inference fuse, resource
//! limit — is typed (`N3Error` variant at the engine boundary, `RefusalKind`
//! over the ABI), never silent (`Ok` with empty derivations). This court
//! measures time-to-typed-refusal for the honest latency ladder:
//!
//!   1. **parse halt** — malformed N3 is refused before the fixpoint loop
//!      starts (microseconds; the cheapest typed halt).
//!   2. **best case** — the fuse premise is provable from the explicit data
//!      alone, so the first end-of-round fuse check (fixpoint iteration 1)
//!      fires it: `N3Error::Fuse`.
//!   3. **worst legal fuse** — the fuse premise contains a fact derivable
//!      only at full closure (the last derivation of the sweep), so the run
//!      completes the closure before the round-boundary fuse check fires:
//!      still `N3Error::Fuse`, still before any budget exhaustion. No
//!      mid-sweep preemption exists — the fuse is consulted only between
//!      rounds — so this is the worst latency a legal fuse can incur.
//!   4. **honest ceiling** — the same chain shape *without* the fuse rule
//!      runs past the documented `n3_iterations` budget (`N3_MAX_ITERATIONS`
//!      = 4,000) and stops mid-sweep with the typed `N3Error::Limit`.
//!      In-process derivation cannot be preempted mid-flight: the engine has
//!      no fuse check inside a fixpoint sweep and no wall-clock stopper, so
//!      the ceiling on halt latency IS the budget. Time-to-typed-refusal of
//!      the budget-exhausted run is therefore the honest ceiling, and both
//!      fuse rungs must land inside it.
//!
//! Reference only (WP P03): a host-side fuel/epoch abort is the wall-clock
//! remedy for preemption *between* ABI calls; it does not change what happens
//! inside one `reason_n3_bounded` call, which is what this court pins.
//!
//! Latency table (printed under `--nocapture`) + the memo honesty line close
//! the receipt.
//!
//! Anti-vacuity, in-court: the identical chain *minus* the fuse rule admits
//! and derives the full reach closure (so the fused verdict is semantic, not
//! a limit), the incomplete chain (endpoint unreachable) does not fire the
//! fuse (trigger semantics, not always-fire), and deleting the `Fuse` arm or
//! the budget check flips the court red.

#![cfg(feature = "abi")]

use std::time::Instant;

use graphlaw::abi::call;
use graphlaw::law::{N3_MAX_ITERATIONS, N3Error, reason_n3_bounded};
use serde_json::{Value, json};

const NS: &str = "http://example.org/n3-halt/";

/// Malformed N3: the typed halt at the parse boundary (before the loop).
fn broken_program() -> String {
    format!("@prefix : <{NS}> .\n:a :p :b .\n:b :p\n")
}

/// Best case: the fuse premise is provable from explicit data alone — the
/// first end-of-round fuse check (iteration 1) fires it.
fn best_case_program() -> String {
    format!(
        "@prefix : <{NS}> .\n\
         :a :fuseNow true .\n\
         {{ ?x :fuseNow true }} => false .\n"
    )
}

/// `links`-hop chain: `:n0 :reach :n1 .` plus `:n_i :link :n_{i+1}` for
/// i in 1..=links (`:n_{links+1}` is `:end`), the multi-premise reach rule,
/// and — optionally — the fuse on `:n0 :reach :end`.
///
/// The reach rule joins the one reach seed with consecutive links. Under the
/// semi-naive agenda the whole chain derives within one fixpoint sweep (each
/// new `:reach` fact is itself an agenda trigger), so the endpoint
/// `:n0 :reach :end` becomes provable only once the sweep completes the
/// closure — and the fuse check exists only at the round boundary after the
/// sweep. That is the worst LEGAL fuse placement: the premise turns provable
/// mid-sweep, and no mid-sweep preemption exists to stop the run earlier.
fn chain_program(links: usize, with_fuse: bool) -> String {
    let mut text = format!("@prefix : <{NS}> .\n:n0 :reach :n1 .\n");
    for i in 1..=links {
        let next: String = if i == links {
            ":end".to_string()
        } else {
            format!(":n{}", i + 1)
        };
        text.push_str(&format!(":n{i} :link {next} .\n"));
    }
    text.push_str("{ ?x :reach ?y . ?y :link ?z } => { ?x :reach ?z } .\n");
    if with_fuse {
        text.push_str("{ :n0 :reach :end } => false .\n");
    }
    text
}

/// Chain minus the fuse rule: the flip pair. Same shape, admits to full closure.
fn unfused_chain(links: usize) -> String {
    chain_program(links, false)
}

/// Chain with the fuse rule but the endpoint unreachable: the trigger guard.
fn incomplete_chain(links: usize) -> String {
    // Same program, but the last link points at :other, never :end — the
    // fuse premise is never provable and must not fire.
    let mut text = format!("@prefix : <{NS}> .\n:n0 :reach :n1 .\n");
    for i in 1..links {
        text.push_str(&format!(":n{} :link :n{} .\n", i, i + 1));
    }
    text.push_str(&format!(":n{links} :link :other .\n"));
    text.push_str("{ ?x :reach ?y . ?y :link ?z } => { ?x :reach ?z } .\n");
    text.push_str("{ :n0 :reach :end } => false .\n");
    text
}

/// Iterations of the chain program: every closure fact (explicit + derived)
/// is consumed by the agenda exactly once, each consumption is one counted
/// iteration, so a chain of `links` costs ~`2·links + 3` fixpoint iterations.
/// Both probe sizes are pinned against the documented 4,000-iteration
/// budget: the worst-legal chain stays inside it, the ceiling chain must
/// overshoot it mid-sweep.
const WORST_LEGAL_LINKS: usize = 600;
const CEILING_LINKS: usize = 2_100;

fn run(req: Value) -> Value {
    serde_json::from_slice(&call(req.to_string().as_bytes())).expect("GraphLaw ABI returns JSON")
}

/// Time parse+derive of `text` to its typed error. A `Ok` here is the
/// silent-halt defect this court exists to catch.
fn time_to_typed_error(text: &str) -> (std::time::Duration, N3Error) {
    let start = Instant::now();
    let err = match reason_n3_bounded(text) {
        Ok(derived) => panic!(
            "SILENT HALT: fused/limited run answered Ok({derived:?}) — \
             the typed refusal never surfaced"
        ),
        Err(e) => e,
    };
    (start.elapsed(), err)
}

#[test]
fn parse_halt_is_typed_and_cheapest() {
    let (elapsed, err) = time_to_typed_error(&broken_program());
    match &err {
        N3Error::Refused(refusal) => {
            // The engine boundary wraps the parse error as a typed refusal
            // (engine-rejected), distinct from Fuse and Limit: the halt is
            // classified before the fixpoint loop ever starts.
            assert_eq!(
                refusal.kind,
                graphlaw::dialect::RefusalKind::EngineRejected,
                "got {refusal:?}"
            );
        }
        _ => panic!("expected the parse-boundary refusal, got {err:?}"),
    }
    println!("parse halt: typed Refused in {elapsed:?}");
}

#[test]
fn best_case_fuse_fires_at_first_check_inside_budget() {
    let (elapsed_best, err) = time_to_typed_error(&best_case_program());
    let report = match &err {
        N3Error::Fuse { report } => report,
        N3Error::Limit { .. } => panic!("fuse must win the race against the budget, got {err:?}"),
        N3Error::Refused(r) => panic!("fixture must parse, got {r:?}"),
        _ => panic!("expected N3Error::Fuse at the first check, got {err:?}"),
    };
    assert!(
        report.contains("# Inference fuse triggered."),
        "report must carry the fuse header, got: {report}"
    );
    assert!(
        report.contains("fuseNow"),
        "report names the fired premise, got: {report}"
    );

    // Op surface: the same run answers the typed Fuse envelope.
    let out = run(json!({"op": "n3", "text": best_case_program()}));
    assert_eq!(out["ok"], false, "op surface must refuse, got {out}");
    assert_eq!(out["error"]["kind"], "Fuse", "typed kind, got {out}");
    assert_eq!(out["error"]["details"]["code"], "Fuse", "got {out}");

    println!("best case (premise provable at iteration 1): {elapsed_best:?} to N3Error::Fuse");
}

#[test]
fn worst_legal_fuse_halts_at_full_closure_inside_budget() {
    let program = chain_program(WORST_LEGAL_LINKS, true);
    let (elapsed_worst, err) = time_to_typed_error(&program);
    match &err {
        N3Error::Fuse { report } => {
            assert!(
                report.contains("# Inference fuse triggered."),
                "report must carry the fuse header, got: {report}"
            );
            assert!(
                report.contains(":end") || report.contains("end"),
                "the fired instance must be the closure endpoint, got: {report}"
            );
        }
        N3Error::Limit {
            limit,
            observed,
            max,
            ..
        } => panic!(
            "the worst LEGAL fuse must halt before budget exhaustion, \
             got Limit {limit} ({observed}/{max}) — the fuse never fired"
        ),
        _ => panic!("fixture must parse, got {err:?}"),
    }

    // Flip pair: the identical chain minus the fuse rule ADMITS and derives
    // the endpoint — proving the fuse premise was provable only at full
    // closure and the verdict is semantic, not a resource cap.
    let out = run(json!({"op": "n3", "text": unfused_chain(WORST_LEGAL_LINKS)}));
    assert_eq!(out["ok"], true, "unfused chain must admit, got {out}");
    let derived = out["derived"].as_str().expect("derived text");
    assert!(
        derived.contains("end"),
        "the unfused chain must derive the full closure incl. the endpoint, got: {derived}"
    );

    println!(
        "worst legal (premise provable only at full closure, {WORST_LEGAL_LINKS} links): \
         {elapsed_worst:?} to N3Error::Fuse"
    );
}

#[test]
fn budget_exhaustion_is_the_honest_ceiling() {
    let program = unfused_chain(CEILING_LINKS);
    let (elapsed_ceiling, err) = time_to_typed_error(&program);
    match &err {
        N3Error::Limit {
            limit,
            observed,
            max,
            ..
        } => {
            assert_eq!(
                *limit, "n3_iterations",
                "the documented budget fires, got {err:?}"
            );
            assert_eq!(
                *max, N3_MAX_ITERATIONS,
                "the documented ceiling, got {err:?}"
            );
            assert!(
                *observed >= N3_MAX_ITERATIONS,
                "observed iterations must reach the documented budget, got {err:?}"
            );
        }
        N3Error::Fuse { .. } => panic!("no fuse rule in the ceiling fixture, got {err:?}"),
        _ => panic!("fixture must parse, got {err:?}"),
    }

    // Op surface: the budget-exhausted run is typed over the ABI too.
    let out = run(json!({"op": "n3", "text": unfused_chain(CEILING_LINKS)}));
    assert_eq!(out["ok"], false, "budget exhaustion must refuse, got {out}");
    assert_eq!(out["error"]["kind"], "ResourceLimit", "got {out}");
    assert_eq!(
        out["error"]["details"]["code"], "ResourceLimit",
        "got {out}"
    );
    assert_eq!(
        out["error"]["details"]["limit"], "n3_iterations",
        "documented limit name, got {out}"
    );
    assert_eq!(
        out["error"]["details"]["max"],
        json!(N3_MAX_ITERATIONS),
        "documented ceiling, got {out}"
    );

    println!(
        "honest ceiling (budget-exhausted, {CEILING_LINKS} links, no fuse): \
         {elapsed_ceiling:?} to N3Error::Limit(n3_iterations)"
    );
}

#[test]
fn trigger_guard_fuse_does_not_fire_without_provable_premise() {
    let out = run(json!({"op": "n3", "text": incomplete_chain(WORST_LEGAL_LINKS)}));
    assert_eq!(
        out["ok"], true,
        "an unprovable fuse premise must not fire, got {out}"
    );
    let derived = out["derived"].as_str().expect("derived text");
    // The chain DID saturate — reachability reaches the last link's target
    // (:other) — so the fuse rule was loaded and the premise genuinely never
    // became provable, rather than the run short-circuiting.
    assert!(
        derived.contains(":other"),
        "the chain must derive reach through the last link (:other), got: {derived}"
    );
    assert!(
        !derived.contains("end"),
        "the endpoint is unreachable, so the fuse premise stays unprovable, got: {derived}"
    );
}

#[test]
fn halt_latency_table_and_honesty_memo() {
    // The three rungs, re-measured back to back so one receipt carries one
    // comparable table.
    let (e_parse, p_err) = time_to_typed_error(&broken_program());
    assert!(matches!(p_err, N3Error::Refused(_)));
    let (e_best, b_err) = time_to_typed_error(&best_case_program());
    assert!(matches!(b_err, N3Error::Fuse { .. }));
    let (e_worst, w_err) = time_to_typed_error(&chain_program(WORST_LEGAL_LINKS, true));
    assert!(matches!(w_err, N3Error::Fuse { .. }));
    let (e_ceiling, c_err) = time_to_typed_error(&unfused_chain(CEILING_LINKS));
    assert!(matches!(c_err, N3Error::Limit { .. }));

    // The honesty claim, as assertions: every typed halt lands within the
    // budget-exhaustion ceiling. (The per-rung tests re-classify the errors;
    // here only the clock order is pinned.)
    assert!(
        e_parse <= e_ceiling && e_best <= e_ceiling && e_worst <= e_ceiling,
        "every typed halt must land inside the budget ceiling \
         (parse {:?}, best {:?}, worst {:?} vs ceiling {:?})",
        e_parse,
        e_best,
        e_worst,
        e_ceiling
    );
    // The worst legal fuse is strictly more work than the best case (it
    // derives the full closure first); the ordering is the honest ladder.
    assert!(
        e_best <= e_worst,
        "best case must not be slower than the worst legal fuse \
         (best {:?} vs worst {:?})",
        e_best,
        e_worst
    );

    println!("=== N3 halt-latency table (WP P14) ===");
    println!(
        "| {:<38} | {:<28} | {:>13} |",
        "rung", "typed halt", "latency"
    );
    println!("|{:-<40}|{:-<30}|{:-<15}|", "", "", "");
    println!(
        "| {:<38} | {:<28} | {:>13?} |",
        "parse boundary", "N3Error::Refused(engine)", e_parse
    );
    println!(
        "| {:<38} | {:<28} | {:>13?} |",
        "best case (fuse @ first check)", "N3Error::Fuse", e_best
    );
    println!(
        "| {:<38} | {:<28} | {:>13?} |",
        format_args!("worst legal (fuse @ closure, {WORST_LEGAL_LINKS} links)"),
        "N3Error::Fuse",
        e_worst
    );
    println!(
        "| {:<38} | {:<28} | {:>13?} |",
        format_args!("honest ceiling (budget, {CEILING_LINKS} links)"),
        "N3Error::Limit(n3_iterations)",
        e_ceiling
    );
    println!(
        "\nMEMO (honesty line): the ceiling is the budget, not a wall clock. \
         In-process derivation cannot be preempted mid-flight — the engine \
         checks the fuse only between fixpoint rounds and has no wall-clock \
         stopper — so time-to-typed-refusal of a budget-exhausted run is the \
         honest ceiling, and every legal halt (parse, fuse at any round) \
         lands inside it, typed, never silent. \
         Wall-clock preemption between ABI calls is the P03 host-side \
         fuel/epoch abort (reference only, out of scope here)."
    );
}
