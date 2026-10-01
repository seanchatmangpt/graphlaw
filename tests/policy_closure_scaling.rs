//! Scaling and equivalence falsifiers for the policy backward closure, plus the
//! receipt-store decode size cap and long-string decode (no mocks).

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use graphlaw::dialect::Dialect;
use graphlaw::law::LawState;
use graphlaw::plan::{Action, Plan};
use graphlaw::policy::{Entry, Outcome, PolicyRefusalKind, Problem, admit_parsed};
use graphlaw::receipt_store::{MAX_RECEIPT_BYTES, ReceiptStore, StoreError};

fn facts(f: &[&str]) -> BTreeSet<String> {
    f.iter().map(|s| s.to_string()).collect()
}

fn entry(state: &str, action: &str, outs: &[&str]) -> Entry {
    let mass = 1_000_000 / outs.len() as u64;
    let mut outcomes: Vec<Outcome> = outs
        .iter()
        .map(|o| Outcome {
            state: o.to_string(),
            probability_ppm: mass,
        })
        .collect();
    outcomes[0].probability_ppm += 1_000_000 - mass * outs.len() as u64;
    Entry {
        state: state.into(),
        action: action.into(),
        outcomes,
    }
}

#[test]
fn long_chain_with_last_sorting_goal_is_admitted_quickly() {
    // Goal sorts after every chain state, so a pass-per-hop fixpoint would need
    // one pass per state; the reverse BFS must do one linear sweep.
    const N: usize = 50_000;
    let name = |i: usize| format!("a{i:07}");
    let mut problem = Problem {
        initial: vec![name(0)],
        goal_facts: facts(&["done"]),
        ..Default::default()
    };
    let mut policy = Vec::with_capacity(N);
    for i in 0..N {
        let next = if i + 1 == N {
            "zgoal".to_string()
        } else {
            name(i + 1)
        };
        problem.states.insert(name(i), BTreeSet::new());
        problem
            .transitions
            .insert(("go".into(), name(i), next.clone()));
        policy.push(entry(&name(i), "go", &[next.as_str()]));
    }
    problem.states.insert("zgoal".into(), facts(&["done"]));
    let a = admit_parsed(&problem, &policy).expect("admitted");
    assert_eq!(a.reachable.len(), N + 1);
    assert_eq!(a.goal_states, vec!["zgoal".to_string()]);
    assert_eq!(a.entries.len(), N);
}

#[test]
fn chain_ending_in_a_trap_cycle_is_a_dead_end() {
    let mut problem = Problem {
        initial: vec!["a0".into()],
        goal_facts: facts(&["done"]),
        ..Default::default()
    };
    for s in ["a0", "a1", "t0", "t1"] {
        problem.states.insert(s.into(), BTreeSet::new());
    }
    problem.states.insert("zgoal".into(), facts(&["done"]));
    for (f, t) in [("a0", "a1"), ("a1", "t0"), ("t0", "t1"), ("t1", "t0")] {
        problem
            .transitions
            .insert(("go".into(), f.into(), t.into()));
    }
    let policy = [
        entry("a0", "go", &["a1"]),
        entry("a1", "go", &["t0"]),
        entry("t0", "go", &["t1"]),
        entry("t1", "go", &["t0"]),
    ];
    let e = admit_parsed(&problem, &policy).unwrap_err();
    assert_eq!(e.kind, PolicyRefusalKind::DeadEnd);
    assert_eq!(e.state, "a0");
}

/// Deterministic xorshift generator (no rand dependency).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

/// Naive reference: reachability from `initial` (stopping at goals), then a
/// repeated-scan fixpoint of states that can reach a goal.
fn naive(initial: usize, goal: &[bool], succ: &[Vec<usize>]) -> Result<Vec<usize>, ()> {
    let mut seen = BTreeSet::from([initial]);
    let mut stack = vec![initial];
    while let Some(s) = stack.pop() {
        if goal[s] {
            continue;
        }
        for &n in &succ[s] {
            if seen.insert(n) {
                stack.push(n);
            }
        }
    }
    let mut good: BTreeSet<usize> = seen.iter().copied().filter(|&s| goal[s]).collect();
    loop {
        let before = good.len();
        for &s in &seen {
            if !goal[s] && succ[s].iter().any(|n| good.contains(n)) {
                good.insert(s);
            }
        }
        if good.len() == before {
            break;
        }
    }
    if seen.iter().all(|s| good.contains(s)) {
        Ok(seen.into_iter().collect())
    } else {
        Err(())
    }
}

