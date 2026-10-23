//! graphlaw-ocel-export — emit graphlaw law-transition traces as OCEL v2.
//!
//! Replays a real law session through the real transition path
//! (`LawState::parse` -> `transition`), exactly as `tests/law_state.rs` does,
//! and writes each transition as OCEL v2 events in a SQLite database whose
//! schema is byte-shaped for pm4py's OCEL2 SQLite reader (the schema
//! pm4pytest's `read_ocel2_sqlite` consumes), so an external pm4pytest court
//! can judge the trace.
//!
//! Activities: {admit, refuse, receipt}.
//! Objects:    {plan, subject, authority}.
//!
//! SQLite is written through the system `sqlite3` CLI (no rusqlite in the
//! manifest, which is coordinator-owned). A missing CLI is a typed
//! capability refusal (REFUSAL-04 class), never a panic.

use std::process::Command;

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawError, LawState, Receipt, Step};
use graphlaw::tier::validate_n3_rules;

/// One OCEL v2 object (id, type, attribute pairs).
struct OcelObject {
    id: String,
    kind: &'static str,
    attrs: Vec<(String, String)>,
}

/// One OCEL v2 event (id, activity, time, attributes, object relationships).
struct OcelEvent {
    id: String,
    kind: &'static str,
    time: u64,
    attrs: Vec<(String, String)>,
    rels: Vec<(String, String)>,
}

/// The full exported trace.
struct OcelTrace {
    objects: Vec<OcelObject>,
    events: Vec<OcelEvent>,
}

fn read(path: &str) -> String {
    std::fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))).expect(path)
}

/// Monotonic unix-seconds timestamps from the wall clock (never backwards).
fn stamp(prev: &mut u64) -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let t = if now > *prev { now } else { *prev + 1 };
    *prev = t;
    t
}

fn unix_to_ocel_time(unix: u64) -> String {
    // Days->civil algorithm (Howard Hinnant), UTC.
    let days = (unix / 86_400) as i64;
    let secs = unix % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02} {h:02}:{mi:02}:{s:02}+00:00",
        h = secs / 3600,
        mi = (secs % 3600) / 60,
        s = secs % 60
    )
}

/// A refusal/error flattened to a stable code string for the trace.
fn refusal_code(err: &LawError) -> String {
    match err {
        LawError::NotAdmitted { violations, .. } => {
            format!("NOT_ADMITTED({violations})")
        }
        LawError::PlanRefused { index, .. } => format!("PLAN_REFUSED(action={index})"),
        LawError::ReceiptRequired { step } => format!("RECEIPT_REQUIRED({step})"),
        LawError::ReceiptRefused { step, .. } => format!("RECEIPT_REFUSED({step})"),
        LawError::LeaseRefused { lease_id, step, .. } => {
            format!("LEASE_REFUSED({lease_id},{step})")
        }
        other => format!("REFUSED({})", short(other)),
    }
}

fn short(err: &LawError) -> String {
    // First line only, truncated — trace attribute, not a log dump.
    let s = err.to_string();
    let line = s.lines().next().unwrap_or("").to_string();
    if line.len() > 120 {
        format!("{}...", &line[..120])
    } else {
        line
    }
}

