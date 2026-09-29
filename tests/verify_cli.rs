//! The real `graphlaw-verify` binary against real plans, real receipt stores and
//! real Ed25519 keys. One command, no network: ADMITTED (0) or REFUSED (1).
#![cfg(feature = "abi")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use graphlaw::attest::{self, SigningKey};
use graphlaw::dialect::Dialect;
use graphlaw::law::LawState;
use graphlaw::plan::{Action, Plan};
use graphlaw::receipt_store::ReceiptStore;

const SUBJECT: &str = "0123456789abcdef0123456789abcdef01234567";

fn t(o: &str) -> String {
    format!("<urn:p:robot> <urn:p:at> <urn:p:{o}> .\n")
}

fn mv(name: &str, from: &str, to: &str) -> Action {
    Action {
        name: name.into(),
        pre: t(from),
        add: t(to),
        del: t(from),
        ..Default::default()
    }
}

fn plan() -> Plan {
    Plan {
        actions: vec![
            mv("a-b", "a", "b"),
            mv("b-c", "b", "c"),
            mv("c-d", "c", "d"),
        ],
        goal: t("d"),
        ..Default::default()
    }
}

struct Fixture {
    dir: PathBuf,
    trusted_hex: String,
}

impl Fixture {
    fn start(&self) -> PathBuf {
        self.dir.join("start.nt")
    }
    fn plan(&self) -> PathBuf {
        self.dir.join("plan.json")
    }
    fn store(&self) -> PathBuf {
        self.dir.join("receipts")
    }
    fn subject_dir(&self) -> PathBuf {
        self.store().join(SUBJECT)
    }
    fn run(&self, key_hex: Option<&str>) -> Output {
        let mut c = Command::new(env!("CARGO_BIN_EXE_graphlaw-verify"));
        c.arg("--start")
            .arg(self.start())
            .arg("--plan")
            .arg(self.plan())
            .arg("--receipts")
            .arg(self.store());
        if let Some(k) = key_hex {
            c.arg("--trusted-key").arg(k);
        }
        c.output().expect("binary runs")
    }
    fn verify(&self) -> Output {
        self.run(Some(&self.trusted_hex.clone()))
    }
}

