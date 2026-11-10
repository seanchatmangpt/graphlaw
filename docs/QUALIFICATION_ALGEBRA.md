# Qualification Algebra

A qualification is not an assertion suite. It is an executable falsification relation over an exact subject.

## Core object

Q = (S, C, A, O, P, E, H)

- S: exact subject identity (repo/ref/SHA plus load-bearing collaborator identities)
- C: inherited court set
- A: adversarial attempt family
- O: independent observers
- P: temporal/process postconditions
- E: durable evidence projection (including OCEL where applicable)
- H: standing ceiling

A court is non-vacuous iff AttemptObserved AND NOT ViolationObserved. A missing attempt is UNKNOWN, never PASS.

Standing forms a bounded lattice:

UNQUALIFIED < OBSERVED < ATTACKED < QUALIFIED

REFUSED and UNKNOWN are not weaker PASS states. They preserve why standing was not manufactured.

## Composition

Profiles compose by set union of inherited courts plus stricter postconditions. A child profile may add constraints but may not silently remove inherited courts. Domain profiles such as FIBO finance therefore inherit the universal exact-subject, anti-vacuity, observer, temporal, mutation, crash/restart, replay, and differential courts.

## Hardening algebra

A discovered counterexample x becomes permanent negative knowledge only after generalization:

counterexample -> boundary -> generalized law -> falsifier family -> executable court -> inherited profile

This is work hardening when repeated deformation attempts convert an easy failure path into a structural constraint. Grain refinement corresponds to splitting an over-broad trust/load-bearing boundary into independently observable subjects. Precipitation/case hardening corresponds to adding local constraints at high-consequence boundaries without pretending the whole system has stronger standing. Tempering corresponds to relaxing an over-constrained rule only with a preserved falsifier and explicit consequence bound.

The analogy is invalid where it does not produce one of: a subject boundary, constraint, observer, falsifier, profile inheritance rule, or standing transition.

## Mechanical falsifier generation

Semantic laws expose mutation dimensions. For a law requiring exact subject digest equality, the generator substitutes a distinct digest and expects refusal. For temporal laws it permutes required event order. For authority laws it raises claimed authority above the admitted ceiling. Generated attacks carry the law identity and mutation identity so replay proves which law was attacked.

## Crash / UNKNOWN law

If actuation outcome cannot be reconstructed after crash/restart, standing is UNKNOWN. Reconciliation must use durable evidence and independent observation; retry is not evidence of non-execution.

## Candidate / authority boundary

Qualification manufactures evidence and standing only. Candidate != Truth != Authority != DO != Standing.

## Executable binding (re-read against `src/qualification.rs`, 2026-11-09)

The algebra above is implemented by four public items in `src/qualification.rs`; the module
header states it manufactures evidence and bounded standing only, never authority.

- `Standing` enum: `Unqualified`, `Observed`, `Attacked`, `Qualified`, `Refused`, `Unknown`
  (the lattice states plus the two non-lattice outcomes).
- `evaluate(Observation) -> Standing` folds seven booleans (`attempt_observed`,
  `violation_observed`, `independent_observer`, `postconditions_satisfied`,
  `durable_evidence`, `crash_observed`, `crash_reconciled`). Order of decision: no attempt ->
  `Unknown`; violation observed -> `Refused`; missing independent observer, postconditions, or
  durable evidence -> `Unknown`; crash observed and not reconciled -> `Unknown`; otherwise
  `Qualified`. Boundary: `evaluate` never returns `Unqualified`, `Observed`, or `Attacked`;
  those three lattice rungs exist as enum values but are not produced by this function.
- `generate_falsifier(law, seed)` implements the mutation dimensions named under mechanical
  falsifier generation: `replace_with_distinct_digest`, `substitute_collaborator`,
  `alter_replay_digest`, `omit_attempt`, `swap_required_pair`,
  `raise_authority_above_ceiling`, `couple_observer_to_actuator`,
  `drop_reconciliation_evidence`, `set_above_limit`; any other mutation name is a typed
  `Err`. Output always carries `authority: NONE`, `consequence: EVIDENCE_ONLY`.
- `compose_courts(parent, child)` returns the set union of parent and child `courts` and
  errors when the child does not list the parent in `inherits`. It enforces the
  no-silent-removal half of Composition; the "stricter postconditions" half is not computed
  here.
- `temporal_rule_holds(events, rule)` checks one strict `a<b` precedence rule.
