//! Executable Chicago qualification algebra.
//!
//! This module manufactures evidence and bounded standing only. It never grants
//! authority and never performs a consequential action.

use std::collections::BTreeSet;

use serde_json::{Value, json};

/// Bounded qualification standing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// No qualifying observation exists.
    Unqualified,
    /// The subject was observed but not attacked.
    Observed,
    /// A falsifier was attempted, but qualification is not yet complete.
    Attacked,
    /// All required attack, observer, and postcondition evidence is present.
    Qualified,
    /// A violation was independently observed.
    Refused,
    /// Evidence is incomplete or the outcome cannot be reconciled.
    Unknown,
}

/// Evidence required to manufacture qualification standing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Observation {
    /// Whether the adversarial path was actually attempted.
    pub attempt_observed: bool,
    /// Whether an independent observer saw the forbidden outcome.
    pub violation_observed: bool,
    /// Whether observation is independent of the load-bearing collaborator.
    pub independent_observer: bool,
    /// Whether all required postconditions were independently satisfied.
    pub postconditions_satisfied: bool,
    /// Whether durable process evidence (for example OCEL) recorded the attempt and outcome.
    pub durable_evidence: bool,
    /// Whether a crash or restart made the consequence outcome ambiguous.
    pub crash_observed: bool,
    /// Whether durable evidence reconciled a crash-ambiguous outcome.
    pub crash_reconciled: bool,
}

/// Evaluate one Chicago observation without manufacturing authority.
pub fn evaluate(observation: Observation) -> Standing {
    if !observation.attempt_observed {
        return Standing::Unknown;
    }
    if observation.violation_observed {
        return Standing::Refused;
    }
    if !observation.independent_observer
        || !observation.postconditions_satisfied
        || !observation.durable_evidence
    {
        return Standing::Unknown;
    }
    if observation.crash_observed && !observation.crash_reconciled {
        return Standing::Unknown;
    }
    Standing::Qualified
}

/// Lattice rung of one observation: [`evaluate`]'s verdict refined so the
/// intermediate rungs are reachable. `evaluate` is unchanged (it stays the
/// admission fold and never returns the three lower rungs).
///
/// - no attempt, independent observer present: [`Standing::Observed`]
/// - no attempt, no observer: [`Standing::Unqualified`]
/// - attempt made, evidence incomplete, no unreconciled crash:
///   [`Standing::Attacked`]
/// - attempt made, unreconciled crash: [`Standing::Unknown`]
/// - otherwise exactly `evaluate`'s `Qualified` / `Refused`.
pub fn rung(observation: Observation) -> Standing {
    match evaluate(observation) {
        Standing::Unknown if !observation.attempt_observed => {
            if observation.independent_observer {
                Standing::Observed
            } else {
                Standing::Unqualified
            }
        }
        Standing::Unknown if observation.crash_observed && !observation.crash_reconciled => {
            Standing::Unknown
        }
        Standing::Unknown => Standing::Attacked,
        other => other,
    }
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {key}"))
}

/// Mechanically generate an adversarial candidate from one semantic law.
///
/// The output is evidence-only and records the exact law, mutation dimension,
/// expected disposition, and mutated candidate for replay.
pub fn generate_falsifier(law: &Value, seed: &Value) -> Result<Value, String> {
    let law_id = required_str(law, "law")?;
    let dimension = required_str(law, "dimension")?;
    let falsifier = law
        .get("falsifier")
        .ok_or_else(|| "missing falsifier".to_string())?;
    let mutation = required_str(falsifier, "mutation")?;
    let expected = required_str(falsifier, "expect")?;

    let mut candidate = seed.clone();
    let object = candidate
        .as_object_mut()
        .ok_or_else(|| "falsifier seed must be a JSON object".to_string())?;

    match mutation {
        "replace_with_distinct_digest" => {
            object.insert(
                dimension.to_string(),
                Value::String(format!("blake3:{}", "f".repeat(64))),
            );
        }
        "substitute_collaborator" => {
            object.insert(
                dimension.to_string(),
                Value::String("urn:chicago:mutated-collaborator".to_string()),
            );
        }
        "alter_replay_digest" => {
            object.insert(
                dimension.to_string(),
                Value::String(format!("blake3:{}", "e".repeat(64))),
            );
        }
        "omit_attempt" => {
            object.insert(dimension.to_string(), Value::Bool(false));
        }
        "swap_required_pair" => {
            let pair = object
                .get_mut(dimension)
                .and_then(Value::as_array_mut)
                .ok_or_else(|| format!("{dimension} must be an array"))?;
            if pair.len() != 2 {
                return Err(format!("{dimension} must contain exactly two events"));
            }
            pair.swap(0, 1);
        }
        "raise_authority_above_ceiling" => {
            object.insert(dimension.to_string(), Value::String("DO".to_string()));
        }
        "couple_observer_to_actuator" => {
            object.insert(dimension.to_string(), Value::Bool(false));
        }
        "drop_reconciliation_evidence" => {
            object.insert(dimension.to_string(), Value::Bool(false));
        }
        "set_above_limit" => {
            let limit = object
                .get("autonomy_limit")
                .and_then(Value::as_u64)
                .ok_or_else(|| "autonomy_limit must be an unsigned integer".to_string())?;
            object.insert(dimension.to_string(), Value::from(limit + 1));
        }
        other => return Err(format!("unsupported falsifier mutation {other}")),
    }

    Ok(json!({
        "law": law_id,
        "dimension": dimension,
        "mutation": mutation,
        "expected": expected,
        "candidate": candidate,
        "authority": "NONE",
        "consequence": "EVIDENCE_ONLY"
    }))
}

