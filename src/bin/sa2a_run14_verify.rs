//! Verify the SA2A run14 semantic-evidence-court corpus through GraphLaw's real JSON ABI.
//!
//! Vocabulary: `sa2a:SemanticEvidenceCourt` in exactly two namespaces:
//!   V1 `https://sa2a.dev/ontology#`   (declared `sa2a:expected`, cross-checked against the
//!                                      filename disposition word and `sa2a:courtId`)
//!   V2 `https://chatmangpt.com/sa2a#` (declared `sa2a:expectedDecision`, cross-checked against
//!                                      `sa2a:observedState`, `sa2a:replayKey` and the filename)
//! Ceiling: the disposition is a declared fixture property cross-checked against its own
//! metadata; it is not computed from payload. Evidence only: authority NONE, no DO.
//!
//! Usage: `sa2a-run14-verify [DIR]` (default `conformance/sa2a/run14`). Prints one JSON object;
//! exit 0 iff every fixture is well-typed and self-consistent, 1 on typed failures, 2 on I/O.
use std::{collections::BTreeMap, fs, path::PathBuf, process::ExitCode};

use serde_json::{Value, json};

const V1: &str = "https://sa2a.dev/ontology#";
const V2: &str = "https://chatmangpt.com/sa2a#";

fn corpus_dir() -> PathBuf {
    std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("conformance/sa2a/run14"))
}

fn request(v: Value) -> Value {
    serde_json::from_slice(&graphlaw::abi::call(v.to_string().as_bytes()))
        .expect("GraphLaw ABI returns JSON")
}

/// Predicate local-name -> lexical values, for the single court node in namespace `ns`.
fn court_props(text: &str, ns: &str) -> Result<Option<BTreeMap<String, Vec<String>>>, String> {
    let q = format!("SELECT ?s ?p ?o WHERE {{ ?s a <{ns}SemanticEvidenceCourt> ; ?p ?o }}");
    let out = request(json!({"op":"sparql","data":{"text":text,"dialect":"turtle"},"query":q}));
    if out["ok"] != true {
        return Err(format!("sparql refused: {out}"));
    }
    let rows = out["result"]["rows"]
        .as_array()
        .or_else(|| out["rows"].as_array())
        .ok_or_else(|| format!("no rows array in {out}"))?;
    if rows.is_empty() {
        return Ok(None);
    }
    let mut subjects = std::collections::BTreeSet::new();
    let mut props: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in rows {
        let s = r[0]["value"].as_str().unwrap_or_default().to_string();
        subjects.insert(s);
        let p = r[1]["value"].as_str().unwrap_or_default();
        let o = r[2]["value"].as_str().unwrap_or_default().to_string();
        if let Some(local) = p.strip_prefix(ns) {
            props.entry(local.to_string()).or_default().push(o);
        }
    }
    if subjects.len() != 1 {
        return Err(format!(
            "expected exactly one court node, found {}",
            subjects.len()
        ));
    }
    Ok(Some(props))
}

fn one<'a>(p: &'a BTreeMap<String, Vec<String>>, k: &str) -> Result<&'a str, String> {
    match p.get(k).map(Vec::as_slice) {
        Some([v]) => Ok(v.as_str()),
        Some(v) => Err(format!("sa2a:{k} has {} values, expected 1", v.len())),
        None => Err(format!("missing sa2a:{k}")),
    }
}

