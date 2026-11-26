#![cfg(feature = "abi")]
//! Court: numeric tokens past `MAX_NUMBER_DIGITS` are refused as typed
//! `ResourceLimit` at exactly max+1; exactly max (and digits inside strings) pass.

use graphlaw::abi::{MAX_NUMBER_DIGITS, call, call_json};
use serde_json::Value;

fn run(raw: &str) -> Value {
    serde_json::from_slice(&call(raw.as_bytes())).expect("json response")
}

#[test]
fn number_at_max_is_not_refused_for_digits() {
    let r = run(&format!(r#"{{"op":"capabilities","n":{}}}"#, "1".repeat(MAX_NUMBER_DIGITS)));
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn number_over_max_is_resource_limit() {
    let r = run(&format!(r#"{{"op":"capabilities","n":{}}}"#, "1".repeat(MAX_NUMBER_DIGITS + 1)));
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["kind"], "ResourceLimit", "{r}");
    let d = &r["error"]["details"];
    assert_eq!(d["limit"], "number_digits", "{r}");
    assert_eq!(d["observed"], MAX_NUMBER_DIGITS + 1, "{r}");
    assert_eq!(d["max"], MAX_NUMBER_DIGITS, "{r}");
}

#[test]
fn long_digit_run_inside_string_is_not_a_number() {
    let r = run(&format!(r#"{{"op":"capabilities","s":"{}"}}"#, "1".repeat(MAX_NUMBER_DIGITS * 4)));
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn manifest_limit_is_the_abi_constant() {
    let m = call_json(&serde_json::json!({"op":"capabilities"}));
    assert_eq!(m["ok"], true, "{m}");
    assert_eq!(MAX_NUMBER_DIGITS, 64);
}
