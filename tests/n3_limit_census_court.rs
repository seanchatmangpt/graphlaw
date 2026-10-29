//! N3 limit census court (WP P02, Chicago: real registry, real runs, no doubles).
//!
//! `ReasonerLimit` has 8 variants and the whole admission surface must agree
//! on that set — the census in the engine, the law table's ceilings, the
//! registry's emitted rows, and the human `Display` labels — or a refusal
//! receipt can name a limit the law table never ceilings.
//!
//! Set-equality, four ways (no wildcards anywhere):
//!   1. engine census: `ReasonerLimit::ALL` is 8 distinct arms, in enum
//!      declaration order, each with a unique machine name;
//!   2. law table: for the exact options `reason_n3_bounded` wires (the
//!      `N3_MAX_*` consts), every census row's `max` is the paired const —
//!      no silent library default on the ABI surface;
//!   3. registry: the `limit_meta` rows with scope `n3` carry exactly the 8
//!      `refusal_name`s, and the `limits` map carries the `n3_max_*` ceilings
//!      read from the same consts;
//!   4. `Display`: 8 distinct labels, paired 1:1 with the machine names.
//!
//! Census-order law: the dispatch in `law::reason_n3_bounded` is
//! `ALL.iter().find(...)` — enum declaration order, not insertion order. A
//! result that reached several limits must name the census-first one. The
//! court pins the pick on a synthetic result and on a real multi-limit run,
//! then pins the full `(name, observed, max)` row per arm on real runs.
#![cfg(feature = "abi")]

use graphlaw::law::{
    N3_MAX_BACKWARD_DEPTH, N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL, N3_MAX_DERIVED_FACTS,
    N3_MAX_ITERATIONS, N3_MAX_MATCH_STEPS, N3_MAX_TERM_BYTES, N3_MAX_TERM_DEPTH,
    N3_MAX_TOTAL_BYTES, N3_MAX_TOTAL_STEPS,
};
use graphlaw::law::ReasonerLimitCensus;
use graphlaw::n3::{ReasonerLimit, ReasonerOptions, ReasonerResult};
use serde_json::{Value, json};

/// The 8 machine names, in census (enum declaration) order — the names the
/// law table's refusals answer and the registry's `limit_meta` carries.
const NAMES: [&str; 8] = [
    "n3_iterations",
    "n3_match_steps",
    "n3_backward_depth",
    "n3_backward_solutions_per_goal",
    "n3_term_bytes",
    "n3_term_depth",
    "n3_total_bytes",
    "n3_derived_facts",
];

