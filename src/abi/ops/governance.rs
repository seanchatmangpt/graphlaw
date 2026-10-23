use serde_json::{Value, json};

use super::super::{
    Fail, MAX_ATOMS_PER_FIELD, MAX_LAW_STEPS, MAX_PLAN_ACTIONS, MAX_POLICY_ENTRIES, Res, bad,
    bad_refusal, check_limit, nquads_of, state_field, str_field,
};
use crate::attest::{Attestation, TrustedKeys};
use crate::dialect::{Dialect, Engine, Refusal, RefusalKind};
use crate::hooks::HookPack;
use crate::law::{
    Ceiling, DEFAULT_MAX_SKEW_SECS, LawError, LawState, Lease, Receipt, SignedLease, Step,
    SystemClock,
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

fn law_err(e: LawError) -> Fail {
    let (refusal, details) = match e {
        LawError::Refused(r) => {
            let d = if r.kind == RefusalKind::ResourceLimit {
                // The refusal message is `resource limit `<name>` exceeded: <observed> > <max> ...`.
                match parse_limit_message(r.message.as_str()) {
                    Some((name, observed, max)) => json!({"code": "ResourceLimit", "limit": name,
                           "observed": observed, "max": max}),
                    None => json!({"code": "ResourceLimit", "limit": "n3_iterations",
                           "max": crate::law::N3_MAX_ITERATIONS}),
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

fn lease_obj(l: &Value) -> Res<Lease> {
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

/// How a `law` request's steps are authorized.
enum LeaseAuth {
    /// No lease: steps run unleased.
    None,
    /// `"lease"` + `"now_unix"` + `"unverified_lease": true`: unsigned lease,
    /// caller-chosen time. Proves nothing about authority.
    Unverified(Lease, u64),
    /// `"signed_lease"` + `"trusted_keys"`: Ed25519-verified, expiry judged by
    /// the module's own clock (the request cannot supply a time).
    Signed {
        signed: SignedLease,
        trusted: TrustedKeys,
        max_skew_secs: u64,
    },
}

impl LeaseAuth {
    fn id(&self) -> Option<String> {
        match self {
            LeaseAuth::None => None,
            LeaseAuth::Unverified(l, _) => Some(l.id.clone()),
            LeaseAuth::Signed { signed, .. } => Some(signed.lease.id.clone()),
        }
    }

    fn authorize(&self, step: &str, required: Ceiling) -> Result<(), LawError> {
        match self {
            LeaseAuth::None => Ok(()),
            LeaseAuth::Unverified(l, now) => l.authorize(step, required, *now),
            LeaseAuth::Signed {
                signed,
                trusted,
                max_skew_secs,
            } => signed.authorize(step, required, trusted, &SystemClock, *max_skew_secs),
        }
    }

    fn transition(
        &self,
        state: &LawState,
        step: &Step<'_>,
    ) -> Result<(LawState, Receipt), LawError> {
        match self {
            LeaseAuth::None => state.transition(step),
            LeaseAuth::Unverified(l, now) => state.transition_leased_unverified(l, step, *now),
            LeaseAuth::Signed {
                signed,
                trusted,
                max_skew_secs,
            } => state.transition_authorized(signed, trusted, &SystemClock, *max_skew_secs, step),
        }
    }
}

fn trusted_keys_of(v: &Value) -> Res<TrustedKeys> {
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

fn lease_auth_of(v: &Value) -> Res<LeaseAuth> {
    match (v.get("signed_lease"), v.get("lease")) {
        (None, None) => Ok(LeaseAuth::None),
        (Some(_), Some(_)) => Err(bad("give `signed_lease` or `lease`, not both")),
        (Some(sl), None) => {
            let a = sl
                .get("attestation")
                .ok_or_else(|| bad("`signed_lease` needs an `attestation`"))?;
            let attestation = Attestation::from_json(
                &json!({
                    "key_id": str_field(a, "key_id")?,
                    "payload_blake3": str_field(a, "payload_blake3")?,
                    "signature": str_field(a, "signature")?,
                })
                .to_string(),
            )
            .map_err(|e| bad(format!("`attestation`: {e}")))?;
            let max_skew_secs = match v.get("max_skew_secs") {
                None | Some(Value::Null) => DEFAULT_MAX_SKEW_SECS,
                Some(n) => n
                    .as_u64()
                    .ok_or_else(|| bad("`max_skew_secs` must be an unsigned integer"))?,
            };
            Ok(LeaseAuth::Signed {
                signed: SignedLease {
                    lease: lease_obj(
                        sl.get("lease")
                            .ok_or_else(|| bad("`signed_lease` needs a `lease`"))?,
                    )?,
                    attestation,
                },
                trusted: trusted_keys_of(v)?,
                max_skew_secs,
            })
        }
        (None, Some(l)) => {
            if v.get("unverified_lease") != Some(&Value::Bool(true)) {
                return Err(Fail {
                    refusal: bad_refusal(
                        "an unsigned `lease` is refused: supply `signed_lease` + `trusted_keys`, \
                         or set `\"unverified_lease\": true` to accept a lease no one signed",
                    ),
                    details: Some(json!({"code": "UnverifiedLeaseRefused"})),
                });
            }
            let now = v
                .get("now_unix")
                .and_then(Value::as_u64)
                .ok_or_else(|| bad("an unverified `lease` requires integer `now_unix`"))?;
            Ok(LeaseAuth::Unverified(lease_obj(l)?, now))
        }
    }
}

/// Parse a plan document (`{"actions":[{name,pre,add,del}],"goal"}`) with the
/// ABI's size limits. Used by the `graphlaw-verify` binary.
pub fn plan_from_json(plan: &Value) -> Result<crate::plan::Plan, String> {
    plan_field(&json!({ "plan": plan })).map_err(|f| f.refusal.message.as_str().to_owned())
}

/// `{"op":"policy","problem":<PlanningProblem JSON|string>,"policy":<UniversalPlan|entries JSON|string>}`:
/// independent strong-cyclic admission of a FOND policy (see `policy`).
pub(in crate::abi) fn op_policy(v: &Value) -> Res<Value> {
    let text = |k: &str| -> Res<String> {
        match v.get(k) {
            Some(Value::String(t)) => Ok(t.clone()),
            Some(o) if o.is_object() || o.is_array() => Ok(o.to_string()),
            _ => Err(bad(format!("missing `{k}` (JSON object or string)"))),
        }
    };
    let count = |p: &Value| match p {
        Value::Array(a) => a.len(),
        o @ Value::Object(_) => o
            .get("policy")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        _ => 0,
    };
    let entries = match v.get("policy") {
        Some(Value::String(t)) => serde_json::from_str::<Value>(t).map_or(0, |p| count(&p)),
        Some(p) => count(p),
        None => 0,
    };
    check_limit("policy_entries", entries, MAX_POLICY_ENTRIES)?;
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

pub(in crate::abi) fn op_law(v: &Value) -> Res<Value> {
    let lease = lease_auth_of(v)?;
    let lease_id = lease.id();
    let mut state = state_field(v, "data")?;
    let mut ids = vec![state.id().to_string()];
    let mut receipts = Vec::new();
    let mut produced: Vec<crate::law::Receipt> = Vec::new();
    let mut recorded = 0usize;
    let steps = v
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing `steps` array"))?;
    for (n, step) in steps.iter().enumerate() {
        check_limit("law_steps", n + 1, MAX_LAW_STEPS)?;
        if str_field(step, "step")? == "plan" {
            let plan = plan_field(step)?;
            lease
                .authorize("admit:plan", Ceiling::Select)
                .map_err(law_err)?;
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
        let (child, r) = lease.transition(&state, &step_ref).map_err(law_err)?;
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
