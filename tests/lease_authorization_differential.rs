//! Differential court over the kernel's lease-authorization law surface
//! (composition C14 seed, lane L-wave-C14).
//!
//! Prior coverage was point assertions only: no court anywhere compared the
//! kernel's own law *decision* code (`Lease::authorize`, `SignedLease::authorize`,
//! `Ceiling` algebra) against an independent second implementation of the same
//! semantics over a generated corpus. This file adds that court.
//!
//! Subset chosen (bounded + fully specified by the doc contracts in
//! `src/law.rs`):
//!
//! 1. unsigned lease authorization — refuse order is `expired` (iff
//!    `now >= expires_unix`), then `out_of_scope` (step name not in scope),
//!    then `ceiling` (required > lease ceiling); otherwise authorized;
//! 2. signed lease authorization — signature-trust and issuer-clock-skew
//!    gates *precede* the lease checks (`untrusted_key`/`bad_signature`, then
//!    `clock_skew` iff `issued > now + max_skew`);
//! 3. ceiling algebra — total order `observe < select < construct`,
//!    `parse`/`name` inverses;
//! 4. registry consistency — every `Step` variant's `name()` /
//!    `required_ceiling()` agrees with the wire-registry `LAW_STEPS` row of
//!    the same operation (`admit:`/`derive:` + registry name).
//!
//! Semantics are NOT the chain fold (that already has dual courts) and NOT
//! SPARQL gate row-ness (strata_gate_differential covers engine-vs-rdflib).
//! The reference evaluator below is written from the doc-comment spec only,
//! sharing no code with the kernel. Fail-closed: any divergence aborts the
//! court with a typed message naming every input of the diverging case.
//!
//! Sabotage twin: `sabotaged_reference` flips one case's verdict; a dedicated
//! test proves the court detects it (the corpus runner is pure, so the twin
//! mutates the reference, not the fixture).

use graphlaw::attest::{SigningKey, TrustedKeys, sign_lease};
use graphlaw::dialect::Dialect;
use graphlaw::law::{Ceiling, FixedClock, LawState, Lease, Step};
use graphlaw::registry::{LAW_STEPS, LEASE_CEILINGS};

// ---------------------------------------------------------------------------
// Deterministic corpus generation (no rand dependency; xorshift64* generator)
// ---------------------------------------------------------------------------

/// Fixed seed so the corpus is byte-stable across runs and machines.
const CORPUS_SEED: u64 = 0xC14_5EED_0001;

struct XorShift(u64);

impl XorShift {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len() as u64) as usize]
    }
}

/// The 8 step names the kernel can be asked to authorize.
const STEP_NAMES: [&str; 8] = [
    "admit:shacl",
    "derive:n3",
    "derive:hooks",
    "admit:plan",
    "admit:require-receipt",
    "admit:require-signed-receipt",
    "derive:rdfs",
    "derive:owl-rl",
];

const CEILING_NAMES: [&str; 3] = ["observe", "select", "construct"];

/// One generated authorization question: lease + step + required ceiling +
/// verifier clock (+ signing trust for signed cases).
#[derive(Debug, Clone)]
struct Case {
    id: usize,
    lease_id: String,
    holder: String,
    ceiling: &'static str,
    scope: Vec<&'static str>,
    expires_unix: u64,
    /// Signed case: the lease is attested by the trusted key and checked with
    /// `issued_unix` at `max_skew_secs`. `None` = unsigned path.
    signed: Option<SignedCase>,
    step: &'static str,
    required: &'static str,
    now_unix: u64,
}

#[derive(Debug, Clone, Copy)]
struct SignedCase {
    issued_unix: u64,
    max_skew_secs: u64,
    /// `true` = verifier trusts the issuing key; `false` = verifier trusts
    /// nothing (kernel must refuse `untrusted_key` before any lease check).
    trusted: bool,
}

/// Boundary-heavy `now`/`expires`/`issued` triples around the decision edges.
const BOUNDARY_NOWS: [u64; 6] = [0, 999, 1_000, 1_001, 1_500, u64::MAX];

