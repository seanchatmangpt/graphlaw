use serde_json::{Value, json};

use super::super::{
    lease::LeaseContext,
    Fail, MAX_ATOMS_PER_FIELD, MAX_LAW_STEPS, MAX_PLAN_ACTIONS, MAX_POLICY_ENTRIES, Res, bad,
    check_limit, nquads_of, state_field, str_field,
};
use crate::attest::TrustedKeys;
use crate::dialect::{Dialect, Engine, Refusal, RefusalKind};
use crate::hooks::HookPack;
use crate::law::{
    Ceiling, Clock, FixedClock, LawError, Lease, Step, SystemClock,
};

/// `(name, observed, max)` from a `resource limit `name` exceeded: a > b` message.
fn parse_limit_message(message: &str) -> Option<(String, usize, usize)> {
    let rest = message.strip_prefix("resource limit `")?;
    let (name, rest) = rest.split_once("` exceeded: ")?;
    let mut nums = rest.split_whitespace();
    let observed = nums.next()?.parse().ok()?;
    if nums.next()? != ">" {
        return None;
    }
    let max = nums.next()?.parse().ok()?;
    Some((name.to_string(), observed, max))
}

pub(in crate::abi) fn law_err(e: LawError) -> Fail {
    let (refusal, details) = match e {
        LawError::Refused(r) => {
            let d = if r.kind == RefusalKind::ResourceLimit {
                // The refusal message is `resource limit `<name>` exceeded: <observed> > <max> ...`.
                match parse_limit_message(r.message.as_str()) {
                    Some((name, observed, max)) => json!({"code": "ResourceLimit", "limit": name,
                           "observed": observed, "max": max}),
                    // A ResourceLimit refusal whose message no longer parses
                    // carries no honest counter: emit the code alone rather
                    // than fabricate an `n3_iterations` row the run never hit.
                    None => json!({"code": "ResourceLimit"}),
                }
            } else {
                json!({"code": "Refused", "kind": format!("{:?}", r.kind)})
            };
            (r, d)
        }
        LawError::PlanRefused {
            index,
            action,
            missing,
            violated_absent,
        } => {
            let mut message = format!(
                "plan refused at step {index} (`{action}`): unmet {}",
                missing.join(" ")
            );
            if !violated_absent.is_empty() {
                message = format!(
                    "plan refused at step {index} (`{action}`): forbidden triple(s) present {}",
                    violated_absent.join(" ")
                );
                if !missing.is_empty() {
                    message.push_str(&format!("; unmet {}", missing.join(" ")));
                }
            }
            (
                Refusal {
                    kind: RefusalKind::EngineRejected,
                    dialect: Some(Dialect::NTriples),
                    engine: Some(Engine::PurRdf),
                    message: message.into(),
                },
                json!({"code": "PlanRefused", "index": index, "action": action,
                       "unmet": missing, "violated_absent": violated_absent}),
            )
        }
        LawError::ReceiptRequired { step } => (
            Refusal {
                kind: RefusalKind::EngineRejected,
                dialect: None,
                engine: Some(Engine::PurRdf),
                message: format!("receipt required: no recorded receipt for step `{step}`").into(),
            },
            json!({"code": "ReceiptRequired", "step": step}),
        ),
        LawError::ReceiptRefused { step, reason } => (
            Refusal {
                kind: RefusalKind::EngineRejected,
                dialect: None,
                engine: None,
                message: format!("receipt for step `{step}` refused: {}", reason.as_str()).into(),
            },
            json!({"code": "ReceiptRefused", "step": step, "reason": reason.as_str()}),
        ),
        e @ LawError::LeaseRefused { .. } => {
            let d = match &e {
                LawError::LeaseRefused {
                    lease_id,
                    step,
                    reason,
                } => json!({
                    "code": "LeaseRefused",
                    "reason": reason.as_str(),
                    "lease_id": lease_id,
                    "step": step,
                }),
                _ => unreachable!("matched LeaseRefused above"),
            };
            (
                Refusal {
                    kind: RefusalKind::EngineRejected,
                    dialect: None,
                    engine: None,
                    message: e.to_string().into(),
                },
                d,
            )
        }
        LawError::NotAdmitted {
            violations,
            results,
        } => (
            Refusal {
                kind: RefusalKind::EngineRejected,
                dialect: Some(Dialect::Turtle),
                engine: Some(Engine::PurRdf),
                message: format!("SHACL admission refused: {violations} violation(s)").into(),
            },
            json!({
                "code": "NotAdmitted",
                "violations": results.iter().map(|v| json!({
                    "focus": v.focus, "path": v.path, "component": v.component,
                    "message": v.message, "severity": v.severity,
                })).collect::<Vec<_>>(),
            }),
        ),
    };
    Fail {
        refusal,
        details: Some(details),
    }
}

