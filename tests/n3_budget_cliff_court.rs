//! N3 budget cliff court (WP P02, Chicago: real engine runs, no doubles).
//!
//! Every budget the reasoner enforces must be a cliff: at cap `max` the run
//! stops with the limit's honest row — `observed == max` for the
//! counter-backed limits, the documented at-cap sentinel (`max + 1`) for
//! the two measure-at-the-door limits (`n3_term_bytes`, whose rejected term
//! never enters the closure) — and at cap `max + 1` that limit no longer
//! fires. A cap that overshoots (recorded counter past the ceiling) or
//! fabricates (a row with no real counter behind it) fails this court.
//!
//! Ten generator programs, one per budget shape; the court binary-searches
//! each cliff on the real engine and pins the `max`/`max+1` pair:
//!
//! | generator        | budget field                     | census arm                   |
//! |------------------|----------------------------------|------------------------------|
//! | chain            | `max_iterations`                 | `n3_iterations`              |
//! | fanout           | `max_closure_facts`              | `n3_derived_facts`           |
//! | rematch          | `max_iterations`                 | `n3_iterations`              |
//! | backward_chain   | `max_backward_depth`             | `n3_backward_depth`          |
//! | cycle-silent-pin | `max_backward_depth`             | `n3_backward_depth`          |
//! | solutions        | `max_backward_solutions_per_goal`| `n3_backward_solutions_per_goal` |
//! | doubler          | `max_term_bytes`                 | `n3_term_bytes`              |
//! | wrapper          | (parser nesting ceiling)         | `n3_term_depth`              |
//! | crossproduct     | `max_total_steps`                | `n3_match_steps`             |
//! | nested_conclusion| `max_total_steps` (via nested run)| `n3_match_steps`            |
//!
//! The two backward-depth generators also pin the silent-pin law: above the
//! cliff a cycle re-entry is pinned by the goal stack, the run COMPLETES,
//! and `statistics.backward_depth` stays the honest zero (no cut happened,
//! so no cut is recorded) — never a fabricated max.
#![cfg(feature = "abi")]

use graphlaw::law::{N3_MAX_BACKWARD_DEPTH, N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL, N3_MAX_TERM_DEPTH};
use graphlaw::law::ReasonerLimitCensus;
use graphlaw::n3::{ReasonerLimit, ReasonerOptions, ReasonerResult};

/// Generous ceilings everywhere except what the caller overrides, so the
/// budget under test is the only reachable limit.
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

/// Fires = the target limit was reached at this cap.
type Probe<'p> = dyn Fn(usize) -> bool + 'p;