fn gen_corpus() -> Vec<Case> {
    let mut rng = XorShift(CORPUS_SEED);
    let mut cases = Vec::new();

    // 1. Pinned boundary cases (exact edges of each refusal arm).
    let mk = |id: usize,
              lease_id: &str,
              ceiling: &str,
              scope: &[&str],
              expires: u64,
              step: &str,
              required: &str,
              now: u64| Case {
        id,
        lease_id: lease_id.into(),
        holder: "court".into(),
        ceiling: leak(ceiling),
        scope: scope.iter().map(|s| leak(s)).collect(),
        expires_unix: expires,
        signed: None,
        step: leak(step),
        required: leak(required),
        now_unix: now,
    };
    cases.push(mk(
        0,
        "b0",
        "construct",
        &["derive:rdfs"],
        1_000,
        "derive:rdfs",
        "construct",
        999,
    ));
    cases.push(mk(
        1,
        "b1",
        "construct",
        &["derive:rdfs"],
        1_000,
        "derive:rdfs",
        "construct",
        1_000,
    ));
    cases.push(mk(
        2,
        "b2",
        "construct",
        &["derive:rdfs"],
        1_000,
        "derive:rdfs",
        "construct",
        1_001,
    ));
    cases.push(mk(
        3,
        "b3",
        "observe",
        &[],
        1_000,
        "derive:rdfs",
        "observe",
        500,
    ));
    cases.push(mk(
        4,
        "b4",
        "select",
        &["admit:plan"],
        1_000,
        "admit:plan",
        "select",
        500,
    ));
    cases.push(mk(
        5,
        "b5",
        "observe",
        &["admit:plan"],
        1_000,
        "admit:plan",
        "select",
        500,
    ));
    cases.push(mk(
        6,
        "b6",
        "select",
        &["admit:plan"],
        1_000,
        "admit:plan",
        "construct",
        500,
    ));
    cases.push(mk(
        7,
        "b7",
        "construct",
        &["derive:rdfs"],
        0,
        "derive:rdfs",
        "observe",
        0,
    ));
    cases.push(mk(
        8,
        "b8",
        "construct",
        &["derive:rdfs"],
        1_000,
        "derive:nonexistent",
        "construct",
        500,
    ));
    cases.push(mk(
        9,
        "b9",
        "observe",
        &["admit:shacl", "admit:plan"],
        1_000,
        "admit:shacl",
        "observe",
        500,
    ));
    cases.push(mk(
        10,
        "b10",
        "select",
        &["admit:plan"],
        u64::MAX,
        "admit:plan",
        "select",
        u64::MAX,
    ));

    // 2. Random unsigned cases.
    while cases.len() < 260 {
        let id = cases.len();
        let scope: Vec<&'static str> = STEP_NAMES
            .iter()
            .copied()
            .filter(|_| rng.below(2) == 0)
            .collect();
        cases.push(Case {
            id,
            lease_id: format!("lease-{id}"),
            holder: format!("agent-{}", rng.below(4)),
            ceiling: rng.pick(&CEILING_NAMES),
            scope,
            expires_unix: *rng.pick(&BOUNDARY_NOWS),
            signed: None,
            step: if rng.below(8) == 0 {
                "derive:nonexistent"
            } else {
                rng.pick(&STEP_NAMES)
            },
            required: rng.pick(&CEILING_NAMES),
            now_unix: *rng.pick(&BOUNDARY_NOWS),
        });
    }

    // 3. Signed cases: trusted + untrusted, skew boundary at exactly
    //    issued == now+skew (allowed) and now+skew+1 (refused).
    let mut id = cases.len();
    for &trusted in &[true, false] {
        for &(issued, skew, now) in &[
            (1_000u64, 60u64, 1_000u64), // issued == now: fine
            (1_060, 60, 1_000),          // issued == now + skew: allowed
            (1_061, 60, 1_000),          // one past: clock_skew
            (2_000, 60, 1_000),          // far future: clock_skew
            (500, 60, 1_000),            // issued before now: fine
        ] {
            cases.push(Case {
                id,
                lease_id: format!("signed-{id}"),
                holder: "issuer".into(),
                ceiling: "construct",
                scope: vec!["derive:rdfs"],
                expires_unix: 1_500,
                signed: Some(SignedCase {
                    issued_unix: issued,
                    max_skew_secs: skew,
                    trusted,
                }),
                step: "derive:rdfs",
                required: "construct",
                now_unix: now,
            });
            id += 1;
        }
    }

    // 4. Random signed cases on top of random lease geometry.
    while cases.len() < 420 {
        let id = cases.len();
        let scope: Vec<&'static str> = STEP_NAMES
            .iter()
            .copied()
            .filter(|_| rng.below(2) == 0)
            .collect();
        cases.push(Case {
            id,
            lease_id: format!("signed-{id}"),
            holder: format!("agent-{}", rng.below(4)),
            ceiling: rng.pick(&CEILING_NAMES),
            scope,
            expires_unix: *rng.pick(&BOUNDARY_NOWS),
            signed: Some(SignedCase {
                issued_unix: *rng.pick(&BOUNDARY_NOWS),
                max_skew_secs: [0, 60, 600][rng.below(3) as usize],
                trusted: rng.below(4) != 0,
            }),
            step: rng.pick(&STEP_NAMES),
            required: rng.pick(&CEILING_NAMES),
            now_unix: *rng.pick(&BOUNDARY_NOWS),
        });
    }

    cases
}

