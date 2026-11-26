//! Lease admission court: Phi_lease => Valid before dispatch.
//!
//! A bad, expired, untrusted or under-ceiling lease is refused by
//! `LeaseContext::admit` / the upfront step authorization BEFORE any step
//! runs: the refusal names the offending step (not step 1) and the response
//! carries no `states`/`receipts`. `plan` honors a presented lease at Select;
//! `record-receipts` honors it at Observe (unscoped). Real ABI, real Ed25519.

use graphlaw::attest::{SigningKey, sign_lease};
use graphlaw::law::{Ceiling, Lease};
use serde_json::{Value, json};

const NOW_FAR: u64 = 4_000_000_000_000;

fn call(v: Value) -> Value {
    graphlaw::abi::call_json(&v)
}

fn data() -> Value {
    json!({"text": "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n<urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n", "dialect": "ntriples"})
}

fn plan_step() -> Value {
    let pre = "<urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .";
    let add = "<urn:a:x> <urn:a:p> <urn:a:y> .";
    json!({"step": "plan", "plan": {"actions": [
        {"name": "a1", "pre": pre, "add": add, "del": ""}], "goal": add}})
}

fn lease_json(ceiling: &str, scope: &[&str], expires: u64, issued: u64) -> Value {
    json!({"id": "L1", "holder": "h", "ceiling": ceiling, "scope": scope,
           "expires_unix": expires, "issued_unix": issued})
}

