//! Fixture producer for the external pytest black-box court
//! (`tests_py/test_blackbox_ops.py`). Not a test; no assertions.
//!
//! Writes a real admitted-plan receipt store exactly like
//! `tests/verify_cli.rs::fixture` — real `Plan::admit`, real `ReceiptStore`
//! with Ed25519-signed receipts — into a directory pytest chooses, and prints
//! the trusted public key hex as the last stdout line.
//!
//! Run: cargo run --example blackbox_fixture -- <outdir>
//! Exit 0 on success, 1 on any fixture fault (never a partial fixture with
//! exit 0).

use std::path::PathBuf;
use std::process::ExitCode;

use graphlaw::attest::{self, SigningKey};
use graphlaw::dialect::Dialect;
use graphlaw::law::LawState;
use graphlaw::plan::{Action, Plan};
use graphlaw::receipt_store::ReceiptStore;

const SUBJECT: &str = "0123456789abcdef0123456789abcdef01234567";

fn t(o: &str) -> String {
    format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n")
}

fn plan() -> Plan {
    Plan {
        actions: vec![
            Action {
                name: "a-b".into(),
                pre: t("a"),
                add: t("b"),
                del: t("a"),
                ..Default::default()
            },
            Action {
                name: "b-c".into(),
                pre: t("b"),
                add: t("c"),
                del: t("b"),
                ..Default::default()
            },
            Action {
                name: "c-d".into(),
                pre: t("c"),
                add: t("d"),
                del: t("c"),
                ..Default::default()
            },
        ],
        goal: t("d"),
        ..Default::default()
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(dir) = args.next().map(PathBuf::from) else {
        eprintln!("usage: blackbox_fixture <outdir>");
        return ExitCode::from(2);
    };
    if let Err(e) = run(&dir) {
        let _ = std::fs::remove_dir_all(&dir);
        eprintln!("blackbox_fixture: {e}");
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn run(dir: &PathBuf) -> Result<(), String> {
    let key = SigningKey::from_seed([1; 32]);
    let start =
        LawState::parse(t("a").as_bytes(), Dialect::NTriples, None).map_err(|e| e.to_string())?;
    let plan = plan();
    let admitted = plan.admit(&start).map_err(|e| e.to_string())?;

    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("start.nt"), t("a")).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("plan.json"), plan.canonical_json()).map_err(|e| e.to_string())?;
    let store = ReceiptStore::open(dir.join("receipts")).map_err(|e| e.to_string())?;
    for r in &admitted.receipts {
        store
            .put_signed(r, SUBJECT, &attest::sign_receipt(&key, r))
            .map_err(|e| e.to_string())?;
    }
    println!("{}", key.verifying_key().to_hex());
    Ok(())
}
