use serde_json::{Value, json};

use super::super::{ABI_VERSION, Res, bad, opt_str, state_field, str_field};
use crate::dialect::sniff;

/// `frozen-duckdb` is a normal dependency of every target — including the wasm
/// unit — so every backend authority is reported on all targets.
fn report_authority(_authority: &str) -> bool {
    true
}

pub(in crate::abi) fn capabilities() -> Value {
    // P28-SIL overflow policy: the evidentiary counters are runtime facts, not
    // hashed surface inputs — cfg-gated additive keys (absent with the feature
    // off, so target-invariance / digest inputs are unchanged).
    #[cfg(all(feature = "evidentiary-ring", not(target_arch = "wasm32")))]
    let evidentiary_counters = {
        let ring = crate::evidence_ring::EvidenceRing::global();
        let dropped = ring.map(|r| r.dropped_total()).unwrap_or(0);
        // Gap markers are counted read-only from the ledger's `gaps/` dir; a
        // full QueryLedger::open here would re-verify the chain under the
        // drainer's exclusive handle (racy), so we only read_dir the markers.
        let gaps = ring
            .and_then(|r| r.ledger_dir().map(<std::path::Path>::to_path_buf))
            .and_then(|d| {
                std::fs::read_dir(d.join("gaps"))
                    .ok()?
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .try_fold(0u64, |n, name| {
                        if name.starts_with("gap-") && name.ends_with(".json") {
                            Ok::<_, std::io::Error>(n + 1)
                        } else {
                            Ok(n)
                        }
                    })
                    .ok()
            })
            .unwrap_or(0);
        json!({
            "evidentiary_ring_dropped": dropped,
            "evidentiary_ledger_gaps": gaps,
        })
    };
    #[cfg(not(all(feature = "evidentiary-ring", not(target_arch = "wasm32"))))]
    let evidentiary_counters = json!({});
    let mut caps = json!({
        "abi": ABI_VERSION,
        "abi_version": ABI_VERSION,
        "crate": env!("CARGO_PKG_VERSION"),
        "authorities": crate::BACKEND_AUTHORITIES.iter().filter(|a| report_authority(a.authority)).map(|a| json!({
            "capability": a.capability, "authority": a.authority, "revision": a.revision,
        })).collect::<Vec<_>>(),
        "rdf_dialects": crate::registry::rdf_dialect_names(),
        "other_dialects": crate::registry::other_dialect_names(),
        "ops": crate::registry::op_names(),
        "registry_schema": crate::registry::REGISTRY_SCHEMA,
        "registry_blake3": crate::registry::registry_blake3(),
        "surface_blake3": crate::registry::surface_blake3(),
    });
    let obj = caps.as_object_mut().expect("capabilities object");
    for (k, v) in evidentiary_counters.as_object().expect("json object") {
        obj.insert(k.clone(), v.clone());
    }
    caps
}

pub(in crate::abi) fn op_sniff(v: &Value) -> Res<Value> {
    let d = sniff(str_field(v, "text")?.as_bytes(), opt_str(v, "hint"))?;
    Ok(json!({"dialect": format!("{d:?}"), "engine": format!("{:?}", d.engine())}))
}

pub(in crate::abi) fn op_canonical(v: &Value) -> Res<Value> {
    let s = state_field(v, "data")?;
    let c = purrdf::try_canonicalize(s.dataset()).map_err(|e| bad(format!("{e:?}")))?;
    Ok(json!({"id": s.id(), "nquads": c.nquads, "quads": s.quad_count()}))
}
