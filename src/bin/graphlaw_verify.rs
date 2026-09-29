//! `graphlaw-verify`: offline third-party check of an admitted plan.
//!
//! ```text
//! graphlaw-verify --start <state.nq|.nt|.ttl|.trig> --plan <plan.json> --receipts <dir>
//!                 [--subject <key>] --trusted-key <hex> [--trusted-key <hex>...]
//! ```
//!
//! Replays `Plan::admit` from the start state, checks the receipt store equals
//! the replayed chain (parent/child ids, step, authority, plan digest), and
//! verifies every receipt's Ed25519 attestation against the trusted keys. No
//! network. Prints `ADMITTED` or `REFUSED: <code>: <detail>`.
//! Exit codes: 0 admitted, 1 refused, 2 usage or I/O error.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use graphlaw::abi::plan_from_json;
use graphlaw::attest::{AttestError, TrustedKeys};
use graphlaw::dialect::Dialect;
use graphlaw::law::LawState;
use graphlaw::receipt_store::{ReceiptStore, StoreError};

struct Args {
    start: PathBuf,
    plan: PathBuf,
    receipts: PathBuf,
    subject: Option<String>,
    keys: Vec<String>,
}

fn usage(msg: &str) -> ExitCode {
    eprintln!("graphlaw-verify: {msg}");
    eprintln!(
        "usage: graphlaw-verify --start <state.nq> --plan <plan.json> --receipts <dir> \
         [--subject <key>] --trusted-key <hex> [--trusted-key <hex>...]"
    );
    ExitCode::from(2)
}

fn parse_args() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let (mut start, mut plan, mut receipts, mut subject) = (None, None, None, None);
    let mut keys = Vec::new();
    while let Some(flag) = it.next() {
        let mut val = || it.next().ok_or_else(|| format!("`{flag}` needs a value"));
        match flag.as_str() {
            "--start" => start = Some(PathBuf::from(val()?)),
            "--plan" => plan = Some(PathBuf::from(val()?)),
            "--receipts" => receipts = Some(PathBuf::from(val()?)),
            "--subject" => subject = Some(val()?),
            "--trusted-key" => keys.push(val()?),
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    if keys.is_empty() {
        return Err(
            "at least one --trusted-key is required: without one no attestation can be verified"
                .into(),
        );
    }
    Ok(Args {
        start: start.ok_or("missing --start")?,
        plan: plan.ok_or("missing --plan")?,
        receipts: receipts.ok_or("missing --receipts")?,
        subject,
        keys,
    })
}

fn dialect_of(p: &Path) -> Dialect {
    match p.extension().and_then(|e| e.to_str()) {
        Some("nt") => Dialect::NTriples,
        Some("ttl") => Dialect::Turtle,
        Some("trig") => Dialect::TriG,
        _ => Dialect::NQuads,
    }
}

/// The subject key: `--subject`, else the only subdirectory of the store root.
fn subject_key(dir: &Path, given: Option<String>) -> Result<String, String> {
    if let Some(s) = given {
        return Ok(s);
    }
    let mut names = Vec::new();
    for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let e = e.map_err(|e| e.to_string())?;
        if e.path().is_dir() {
            names.push(e.file_name().to_string_lossy().into_owned());
        }
    }
    match names.as_slice() {
        [one] => Ok(one.clone()),
        _ => Err(format!(
            "{} holds {} subject directories; pass --subject",
            dir.display(),
            names.len()
        )),
    }
}

fn refused(code: &str, detail: impl std::fmt::Display) -> ExitCode {
    println!("REFUSED: {code}: {detail}");
    ExitCode::from(1)
}

fn read(p: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))
}

fn run(a: Args) -> Result<ExitCode, String> {
    let trusted = TrustedKeys::from_hex(&a.keys).map_err(|e| format!("--trusted-key: {e}"))?;
    let start = LawState::parse(&read(&a.start)?, dialect_of(&a.start), None)
        .map_err(|e| format!("--start: {e}"))?;
    let plan_json: serde_json::Value =
        serde_json::from_slice(&read(&a.plan)?).map_err(|e| format!("--plan: {e}"))?;
    let plan = plan_from_json(&plan_json).map_err(|e| format!("--plan: {e}"))?;
    let subject = subject_key(&a.receipts, a.subject)?;
    let store = ReceiptStore::open(&a.receipts).map_err(|e| e.to_string())?;

    let replayed = match plan.admit(&start) {
        Ok(admitted) => admitted,
        Err(e) => return Ok(refused("PlanRefused", e)),
    };
    let stored = match store.verify_attested(&subject, &trusted) {
        Ok(rs) => rs,
        Err(e) => {
            let code = match &e {
                StoreError::Io(_) | StoreError::InvalidSubject(_) => {
                    return Err(e.to_string());
                }
                StoreError::Malformed { .. } | StoreError::DigestMismatch { .. } => {
                    "ReceiptTampered"
                }
                StoreError::BrokenChain { .. } => "ChainBroken",
                StoreError::Unattested { .. } => "Unattested",
                StoreError::Attestation { error, .. } => match error {
                    AttestError::BadSignature => "BadSignature",
                    AttestError::UnknownKey => "UntrustedKey",
                    AttestError::Malformed(_) => "MalformedAttestation",
                    // Non-exhaustive: an attestation error added later still refuses.
                    _ => "BadSignature",
                },
                // Non-exhaustive: a store error added later still refuses.
                _ => "ReceiptTampered",
            };
            return Ok(refused(code, e));
        }
    };
    if stored.len() != replayed.receipts.len() {
        return Ok(refused(
            "ReceiptMismatch",
            format!(
                "store has {} receipts, replay produced {}",
                stored.len(),
                replayed.receipts.len()
            ),
        ));
    }
    for (i, (s, r)) in stored.iter().zip(&replayed.receipts).enumerate() {
        let same = s.parent == r.parent
            && s.child == r.child
            && s.step == r.step
            && s.authority == r.authority
            && s.added == r.added
            && s.plan_sha256 == r.plan_sha256;
        if !same {
            return Ok(refused(
                "ReceiptMismatch",
                format!("receipt {i} differs from the replayed chain"),
            ));
        }
    }
    println!("ADMITTED");
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match parse_args() {
        Err(m) => usage(&m),
        Ok(a) => run(a).unwrap_or_else(|m| usage(&m)),
    }
}