#[test]
fn closure_matches_naive_reference_on_small_graphs() {
    let mut rng = Lcg(0x9e37_79b9_7f4a_7c15);
    let mut admitted = 0;
    let mut refused = 0;
    for _ in 0..500 {
        let n = 2 + rng.next(7);
        let goal: Vec<bool> = (0..n).map(|_| rng.next(4) == 0).collect();
        let succ: Vec<Vec<usize>> = (0..n)
            .map(|_| {
                let k = 1 + rng.next(2);
                let mut v: Vec<usize> = (0..k).map(|_| rng.next(n)).collect();
                v.sort();
                v.dedup();
                v
            })
            .collect();
        let name = |i: usize| format!("s{i}");
        let mut problem = Problem {
            initial: vec![name(0)],
            goal_facts: facts(&["done"]),
            ..Default::default()
        };
        let mut policy = Vec::new();
        for i in 0..n {
            problem.states.insert(
                name(i),
                if goal[i] {
                    facts(&["done"])
                } else {
                    facts(&[])
                },
            );
            if goal[i] {
                continue;
            }
            let outs: Vec<String> = succ[i].iter().map(|&t| name(t)).collect();
            for o in &outs {
                problem
                    .transitions
                    .insert(("go".into(), name(i), o.clone()));
            }
            let refs: Vec<&str> = outs.iter().map(String::as_str).collect();
            policy.push(entry(&name(i), "go", &refs));
        }
        match (admit_parsed(&problem, &policy), naive(0, &goal, &succ)) {
            (Ok(a), Ok(reach)) => {
                admitted += 1;
                let mut want: Vec<String> = reach.into_iter().map(name).collect();
                want.sort();
                assert_eq!(a.reachable, want);
                let mut gs: Vec<String> = (0..n)
                    .filter(|&i| goal[i] && a.reachable.contains(&name(i)))
                    .map(name)
                    .collect();
                gs.sort();
                assert_eq!(a.goal_states, gs);
            }
            (Err(e), Err(())) => {
                refused += 1;
                assert_eq!(e.kind, PolicyRefusalKind::DeadEnd);
            }
            (got, want) => panic!("divergence: got {got:?}, naive {want:?}"),
        }
    }
    // Both outcomes must be exercised for the equivalence to mean anything.
    assert!(
        admitted > 0 && refused > 0,
        "admitted {admitted}, refused {refused}"
    );
}

// ---- receipt store: decode cap and long-string decode ----

fn t(o: &str) -> String {
    format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n")
}

fn first_receipt() -> graphlaw::law::Receipt {
    let s = LawState::parse(t("a").as_bytes(), Dialect::NTriples, None).unwrap();
    let plan = Plan {
        actions: vec![Action {
            name: "a-b".into(),
            pre: t("a"),
            add: t("b"),
            del: t("a"),
            ..Default::default()
        }],
        goal: t("b"),
        ..Default::default()
    };
    plan.admit(&s).unwrap().receipts.remove(0)
}

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("graphlaw-closure-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    d
}

const SUBJECT: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn long_multibyte_strings_round_trip_in_linear_time() {
    let dir = scratch("longstr");
    let mut r = first_receipt();
    // ~600 KB of 1-, 2-, 3- and 4-byte chars plus escapes, under the size cap;
    // a per-char whole-tail UTF-8 validation would take minutes here.
    r.child = "a\u{e9}\u{20ac}\u{1f600}\"\\\n".repeat(40_000);
    let store = ReceiptStore::open(&dir).unwrap();
    store.put(&r, SUBJECT).unwrap();
    assert_eq!(store.verify(SUBJECT).unwrap(), vec![r]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn oversized_receipt_file_is_refused_unread() {
    let dir = scratch("toolarge");
    let store = ReceiptStore::open(&dir).unwrap();
    let sd = dir.join(SUBJECT);
    fs::create_dir_all(&sd).unwrap();
    fs::write(sd.join("big.json"), vec![b' '; MAX_RECEIPT_BYTES + 1]).unwrap();
    match store.verify(SUBJECT) {
        Err(StoreError::TooLarge { file, size, limit }) => {
            assert_eq!(file, "big.json");
            assert_eq!(size, limit + 1);
        }
        other => panic!("expected TooLarge, got {other:?}"),
    }
    let _ = fs::remove_dir_all(&dir);
}
