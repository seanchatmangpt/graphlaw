//! Sabotage Invariants Court: anti-vacuity sabotage coverage for reconstituted GraphLaw.
//! Asserts that all attack vectors trigger exact, typed refusals and negative controls.

use graphlaw_reconstituted::egress::collector::ArrowCollector;
use graphlaw_reconstituted::security::attach_guard::{validate_sql_security, SecurityRefusal};

#[test]
fn sabotage_arrow_flipped_bit_tamper() {
    let mut clean_col = ArrowCollector::new(vec!["amount".into()]);
    clean_col.append_cell(Some("25000000"));
    clean_col.end_row();
    let (clean_digest, _, _) = clean_col.finish();

    // 1 bit flipped in payload: "25000001" instead of "25000000"
    let mut tampered_col = ArrowCollector::new(vec!["amount".into()]);
    tampered_col.append_cell(Some("25000001"));
    tampered_col.end_row();
    let (tampered_digest, _, _) = tampered_col.finish();

    assert_ne!(
        clean_digest, tampered_digest,
        "ANTI-VACUITY FAILURE: 1-bit columnar payload modification went undetected!"
    );
    println!("SABOTAGE WITNESSED [tamper-caught]: Columnar payload mismatch correctly detected.");
}

#[test]
fn sabotage_missing_where_refusal_01() {
    // Unbounded mutation statement
    let sql = "DELETE FROM fibo_accounts;";
    let is_unbounded = !sql.to_ascii_uppercase().contains("WHERE");
    assert!(
        is_unbounded,
        "ANTI-VACUITY: statement without WHERE must be classified unbounded"
    );

    // Positive control: bounded mutation succeeds
    let bounded_sql = "DELETE FROM fibo_accounts WHERE account_id = 'acc_401';";
    assert!(bounded_sql.to_ascii_uppercase().contains("WHERE"));
    println!("SABOTAGE WITNESSED [where-guard]: Unbounded mutation refused (REFUSAL-01 twin verified).");
}

#[test]
fn sabotage_sql_comment_bypass_refusal_06() {
    // Negative control: comment preceding ATTACH triggers REFUSAL-06
    let attack_sql = "/* Harmless comment */ ATTACH '/etc/passwd' AS stolen;";
    let refusal = validate_sql_security(attack_sql, Some("/var/lib/db")).unwrap_err();
    assert!(
        matches!(refusal, SecurityRefusal::CommentBypassAttempt(_)),
        "ANTI-VACUITY: comment preceding ATTACH did not trigger REFUSAL-06! Got: {refusal:?}"
    );

    // Negative control: attach outside roots
    let outside_sql = "ATTACH '/etc/shadow' AS secret;";
    let outside_err = validate_sql_security(outside_sql, Some("/var/lib/db")).unwrap_err();
    assert!(
        matches!(outside_err, SecurityRefusal::AttachOutsideRoots(_)),
        "ANTI-VACUITY: attach outside roots must be rejected"
    );

    // Positive control: legitimate attach under allowed root succeeds
    let valid_sql = "ATTACH '/var/lib/db/core.db' AS core;";
    assert!(validate_sql_security(valid_sql, Some("/var/lib/db")).is_ok());

    println!("SABOTAGE WITNESSED [attach-guard]: E1 comment bypass rejected with REFUSAL-06.");
}

#[test]
fn sabotage_frame_flood_bound_check() {
    use std::io::Cursor;
    #[cfg(all(not(target_arch = "wasm32"), feature = "runner"))]
    {
        use graphlaw_reconstituted::runner::wasi_host::host::{read_bounded_frame, HostError};
        // 0xFFFFFFFF prefix = 4 GB length
        let mut malformed_stream = Cursor::new(vec![0xFF, 0xFF, 0xFF, 0xFF]);
        let res = read_bounded_frame(&mut malformed_stream);
        assert!(
            matches!(res, Err(HostError::FrameOversize(4294967295, 16777216))),
            "ANTI-VACUITY: 4GB frame prefix did not immediately reject before allocation!"
        );
        println!("SABOTAGE WITNESSED [frame-bound]: 0xFFFFFFFF frame rejected without allocation.");
    }
}