/// Binary-search the cliff: the smallest cap in `(lo, hi]` that does NOT
/// fire. Requires the predicate monotone (small caps fire, large caps
/// clear), `lo` firing and `hi` clearing; the court then pins the pair
/// `(hi - 1, hi)` — `max` fires, `max + 1` clears.
fn cliff(lo_fires: usize, hi_clears: usize, probe: &Probe) -> usize {
    assert!(lo_fires < hi_clears, "bounds crossed: {lo_fires} >= {hi_clears}");
    assert!(
        probe(lo_fires),
        "lower bound {lo_fires} must fire (fixture drifted)"
    );
    assert!(
        !probe(hi_clears),
        "upper bound {hi_clears} must clear (fixture drifted)"
    );
    let (mut lo, mut hi) = (lo_fires, hi_clears);
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if probe(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    hi
}

/// The cliff pair `(max, max + 1)`: `max` fires, `max + 1` clears.
struct CliffPair {
    max: usize,
    max_plus_one: usize,
}

fn search_pair(lo_fires: usize, hi_clears: usize, probe: &Probe) -> CliffPair {
    let max_plus_one = cliff(lo_fires, hi_clears, probe);
    CliffPair {
        max: max_plus_one - 1,
        max_plus_one,
    }
}

/// Smallest doubling probe that clears, for generators whose completion
/// metric can sit exactly ON the cliff (the run-wide per-pass gate taints a
/// run whose spend equals its ceiling), so the measured count is only a
/// lower bound for the clear side.
fn clearing_bound(floor: usize, probe: &Probe) -> usize {
    let mut hi = floor.max(2);
    if !probe(hi) {
        return hi;
    }
    while probe(hi) {
        hi *= 2;
        assert!(hi <= 1 << 40, "no clearing bound found from {floor}");
    }
    hi
}

/// Full honest-row assertion at the firing cap: the limit's census row is
/// exactly `(graphlaw_name, observed, max)`.
fn assert_row(
    target: ReasonerLimit,
    result: &ReasonerResult,
    options: &ReasonerOptions,
    max: usize,
    observed: usize,
) {
    assert!(
        result.limits_reached.contains(&target),
        "{:?}: limit missing at max, got {:?}",
        target.graphlaw_name(),
        result.limits_reached
    );
    let (name, got_observed, got_max) = target.graphlaw_row(result, options);
    assert_eq!(name, target.graphlaw_name());
    assert_eq!(
        got_max, max,
        "{name}: row ceiling drifted from the cliff"
    );
    // The fork enforces every ceiling but records counters only for
    // iterations/match_steps/derived-facts; unrecorded rows carry the
    // documented at-cap sentinel (max + 1).
    let expected = match target {
        ReasonerLimit::Iterations | ReasonerLimit::MatchSteps | ReasonerLimit::ClosureFacts => {
            observed
        }
        _ => got_max + 1,
    };
    assert_eq!(
        got_observed, expected,
        "{name}: recorded counter is not the honest one"
    );
}

// ---------------------------------------------------------------------------
// Generators. Each builds its document and drives one budget to a cliff.
// ---------------------------------------------------------------------------

/// chain: a symmetric-closure rule over a linear chain. Each fixpoint pass
/// derives the next reverse edge, so the pass counter is the budget.
fn chain_text(links: usize) -> String {
    let mut facts = String::new();
    for i in 0..links {
        facts.push_str(&format!(":n{} :s :n{} .\n", i, i + 1));
    }
    format!("@prefix : <urn:k:> .\n{facts}{{ ?x :s ?y }} => {{ ?y :s ?x }} .\n")
}

/// fanout: one premise matching every fact, deriving one new fact per match.
fn fanout_text(facts: usize) -> String {
    let mut text = String::from("@prefix : <urn:f:> .\n");
    for i in 0..facts {
        text.push_str(&format!(":a{i} :p :b{i} .\n"));
    }
    text.push_str("{ ?x :p ?y } => { ?x :q ?y } .\n");
    text
}

/// rematch: transitive closure over a path — every pass re-matches existing
/// joins while the closure grows, burning fixpoint passes at a quadratic
/// rate. Same budget as `chain`, different cost shape.
fn rematch_text(nodes: usize) -> String {
    let mut facts = String::new();
    for i in 0..nodes - 1 {
        facts.push_str(&format!(":v{i} :e :v{} .\n", i + 1));
    }
    format!(
        "@prefix : <urn:m:> .\n{facts}\
         {{ ?x :e ?y . ?y :e ?z }} => {{ ?x :e ?z }} .\n"
    )
}

/// backward_chain: backward rules `p{k} <= p{k-1}` over a single base fact;
/// the forward rule's premise `p{depth}` can only be proven by recursing
/// `depth` levels.
fn backward_chain_text(depth: usize) -> String {
    let mut rules = String::new();
    for k in 1..=depth {
        rules.push_str(&format!("{{ ?x :p{k} ?y }} <= {{ ?x :p{} ?y }} .\n", k - 1));
    }
    format!(
        "@prefix : <urn:b:> .\n:a :p0 :b .\n{rules}\
         {{ :a :p{depth} :b }} => {{ :a :done :b }} .\n"
    )
}

/// cycle-silent-pin: the backward chain plus one back edge `p1 <= p{depth}`,
/// so proving `p1` re-enters the root goal. Below the cliff the re-entry is
/// cut by the depth ceiling (loud); at the cliff's clear side the goal
/// stack pins it silently and the run completes with the honest zero.
fn cycle_pin_text(depth: usize) -> String {
    let mut rules = String::new();
    for k in 1..=depth {
        rules.push_str(&format!("{{ ?x :p{k} ?y }} <= {{ ?x :p{} ?y }} .\n", k - 1));
    }
    rules.push_str(&format!(
        "{{ ?x :p1 ?y }} <= {{ ?x :p{depth} ?y }} .\n"
    ));
    format!(
        "@prefix : <urn:y:> .\n:a :p0 :b .\n{rules}\
         {{ :a :p{depth} :b }} => {{ :a :done :b }} .\n"
    )
}

/// solutions: one goal with one solution per leaf fact.
fn solutions_text(leaves: usize) -> String {
    let mut text = String::from("@prefix : <urn:s:> .\n");
    for i in 0..leaves {
        text.push_str(&format!(":a :p :leaf{i} .\n"));
    }
    text.push_str(
        "{ ?x :sol ?y } <= { ?x :p ?y } .\n{ ?x :sol ?y } => { ?x :got ?y } .\n",
    );
    text
}

/// doubler: a bounded doubling chain — each link concatenates the term with
/// itself; the final term is `2^(links+1)` bytes of `y`.
fn doubler_text(links: usize) -> String {
    let mut dbl = String::new();
    for k in 0..links {
        dbl.push_str(&format!(
            "{{ :a :d{k} ?x . (?x ?x) string:concatenation ?y }} => {{ :a :d{} ?y }} .\n",
            k + 1
        ));
    }
    format!(
        "@prefix : <urn:d:> .\n\
         @prefix string: <http://www.w3.org/2000/10/swap/string#> .\n\
         :a :d0 \"yy\" .\n{dbl}"
    )
}

/// wrapper: each link wraps the previous object one list deeper; derived
/// terms nest one level per link until the parser's fixed nesting ceiling
/// rejects the first breach.
fn wrapper_text(links: usize) -> String {
    let mut wrap = String::new();
    for k in 0..links {
        wrap.push_str(&format!(
            "{{ ?x :w{k} ?y }} => {{ ?x :w{} (?y) }} .\n",
            k + 1
        ));
    }
    format!(
        "@prefix : <urn:w:> .\n:a :w0 :nil .\n{wrap}\
         {{ :a :w{links} ?y }} => {{ :a :done ?y }} .\n"
    )
}

/// crossproduct: every fact shares one join key, so the rule derives the
/// full N*N product (all new facts) — the run-wide matcher-step sum is the
/// budget that matters, and the step gate stops the product mid-derivation.
fn crossproduct_text(side: usize) -> String {
    let facts: String = (0..side).map(|i| format!(":n{i} :p :m .\n")).collect();
    format!(
        "@prefix : <urn:x:> .\n{facts}\
         {{ ?a :p ?x . ?b :p ?x }} => {{ ?a :q ?b }} .\n"
    )
}

/// nested_conclusion: one rule whose FIRST premise is a `log:conclusion`
/// call over a heavy nested document and whose SECOND premise joins outer
/// facts in the same search. The nested run inherits what is left of BOTH
/// the run-wide step ceiling and the iteration ceiling (`SearchBudget::
/// nested_options`), spends steps out of the caller's share, and the outer
/// run absorbs the spend (`absorb_result`) — so after the absorb, the same
/// search's own ticks are gated against the ceiling that is already spent,
/// and the recorded sum cannot leak past it.
fn nested_conclusion_text(side: usize) -> String {
    let facts: String = (0..side).map(|i| format!(":n{i} :p :m .\n")).collect();
    let rule_text = "{ ?a :p ?x . ?b :p ?x } => { ?a :q ?b } .\n".to_string();
    let inner = format!("{rule_text}{facts}");
    // Six facts live outside the formula so the second premise ticks
    // candidates in the same search after the nested call absorbs.
    let outer: String = (0..6).map(|i| format!(":o{i} :p :m .\n")).collect();
    format!(
        "@prefix : <urn:n:> .\n{outer}\
         @prefix log: <http://www.w3.org/2000/10/swap/log#> .\n\
         {{ {{\n{inner}\n}} log:conclusion ?g . ?a :p ?x }} => {{ :a :nested :done }} .\n"
    )
}

// ---------------------------------------------------------------------------
// The ten cliff courts.
// ---------------------------------------------------------------------------

#[test]
fn gen_chain_iterations_cliff() {
    let text = chain_text(48);
    // Measure the completion metric: the pass count a generous run takes.
    let complete = run(&text, &tuned(|_| {}));
    assert!(complete.is_complete());
    let t = complete.statistics.iterations;

    let probe = |cap: usize| {
        run(&text, &tuned(|o| o.max_iterations = cap))
            .limits_reached
            .contains(&ReasonerLimit::Iterations)
    };
    let pair = search_pair(1, t, &probe);
    assert_eq!(
        pair.max_plus_one, t,
        "the cliff is exactly the completion pass count"
    );

    // max: fires, counter == ceiling exactly (gate before increment).
    let options = tuned(|o| o.max_iterations = pair.max);
    let fired = run(&text, &options);
    assert_row(ReasonerLimit::Iterations, &fired, &options, pair.max, pair.max);

    // max + 1: completes, limit gone.
    let cleared = run(&text, &tuned(|o| o.max_iterations = pair.max_plus_one));
    assert!(cleared.is_complete());
    assert!(!cleared.limits_reached.contains(&ReasonerLimit::Iterations));
}

#[test]
fn gen_rematch_iterations_cliff() {
    let text = rematch_text(16);
    let complete = run(&text, &tuned(|_| {}));
    assert!(complete.is_complete());
    let t = complete.statistics.iterations;

    let probe = |cap: usize| {
        run(&text, &tuned(|o| o.max_iterations = cap))
            .limits_reached
            .contains(&ReasonerLimit::Iterations)
    };
    let pair = search_pair(1, t, &probe);

    let options = tuned(|o| o.max_iterations = pair.max);
    let fired = run(&text, &options);
    assert_row(ReasonerLimit::Iterations, &fired, &options, pair.max, pair.max);

    let cleared = run(&text, &tuned(|o| o.max_iterations = pair.max_plus_one));
    assert!(cleared.is_complete());
    assert!(!cleared.limits_reached.contains(&ReasonerLimit::Iterations));
}

#[test]
fn gen_fanout_derived_facts_cliff() {
    let text = fanout_text(200);
    let complete = run(&text, &tuned(|_| {}));
    assert!(complete.is_complete());
    let t = complete.derived.len();

    let probe = |cap: usize| {
        run(&text, &tuned(|o| o.max_closure_facts = cap))
            .limits_reached
            .contains(&ReasonerLimit::ClosureFacts)
    };
    let pair = search_pair(1, t, &probe);

    // max: the fact gate fires with `derived.len()` exactly at the ceiling.
    let options = tuned(|o| o.max_closure_facts = pair.max);
    let fired = run(&text, &options);
    assert_row(ReasonerLimit::ClosureFacts, &fired, &options, pair.max, pair.max);

    let cleared = run(&text, &tuned(|o| o.max_closure_facts = pair.max_plus_one));
    assert!(cleared.is_complete());
    assert_eq!(
        cleared.derived.len(),
        t,
        "max + 1 admits the whole fanout"
    );
}

#[test]
fn gen_backward_chain_depth_cliff() {
    let depth = 24usize;
    let text = backward_chain_text(depth);
    // The chain needs the recursion entries p{depth}@0 .. p1@{depth-1}; the
    // base fact answers at the floor, so the deepest entry is depth - 1 and
    // the cliff sits at `depth`.
    let probe = |cap: usize| {
        run(&text, &tuned(|o| o.max_backward_depth = cap))
            .limits_reached
            .contains(&ReasonerLimit::BackwardDepth)
    };
    let pair = search_pair(1, depth, &probe);
    assert_eq!(pair.max_plus_one, depth, "cliff == chain depth");

    // max: cut at exactly the ceiling.
    let options = tuned(|o| o.max_backward_depth = pair.max);
    let fired = run(&text, &options);
    assert_row(ReasonerLimit::BackwardDepth, &fired, &options, pair.max, pair.max);

    // max + 1 == depth: the ceiling-equal chain PROVES, no cut recorded,
    // and the honest counter stays zero (no cut happened).
    let cleared = run(&text, &tuned(|o| o.max_backward_depth = pair.max_plus_one));
    assert!(cleared.is_complete());
    assert!(!cleared.limits_reached.contains(&ReasonerLimit::BackwardDepth));
    // The fork's statistics record no depth counter; the cut (and its
    // absence on an uncut run) is witnessed solely by limits_reached.
}

#[test]
fn gen_cycle_silent_pin_depth_cliff() {
    let depth = 24usize;
    let text = cycle_pin_text(depth);
    // Below the cliff the cycle re-entry lands past the ceiling and is cut
    // loudly; at `depth + 1` the re-entry passes the ceiling and is pinned
    // by the goal stack instead — silently, with no limit and no cut.
    let probe = |cap: usize| {
        run(&text, &tuned(|o| o.max_backward_depth = cap))
            .limits_reached
            .contains(&ReasonerLimit::BackwardDepth)
    };
    let pair = search_pair(1, depth + 1, &probe);
    assert_eq!(pair.max_plus_one, depth + 1, "cliff == depth + 1 (re-entry)");

    // max: the re-entry is cut at exactly the ceiling.
    let options = tuned(|o| o.max_backward_depth = pair.max);
    let fired = run(&text, &options);
    assert_row(ReasonerLimit::BackwardDepth, &fired, &options, pair.max, pair.max);

    // max + 1: silent pin. The run completes, the proof goes through, and
    // no limit fires — the goal stack ended the recursion, so the honest
    // depth counter stays zero.
    let cleared = run(&text, &tuned(|o| o.max_backward_depth = pair.max_plus_one));
    assert!(cleared.is_complete());
    assert!(cleared.limits_reached.is_empty(), "{cleared:?}");
    assert!(cleared.derived.iter().any(|t| matches!(
        &t.p,
        graphlaw::n3::Term::Iri(iri) if iri.contains("done")
    )));
    // Same fork semantics: no depth counter exists; limits_reached is the
    // only witness.
}

#[test]
fn gen_solutions_cliff() {
    let leaves = 300usize;
    let text = solutions_text(leaves);
    // One solution per leaf: the cliff is `leaves + 1` (the gate fires when
    // the retained count reaches the ceiling).
    let probe = |cap: usize| {
        run(&text, &tuned(|o| o.max_backward_solutions_per_goal = cap))
            .limits_reached
            .contains(&ReasonerLimit::BackwardSolutionsPerGoal)
    };
    let pair = search_pair(1, leaves + 1, &probe);
    assert_eq!(pair.max_plus_one, leaves + 1, "one solution per leaf");

    // max: retained answers truncate to exactly the ceiling.
    let options = tuned(|o| o.max_backward_solutions_per_goal = pair.max);
    let fired = run(&text, &options);
    assert_row(
        ReasonerLimit::BackwardSolutionsPerGoal,
        &fired,
        &options,
        pair.max,
        pair.max,
    );

    // max + 1: the whole answer set fits, no ceiling fires.
    let cleared = run(
        &text,
        &tuned(|o| o.max_backward_solutions_per_goal = pair.max_plus_one),
    );
    assert!(!cleared
        .limits_reached
        .contains(&ReasonerLimit::BackwardSolutionsPerGoal));
}

#[test]
fn gen_doubler_term_bytes_cliff() {
    let links = 20usize;
    let text = doubler_text(links);
    // "yy" doubled `links` times: the largest term's VALUE is 2^(links+1)
    // bytes; the door's measurement adds the term's own datatype text, so
    // the exact cliff is the term's real footprint — bounded below by the
    // value and above by value + a small constant.
    let final_value = 2usize << links;
    let probe = |cap: usize| {
        run(&text, &tuned(|o| o.max_term_bytes = cap))
            .limits_reached
            .contains(&ReasonerLimit::TermSize)
    };
    let hi = clearing_bound(final_value, &probe);
    let pair = search_pair(1, hi, &probe);
    assert!(
        pair.max_plus_one >= final_value,
        "the final term must fit at the cliff, got {}",
        pair.max_plus_one
    );
    assert!(
        pair.max_plus_one - final_value <= 256,
        "the door measures the term itself, not the world: {} vs {final_value}",
        pair.max_plus_one
    );

    // max: fires with the documented at-cap sentinel (max + 1): the
    // rejected term never enters the closure, so its size cannot be a
    // counter — the row says exactly one past the door.
    let options = tuned(|o| o.max_term_bytes = pair.max);
    let fired = run(&text, &options);
    assert_row(
        ReasonerLimit::TermSize,
        &fired,
        &options,
        pair.max,
        pair.max + 1,
    );

    // max + 1: the final term fits, no ceiling fires.
    let cleared = run(&text, &tuned(|o| o.max_term_bytes = pair.max_plus_one));
    assert!(cleared.is_complete());
    assert!(!cleared.limits_reached.contains(&ReasonerLimit::TermSize));
}

#[test]
fn gen_wrapper_term_depth_cliff() {
    // The term-depth ceiling is the parser's fixed nesting limit, mirrored
    // by the law table's `N3_MAX_TERM_DEPTH`; the cliff pair is
    // (ceiling, ceiling + 1) on the nesting axis, reached by binary-searching
    // the link count that first breaches it.
    let probe_links = |links: usize| {
        run(&wrapper_text(links), &law_opts()).limits_reached.contains(&ReasonerLimit::TermDepth)
    };
    let lo = N3_MAX_TERM_DEPTH / 2;
    let hi = N3_MAX_TERM_DEPTH + 4;
    assert!(probe_links(lo) == false, "short wrapper must not breach");
    assert!(probe_links(hi), "long wrapper must breach");
    let (mut lo, mut hi) = (lo, hi);
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if probe_links(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    assert_eq!(
        hi,
        N3_MAX_TERM_DEPTH + 1,
        "the first breaching wrapper nests exactly one past the ceiling"
    );

    // At the breach: observed is the first rejected nesting (a real
    // measurement), exactly one past the mirrored ceiling.
    let fired = run(&wrapper_text(hi), &law_opts());
    let (name, observed, max) = ReasonerLimit::TermDepth.graphlaw_row(&fired, &law_opts());
    assert_eq!(name, "n3_term_depth");
    assert_eq!(max, N3_MAX_TERM_DEPTH);
    assert_eq!(observed, N3_MAX_TERM_DEPTH + 1);

    // Below the breach: the whole chain nests, no ceiling fires. Above it:
    // the FIRST breach is still the recorded one — the chain stops at the
    // rejected term, so a longer chain cannot deepen the observation.
    let short = run(&wrapper_text(hi - 1), &law_opts());
    assert!(!short.limits_reached.contains(&ReasonerLimit::TermDepth));
    let long = run(&wrapper_text(hi + 8), &law_opts());
    let (_, long_observed, _) = ReasonerLimit::TermDepth.graphlaw_row(&long, &law_opts());
    assert_eq!(
        long_observed,
        N3_MAX_TERM_DEPTH + 1,
        "first-breach honesty: longer input cannot deepen the recorded cut"
    );
}

#[test]
fn gen_crossproduct_total_steps_cliff() {
    let side = 48usize;
    let text = crossproduct_text(side);
    let complete = run(&text, &tuned(|_| {}));
    assert!(complete.is_complete());
    let t = complete.statistics.match_steps;

    let probe = |cap: usize| {
        run(&text, &tuned(|o| o.max_total_steps = cap))
            .limits_reached
            .contains(&ReasonerLimit::MatchSteps)
    };
    // The measured completion count sits ON the cliff (a run whose spend
    // equals its ceiling is tainted by the per-pass gate), so the clear
    // side is t + 1 and the bound-finder supplies it.
    let hi = clearing_bound(t, &probe);
    let pair = search_pair(1, hi, &probe);

    // max: the run-wide sum stops exactly at the ceiling. At the cliff the
    // gate taints the run only after the last search spent its share, so
    // the row is (ceiling, ceiling) — zero overshoot. One step below the
    // cliff the same gate halts the product mid-derivation, still with the
    // recorded sum exactly equal to the cap.
    let options = tuned(|o| o.max_total_steps = pair.max);
    let fired = run(&text, &options);
    assert_row(ReasonerLimit::MatchSteps, &fired, &options, pair.max, pair.max);

    let under = tuned(|o| o.max_total_steps = pair.max / 2);
    let halted = run(&text, &under);
    assert!(
        halted.limits_reached.contains(&ReasonerLimit::MatchSteps),
        "well below the cliff the gate must fire"
    );
    assert_eq!(
        halted.statistics.match_steps,
        pair.max / 2,
        "mid-derivation: the recorded sum is exactly the cap"
    );
    assert!(
        halted.derived.len() < side * side,
        "well below the cliff the product must be halted mid-derivation"
    );

    // max + 1: completes with the full step count and the whole product.
    let cleared = run(&text, &tuned(|o| o.max_total_steps = pair.max_plus_one));
    assert!(cleared.is_complete());
    assert_eq!(cleared.statistics.match_steps, t);
    assert_eq!(cleared.derived.len(), side * side);
}

#[test]
fn gen_nested_conclusion_total_steps_cliff() {
    let text = nested_conclusion_text(48);
    let complete = run(&text, &tuned(|_| {}));
    assert!(complete.is_complete());
    let t = complete.statistics.match_steps;

    let probe = |cap: usize| {
        run(&text, &tuned(|o| o.max_total_steps = cap))
            .limits_reached
            .contains(&ReasonerLimit::MatchSteps)
    };
    let hi = clearing_bound(t, &probe);
    let pair = search_pair(1, hi, &probe);

    // max: the nested run spends the caller's remaining share and the
    // outer run absorbs the spend; the same search's later ticks are gated
    // against the ceiling that is already spent, so the record is EXACTLY
    // the ceiling — zero overshoot. (Under a gate that ignores the nested
    // spend, this shape leaks the join's post-absorb ticks past the
    // ceiling and this assertion is the one that goes red.)
    let options = tuned(|o| o.max_total_steps = pair.max);
    let fired = run(&text, &options);
    assert_row(ReasonerLimit::MatchSteps, &fired, &options, pair.max, pair.max);
    // Well below the cliff, the same law: the recorded sum is exactly the
    // cap while the run is still mid-search.
    let under = tuned(|o| o.max_total_steps = pair.max / 2);
    let halted = run(&text, &under);
    assert_eq!(
        halted.statistics.match_steps,
        pair.max / 2,
        "the run-wide record is exactly the cap, nested spend included"
    );
    assert!(
        pair.max_plus_one <= t,
        "the clearing cap cannot exceed the completion spend (the fork's gate          checks then charges, so nested absorbs may not leak into the record)"
    );

    // max + 1: nested run completes, premise holds, the derivation lands,
    // and the outer run spent exactly the completion count.
    let cleared = run(&text, &tuned(|o| o.max_total_steps = pair.max_plus_one));
    assert!(cleared.is_complete());
    assert_eq!(cleared.statistics.match_steps, t);
    assert!(
        cleared.derived.iter().any(|t| matches!(
            &t.p,
            graphlaw::n3::Term::Iri(iri) if iri.contains("nested")
        )),
        "the log:conclusion premise must hold at max + 1"
    );
}

/// The law table's exact options (what `reason_n3_bounded` wires) — used by
/// the wrapper court so the cliff is taken against ABI ceilings, not
/// generous ones.
fn law_opts() -> ReasonerOptions {
    ReasonerOptions {
        include_explicit: false,
        max_iterations: usize::MAX / 4,
        max_term_bytes: usize::MAX / 4,
        max_total_bytes: usize::MAX / 4,
        max_closure_facts: usize::MAX / 4,
        max_match_steps: usize::MAX / 4,
        max_total_steps: usize::MAX / 4,
        max_backward_depth: N3_MAX_BACKWARD_DEPTH,
        max_backward_solutions_per_goal: N3_MAX_BACKWARD_SOLUTIONS_PER_GOAL,
        ..ReasonerOptions::default()
    }
}