fn leak(s: &str) -> &'static str {
    // Test-local corpus constants: the strings above are 'static anyway.
    Box::leak(s.to_string().into_boxed_str())
}

// ---------------------------------------------------------------------------
// Independent reference evaluator (written from the doc-comment spec only)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Ok,
    Refused(&'static str),
}

/// Reference model of the lease decision, deliberately re-implemented.
///
/// `sabotage_case`: when `Some(id)`, the reference's verdict for that case id
/// is flipped — the sabotage twin that must make the court fail.
struct Reference {
    sabotage_case: Option<usize>,
}

impl Reference {
    fn evaluate(&self, c: &Case) -> Verdict {
        if self.sabotage_case == Some(c.id) {
            // Sabotage twin: flip the verdict for exactly one case.
            return Verdict::Refused("ceiling");
        }
        // Signed path: trust gate first, then issuer-clock skew, then lease.
        if let Some(s) = &c.signed {
            if !s.trusted {
                return Verdict::Refused("untrusted_key");
            }
            if s.issued_unix > s.max_skew_secs.saturating_add(c.now_unix) {
                return Verdict::Refused("clock_skew");
            }
        }
        if c.now_unix >= c.expires_unix {
            return Verdict::Refused("expired");
        }
        if !c.scope.contains(&c.step) {
            return Verdict::Refused("out_of_scope");
        }
        if rank(c.required) > rank(c.ceiling) {
            return Verdict::Refused("ceiling");
        }
        Verdict::Ok
    }
}

/// Reference ceiling rank, independent of `graphlaw::law::Ceiling`.
fn rank(ceiling: &str) -> u8 {
    match ceiling {
        "observe" => 0,
        "select" => 1,
        "construct" => 2,
        other => panic!("reference model: unknown ceiling `{other}`"),
    }
}

// ---------------------------------------------------------------------------
// Kernel evaluation of the same case
// ---------------------------------------------------------------------------