pub(in crate::abi) fn op_hooks(v: &Value) -> Res<Value> {
    let data = state_field(v, "data")?;
    let pack = HookPack::load(&state_field(v, "pack")?)?;
    let m = pack.materialize(&data)?;
    Ok(json!({
        "id": m.state.id(),
        "rounds": m.rounds,
        "quads": m.state.quad_count(),
        "nquads": nquads_of(m.state.dataset())?,
        "firings": m.firings.iter().map(|f| json!({
            "hook": f.hook, "round": f.round, "added": f.added,
            "row": f.row.iter().map(|(k, t)| json!([k, t])).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    }))
}

fn plan_field(step: &Value) -> Res<crate::plan::Plan> {
    let p = step.get("plan").ok_or_else(|| bad("missing `plan`"))?;
    let text = |o: &Value, k: &str| -> Res<String> {
        match o.get(k) {
            None | Some(Value::Null) => Ok(String::new()),
            Some(Value::String(t)) => {
                let atoms = t.lines().filter(|l| !l.trim().is_empty()).count();
                check_limit("atoms_per_field", atoms, MAX_ATOMS_PER_FIELD)?;
                Ok(t.clone())
            }
            Some(_) => Err(bad(format!("`{k}` must be an N-Triples string"))),
        }
    };
    let raw = p
        .get("actions")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("plan needs an `actions` array"))?;
    check_limit("plan_actions", raw.len(), MAX_PLAN_ACTIONS)?;
    let actions = raw
        .iter()
        .map(|a| {
            Ok(crate::plan::Action {
                name: text(a, "name")?,
                pre: text(a, "pre")?,
                pre_not: text(a, "pre_not")?,
                add: text(a, "add")?,
                del: text(a, "del")?,
            })
        })
        .collect::<Res<Vec<_>>>()?;
    Ok(crate::plan::Plan {
        actions,
        goal: text(p, "goal")?,
        goal_not: text(p, "goal_not")?,
    })
}

pub(in crate::abi) fn lease_obj(l: &Value) -> Res<Lease> {
    let ceiling = Ceiling::parse(str_field(l, "ceiling")?)
        .ok_or_else(|| bad("lease `ceiling` must be observe|select|construct"))?;
    let scope = l
        .get("scope")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("lease `scope` must be an array of step names"))?
        .iter()
        .map(|s| {
            s.as_str()
                .map(str::to_string)
                .ok_or_else(|| bad("lease `scope` entries are strings"))
        })
        .collect::<Res<Vec<_>>>()?;
    let expires_unix = l
        .get("expires_unix")
        .and_then(Value::as_u64)
        .ok_or_else(|| bad("lease `expires_unix` must be an unsigned integer"))?;
    let issued_unix = match l.get("issued_unix") {
        None | Some(Value::Null) => 0,
        Some(n) => n
            .as_u64()
            .ok_or_else(|| bad("lease `issued_unix` must be an unsigned integer"))?,
    };
    Ok(Lease {
        id: str_field(l, "id")?.to_string(),
        holder: str_field(l, "holder")?.to_string(),
        ceiling,
        scope,
        expires_unix,
        issued_unix,
    })
}

pub(in crate::abi) fn trusted_keys_of(v: &Value) -> Res<TrustedKeys> {
    let arr = v
        .get("trusted_keys")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("`trusted_keys` must be an array of hex Ed25519 public keys"))?;
    let hexes = arr
        .iter()
        .map(|k| {
            k.as_str()
                .ok_or_else(|| bad("`trusted_keys` entries are hex strings"))
        })
        .collect::<Res<Vec<_>>>()?;
    TrustedKeys::from_hex(hexes).map_err(|e| bad(format!("`trusted_keys`: {e}")))
}

/// Parse a plan document (`{"actions":[{name,pre,add,del}],"goal"}`) with the
/// ABI's size limits. Used by the `graphlaw-verify` binary.
pub fn plan_from_json(plan: &Value) -> Result<crate::plan::Plan, String> {
    plan_field(&json!({ "plan": plan })).map_err(|f| f.refusal.message.as_str().to_owned())
}