fn sql_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// Emit the trace to `path` as a pm4py-OCEL2-SQLite-schema database via the
/// system sqlite3 CLI. Returns the count of events written.
fn write_ocel2_sqlite(trace: &OcelTrace, path: &str) -> Result<usize, String> {
    let mut sql = String::new();
    sql.push_str("PRAGMA writable_schema=OFF;\nBEGIN;\n");
    sql.push_str(
        "CREATE TABLE IF NOT EXISTS \"event\" (\"ocel_id\" TEXT, \"ocel_type\" TEXT);\n\
         CREATE TABLE IF NOT EXISTS \"object\" (\"ocel_id\" TEXT, \"ocel_type\" TEXT);\n\
         CREATE TABLE IF NOT EXISTS \"event_map_type\" (\"ocel_type\" TEXT, \"ocel_type_map\" TEXT);\n\
         CREATE TABLE IF NOT EXISTS \"object_map_type\" (\"ocel_type\" TEXT, \"ocel_type_map\" TEXT);\n\
         CREATE TABLE IF NOT EXISTS \"event_object\" (\"ocel_event_id\" TEXT, \"ocel_object_id\" TEXT, \"ocel_qualifier\" TEXT);\n\
         CREATE TABLE IF NOT EXISTS \"object_object\" (\"ocel_source_id\" TEXT, \"ocel_target_id\" TEXT, \"ocel_qualifier\" TEXT);\n",
    );
    let mut event_types: Vec<&str> = trace.events.iter().map(|e| e.kind).collect();
    event_types.sort_unstable();
    event_types.dedup();
    let mut object_types: Vec<&str> = trace.objects.iter().map(|o| o.kind).collect();
    object_types.sort_unstable();
    object_types.dedup();
    for t in &event_types {
        let mut cap = String::from(*t);
        if let Some(c) = cap.get_mut(0..1) {
            cap.get_mut(0..1).unwrap().make_ascii_uppercase();
        }
        sql.push_str(&format!(
            "CREATE TABLE IF NOT EXISTS \"event_{cap}\" (\"ocel_id\" TEXT, \"ocel_time\" TIMESTAMP, \"ocel:activity\" TEXT);\n"
        ));
    }
    for t in &object_types {
        let mut cap = String::from(*t);
        if let Some(c) = cap.get_mut(0..1) {
            cap.get_mut(0..1).unwrap().make_ascii_uppercase();
        }
        sql.push_str(&format!(
            "CREATE TABLE IF NOT EXISTS \"object_{cap}\" (\"ocel_id\" TEXT);\n"
        ));
    }
    // Attribute columns are fixed by activity/object type.
    let attr_cols: &[(&str, &[&str])] = &[
        ("admit", &["step", "added", "parent", "child"]),
        ("refuse", &["step", "code", "detail"]),
        (
            "receipt",
            &["step", "authority", "parent", "child", "subject"],
        ),
    ];
    for (kind, cols) in attr_cols {
        let mut cap = String::from(*kind);
        cap.get_mut(0..1).unwrap().make_ascii_uppercase();
        for c in cols.iter() {
            sql.push_str(&format!(
                "ALTER TABLE \"event_{cap}\" ADD COLUMN {c} TEXT;\n"
            ));
        }
    }
    let obj_cols: &[(&str, &[&str])] = &[
        ("plan", &["digest"]),
        ("subject", &["digest"]),
        ("authority", &["backend"]),
    ];
    for (kind, cols) in obj_cols {
        let mut cap = String::from(*kind);
        cap.get_mut(0..1).unwrap().make_ascii_uppercase();
        for c in cols.iter() {
            sql.push_str(&format!(
                "ALTER TABLE \"object_{cap}\" ADD COLUMN {c} TEXT;\n"
            ));
        }
    }

    for o in &trace.objects {
        sql.push_str(&format!(
            "INSERT INTO \"object\" VALUES ({}, {});\n",
            sql_quote(&o.id),
            sql_quote(o.kind)
        ));
        let mut cap = String::from(o.kind);
        cap.get_mut(0..1).unwrap().make_ascii_uppercase();
        let mut row = vec![sql_quote(&o.id)];
        for name in obj_cols
            .iter()
            .find(|(k, _)| **k == *o.kind)
            .expect("known object type")
            .1
            .iter()
            .copied()
        {
            let v = o
                .attrs
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            let _ = name;
            row.push(sql_quote(&v));
        }
        sql.push_str(&format!(
            "INSERT INTO \"object_{cap}\" VALUES ({});\n",
            row.join(", ")
        ));
    }
    for t in &object_types {
        let mut cap = String::from(*t);
        cap.get_mut(0..1).unwrap().make_ascii_uppercase();
        sql.push_str(&format!(
            "INSERT INTO \"object_map_type\" VALUES ({}, {});\n",
            sql_quote(t),
            sql_quote(&cap)
        ));
    }
    for e in &trace.events {
        sql.push_str(&format!(
            "INSERT INTO \"event\" VALUES ({}, {});\n",
            sql_quote(&e.id),
            sql_quote(e.kind)
        ));
        let mut cap = String::from(e.kind);
        cap.get_mut(0..1).unwrap().make_ascii_uppercase();
        let cols = attr_cols
            .iter()
            .find(|(k, _)| **k == *e.kind)
            .ok_or_else(|| format!("unknown activity type {}", e.kind))?
            .1;
        let mut row = vec![sql_quote(&e.id), sql_quote(&unix_to_ocel_time(e.time))];
        row.push(sql_quote(e.kind));
        for name in cols.iter().copied() {
            let v = e
                .attrs
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            row.push(sql_quote(&v));
        }
        sql.push_str(&format!(
            "INSERT INTO \"event_{cap}\" VALUES ({});\n",
            row.join(", ")
        ));
        for (oid, qual) in &e.rels {
            sql.push_str(&format!(
                "INSERT INTO \"event_object\" VALUES ({}, {}, {});\n",
                sql_quote(&e.id),
                sql_quote(oid),
                sql_quote(qual)
            ));
        }
    }
    for t in &event_types {
        let mut cap = String::from(*t);
        cap.get_mut(0..1).unwrap().make_ascii_uppercase();
        sql.push_str(&format!(
            "INSERT INTO \"event_map_type\" VALUES ({}, {});\n",
            sql_quote(t),
            sql_quote(&cap)
        ));
    }
    sql.push_str("COMMIT;\n");

    let mut child = Command::new("sqlite3")
        .arg(path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!(
                "REFUSAL-04 CAPABILITY: cannot execute system sqlite3 CLI ({e}); \
                 OCEL v2 export requires it to write the pm4py-schema database"
            )
        })?;
    use std::io::Write as _;
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(sql.as_bytes())
        .map_err(|e| format!("sqlite3 stdin write failed: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("sqlite3 wait: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "sqlite3 exited {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(trace.events.len())
}

/// Run the real law session and build the OCEL trace.
///
/// Session (all through the real `transition` path):
/// 1. `s0 --DeriveN3--> s1`              admit + receipt
/// 2. `s1 --AdmitShacl--> s1`            admit + receipt
/// 3. `s0 --AdmitShacl--> Err`           refuse (pre-derivation state lacks
///                                       the mandatory ERRC surface)
fn build_trace() -> OcelTrace {
    let rules = read("innovation/blue-ocean-triz.n3");
    let ruleset = validate_n3_rules(rules.as_bytes(), None).expect("rules validate");
    let shapes = read("innovation/candidate.shacl.ttl");
    let s0 = LawState::parse(
        read("innovation/industry-fixture.n3").as_bytes(),
        Dialect::Turtle,
        None,
    )
    .expect("fixture is plain RDF");

    let mut events: Vec<OcelEvent> = Vec::new();
    let mut objects: Vec<OcelObject> = Vec::new();
    let mut clock: u64 = 0;
    let mut eid: usize = 0;

    let mut emit = |activity: &'static str,
                    receipt: Option<&Receipt>,
                    code: Option<String>,
                    detail: String,
                    parent: &str,
                    child: &str,
                    subject: &str,
                    authority: &str,
                    events: &mut Vec<OcelEvent>,
                    eid: &mut usize,
                    clock: &mut u64| {
        let ts = stamp(clock);
        let id = format!("e{eid}");
        *eid += 1;
        let attrs = match activity {
            "admit" => vec![
                ("step".to_string(), receipt.unwrap().step.to_string()),
                ("added".to_string(), receipt.unwrap().added.to_string()),
                ("parent".to_string(), parent.to_string()),
                ("child".to_string(), child.to_string()),
                ("authority".to_string(), authority.to_string()),
            ],
            "refuse" => vec![
                ("step".to_string(), "admit:shacl".to_string()),
                ("code".to_string(), code.unwrap_or_default()),
                ("detail".to_string(), detail.to_string()),
            ],
            "receipt" => vec![
                ("step".to_string(), receipt.unwrap().step.to_string()),
                ("authority".to_string(), authority.to_string()),
                ("parent".to_string(), parent.to_string()),
                ("child".to_string(), child.to_string()),
                (
                    "subject".to_string(),
                    receipt
                        .unwrap()
                        .subject_blake3
                        .clone()
                        .unwrap_or_else(|| "unset".to_string()),
                ),
            ],
            _ => Vec::new(),
        };
        let subject_id = format!("subject:{subject}");
        let authority_id = format!("authority:{authority}");
        let rels = vec![
            (format!("plan:{parent}"), "from_plan".to_string()),
            (format!("plan:{child}"), "to_plan".to_string()),
            (subject_id, "subject".to_string()),
            (authority_id, "witnessed_by".to_string()),
        ];
        events.push(OcelEvent {
            id,
            kind: activity,
            time: ts,
            attrs,
            rels: rels,
        });
    };

    // Step 1: DeriveN3.
    let (s1, r1) = s0
        .transition(&Step::DeriveN3 {
            rules: &rules,
            ruleset: &ruleset,
        })
        .expect("derivation succeeds on the fixture");
    emit(
        "admit",
        Some(&r1),
        None,
        String::new(),
        s0.id(),
        s1.id(),
        r1.subject_blake3
            .clone()
            .unwrap_or_else(|| s1.id().to_string())
            .as_str(),
        r1.authority.authority.clone(),
        &mut events,
        &mut eid,
        &mut clock,
    );
    emit(
        "receipt",
        Some(&r1),
        None,
        String::new(),
        r1.parent.as_str(),
        r1.child.as_str(),
        r1.subject_blake3
            .clone()
            .unwrap_or_else(|| s1.id().to_string())
            .as_str(),
        r1.authority.authority.clone(),
        &mut events,
        &mut eid,
        &mut clock,
    );

    // Step 2: AdmitShacl on the derived state — conforms.
    let (s2, r2) = s1
        .transition(&Step::AdmitShacl {
            shapes_ttl: &shapes,
        })
        .expect("derived state conforms to the admission shapes");
    emit(
        "admit",
        Some(&r2),
        None,
        String::new(),
        s1.id(),
        s2.id(),
        r2.subject_blake3
            .clone()
            .unwrap_or_else(|| s2.id().to_string())
            .as_str(),
        r2.authority.authority.clone(),
        &mut events,
        &mut eid,
        &mut clock,
    );
    emit(
        "receipt",
        Some(&r2),
        None,
        String::new(),
        r2.parent.as_str(),
        r2.child.as_str(),
        r2.subject_blake3
            .clone()
            .unwrap_or_else(|| s2.id().to_string())
            .as_str(),
        r2.authority.authority.clone(),
        &mut events,
        &mut eid,
        &mut clock,
    );

    // Step 3: refusal path — a shape that requires a predicate no node in
    // the derived state carries, so SHACL reports non-conformance.
    let refusing_shapes = format!(
        "@prefix sh: <http://www.w3.org/ns/shacl#> .\n\
         @prefix inv: <https://graphlaw.chatman.io/innovation#> .\n\
         inv:RefusalProbeShape a sh:NodeShape ;\n\
             sh:targetSubjectsOf inv:replaces ;\n\
             sh:property [ sh:path inv:ocelRefusalProbe ; sh:minCount 1 ] .\n"
    );
    let refuse_err = s1
        .transition(&Step::AdmitShacl {
            shapes_ttl: &refusing_shapes,
        })
        .expect_err("a mandatory absent predicate must refuse admission");
    let code = refusal_code(&refuse_err);
    let detail = short(&refuse_err);
    emit(
        "refuse",
        None,
        Some(code),
        detail,
        s0.id(),
        s0.id(),
        s0.id(),
        "purrdf::shapes",
        &mut events,
        &mut eid,
        &mut clock,
    );

    // Objects (deduped by id, order-stable).
    let mut seen: Vec<String> = Vec::new();
    for e in &events {
        for (oid, _) in &e.rels {
            if seen.contains(oid) {
                continue;
            }
            seen.push(oid.clone());
            let (kind, attrs) = if let Some(rest) = oid.strip_prefix("plan:") {
                ("plan", vec![("digest".to_string(), rest.to_string())])
            } else if let Some(rest) = oid.strip_prefix("subject:") {
                ("subject", vec![("digest".to_string(), rest.to_string())])
            } else if let Some(rest) = oid.strip_prefix("authority:") {
                ("authority", vec![("backend".to_string(), rest.to_string())])
            } else {
                continue;
            };
            objects.push(OcelObject {
                id: oid.clone(),
                kind,
                attrs,
            });
        }
    }
    OcelTrace { objects, events }
}

fn main() {
    let out_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: graphlaw-ocel-export <out.sqlite>");
        std::process::exit(2);
    });
    let trace = build_trace();
    let n = write_ocel2_sqlite(&trace, &out_path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });
    println!(
        "exported {n} events / {} objects to {out_path}",
        trace.objects.len()
    );
}