/// (namespace label, disposition) or typed (code, detail).
fn judge(stem: &str, text: &str) -> Result<(&'static str, String), (&'static str, String)> {
    let parsed = request(json!({"op":"parse","text":text,"dialect":"turtle"}));
    if parsed["ok"] != true {
        return Err(("PARSE_REFUSED", parsed.to_string()));
    }
    let canon = request(json!({"op":"canonical","data":{"text":text,"dialect":"turtle"}}));
    if canon["ok"] != true {
        return Err(("CANONICALIZATION_REFUSED", canon.to_string()));
    }
    let v1 = court_props(text, V1).map_err(|e| ("QUERY_FAILED", e))?;
    let v2 = court_props(text, V2).map_err(|e| ("QUERY_FAILED", e))?;
    let (label, ns, props) = match (v1, v2) {
        (Some(p), None) => ("v1", V1, p),
        (None, Some(p)) => ("v2", V2, p),
        (None, None) => {
            return Err((
                "UNKNOWN_NAMESPACE",
                "no sa2a:SemanticEvidenceCourt in a recognised namespace".into(),
            ));
        }
        (Some(_), Some(_)) => {
            return Err((
                "AMBIGUOUS_NAMESPACE",
                "court declared in both namespaces".into(),
            ));
        }
    };
    let _ = ns;
    let bad = |e: String| ("MALFORMED_COURT", e);
    for (k, want) in [
        ("authority", "NONE"),
        ("consequence", "EVIDENCE_ONLY"),
        ("canonicalization", "RDFC-1.0"),
    ] {
        let got = one(&props, k).map_err(bad)?;
        if got != want {
            return Err((
                "INVARIANT_VIOLATED",
                format!("sa2a:{k} is {got:?}, must be {want:?}"),
            ));
        }
    }
    one(&props, "exactSubject").map_err(bad)?;
    one(&props, "falsifier").map_err(bad)?;
    let name = stem.split_once('_').map_or(stem, |(_, n)| n);
    let disposition;
    if label == "v1" {
        disposition = one(&props, "expected").map_err(bad)?.to_string();
        let court = one(&props, "courtId").map_err(bad)?;
        if court != name {
            return Err((
                "COURT_ID_MISMATCH",
                format!("courtId {court:?} != filename {name:?}"),
            ));
        }
        let by_name = if name.ends_with("_bound") {
            "ADMIT"
        } else if name.ends_with("_drift") || name.ends_with("_missing") {
            "REFUSE"
        } else {
            return Err((
                "UNCLASSIFIABLE_FILENAME",
                format!("no disposition word in {name:?}"),
            ));
        };
        if by_name != disposition {
            return Err((
                "DISPOSITION_MISMATCH",
                format!("declared {disposition} but filename implies {by_name}"),
            ));
        }
    } else {
        disposition = one(&props, "expectedDecision").map_err(bad)?.to_string();
        let num = stem.split('_').next().unwrap_or_default();
        let rk = one(&props, "replayKey").map_err(bad)?;
        if rk != format!("run14:{num}:{name}") {
            return Err((
                "REPLAY_KEY_MISMATCH",
                format!("replayKey {rk:?} != run14:{num}:{name}"),
            ));
        }
        one(&props, "dimension").map_err(bad)?;
        if let Ok(state) = one(&props, "observedState") {
            let by_state = match state {
                "BOUND" | "NONE" | "EVIDENCE_ONLY" | "POWERLESS" | "RDFC-1.0" => "ADMIT",
                "MISSING" | "DRIFT" | "NON_RDFC" | "NON_NONE" | "DO" | "AUTHORITATIVE" => "REFUSE",
                other => return Err(("UNKNOWN_OBSERVED_STATE", other.to_string())),
            };
            if by_state != disposition {
                return Err((
                    "DISPOSITION_MISMATCH",
                    format!("declared {disposition} but observedState {state} implies {by_state}"),
                ));
            }
        }
    }
    if disposition != "ADMIT" && disposition != "REFUSE" {
        return Err(("UNKNOWN_DISPOSITION", disposition));
    }
    Ok((label, disposition))
}

fn main() -> ExitCode {
    let dir = corpus_dir();
    let mut files: Vec<PathBuf> = match fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
            .collect(),
        Err(e) => {
            println!(
                "{}",
                json!({"ok":false,"failures":[{"file":dir.display().to_string(),"code":"IO","detail":e.to_string()}]})
            );
            return ExitCode::from(2);
        }
    };
    files.sort();
    let (mut admit, mut refuse, mut v1n, mut v2n) = (0usize, 0usize, 0usize, 0usize);
    let (mut fixtures, mut failures) = (vec![], vec![]);
    for path in &files {
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(json!({"file":file,"code":"IO","detail":e.to_string()}));
                continue;
            }
        };
        match judge(&stem, &text) {
            Ok((label, d)) => {
                if d == "ADMIT" {
                    admit += 1
                } else {
                    refuse += 1
                }
                if label == "v1" {
                    v1n += 1
                } else {
                    v2n += 1
                }
                fixtures.push(json!({"file":file,"namespace":label,"disposition":d}));
            }
            Err((code, detail)) => failures.push(json!({"file":file,"code":code,"detail":detail})),
        }
    }
    let ok = failures.is_empty();
    println!(
        "{}",
        json!({"ok":ok,"vectors":files.len(),"admit":admit,"refuse":refuse,
        "v1":v1n,"v2":v2n,"authority":"NONE","fixtures":fixtures,"failures":failures})
    );
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