/// `{"op":"policy","problem":<PlanningProblem JSON|string>,"policy":<UniversalPlan|entries JSON|string>}`:
/// independent strong-cyclic admission of a FOND policy (see `policy`).
/// `(entries, total outcomes)` of a policy value, counted through the same
/// shape `policy::admit` reads (`[..]` or `{"policy":[..]}`).
pub(in crate::abi) fn policy_counts(p: Option<&Value>) -> (usize, usize) {
    let arr = match p {
        Some(Value::Array(a)) => Some(a),
        Some(o @ Value::Object(_)) => o.get("policy").and_then(Value::as_array),
        _ => None,
    };
    arr.map_or((0, 0), |a| {
        let outcomes = a
            .iter()
            .map(|e| e.get("outcomes").and_then(Value::as_array).map_or(0, Vec::len))
            .sum();
        (a.len(), outcomes)
    })
}

pub(in crate::abi) fn op_policy(v: &Value) -> Res<Value> {
    let text = |k: &str| -> Res<String> {
        match v.get(k) {
            Some(Value::String(t)) => Ok(t.clone()),
            Some(o) if o.is_object() || o.is_array() => Ok(o.to_string()),
            _ => Err(bad(format!("missing `{k}` (JSON object or string)"))),
        }
    };
    // Count by the key `policy::admit` actually reads: a bare array, or an
    // object's `policy` array; string forms are parsed first.
    let parsed;
    let policy_val: Option<&Value> = match v.get("policy") {
        Some(Value::String(t)) => {
            parsed = serde_json::from_str::<Value>(t).ok();
            parsed.as_ref()
        }
        other => other,
    };
    let (entries, outcomes) = policy_counts(policy_val);
    check_limit("policy_entries", entries, MAX_POLICY_ENTRIES)?;
    check_limit("policy_outcomes", outcomes, MAX_POLICY_ENTRIES)?;
    let admitted = crate::policy::admit(&text("problem")?, &text("policy")?).map_err(|r| Fail {
        refusal: Refusal {
            kind: RefusalKind::EngineRejected,
            dialect: None,
            engine: None,
            message: format!(
                "policy refused ({}) at state `{}` action `{}`: {}",
                r.kind.as_str(),
                r.state,
                r.action,
                r.message
            )
            .into(),
        },
        details: Some(json!({
            "code": "PolicyRefused", "policy_kind": r.kind.as_str(),
            "state": r.state, "action": r.action,
        })),
    })?;
    Ok(json!({
        "initial_states": admitted.initial_states,
        "reachable": admitted.reachable,
        "goal_states": admitted.goal_states,
        "entries": admitted.entries.iter().map(|(s, a)| json!([s, a])).collect::<Vec<_>>(),
        "ntriples": admitted.to_ntriples(),
    }))
}

/// `(lease step name, required ceiling, scoped)` of a wire step. `scoped` is
/// false for `record-receipts`, which has no scope row: a presented lease must
/// still reach Observe but need not list it.
fn step_requirement(step: &Value) -> Res<(&'static str, Ceiling, bool)> {
    Ok(match str_field(step, "step")? {
        "shacl" => ("admit:shacl", Ceiling::Observe, true),
        "sql" => ("admit:sql", Ceiling::Observe, true),
        "require-receipt" => ("admit:require-receipt", Ceiling::Observe, true),
        "require-signed-receipt" => ("admit:require-signed-receipt", Ceiling::Observe, true),
        "plan" => ("admit:plan", Ceiling::Select, true),
        "n3" => ("derive:n3", Ceiling::Construct, true),
        "hooks" => ("derive:hooks", Ceiling::Construct, true),
        "rdfs" => ("derive:rdfs", Ceiling::Construct, true),
        "owl-rl" => ("derive:owl-rl", Ceiling::Construct, true),
        "record-receipts" => ("admit:record-receipts", Ceiling::Observe, false),
        other => return Err(bad(format!("unknown step `{other}`"))),
    })
}