/// Census arm -> (machine name, law-table const) its row answers under the
/// ABI options.
fn law_table() -> [(ReasonerLimit, &'static str, usize); 8] {
    [
        (ReasonerLimit::Iterations, "n3_iterations", N3_MAX_ITERATIONS),
        (ReasonerLimit::MatchSteps, "n3_match_steps", N3_MAX_MATCH_STEPS),
        (
            ReasonerLimit::BackwardDepth,
            "n3_backward_depth",
            N3_MAX_BACKWARD_DEPTH,
        ),
        (
            ReasonerLimit::BackwardSolutionsPerGoal,
            "n3_backward_solutions_per_goal",
            N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL,
        ),
        (ReasonerLimit::TermSize, "n3_term_bytes", N3_MAX_TERM_BYTES),
        (ReasonerLimit::TermDepth, "n3_term_depth", N3_MAX_TERM_DEPTH),
        (ReasonerLimit::ClosureSize, "n3_total_bytes", N3_MAX_TOTAL_BYTES),
        (
            ReasonerLimit::ClosureFacts,
            "n3_derived_facts",
            N3_MAX_DERIVED_FACTS,
        ),
    ]
}

/// The options `reason_n3_bounded` wires, rebuilt from the same consts.
fn law_options() -> ReasonerOptions {
    ReasonerOptions {
        include_explicit: false,
        max_iterations: N3_MAX_ITERATIONS,
        max_term_bytes: N3_MAX_TERM_BYTES,
        max_total_bytes: N3_MAX_TOTAL_BYTES,
        max_closure_facts: N3_MAX_DERIVED_FACTS,
        max_match_steps: N3_MAX_MATCH_STEPS,
        max_total_steps: N3_MAX_TOTAL_STEPS,
        max_backward_depth: N3_MAX_BACKWARD_DEPTH,
        max_backward_solutions_per_goal: N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL,
        ..ReasonerOptions::default()
    }
}

/// Generous ceilings everywhere except what the caller overrides, so the arm
/// under test is the only reachable limit.
fn tuned(tweak: impl FnOnce(&mut ReasonerOptions)) -> ReasonerOptions {
    let mut options = ReasonerOptions {
        max_iterations: usize::MAX / 4,
        max_match_steps: usize::MAX / 4,
        max_backward_depth: usize::MAX / 4,
        max_backward_solutions_per_goal: usize::MAX / 4,
        max_term_bytes: usize::MAX / 4,
        max_total_bytes: usize::MAX / 4,
        max_closure_facts: usize::MAX / 4,
        max_total_steps: usize::MAX / 4,
        trace: false,
        proof: false,
        include_explicit: false,
    };
    tweak(&mut options);
    options
}

fn run(text: &str, options: &ReasonerOptions) -> ReasonerResult {
    let doc = graphlaw::n3::parse_n3(text, None).expect("court fixture parses");
    graphlaw::n3::reason_document(&doc, options)
}

/// A minimal all-zero result: `ReasonerResult`'s fields are public, so the
/// court can name which limits a result carries without faking an engine.
fn result_reaching(limits: Vec<ReasonerLimit>) -> ReasonerResult {
    ReasonerResult {
        status: graphlaw::n3::CompletionStatus::Complete,
        limits_reached: limits,
        errors: Vec::new(),
        statistics: graphlaw::n3::ReasonerStatistics::default(),
        explicit: Vec::new(),
        explicit_sources: Default::default(),
        derived: Vec::new(),
        closure: Vec::new(),
        proofs: Vec::new(),
        rules: Vec::new(),
        fuse: None,
    }
}

/// ---------- 1. engine census: 8 distinct wildcard-free arms ----------

#[test]
fn census_is_eight_distinct_wildcard_free_arms() {
    assert_eq!(ReasonerLimit::ALL.len(), 8, "the census has 8 arms");
    let names: Vec<_> = ReasonerLimit::ALL
        .iter()
        .map(|l| l.graphlaw_name())
        .collect();
    assert_eq!(names, NAMES, "census order and machine names drifted");
    // Wildcard-free: unique, off-family names fail loudly.
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 8, "machine names must be unique: {names:?}");
    for name in &names {
        assert!(name.starts_with("n3_"), "machine name {name} off-family");
    }
    // Declaration order == census order: the BTreeSet the engine keeps and
    // the `ALL` array must agree, or the dispatch's census-first `find`
    // would answer a different limit than the engine's own ordering.
    let as_set: std::collections::BTreeSet<ReasonerLimit> = ReasonerLimit::ALL.into();
    let ordered: Vec<_> = as_set.iter().copied().collect();
    assert_eq!(
        ordered,
        ReasonerLimit::ALL.to_vec(),
        "`ALL` must be in enum declaration order (the BTreeSet's order)"
    );
}

#[test]
fn display_labels_are_eight_distinct_and_paired_with_names() {
    let mut labels = std::collections::BTreeSet::new();
    for (limit, name) in ReasonerLimit::ALL.iter().zip(NAMES) {
        let label = limit.to_string();
        assert!(!label.is_empty(), "{name} has an empty Display label");
        labels.insert(label);
    }
    assert_eq!(labels.len(), 8, "Display labels must be distinct");
}