fn kernel_verdict(c: &Case) -> Verdict {
    let lease = Lease {
        id: c.lease_id.clone(),
        holder: c.holder.clone(),
        ceiling: Ceiling::parse(c.ceiling).expect("corpus ceilings are registry names"),
        scope: c.scope.iter().map(|s| (*s).to_string()).collect(),
        expires_unix: c.expires_unix,
        issued_unix: c.signed.map_or(0, |s| s.issued_unix),
    };
    match &c.signed {
        None => lease
            .authorize(c.step, Ceiling::parse(c.required).unwrap(), c.now_unix)
            .map_or_else(
                |e| match e {
                    graphlaw::law::LawError::LeaseRefused { reason, .. } => {
                        Verdict::Refused(reason.as_str())
                    }
                    other => panic!("unexpected kernel error {other:?} on case {}", c.id),
                },
                |()| Verdict::Ok,
            ),
        Some(s) => {
            let key = SigningKey::from_seed([7; 32]);
            let mut trusted = TrustedKeys::new();
            if s.trusted {
                trusted.insert(key.verifying_key());
            }
            let signed = sign_lease(&key, lease);
            let clock = FixedClock(c.now_unix);
            signed
                .authorize(
                    c.step,
                    Ceiling::parse(c.required).unwrap(),
                    &trusted,
                    &clock,
                    s.max_skew_secs,
                )
                .map_or_else(
                    |e| match e {
                        graphlaw::law::LawError::LeaseRefused { reason, .. } => {
                            Verdict::Refused(reason.as_str())
                        }
                        other => panic!("unexpected kernel error {other:?} on case {}", c.id),
                    },
                    |()| Verdict::Ok,
                )
        }
    }
}

// ---------------------------------------------------------------------------
// The court
// ---------------------------------------------------------------------------

/// Runs the whole corpus; returns the divergences (empty = agreement).
fn run_corpus(reference: &Reference) -> Vec<String> {
    let corpus = gen_corpus();
    let mut divergences = Vec::new();
    for c in &corpus {
        let want = reference.evaluate(c);
        let got = kernel_verdict(c);
        if want != got {
            divergences.push(format!(
                "lease_authorization_divergence[case {}]: \
                 lease_id={} holder={} ceiling={} scope={:?} expires_unix={} \
                 signed={:?} step={} required={} now_unix={} \
                 -- reference says {:?}, kernel says {:?}",
                c.id,
                c.lease_id,
                c.holder,
                c.ceiling,
                c.scope,
                c.expires_unix,
                c.signed,
                c.step,
                c.required,
                c.now_unix,
                want,
                got
            ));
        }
    }
    divergences
}

const CORPUS_LEN: usize = 420;

#[test]
fn lease_authorization_kernel_matches_reference_over_corpus() {
    let divergences = run_corpus(&Reference {
        sabotage_case: None,
    });
    assert!(
        divergences.is_empty(),
        "kernel lease authorization diverged from the reference evaluator in {} of {CORPUS_LEN} \
         cases (fail-closed):\n{}",
        divergences.len(),
        divergences.join("\n")
    );
}

#[test]
fn sabotage_twin_flipping_one_reference_verdict_fails_the_court() {
    const SABOTAGED: usize = 137;
    let divergences = run_corpus(&Reference {
        sabotage_case: Some(SABOTAGED),
    });
    assert_eq!(
        divergences.len(),
        1,
        "sabotage twin must produce exactly one divergence, got: {divergences:?}"
    );
    assert!(
        divergences[0].contains(&format!("case {SABOTAGED}")),
        "divergence must name the sabotaged case: {}",
        divergences[0]
    );
    assert!(
        divergences[0].starts_with("lease_authorization_divergence"),
        "divergence must carry the typed divergence prefix: {}",
        divergences[0]
    );
}

// ---------------------------------------------------------------------------
// Ceiling algebra + registry consistency courts
// ---------------------------------------------------------------------------

