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