fn signed(l: &Value, key: &SigningKey) -> Value {
    let parsed = Lease {
        id: "L1".into(),
        holder: "h".into(),
        ceiling: Ceiling::parse(l["ceiling"].as_str().unwrap()).unwrap(),
        scope: l["scope"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect(),
        expires_unix: l["expires_unix"].as_u64().unwrap(),
        issued_unix: l["issued_unix"].as_u64().unwrap(),
    };
    let s = sign_lease(key, parsed);
    json!({"lease": l, "attestation": {"key_id": s.attestation.key_id,
           "payload_blake3": s.attestation.payload_blake3,
           "signature": s.attestation.signature}})
}

fn signed_req(l: &Value, signer: &SigningKey, trusted: &SigningKey, steps: Value) -> Value {
    json!({"op": "law", "data": data(), "signed_lease": signed(l, signer),
           "trusted_keys": [trusted.verifying_key().to_hex()], "steps": steps})
}

fn unverified_req(l: Value, steps: Value) -> Value {
    json!({"op": "law", "data": data(), "lease": l, "unverified_lease": true,
           "now_unix": 1000, "steps": steps})
}

/// `(code, reason, step)` of a refusal; also asserts no step ran.
fn refused(r: &Value) -> (String, String, String) {
    assert_eq!(r["ok"], false, "expected refusal: {r}");
    assert!(r.get("receipts").is_none() && r.get("states").is_none(), "{r}");
    let d = &r["error"]["details"];
    // A detail key may be legitimately absent/null (e.g. no step); any other
    // non-string shape is a loud failure.
    let s = |k: &str| match &d[k] {
        Value::String(v) => v.clone(),
        Value::Null => String::new(),
        other => panic!("refusal details.{k} must be a string or absent, got {other}: {r}"),
    };
    (s("code"), s("reason"), s("step"))
}

fn key() -> SigningKey {
    SigningKey::from_seed([5; 32])
}

#[test]
fn untrusted_key_refused_before_any_step() {
    let l = lease_json("construct", &["derive:rdfs"], NOW_FAR, 0);
    let req = signed_req(&l, &key(), &SigningKey::from_seed([6; 32]), json!([{"step": "rdfs"}]));
    let (code, reason, _) = refused(&call(req));
    assert_eq!((code.as_str(), reason.as_str()), ("LeaseRefused", "untrusted_key"));
}

#[test]
fn tampered_lease_refused_bad_signature() {
    let k = key();
    let l = lease_json("observe", &["derive:rdfs"], NOW_FAR, 0);
    let mut req = signed_req(&l, &k, &k, json!([{"step": "rdfs"}]));
    // Escalate the ceiling after signing.
    req["signed_lease"]["lease"]["ceiling"] = json!("construct");
    let (code, reason, _) = refused(&call(req));
    assert_eq!((code.as_str(), reason.as_str()), ("LeaseRefused", "bad_signature"));
}

#[test]
fn expired_lease_refused_even_when_steps_are_in_scope() {
    let k = key();
    let l = lease_json("construct", &["derive:rdfs"], 1, 0);
    let (code, reason, _) = refused(&call(signed_req(&l, &k, &k, json!([{"step": "rdfs"}]))));
    assert_eq!((code.as_str(), reason.as_str()), ("LeaseRefused", "expired"));
    let l = lease_json("construct", &["derive:rdfs"], 1000, 0);
    let (_, reason, _) = refused(&call(unverified_req(l, json!([{"step": "rdfs"}]))));
    assert_eq!(reason, "expired");
}

#[test]
fn later_bad_step_refuses_whole_request_up_front() {
    let k = key();
    // Step 1 (rdfs) is authorized; step 2 (n3) is out of scope. The refusal
    // is raised up front: no states/receipts, and it names step 2.
    let l = lease_json("construct", &["derive:rdfs"], NOW_FAR, 0);
    let steps = json!([{"step": "rdfs"}, {"step": "n3", "rules": "{ } => { } ."}]);
    let (code, reason, step) = refused(&call(signed_req(&l, &k, &k, steps)));
    assert_eq!(
        (code.as_str(), reason.as_str(), step.as_str()),
        ("LeaseRefused", "out_of_scope", "derive:n3")
    );
}

#[test]
fn under_ceiling_lease_refused_before_any_step() {
    let k = key();
    let l = lease_json("observe", &["admit:shacl", "derive:rdfs"], NOW_FAR, 0);
    let steps = json!([{"step": "shacl", "shapes": ""}, {"step": "rdfs"}]);
    let (_, reason, step) = refused(&call(signed_req(&l, &k, &k, steps)));
    assert_eq!((reason.as_str(), step.as_str()), ("ceiling", "derive:rdfs"));
}

#[test]
fn plan_honors_presented_lease() {
    let k = key();
    // Admitted at Select, receipt carries the lease id.
    let ok = call(signed_req(
        &lease_json("select", &["admit:plan"], NOW_FAR, 0),
        &k,
        &k,
        json!([plan_step()]),
    ));
    assert_eq!(ok["ok"], true, "{ok}");
    assert_eq!(ok["receipts"][0]["lease_id"], "L1");
    // Observe ceiling < Select.
    let (_, reason, step) = refused(&call(signed_req(
        &lease_json("observe", &["admit:plan"], NOW_FAR, 0),
        &k,
        &k,
        json!([plan_step()]),
    )));
    assert_eq!((reason.as_str(), step.as_str()), ("ceiling", "admit:plan"));
    // Out of scope.
    let (_, reason, _) = refused(&call(signed_req(
        &lease_json("select", &["derive:rdfs"], NOW_FAR, 0),
        &k,
        &k,
        json!([plan_step()]),
    )));
    assert_eq!(reason, "out_of_scope");
    // Expired / untrusted never reach the plan.
    let (_, reason, _) = refused(&call(signed_req(
        &lease_json("select", &["admit:plan"], 1, 0),
        &k,
        &k,
        json!([plan_step()]),
    )));
    assert_eq!(reason, "expired");
}

#[test]
fn record_receipts_honors_presented_lease_without_scope_row() {
    let k = key();
    let steps = json!([plan_step(), {"step": "record-receipts"}]);
    // Valid lease, record-receipts not in scope: admitted (unscoped, Observe).
    let ok = call(signed_req(
        &lease_json("select", &["admit:plan"], NOW_FAR, 0),
        &k,
        &k,
        steps.clone(),
    ));
    assert_eq!(ok["ok"], true, "{ok}");
    // Expired lease: record-receipts alone is refused, not waved through.
    let (_, reason, step) = refused(&call(signed_req(
        &lease_json("observe", &[], 1, 0),
        &k,
        &k,
        json!([{"step": "record-receipts"}]),
    )));
    assert_eq!((reason.as_str(), step.as_str()), ("expired", "admit:record-receipts"));
    // Untrusted signer: refused for a record-receipts-only request.
    let (_, reason, _) = refused(&call(signed_req(
        &lease_json("observe", &[], NOW_FAR, 0),
        &k,
        &SigningKey::from_seed([6; 32]),
        json!([{"step": "record-receipts"}]),
    )));
    assert_eq!(reason, "untrusted_key");
}

#[test]
fn lease_policy_required_refuses_leaseless_before_any_step() {
    let steps = json!([{"step": "rdfs"}]);
    // Default (NoneAllowed): lease-less rdfs chain still admits.
    let ok = call(json!({"op": "law", "data": data(), "steps": steps}));
    assert_eq!(ok["ok"], true, "{ok}");
    // Explicit request-level tightening refuses with a typed code.
    let r = call(json!({"op": "law", "data": data(), "steps": steps,
                        "lease_policy": "required"}));
    let (code, _, _) = refused(&r);
    assert_eq!(code, "LeaseRequired");
    // Explicit none-allowed is the default.
    let r = call(json!({"op": "law", "data": data(), "steps": steps,
                        "lease_policy": "none-allowed"}));
    assert_eq!(r["ok"], true, "{r}");
    // Unknown policy value is a typed bad request, not silently permissive.
    let r = call(json!({"op": "law", "data": data(), "steps": steps,
                        "lease_policy": "whatever"}));
    assert_eq!(r["ok"], false, "{r}");
    // A valid lease satisfies `required`.
    let k = key();
    let mut req = signed_req(
        &lease_json("construct", &["derive:rdfs"], NOW_FAR, 0),
        &k,
        &k,
        steps,
    );
    req["lease_policy"] = json!("required");
    let r = call(req);
    assert_eq!(r["ok"], true, "{r}");
}