#[test]
fn ceiling_order_and_parse_name_inverse_agree_with_reference_ranking() {
    // parse/name inverses over the registry's ceiling names.
    for name in LEASE_CEILINGS {
        let c =
            Ceiling::parse(name).unwrap_or_else(|| panic!("registry ceiling {name} unparsable"));
        assert_eq!(
            c.name(),
            *name,
            "Ceiling::name/parse not inverse for {name}"
        );
    }
    assert!(
        Ceiling::parse("teleport").is_none(),
        "unknown ceiling must not parse"
    );
    // The kernel total order matches the reference ranking on every pair.
    let rank_of = |name: &str| rank(name);
    for a in LEASE_CEILINGS {
        for b in LEASE_CEILINGS {
            let (ca, cb) = (Ceiling::parse(a).unwrap(), Ceiling::parse(b).unwrap());
            assert_eq!(
                ca < cb,
                rank_of(a) < rank_of(b),
                "kernel ceiling order disagrees with reference ranking on {a} vs {b}"
            );
            assert_eq!(
                ca == cb,
                a == b,
                "kernel ceiling equality disagrees with reference on {a} vs {b}"
            );
            assert_eq!(
                ca > cb,
                rank_of(a) > rank_of(b),
                "kernel ceiling order disagrees with reference ranking on {a} vs {b}"
            );
        }
    }
}

#[test]
fn registry_law_steps_agree_with_kernel_step_names_and_ceilings() {
    use graphlaw::hooks::HookPack;

    // An empty state yields an empty HookPack — the court only reads
    // `required_ceiling`, never materializes.
    let empty_state =
        LawState::parse(b"", Dialect::NTriples, None).expect("empty N-Triples state parses");
    let empty_pack = Box::leak(Box::new(
        HookPack::load(&empty_state).expect("empty pack loads"),
    ));
    let empty_plan = Box::leak(Box::new(
        graphlaw::plan::Plan::builder()
            .action("a")
            .build()
            .expect("empty plan builds"),
    ));
    let no_trusted = Box::leak(Box::new(TrustedKeys::new()));
    let empty_ruleset =
        Box::leak(Box::new(graphlaw::tier::validate_n3_rules(b"", None).expect("empty rules")));

    // Registry wire name -> kernel Step variant of the same operation.
    let kernel_steps: [(&str, &Step<'static>); 8] = [
        ("shacl", &Step::AdmitShacl { shapes_ttl: "" }),
        (
            "n3",
            &Step::DeriveN3 {
                rules: "",
                ruleset: empty_ruleset,
            },
        ),
        ("rdfs", &Step::EntailRdfs),
        ("owl-rl", &Step::EntailOwlRl),
        ("hooks", &Step::Hooks { pack: empty_pack }),
        ("plan", &Step::Plan { plan: empty_plan }),
        ("require-receipt", &Step::RequireReceipt { step: "x" }),
        (
            "require-signed-receipt",
            &Step::RequireSignedReceipt {
                step: "x",
                trusted: &*no_trusted,
            },
        ),
    ];
    // Kernel step names must be exactly the prefix-joined registry names.
    let expected: Vec<String> = LAW_STEPS
        .iter()
        .filter(|r| r.ceiling.is_some())
        .map(|r| prefix_join(r.name))
        .collect();
    let actual: Vec<&str> = kernel_steps.iter().map(|(_, s)| (*s).name()).collect();
    assert_eq!(
        actual, expected,
        "kernel Step names and registry law-step rows disagree"
    );

    for (reg_name, reg_ceiling) in LAW_STEPS
        .iter()
        .filter(|r| r.ceiling.is_some())
        .map(|r| (r.name, r.ceiling.unwrap()))
    {
        let step = kernel_steps
            .iter()
            .find(|(_, s)| (*s).name() == prefix_join(reg_name))
            .map(|(_, s)| *s)
            .unwrap_or_else(|| panic!("registry law step `{reg_name}` has no kernel Step variant"));
        assert_eq!(
            step.required_ceiling().name(),
            reg_ceiling,
            "ceiling mismatch for registry step `{reg_name}`"
        );
    }
    // `record-receipts` is the one registry row with no transition gate.
    assert!(
        LAW_STEPS
            .iter()
            .any(|r| r.name == "record-receipts" && r.ceiling.is_none()),
        "registry must carry the ceiling-less `record-receipts` row"
    );
}

fn prefix_join(reg_name: &str) -> String {
    let prefix = if reg_name == "shacl" || reg_name.starts_with("require") || reg_name == "plan" {
        "admit:"
    } else {
        "derive:"
    };
    format!("{prefix}{reg_name}")
}