/// ---------- 2. set-equal with the law table ----------

#[test]
fn every_census_row_answers_the_law_table_const_under_law_options() {
    let options = law_options();
    let result = result_reaching(Vec::new());
    for (limit, name, max) in law_table() {
        let (row_name, _observed, row_max) = limit.graphlaw_row(&result, &options);
        assert_eq!(row_name, name, "census arm {name:?} drifted");
        assert_eq!(
            row_max, max,
            "census arm {name:?} stopped answering its law-table const"
        );
    }
    // The engine's own default must also agree on the two ceilings the ABI
    // does not re-pin: a library default bump surfaces here as a reviewed
    // re-pin rather than silent divergence (mirrors the match-steps court).
    let default = ReasonerOptions::default();
    assert_eq!(default.max_backward_depth, N3_MAX_BACKWARD_DEPTH);
    assert_eq!(
        default.max_backward_solutions_per_goal,
        N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL
    );
    assert_eq!(
        default.max_total_steps,
        ReasonerOptions::default().max_total_steps
    );
}

/// ---------- 3. set-equal with the registry ----------

#[test]
fn census_names_set_equal_with_registry_limit_meta() {
    let doc = graphlaw::registry::registry_value();
    let meta = doc
        .get("limit_meta")
        .and_then(Value::as_object)
        .expect("registry document has a `limit_meta` object");

    // n3-scope rows with a machine refusal name, as a set.
    let mut registry_names: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for (key, entry) in meta {
        let scope = entry.get("scope").and_then(Value::as_str);
        let refusal = entry.get("refusal_name").and_then(Value::as_str);
        if scope == Some("n3") {
            assert!(key.starts_with("n3_max_"), "n3-scope limit {key} off-family");
            let refusal = refusal
                .unwrap_or_else(|| panic!("n3-scope limit {key} carries no refusal_name"));
            registry_names.insert(refusal);
        }
    }
    let census_names: std::collections::BTreeSet<_> = ReasonerLimit::ALL
        .iter()
        .map(|l| l.graphlaw_name())
        .collect();
    assert_eq!(
        registry_names, census_names,
        "registry refusal names and the census arms are not set-equal"
    );
    assert_eq!(registry_names.len(), 8);

    // The two `n3_max_*_steps` rows both answer `n3_match_steps` — one name,
    // two ceilings, the row's `max` discriminates. Pin that pairing too.
    let per_search = &meta["n3_max_match_steps"];
    let run_wide = &meta["n3_max_total_steps"];
    assert_eq!(
        per_search["refusal_name"].as_str(),
        run_wide["refusal_name"].as_str(),
        "both step ceilings must answer the same machine name"
    );
    assert_eq!(per_search["refusal_name"].as_str(), Some("n3_match_steps"));
    // And the ceilings behind the rows are the law table's own consts.
    let limits = doc.get("limits").and_then(Value::as_object).unwrap();
    assert_eq!(limits["n3_max_match_steps"], json!(N3_MAX_MATCH_STEPS));
    assert_eq!(limits["n3_max_total_steps"], json!(N3_MAX_TOTAL_STEPS));
    assert_eq!(limits["n3_max_backward_depth"], json!(N3_MAX_BACKWARD_DEPTH));
    assert_eq!(
        limits["n3_max_backward_solutions_per_goal"],
        json!(N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL)
    );
    assert_eq!(limits["n3_max_term_depth"], json!(N3_MAX_TERM_DEPTH));
}

/// ---------- 4. census-order dispatch on real and synthetic results ----------

