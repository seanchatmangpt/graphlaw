//! OCEL v2 export court (lane PM4).
//!
//! Chicago: the bin runs a real law session through the real transition path
//! and writes a real SQLite database; the court runs the real bin and asserts
//! on real SQLite readbacks plus a real pm4py OCEL2 readback — no mocks.

use std::path::PathBuf;
use std::process::Command;

fn run_bin(db_path: &str) -> (bool, String) {
    let _ = std::fs::remove_file(db_path);
    let st = Command::new(env!("CARGO_BIN_EXE_graphlaw_ocel_export"))
        .arg(db_path)
        .output()
        .expect("spawn bin");
    (
        st.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&st.stdout),
            String::from_utf8_lossy(&st.stderr)
        ),
    )
}

fn sqlite(db_path: &str, query: &str) -> String {
    let o = Command::new("sqlite3")
        .args(["-list", "-noheader", db_path, query])
        .output()
        .expect("spawn sqlite3");
    assert!(
        o.status.success(),
        "sqlite3 failed ({query}): {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).trim_end().to_string()
}

fn temp_db(name: &str) -> PathBuf {
    let tmp =
        std::env::temp_dir().join(format!("graphlaw-ocel-court-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&tmp).expect("temp dir");
    tmp.join("trace.sqlite")
}

#[test]
fn ocel_export_sqlite_readback() {
    let db = temp_db("readback");
    let db_path = db.to_str().expect("utf8 path");

    let (ok, output) = run_bin(db_path);
    assert!(ok, "bin failed: {output}");

    // 1. Activities are exactly {admit, refuse, receipt}.
    let kinds = sqlite(db_path, "SELECT DISTINCT ocel_type FROM event ORDER BY 1");
    assert_eq!(kinds, "admit\nreceipt\nrefuse", "event activities");

    // 2. Object types are exactly {plan, subject, authority}.
    let obj_kinds = sqlite(db_path, "SELECT DISTINCT ocel_type FROM object ORDER BY 1");
    assert_eq!(obj_kinds, "authority\nplan\nsubject", "object types");

    // 3. Admit events carry the real derivation receipt facts.
    let derive_added = sqlite(
        db_path,
        "SELECT added FROM event_Admit WHERE step='derive:n3'",
    );
    assert!(
        derive_added.parse::<u64>().expect("added numeric") >= 20,
        "derive:n3 added={derive_added}, expected the ERRC surface (>=20)"
    );

    // 4. Refuse event exists with a typed code, linked to the refused plan.
    let refuse = sqlite(db_path, "SELECT code FROM event_Refuse");
    assert_eq!(refuse, "NOT_ADMITTED(1)", "refusal code");
    let refuse_plan = sqlite(
        db_path,
        "SELECT count(*) FROM event_object eo JOIN event e ON e.ocel_id=eo.ocel_event_id \
         WHERE e.ocel_type='refuse' AND eo.ocel_qualifier='from_plan'",
    );
    assert!(
        refuse_plan == "1",
        "refuse event must link its plan, got {refuse_plan}"
    );

    // 5. Receipt events witness real authorities.
    let auths = sqlite(
        db_path,
        "SELECT DISTINCT authority FROM event_Receipt ORDER BY 1",
    );
    assert_eq!(auths, "purrdf::n3\npurrdf::shapes", "receipt authorities");

    // 6. Every event relates at least one plan, subject and authority object.
    let orphans = sqlite(
        db_path,
        "SELECT count(*) FROM event e WHERE \
         (SELECT count(*) FROM event_object eo WHERE eo.ocel_event_id=e.ocel_id AND eo.ocel_object_id LIKE 'plan:%')=0 \
         OR (SELECT count(*) FROM event_object eo WHERE eo.ocel_event_id=e.ocel_id AND eo.ocel_object_id LIKE 'subject:%')=0 \
         OR (SELECT count(*) FROM event_object eo WHERE eo.ocel_event_id=e.ocel_id AND eo.ocel_object_id LIKE 'authority:%')=0",
    );
    assert_eq!(orphans, "0", "every event relates plan+subject+authority");
}

#[test]
fn ocel_export_pm4py_reads_the_sqlite() {
    let db = temp_db("pm4py");
    let db_path = db.to_str().expect("utf8 path");
    let (ok, output) = run_bin(db_path);
    assert!(ok, "bin failed: {output}");

    // External judge: pm4py must parse the emitted database as OCEL v2 and
    // see the same activity set and multi-object relations.
    let py = format!(
        r#"
import pm4py, sys
o = pm4py.read_ocel2_sqlite({db_path:?})
acts = sorted(o.events["ocel:activity"].unique().tolist())
objs = sorted(o.objects["ocel:type"].unique().tolist())
rels = o.relations[["ocel:activity","ocel:oid","ocel:qualifier"]]
assert acts == ["admit","receipt","refuse"], acts
assert objs == ["authority","plan","subject"], objs
refuse_oids = rels[rels["ocel:activity"]=="refuse"]["ocel:oid"].tolist()
assert any(x.startswith("plan:") for x in refuse_oids), refuse_oids
print("PM4PY-OK", len(o.events), "events", len(o.objects), "objects")
"#
    );
    let o = Command::new("python3")
        .args(["-c", &py])
        .output()
        .expect("spawn python3");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(
        o.status.success() && combined.contains("PM4PY-OK"),
        "pm4py readback failed: {combined}"
    );
    assert!(
        combined.contains("5 events"),
        "expected 5 events (admit+receipt x2, refuse), got: {combined}"
    );
}
