//! Court: duckdb op policy seams (wasm ATTACH refusal, memory split, path
//! allowlist for views/COPY, incremental page-byte cap).
//!
//! The `cfg(target_arch = "wasm32")` branches cannot execute on a native
//! host; they are covered by a source-law scan plus native tests of the pure
//! allowlist the wasm/native split shares.

use graphlaw::duckdb_op::{
    MAX_SESSIONS, conn_memory_limit_mib, enforce_copy_policy_with, enforce_path_policy_with,
    path_under_roots,
};

fn root() -> String {
    let d = std::env::temp_dir().join(format!("graphlaw-wasm-policy-{}", std::process::id()));
    std::fs::create_dir_all(&d).expect("root dir");
    std::fs::canonicalize(&d).expect("canon").to_string_lossy().into_owned()
}

#[test]
fn path_inside_root_admits_outside_refuses() {
    let r = root();
    let f = format!("{r}/a.csv");
    std::fs::write(&f, "x\n1\n").expect("write");
    assert!(path_under_roots(&f, &r, false));
    assert!(path_under_roots(&format!("{r}/*.csv"), &r, false));
    assert!(!path_under_roots("/etc/passwd", &r, false));
    assert!(!path_under_roots(&format!("{r}/../etc/passwd"), &r, false));
    assert!(!path_under_roots("/etc/*.conf", &r, false));
    assert!(!path_under_roots("relative.csv", &r, false));
    assert!(!path_under_roots(&f, "", false));
}

#[test]
fn env_unset_is_open_env_empty_is_closed_for_mounts() {
    assert!(enforce_path_policy_with("/etc/passwd", None, false).is_ok());
    let err = enforce_path_policy_with("/etc/passwd", Some(""), false).expect_err("closed");
    assert!(format!("{err:?}").contains("GRAPHLAW_ATTACH_ROOTS"));
}

#[test]
fn copy_to_and_from_go_through_the_allowlist() {
    let r = root();
    let ok_to = format!("COPY (SELECT 'x' AS a) TO '{r}/out.csv' (FORMAT CSV)");
    assert!(enforce_copy_policy_with(&ok_to, Some(&r)).is_ok(), "missing leaf under root");
    let bad_to = "COPY (SELECT 1) TO '/tmp/escape-graphlaw.csv'";
    assert!(enforce_copy_policy_with(bad_to, Some(&r)).is_err());
    let bad_from = "COPY t FROM '/etc/passwd'";
    assert!(enforce_copy_policy_with(bad_from, Some(&r)).is_err());
    // decoy literal inside the subquery does not satisfy the check
    let decoy = format!("COPY (SELECT '{r}/x') TO '/tmp/escape-graphlaw.csv'");
    assert!(enforce_copy_policy_with(&decoy, Some(&r)).is_err());
    // non-COPY statements and unset env are untouched
    assert!(enforce_copy_policy_with("SELECT 1", Some(&r)).is_ok());
    assert!(enforce_copy_policy_with(bad_to, None).is_ok());
}

#[test]
fn wasm_memory_is_divided_and_sessions_lowered() {
    assert_eq!(conn_memory_limit_mib(false), 32);
    let w = conn_memory_limit_mib(true);
    assert!(w * (2 + 1) <= 32, "wasm per-connection limit sums within 32 MiB");
    #[cfg(not(target_arch = "wasm32"))]
    assert_eq!(MAX_SESSIONS, 16);
    #[cfg(target_arch = "wasm32")]
    assert_eq!(MAX_SESSIONS, 2);
}

#[test]
fn source_law_wasm_branches_present() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/duckdb_op.rs"))
        .expect("read duckdb_op.rs");
    let wasm_attach = src
        .split("fn enforce_attach_policy(sql: &str)")
        .nth(1)
        .and_then(|t| t.split("#[cfg(not(target_arch = \"wasm32\"))]").next())
        .expect("enforce_attach_policy body");
    assert!(wasm_attach.contains("#[cfg(target_arch = \"wasm32\")]"));
    assert!(wasm_attach.contains("attach_refused_on_wasm(sql)"));
    assert!(!wasm_attach.contains("GRAPHLAW_ATTACH_ROOTS\")"), "wasm branch must not read env");
    assert!(!wasm_attach.contains("canonicalize"));
    assert!(src.contains("pub const MAX_SESSIONS: usize = 2;"));
    assert!(src.contains("conn_memory_limit_mib(cfg!(target_arch = \"wasm32\"))"));
    assert!(src.contains("enforce_path_policy(glob, false)?;"), "mount_view must be gated");
    assert!(src.contains("let projected = nt.len() + line.len() + 1;"), "incremental page cap");
}