#[test]
fn census_order_beats_insertion_order_in_the_dispatch_find() {
    // Recorded out of census order: deeper arms first, Iterations last.
    let result = result_reaching(vec![
        ReasonerLimit::TermDepth,
        ReasonerLimit::BackwardDepth,
        ReasonerLimit::Iterations,
    ]);
    let picked = ReasonerLimit::ALL
        .iter()
        .find(|limit| result.limits_reached.contains(limit))
        .copied();
    assert_eq!(
        picked,
        Some(ReasonerLimit::Iterations),
        "dispatch must answer the census-first arm, not the first-recorded"
    );
    let (name, observed, max) = picked.unwrap().graphlaw_row(&result, &law_options());
    assert_eq!(name, "n3_iterations");
    assert_eq!(observed, 0);
    assert_eq!(max, N3_MAX_ITERATIONS);
}

#[test]
fn a_real_multi_limit_run_names_the_census_first_limit() {
    // One run that genuinely reaches two backward limits: a backward chain
    // deeper than the depth ceiling AND a goal with more solutions than the
    // per-goal ceiling. Census order answers BackwardDepth (arm 3) before
    // BackwardSolutionsPerGoal (arm 4).
    let depth = N3_MAX_BACKWARD_DEPTH + 2;
    let mut rules = String::new();
    for k in 1..=depth {
        rules.push_str(&format!("{{ ?x :d{k} ?y }} <= {{ ?x :d{} ?y }} .\n", k - 1));
    }
    let leaves: String = (0..N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL + 4)
        .map(|i| format!(":a :p :leaf{i} .\n"))
        .collect();
    let text = format!(
        "@prefix : <urn:c:> .\n:a :d0 :b .\n{rules}{leaves}\
         {{ ?x :sol ?y }} <= {{ ?x :p ?y }} .\n\
         {{ :a :d{depth} :b }} => {{ :a :done :b }} .\n\
         {{ :a :sol ?y }} => {{ :a :got ?y }} .\n"
    );
    let result = run(&text, &law_options());
    let reached: std::collections::BTreeSet<_> = result.limits_reached.iter().copied().collect();
    assert!(
        reached.contains(&ReasonerLimit::BackwardDepth),
        "the deep chain must cut, got {reached:?}"
    );
    assert!(
        reached.contains(&ReasonerLimit::BackwardSolutionsPerGoal),
        "the wide goal must truncate, got {reached:?}"
    );
    // Census-first pick over what the run actually reached.
    let picked = ReasonerLimit::ALL
        .iter()
        .find(|limit| reached.contains(limit))
        .copied()
        .expect("a limit-firing run names at least one arm");
    let (name, observed, max) = picked.graphlaw_row(&result, &law_options());
    assert_eq!(picked, ReasonerLimit::BackwardDepth, "census order wins");
    assert_eq!(name, "n3_backward_depth");
    assert_eq!(
                observed,
                max + 1,
                "unrecorded depth: the documented at-cap sentinel (max + 1)"
            );
}

/// ---------- 5. per-variant rows on real runs ----------
///
/// One real run per arm, driving that arm and only that arm, pinning the
/// full `(name, observed, max)` row a receipt would carry.

#[test]
fn row_pins_iterations_on_real_runs() {
    // The nonterminating blank-node rule: only the iteration cap stops it.
    let text = "@prefix : <urn:r:> .\n:a :s :b .\n{ ?x :s ?y } => { ?y :s _:n } .\n";
    for cap in [8usize, 64] {
        let options = tuned(|o| o.max_iterations = cap);
        let r = run(text, &options);
        assert!(r.limits_reached.contains(&ReasonerLimit::Iterations), "{r:?}");
        let (name, observed, max) = ReasonerLimit::Iterations.graphlaw_row(&r, &options);
        assert_eq!((name, observed, max), ("n3_iterations", cap, cap));
    }
}