pub(in crate::abi) fn op_law(v: &Value) -> Res<Value> {
    // One clock read per request: the lease is judged once, at this instant.
    let clock = FixedClock(SystemClock.now_unix());
    let steps = v
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing `steps` array"))?;
    // Phi_lease => Valid before dispatch: admit the lease and authorize every
    // requested step BEFORE any step (or data parse) runs.
    let mut requirements = Vec::with_capacity(steps.len().min(MAX_LAW_STEPS));
    for (n, step) in steps.iter().enumerate() {
        check_limit("law_steps", n + 1, MAX_LAW_STEPS)?;
        requirements.push(step_requirement(step)?);
    }
    let first = requirements.first().map_or("request", |r| r.0);
    let lease = LeaseContext::admit(v, &clock, first)?;
    for (name, ceiling, scoped) in &requirements {
        if *scoped {
            lease.authorize(name, *ceiling).map_err(law_err)?;
        } else {
            lease.authorize_unscoped(name, *ceiling).map_err(law_err)?;
        }
    }
    let lease_id = lease.id();
    let mut state = state_field(v, "data")?;
    let mut ids = vec![state.id().to_string()];
    let mut receipts = Vec::new();
    let mut produced: Vec<crate::law::Receipt> = Vec::new();
    let mut recorded = 0usize;
    for step in steps.iter() {
        if str_field(step, "step")? == "plan" {
            let plan = plan_field(step)?;
            let admitted = plan.admit(&state).map_err(law_err)?;
            for (index, r) in admitted.receipts.iter().enumerate() {
                let mut rj = json!({
                    "step": r.step, "parent": r.parent, "child": r.child, "added": r.added,
                    "authority": r.authority.authority, "revision": r.authority.revision,
                    "plan_blake3": admitted.plan_digest, "index": index,
                });
                if let Some(id) = &lease_id {
                    rj["lease_id"] = json!(id);
                }
                receipts.push(rj);
                ids.push(r.child.clone());
                let mut r = r.clone();
                r.lease_id = lease_id.clone();
                produced.push(r);
            }
            state = admitted.state;
            continue;
        }
        if str_field(step, "step")? == "record-receipts" {
            for r in &produced[recorded..] {
                state = crate::receipt::record(&state, r).map_err(law_err)?;
            }
            recorded = produced.len();
            ids.push(state.id().to_string());
            continue;
        }
        let pack;
        let shapes;
        let rules;
        let ruleset;
        let step_name;
        let step_trusted;
        let step_ref = match str_field(step, "step")? {
            "shacl" => {
                shapes = str_field(step, "shapes")?.to_string();
                Step::AdmitShacl {
                    shapes_ttl: &shapes,
                }
            }
            "n3" => {
                rules = str_field(step, "rules")?.to_string();
                ruleset = crate::tier::validate_n3_rules(rules.as_bytes(), None)
                    .map_err(|r| law_err(LawError::Refused(r)))?;
                Step::DeriveN3 {
                    rules: &rules,
                    ruleset: &ruleset,
                }
            }
            "rdfs" => Step::EntailRdfs,
            "owl-rl" => Step::EntailOwlRl,
            "sql" => Step::AdmitSql,
            "require-receipt" => {
                step_name = str_field(step, "step_name")?.to_string();
                Step::RequireReceipt { step: &step_name }
            }
            "require-signed-receipt" => {
                step_name = str_field(step, "step_name")?.to_string();
                step_trusted = trusted_keys_of(step)?;
                Step::RequireSignedReceipt {
                    step: &step_name,
                    trusted: &step_trusted,
                }
            }
            "hooks" => {
                pack = HookPack::load(&state_field(step, "pack")?)?;
                Step::Hooks { pack: &pack }
            }
            other => return Err(bad(format!("unknown step `{other}`"))),
        };
        // Proof-consuming: the lease was admitted above; this is a pure
        // scope/ceiling predicate, never a signature or clock check.
        lease
            .authorize(step_ref.name(), step_ref.required_ceiling())
            .map_err(law_err)?;
        let (child, mut r) = state.transition(&step_ref).map_err(law_err)?;
        r.lease_id = lease_id.clone();
        let mut rj = json!({
            "step": r.step, "parent": r.parent, "child": r.child, "added": r.added,
            "authority": r.authority.authority, "revision": r.authority.revision,
        });
        if let Some(id) = &r.lease_id {
            rj["lease_id"] = json!(id);
        }
        receipts.push(rj);
        ids.push(child.id().to_string());
        produced.push(r);
        state = child;
    }
    Ok(json!({"states": ids, "receipts": receipts, "nquads": nquads_of(state.dataset())?}))
}