/// Build a real admitted plan and a fully signed receipt store under a fresh dir.
fn fixture(name: &str, signer: &SigningKey, trusted: &SigningKey) -> Fixture {
    let dir = std::env::temp_dir().join(format!("graphlaw-verify-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let start = LawState::parse(t("a").as_bytes(), Dialect::NTriples, None).unwrap();
    let plan = plan();
    let admitted = plan.admit(&start).unwrap();
    fs::write(dir.join("start.nt"), t("a")).unwrap();
    fs::write(dir.join("plan.json"), plan.canonical_json()).unwrap();
    let store = ReceiptStore::open(dir.join("receipts")).unwrap();
    for r in &admitted.receipts {
        store
            .put_signed(r, SUBJECT, &attest::sign_receipt(signer, r))
            .unwrap();
    }
    Fixture {
        dir,
        trusted_hex: trusted.verifying_key().to_hex(),
    }
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

fn refused(o: &Output, code: &str) {
    assert_eq!(
        o.status.code(),
        Some(1),
        "stdout={} stderr={}",
        stdout(o),
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(
        stdout(o).starts_with(&format!("REFUSED: {code}:")),
        "expected {code}, got `{}`",
        stdout(o)
    );
}

fn first_with_suffix(dir: &Path, suffix: &str) -> PathBuf {
    let mut v: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with(suffix))
        .collect();
    v.sort();
    v.remove(0)
}

#[test]
fn admitted_plan_with_trusted_signed_receipts_exits_zero() {
    let k = SigningKey::from_seed([1; 32]);
    let f = fixture("ok", &k, &k);
    let o = f.verify();
    assert_eq!(stdout(&o), "ADMITTED");
    assert_eq!(o.status.code(), Some(0));
    let _ = fs::remove_dir_all(&f.dir);
}

#[test]
fn tampered_receipt_byte_is_refused() {
    let k = SigningKey::from_seed([1; 32]);
    let f = fixture("tamper", &k, &k);
    let file = first_with_suffix(&f.subject_dir(), ".json");
    let mut bytes = fs::read(&file).unwrap();
    let pos = bytes.windows(8).position(|w| w == b"\"added\":").unwrap() + 8;
    bytes[pos] = if bytes[pos] == b'9' {
        b'8'
    } else {
        bytes[pos] + 1
    };
    fs::write(&file, &bytes).unwrap();
    refused(&f.verify(), "ReceiptTampered");
    let _ = fs::remove_dir_all(&f.dir);
}

#[test]
fn receipts_signed_by_an_untrusted_key_are_refused() {
    let good = SigningKey::from_seed([1; 32]);
    let mallory = SigningKey::from_seed([9; 32]);
    // whole store signed by mallory, verifier trusts only `good`
    let f = fixture("untrusted", &mallory, &good);
    refused(&f.verify(), "UntrustedKey");
    let _ = fs::remove_dir_all(&f.dir);

    // one attestation swapped for mallory's over the same receipt bytes
    let f = fixture("swap", &good, &good);
    let sig = first_with_suffix(&f.subject_dir(), ".sig");
    let receipts = ReceiptStore::open(f.store())
        .unwrap()
        .verify(SUBJECT)
        .unwrap();
    let digest = sig.file_stem().unwrap().to_string_lossy().into_owned();
    let victim = receipts
        .iter()
        .find(|r| graphlaw::receipt_store::receipt_digest(r) == digest)
        .unwrap();
    fs::write(&sig, attest::sign_receipt(&mallory, victim).to_json()).unwrap();
    refused(&f.verify(), "UntrustedKey");
    let _ = fs::remove_dir_all(&f.dir);
}

#[test]
fn edited_plan_is_refused() {
    let k = SigningKey::from_seed([1; 32]);

    // still admissible but a different plan: the receipts' plan digest no longer matches
    let f = fixture("plan-rename", &k, &k);
    let mut p = plan();
    p.actions[1].name = "b-c-renamed".into();
    fs::write(f.plan(), p.canonical_json()).unwrap();
    refused(&f.verify(), "ReceiptMismatch");
    let _ = fs::remove_dir_all(&f.dir);

    // not admissible: precondition can no longer be met
    let f = fixture("plan-break", &k, &k);
    let mut p = plan();
    p.actions[0].pre = t("zzz");
    fs::write(f.plan(), p.canonical_json()).unwrap();
    refused(&f.verify(), "PlanRefused");
    let _ = fs::remove_dir_all(&f.dir);
}

#[test]
fn missing_attestation_or_receipt_is_refused() {
    let k = SigningKey::from_seed([1; 32]);
    let f = fixture("unsigned", &k, &k);
    fs::remove_file(first_with_suffix(&f.subject_dir(), ".sig")).unwrap();
    refused(&f.verify(), "Unattested");
    let _ = fs::remove_dir_all(&f.dir);

    let f = fixture("gap", &k, &k);
    fs::remove_file(first_with_suffix(&f.subject_dir(), ".json")).unwrap();
    // a middle/first receipt gone: chain no longer replays
    let o = f.verify();
    assert_eq!(o.status.code(), Some(1), "{}", stdout(&o));
    assert!(stdout(&o).starts_with("REFUSED:"));
    let _ = fs::remove_dir_all(&f.dir);
}

#[test]
fn usage_and_io_errors_exit_two() {
    let k = SigningKey::from_seed([1; 32]);
    let f = fixture("usage", &k, &k);
    // no trusted key: attestations could not be verified, so no verdict is given
    assert_eq!(f.run(None).status.code(), Some(2));
    // unreadable start file
    fs::remove_file(f.start()).unwrap();
    assert_eq!(f.verify().status.code(), Some(2));
    let _ = fs::remove_dir_all(&f.dir);
}