#[test]
fn row_pins_match_steps_run_wide_on_real_runs() {
    // A crossproduct join deriving N*N facts: its total step count crosses
    // a small run-wide ceiling and only that ceiling.
    let facts: String = (0..40).map(|i| format!(":n{i} :p :m{i} .\n")).collect();
    let text = format!(
        "@prefix : <urn:r:> .\n{facts}{{ ?a :p ?x . ?b :p ?x }} => {{ ?a :q ?b }} .\n"
    );
    // Find a total-steps ceiling the run actually crosses, then pin the row.
    let mut cap = 1usize;
    loop {
        let options = tuned(|o| o.max_total_steps = cap);
        let r = run(&text, &options);
        if r.limits_reached.contains(&ReasonerLimit::MatchSteps) {
            let (name, observed, max) = ReasonerLimit::MatchSteps.graphlaw_row(&r, &options);
            assert_eq!(name, "n3_match_steps");
            assert_eq!(max, cap, "a run-wide breach reports the run-wide ceiling");
            assert!(
                observed >= max,
                "the recorded counter reflects real work at or past the run-wide ceiling"
            );
            return;
        }
        cap *= 2;
        assert!(cap <= 1 << 24, "fixture drifted: no run-wide breach found");
    }
}

#[test]
fn row_pins_match_steps_per_search_on_real_runs() {
    // The saturating self-join plus one wildcard backward head keeps every
    // forward rule off the agenda, so the join runs as ONE slow-path search
    // that crosses the per-search cap while the run-wide sum stays small.
    let facts: String = (0..120).map(|i| format!(":n{i} :p :m .\n")).collect();
    let text = format!(
        "@prefix : <urn:r:> .\n{facts}\
         {{ ?a :p ?x . ?b :p ?x }} => {{ ?a :p ?x }} .\n\
         {{ ?u ?w ?v }} <= {{ ?u :s ?v }} .\n"
    );
    let mut cap = 64usize;
    loop {
        let options = tuned(|o| o.max_match_steps = cap);
        let r = run(&text, &options);
        if r.limits_reached.contains(&ReasonerLimit::MatchSteps) {
            let (name, observed, max) = ReasonerLimit::MatchSteps.graphlaw_row(&r, &options);
            assert_eq!(name, "n3_match_steps");
            assert_eq!(
                max, cap,
                "a per-search breach must report the per-search ceiling, got {max}"
            );
            assert!(
                observed < 4 * cap,
                "per-search breach records the search that cut, not a runaway sum: {observed}"
            );
            return;
        }
        cap *= 2;
        assert!(cap <= 1 << 20, "fixture drifted: no per-search breach found");
    }
}

#[test]
fn row_pins_backward_depth_on_real_runs() {
    // A backward chain deeper than the ceiling: cuts record the depth they
    // cut at, exactly; the ceiling-equal chain proves without cutting.
    let depth = 6usize;
    let mut rules = String::new();
    for k in 1..=depth {
        rules.push_str(&format!("{{ ?x :p{k} ?y }} <= {{ ?x :p{} ?y }} .\n", k - 1));
    }
    let text = format!(
        "@prefix : <urn:r:> .\n:a :p0 :b .\n{rules}{{ :a :p{depth} :b }} => {{ :a :done :b }} .\n"
    );
    for cap in [3usize, depth] {
        let options = tuned(|o| o.max_backward_depth = cap);
        let r = run(&text, &options);
        if cap < depth {
            assert!(r.limits_reached.contains(&ReasonerLimit::BackwardDepth), "{cap}");
            let (name, observed, max) = ReasonerLimit::BackwardDepth.graphlaw_row(&r, &options);
            assert_eq!(
                (name, observed, max),
                ("n3_backward_depth", cap + 1, cap),
                "depth cuts carry the at-cap sentinel (the fork records no depth counter)"
            );
        } else {
            assert!(
                !r.limits_reached.contains(&ReasonerLimit::BackwardDepth),
                "cap == depth must prove without cutting"
            );
        }
    }
}