fn court_set(profile: &Value) -> Result<BTreeSet<String>, String> {
    profile
        .get("courts")
        .and_then(Value::as_array)
        .ok_or_else(|| "profile courts must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "court id must be a string".to_string())
        })
        .collect()
}

/// Compose a child qualification profile monotonically over its parent.
///
/// Child profiles may add courts but cannot silently discard inherited courts.
pub fn compose_courts(parent: &Value, child: &Value) -> Result<BTreeSet<String>, String> {
    let parent_id = required_str(parent, "profile")?;
    let inherits = child
        .get("inherits")
        .and_then(Value::as_array)
        .ok_or_else(|| "child inherits must be an array".to_string())?;
    if !inherits
        .iter()
        .any(|value| value.as_str() == Some(parent_id))
    {
        return Err(format!(
            "child profile does not inherit required parent {parent_id}"
        ));
    }

    let mut courts = court_set(parent)?;
    courts.extend(court_set(child)?);
    Ok(courts)
}

/// Check one OCEL-style strict precedence rule such as `prepare<actuate`.
///
/// Semantics (per-`after`-event, 1:1 pairing): scanning left to right, every
/// occurrence of `after` must consume one earlier, not-yet-consumed occurrence
/// of `before`. So `["prepare","actuate","actuate"]` fails (the second
/// `actuate` has no `prepare` of its own) and `["actuate","prepare"]` fails.
/// The rule also requires at least one `after` occurrence (a rule that never
/// fires is not evidence) and refuses the degenerate `a<a` and any rule
/// without a single `<`. Unconsumed trailing `before` events are allowed.
pub fn temporal_rule_holds(events: &[&str], rule: &str) -> bool {
    let Some((before, after)) = rule.split_once('<') else {
        return false;
    };
    if before == after {
        return false;
    }
    let mut pending = 0usize;
    let mut fired = false;
    for event in events {
        if *event == before {
            pending += 1;
        } else if *event == after {
            if pending == 0 {
                return false;
            }
            pending -= 1;
            fired = true;
        }
    }
    fired
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full() -> Observation {
        Observation {
            attempt_observed: true,
            violation_observed: false,
            independent_observer: true,
            postconditions_satisfied: true,
            durable_evidence: true,
            crash_observed: false,
            crash_reconciled: false,
        }
    }

    #[test]
    fn rung_reaches_every_lattice_state() {
        let none = Observation {
            attempt_observed: false,
            independent_observer: false,
            ..full()
        };
        assert_eq!(rung(none), Standing::Unqualified);
        assert_eq!(
            rung(Observation {
                independent_observer: true,
                ..none
            }),
            Standing::Observed
        );
        assert_eq!(
            rung(Observation {
                durable_evidence: false,
                ..full()
            }),
            Standing::Attacked
        );
        assert_eq!(
            rung(Observation {
                crash_observed: true,
                ..full()
            }),
            Standing::Unknown
        );
        assert_eq!(rung(full()), Standing::Qualified);
        assert_eq!(
            rung(Observation {
                violation_observed: true,
                ..full()
            }),
            Standing::Refused
        );
    }

    #[test]
    fn temporal_rule_is_enforced_per_after_event() {
        assert!(temporal_rule_holds(&["p", "a", "p", "a"], "p<a"));
        assert!(temporal_rule_holds(&["p", "p", "a"], "p<a"));
        assert!(!temporal_rule_holds(&["p", "a", "a"], "p<a"), "second a has no p");
        assert!(!temporal_rule_holds(&["a", "p"], "p<a"));
        assert!(!temporal_rule_holds(&["p"], "p<a"), "rule never fires");
        assert!(!temporal_rule_holds(&["p", "a"], "p"));
        assert!(!temporal_rule_holds(&["p", "p"], "p<p"));
    }
}
