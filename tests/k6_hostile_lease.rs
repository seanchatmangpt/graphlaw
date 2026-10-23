//! Lane K6 adversarial court — null hypothesis: "typed refusals make the
//! authority model unforgeable." Every attack either flips to a typed refusal
//! (concede) or demonstrates a bypass (file:line cited in the lane report).
//!
//! Real frozen surface: `graphlaw::abi::call_json` on the host ABI, real
//! Ed25519 keys, real state transitions. No mocks. Read-only vs src/.

use serde_json::{Value, json};

fn call(v: Value) -> Value {
    graphlaw::abi::call_json(&v)
}

fn refusal_of(r: &Value) -> (String, String) {
    assert_eq!(r["ok"], false, "{r}");
    (
        r["error"]["details"]["code"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        r["error"]["details"]["reason"]
            .as_str()
            .unwrap_or("")
            .to_string(),
    )
}

fn data() -> Value {
    json!({"text": "<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n<urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n", "dialect": "ntriples"})
}

fn lease(id: &str, ceiling: &str, scope: Value, expires_unix: u64, issued_unix: u64) -> Value {
    json!({"id": id, "holder": "h", "ceiling": ceiling, "scope": scope,
           "expires_unix": expires_unix, "issued_unix": issued_unix})
}

fn signed_lease_of(l: Value, key: &graphlaw::attest::SigningKey) -> Value {
    use graphlaw::law::{Ceiling, Lease};
    let parsed = Lease {
        id: l["id"].as_str().unwrap().into(),
        holder: l["holder"].as_str().unwrap().into(),
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
    let signed = graphlaw::attest::sign_lease(key, parsed);
    json!({"lease": l, "attestation": {"key_id": signed.attestation.key_id,
           "payload_blake3": signed.attestation.payload_blake3,
           "signature": signed.attestation.signature}})
}

fn signed_req(sl: Value, trusted: &graphlaw::attest::SigningKey, steps: Value) -> Value {
    json!({"op": "law", "data": data(), "signed_lease": sl,
           "trusted_keys": [trusted.verifying_key().to_hex()], "steps": steps})
}

fn unverified_req(l: Value, now: u64, steps: Value) -> Value {
    json!({"op": "law", "data": data(), "lease": l, "unverified_lease": true,
           "now_unix": now, "steps": steps})
}

const NOW: u64 = 1_800_000_000; // far from both epoch and u64 wrap

// ---------------------------------------------------------------------------
// Attack 1: expiry wrap-around / 0 / far-future.
// ---------------------------------------------------------------------------
#[test]
fn a1_expiry_zero_is_expired_refusal() {
    let k = graphlaw::attest::SigningKey::from_seed([3; 32]);
    // expires_unix = 0: expired at every now >= 0, including "pre-epoch".
    let r = call(signed_req(
        signed_lease_of(lease("L0", "construct", json!(["derive:rdfs"]), 0, 0), &k),
        &k,
        json!([{"step": "rdfs"}]),
    ));
    assert_eq!(
        refusal_of(&r),
        ("LeaseRefused".to_string(), "expired".to_string())
    );
}

#[test]
fn a1_expiry_far_future_admits_only_with_valid_signature() {
    let k = graphlaw::attest::SigningKey::from_seed([3; 32]);
    let good = call(signed_req(
        signed_lease_of(
            lease("LF", "construct", json!(["derive:rdfs"]), u64::MAX, 0),
            &k,
        ),
        &k,
        json!([{"step": "rdfs"}]),
    ));
    assert_eq!(good["ok"], true, "{good}");
    assert_eq!(good["receipts"][0]["lease_id"], "LF");

    // Same far-future expiry but the lease payload edited after signing:
    // must be bad_signature, never an admission.
    let k2 = graphlaw::attest::SigningKey::from_seed([9; 32]);
    let sl = signed_lease_of(lease("LF", "construct", json!(["derive:rdfs"]), 100, 0), &k);
    let mut tampered = sl.clone();
    tampered["lease"]["expires_unix"] = json!(u64::MAX);
    let r = call(signed_req(tampered, &k, json!([{"step": "rdfs"}])));
    assert_eq!(
        refusal_of(&r),
        ("LeaseRefused".to_string(), "bad_signature".to_string())
    );
    let _ = k2;
}

#[test]
fn a1_issued_far_future_is_clock_skew_refusal() {
    let k = graphlaw::attest::SigningKey::from_seed([3; 32]);
    let r = call(signed_req(
        signed_lease_of(
            lease(
                "LK",
                "construct",
                json!(["derive:rdfs"]),
                u64::MAX,
                u64::MAX,
            ),
            &k,
        ),
        &k,
        json!([{"step": "rdfs"}]),
    ));
    assert_eq!(refusal_of(&r), ("LeaseRefused".into(), "clock_skew".into()));
}

// ---------------------------------------------------------------------------
// Attack 2: clock-skew boundary +-1s (deterministic via FixedClock; the ABI
// path uses SystemClock so only the pure predicate is probed here — it is the
// same predicate SignedLease::authorize calls at src/law.rs:362-365).
// ---------------------------------------------------------------------------
#[test]
fn a2_skew_boundary_is_exact_to_the_second() {
    use graphlaw::attest::{SigningKey, TrustedKeys, sign_lease};
    use graphlaw::law::{FixedClock, Lease, SignedLease};
    let k = SigningKey::from_seed([3; 32]);
    let mut trusted = TrustedKeys::new();
    trusted.insert(k.verifying_key());
    let mk = |issued: u64| SignedLease {
        lease: Lease {
            id: "S".into(),
            holder: "h".into(),
            ceiling: graphlaw::law::Ceiling::Construct,
            scope: vec!["derive:rdfs".into()],
            expires_unix: u64::MAX,
            issued_unix: issued,
        },
        attestation: sign_lease(
            &k,
            Lease {
                id: "S".into(),
                holder: "h".into(),
                ceiling: graphlaw::law::Ceiling::Construct,
                scope: vec!["derive:rdfs".into()],
                expires_unix: u64::MAX,
                issued_unix: issued,
            },
        )
        .attestation,
    };
    let now: u64 = 1_800_000_000;
    let reason_of = |e: graphlaw::law::LawError| match e {
        graphlaw::law::LawError::LeaseRefused { reason, .. } => reason,
        other => panic!("expected LeaseRefused, got {other:?}"),
    };
    // issued = now + 60 == skew bound: admits (documented DEFAULT_MAX_SKEW_SECS).
    assert!(
        mk(now + 60)
            .authorize(
                "derive:rdfs",
                graphlaw::law::Ceiling::Construct,
                &trusted,
                &FixedClock(now),
                60
            )
            .is_ok()
    );
    // issued = now + 61: one second past the bound flips to ClockSkew.
    assert_eq!(
        reason_of(
            mk(now + 61)
                .authorize(
                    "derive:rdfs",
                    graphlaw::law::Ceiling::Construct,
                    &trusted,
                    &FixedClock(now),
                    60
                )
                .unwrap_err()
        ),
        graphlaw::law::LeaseReason::ClockSkew
    );
    // Expiry boundary: expires == now is expired; expires == now+1 admits.
    let mut expired = mk(0);
    expired.lease.expires_unix = now;
    expired.attestation = sign_lease(&k, expired.lease.clone()).attestation;
    assert_eq!(
        reason_of(
            expired
                .authorize(
                    "derive:rdfs",
                    graphlaw::law::Ceiling::Construct,
                    &trusted,
                    &FixedClock(now),
                    60
                )
                .unwrap_err()
        ),
        graphlaw::law::LeaseReason::Expired
    );
    let mut alive = mk(0);
    alive.lease.expires_unix = now + 1;
    alive.attestation = sign_lease(&k, alive.lease.clone()).attestation;
    assert!(
        alive
            .authorize(
                "derive:rdfs",
                graphlaw::law::Ceiling::Construct,
                &trusted,
                &FixedClock(now),
                60
            )
            .is_ok()
    );
}

// ---------------------------------------------------------------------------
// Attack 3: ceiling bypass — Observe lease asked to run a Construct step.
// ---------------------------------------------------------------------------
#[test]
fn a3_observe_lease_cannot_run_construct_step() {
    let k = graphlaw::attest::SigningKey::from_seed([3; 32]);
    // derive:rdfs requires Ceiling::Construct (src/law.rs:120-122).
    let r = call(signed_req(
        signed_lease_of(
            lease("OB", "observe", json!(["derive:rdfs"]), u64::MAX, 0),
            &k,
        ),
        &k,
        json!([{"step": "rdfs"}]),
    ));
    assert_eq!(refusal_of(&r), ("LeaseRefused".into(), "ceiling".into()));

    // Same through the unverified lane: the ceiling check is identical.
    let r2 = call(unverified_req(
        lease("OB", "observe", json!(["derive:rdfs"]), u64::MAX, 0),
        0,
        json!([{"step": "rdfs"}]),
    ));
    assert_eq!(refusal_of(&r2), ("LeaseRefused".into(), "ceiling".into()));
}

// ---------------------------------------------------------------------------
// Attack 4: out-of-scope step (lease scoped to a different step name; the
// scope vocabulary is step names, there is no repo-path scope in the model).
// ---------------------------------------------------------------------------
#[test]
fn a4_step_scoped_to_other_step_is_out_of_scope() {
    let k = graphlaw::attest::SigningKey::from_seed([3; 32]);
    let r = call(signed_req(
        signed_lease_of(
            lease(
                "SC",
                "construct",
                json!(["derive:n3", "admit:shacl"]),
                u64::MAX,
                0,
            ),
            &k,
        ),
        &k,
        json!([{"step": "rdfs"}]),
    ));
    assert_eq!(
        refusal_of(&r),
        ("LeaseRefused".into(), "out_of_scope".into())
    );

    // Multi-step request: lease valid for step 1 only — step 2 must refuse
    // (per-step re-check, no "validated once, valid forever" leak).
    let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix ex: <urn:e:> .\nex:S a sh:NodeShape ; sh:targetClass ex:T ; sh:property [ sh:path ex:p ; sh:minCount 1 ] .\n";
    let two = json!([
        {"step": "shacl", "shapes": shapes},
        {"step": "rdfs"}
    ]);
    let r2 = call(signed_req(
        signed_lease_of(
            lease("SC2", "construct", json!(["admit:shacl"]), u64::MAX, 0),
            &k,
        ),
        &k,
        two,
    ));
    assert_eq!(
        refusal_of(&r2),
        ("LeaseRefused".into(), "out_of_scope".into())
    );
    // And the first step's work must NOT have leaked into a receipt stream
    // that reports success: the whole request errors.
    let _ = r2["error"]["message"].as_str().unwrap_or("");
}

// ---------------------------------------------------------------------------
// Attack 5: receipts — forged but wellformed JSON, tampered, and unsigned.
// ---------------------------------------------------------------------------
#[test]
fn a5_unsigned_receipt_fails_the_signed_gate() {
    use graphlaw::attest::SigningKey;
    use graphlaw::dialect::Dialect;
    use graphlaw::law::{LawState, Step};
    let base = LawState::parse(
        data()["text"].as_str().unwrap().as_bytes(),
        Dialect::NTriples,
        None,
    )
    .unwrap();
    let (child, r) = base.transition(&Step::EntailRdfs).unwrap();
    let k = SigningKey::from_seed([1; 32]);
    let nq = |s: &LawState| {
        String::from_utf8(
            graphlaw::purrdf::serialize_dataset(
                s.dataset().as_ref(),
                "application/n-quads",
                graphlaw::purrdf::SerializeGraph::Dataset,
            )
            .unwrap(),
        )
        .unwrap()
    };
    let unsigned = nq(&graphlaw::receipt::record(&child, &r).unwrap());
    let req = json!({"op": "law", "data": {"text": unsigned, "dialect": "nquads"},
        "steps": [{"step": "require-signed-receipt", "step_name": "derive:rdfs",
                   "trusted_keys": [k.verifying_key().to_hex()]}]});
    assert_eq!(
        refusal_of(&call(req)),
        ("ReceiptRefused".into(), "unattested".into())
    );
}

#[test]
fn a5_tampered_and_forged_receipts_refuse() {
    use graphlaw::attest::{SigningKey, sign_receipt};
    use graphlaw::dialect::Dialect;
    use graphlaw::law::{LawState, Step};
    let base = LawState::parse(
        data()["text"].as_str().unwrap().as_bytes(),
        Dialect::NTriples,
        None,
    )
    .unwrap();
    let (child, r) = base.transition(&Step::EntailRdfs).unwrap();
    let k = SigningKey::from_seed([1; 32]);
    let nq = |s: &LawState| {
        String::from_utf8(
            graphlaw::purrdf::serialize_dataset(
                s.dataset().as_ref(),
                "application/n-quads",
                graphlaw::purrdf::SerializeGraph::Dataset,
            )
            .unwrap(),
        )
        .unwrap()
    };

    // Tampered: validly signed receipt, one extra "added" triple grafted on
    // the recorded receipt subject (subject IRI lifted from the real record).
    let tamper_state = graphlaw::receipt::record_signed(&child, &r, &sign_receipt(&k, &r)).unwrap();
    let doc = nq(&tamper_state);
    let subj = doc
        .lines()
        .find(|l| l.contains("urn:graphlaw:receipt:"))
        .unwrap()
        .split(' ')
        .next()
        .unwrap()
        .to_string();
    let forged = format!(
        "{}\n{} <urn:graphlaw:p:added> \"999\" .\n",
        doc.trim_end(),
        subj
    );
    let tamper_state = LawState::parse(forged.as_bytes(), Dialect::NQuads, None).unwrap();
    let req_t = json!({"op": "law", "data": {"text": nq(&tamper_state), "dialect": "nquads"},
        "steps": [{"step": "require-signed-receipt", "step_name": "derive:rdfs",
                   "trusted_keys": [k.verifying_key().to_hex()]}]});
    let (code, reason) = refusal_of(&call(req_t));
    assert!(
        (code == "ReceiptRefused" && (reason == "bad_signature" || reason == "unattested")),
        "tampered receipt admitted: {code}/{reason}"
    );

    // Forged-but-wellformed: entirely hand-written receipt triples using the
    // exact predicate namespace, signed by nobody.
    let n = format!("<urn:graphlaw:receipt:{}>", r.child);
    let forged_doc = format!(
        "{n} <urn:graphlaw:p:step> \"derive:rdfs\" .\n\
         {n} <urn:graphlaw:p:child> \"{}\" .\n\
         {n} <urn:graphlaw:p:parent> \"deadbeef\" .\n\
         {n} <urn:graphlaw:p:authority> \"purrdf\" .\n\
         {n} <urn:graphlaw:p:revision> \"1\" .\n\
         {n} <urn:graphlaw:p:added> \"0\" .\n",
        r.child
    );
    let req_f = json!({"op": "law", "data": {"text": forged_doc, "dialect": "ntriples"},
        "steps": [{"step": "require-signed-receipt", "step_name": "derive:rdfs",
                   "trusted_keys": [k.verifying_key().to_hex()]}]});
    let (code, reason) = refusal_of(&call(req_f));
    assert!(
        (code == "ReceiptRefused" || code == "ReceiptRequired"),
        "forged receipt admitted: {code}/{reason}"
    );
}

#[test]
fn a5_unsigned_gate_still_requires_a_recorded_receipt() {
    // `require-receipt` (Observe gate) with no receipt at all: ReceiptRequired.
    let req = json!({"op": "law", "data": data(),
        "steps": [{"step": "require-receipt", "step_name": "derive:rdfs"}]});
    assert_eq!(
        refusal_of(&call(req)),
        ("ReceiptRequired".into(), "".into())
    );
}

// ---------------------------------------------------------------------------
// Documented downgrade, not a bypass: the explicit unverified lane lets a
// caller-picked clock revive an expired lease. Receipt still says which lease.
// ---------------------------------------------------------------------------
#[test]
fn d1_unverified_lane_is_a_loud_downgrade_not_a_bypass() {
    let r = call(unverified_req(
        lease("EX", "construct", json!(["derive:rdfs"]), 100, 0),
        0, // caller-chosen "now"
        json!([{"step": "rdfs"}]),
    ));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["receipts"][0]["lease_id"], "EX");
    // Without the explicit flag it refuses.
    let mut req = unverified_req(
        lease("EX", "construct", json!(["derive:rdfs"]), 100, 0),
        0,
        json!([{"step": "rdfs"}]),
    );
    req.as_object_mut().unwrap().remove("unverified_lease");
    assert_eq!(refusal_of(&call(req)).0, "UnverifiedLeaseRefused");
}