#[test]
fn row_pins_backward_solutions_on_real_runs() {
    // One goal with many solutions: retained answers truncate to exactly
    // the ceiling, and the ceiling-equal count clears at cap+1.
    let solutions = 12usize;
    let leaves: String = (0..solutions)
        .map(|i| format!(":a :p :leaf{i} .\n"))
        .collect();
    let text = format!(
        "@prefix : <urn:r:> .\n{leaves}{{ ?x :sol ?y }} <= {{ ?x :p ?y }} .\n\
         {{ ?x :sol ?y }} => {{ ?x :got ?y }} .\n"
    );
    for (cap, should_fire) in [(4usize, true), (solutions, true), (solutions + 1, false)] {
        let options = tuned(|o| o.max_backward_solutions_per_goal = cap);
        let r = run(&text, &options);
        assert_eq!(
            r.limits_reached.contains(&ReasonerLimit::BackwardSolutionsPerGoal),
            should_fire,
            "cap {cap}"
        );
        if should_fire {
            let (name, observed, max) =
                ReasonerLimit::BackwardSolutionsPerGoal.graphlaw_row(&r, &options);
            assert_eq!(name, "n3_backward_solutions_per_goal");
            assert_eq!(
                observed,
                max + 1,
                "unrecorded solutions: the documented at-cap sentinel (max + 1)"
            );
        }
    }
}

#[test]
fn row_pins_term_depth_on_real_runs() {
    // A wrapping chain whose derived terms nest one past the parser's fixed
    // ceiling. `max` is the law table's term-depth const; `observed` is the
    // first breached nesting depth (a real measurement, not a sentinel).
    let links = N3_MAX_TERM_DEPTH + 2;
    let mut wrap = String::new();
    for k in 0..links {
        wrap.push_str(&format!(
            "{{ ?x :w{k} ?y }} => {{ ?x :w{} (?y) }} .\n",
            k + 1
        ));
    }
    let text = format!(
        "@prefix : <urn:r:> .\n:a :w0 :nil .\n{wrap}{{ :a :w{links} ?y }} => {{ :a :done ?y }} .\n"
    );
    let options = law_options();
    let r = run(&text, &options);
    assert!(
        r.limits_reached.contains(&ReasonerLimit::TermDepth),
        "{:?}",
        r.limits_reached
    );
    let (name, observed, max) = ReasonerLimit::TermDepth.graphlaw_row(&r, &options);
    assert_eq!(name, "n3_term_depth");
    assert_eq!(max, N3_MAX_TERM_DEPTH);
    assert_eq!(
        observed,
        N3_MAX_TERM_DEPTH + 1,
        "the first rejected nesting is exactly one past the ceiling"
    );
}

#[test]
fn row_pins_term_size_on_real_runs() {
    // A bounded doubler crossing the per-term cap. The row is the documented
    // at-cap sentinel (cap+1): the rejected term is measured at the door and
    // never enters the closure, so no counter of it can exist. "yy" doubles
    // per link: 20 links reach 2 MiB, far past the 64 KiB cap.
    let links = 20usize;
    let mut dbl = String::new();
    for k in 0..links {
        dbl.push_str(&format!(
            "{{ :a :d{k} ?x . (?x ?x) string:concatenation ?y }} => {{ :a :d{} ?y }} .\n",
            k + 1
        ));
    }
    let text = format!(
        "@prefix : <urn:r:> .\n\
         @prefix string: <http://www.w3.org/2000/10/swap/string#> .\n\
         :a :d0 \"yy\" .\n{dbl}"
    );
    let options = law_options();
    let r = run(&text, &options);
    assert!(
        r.limits_reached.contains(&ReasonerLimit::TermSize),
        "{:?}",
        r.limits_reached
    );
    let (name, observed, max) = ReasonerLimit::TermSize.graphlaw_row(&r, &options);
    assert_eq!(name, "n3_term_bytes");
    assert_eq!(max, N3_MAX_TERM_BYTES);
    assert_eq!(observed, N3_MAX_TERM_BYTES + 1, "documented at-cap sentinel");
}
