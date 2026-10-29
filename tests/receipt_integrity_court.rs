//! receipt_integrity_court.rs — P17 G1: capture-law court over receipts.
//!
//! Law (capture): a receipt must embed REAL command output. A receipt that
//! claims `EXIT=0` while the cited log ends `EXIT=1` is a misclaim; a log with
//! no `EXIT=` tail proves nothing. This court makes receipts citing
//! `receipts/logs/*.log` mechanically auditable.
//!
//! Citation convention (started by this court's own receipt):
//!     receipts/logs/<name>.log EXIT=<n>
//! on a single line — the path token and the claimed exit on the same line.
//!
//! Legs:
//!   1. inline fixture: a temp receipt claiming EXIT=0 over a log ending
//!      EXIT=1 must be flagged (plus positive control and no-tail control);
//!   2. on-tree scan: EVERY receipts/*.md citing receipts/logs paths is
//!      audited — cited exit == log's final EXIT= line, the log must exist,
//!      and an exit-0 citation's log must lack `test result: FAILED` /
//!      line-initial `error:` tails (false-green capture). Receipts citing no
//!      logs are out of scope;
//!   3. count floor: >=1 receipt citing at least one log (the convention must
//!      exist, never be optional).
//!
//! Auto-discovered, std-only. Anchored at CARGO_MANIFEST_DIR. Logs under
//! receipts/logs/ are chain-attested automatically (receipt_chain_sync.sh
//! enumerates receipts/ recursively).

use std::fs;
use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const RECEIPTS_REL: &str = "receipts";
const LOG_PREFIX: &str = "receipts/logs/";
const PATH_CLASS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._/-";

#[derive(Debug)]
struct Citation {
    line: usize,
    log_rel: String,
    claimed_exit: Option<u32>,
}

#[derive(Debug)]
struct Violation {
    receipt: String,
    line: usize,
    msg: String,
}

fn is_path_byte(c: u8) -> bool {
    PATH_CLASS.contains(&c)
}

/// Every `receipts/logs/<name>.log EXIT=<n>` citation in a receipt text.
fn find_citations(receipt: &str, text: &str) -> Vec<Citation> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let mut from = 0;
        while let Some(pos) = line[from..].find(LOG_PREFIX) {
            let start = from + pos;
            let path_start = start + LOG_PREFIX.len();
            let mut end = path_start;
            while end < line.len() && is_path_byte(line.as_bytes()[end]) {
                end += 1;
            }
            let path = &line[path_start..end];
            if path.ends_with(".log") {
                // Convention: the claimed exit sits on the same line as the
                // citation — `receipts/logs/<name>.log EXIT=<n>` — so the
                // first `EXIT=` token on the line carries the claim.
                let claimed_exit = line.find("EXIT=").and_then(|e| {
                    line[e + 5..]
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u32>()
                        .ok()
                });
                out.push(Citation {
                    line: i + 1,
                    log_rel: format!("{LOG_PREFIX}{path}"),
                    claimed_exit,
                });
            }
            from = end.max(start + LOG_PREFIX.len());
            if from >= line.len() {
                break;
            }
        }
    }
    out
}

/// The final `EXIT=<n>` line of a log (the capture-law tail).
fn log_final_exit(log_text: &str) -> Option<u32> {
    let mut last = None;
    for line in log_text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("EXIT=") {
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                last = Some(n);
            }
        }
    }
    last
}

fn audit_receipt(receipt_rel: &str, text: &str, root: &Path) -> (usize, Vec<Violation>) {
    let mut violations = Vec::new();
    let citations = find_citations(receipt_rel, text);
    for c in &citations {
        let log_path = root.join(&c.log_rel);
        let log_text = match fs::read_to_string(&log_path) {
            Ok(t) => t,
            Err(_) => {
                violations.push(Violation {
                    receipt: receipt_rel.into(),
                    line: c.line,
                    msg: format!("cited log '{}' does not exist on disk", c.log_rel),
                });
                continue;
            }
        };
        let actual = match log_final_exit(&log_text) {
            Some(n) => n,
            None => {
                violations.push(Violation {
                    receipt: receipt_rel.into(),
                    line: c.line,
                    msg: format!(
                        "cited log '{}' has no EXIT=<n> tail — capture law: a log that proves nothing is not evidence",
                        c.log_rel
                    ),
                });
                continue;
            }
        };
        match c.claimed_exit {
            None => violations.push(Violation {
                receipt: receipt_rel.into(),
                line: c.line,
                msg: format!(
                    "citation of '{}' lacks the embedded claimed exit — cite as 'receipts/logs/<name>.log EXIT=<n>'",
                    c.log_rel
                ),
            }),
            Some(claimed) => {
                if claimed != actual {
                    violations.push(Violation {
                        receipt: receipt_rel.into(),
                        line: c.line,
                        msg: format!(
                            "RECEIPT-DRIFT: claims EXIT={claimed} but '{}' ends EXIT={actual}",
                            c.log_rel
                        ),
                    });
                }
                if claimed == 0 {
                    for (i, line) in log_text.lines().enumerate() {
                        let t = line.trim_start();
                        let bad_fail = t.contains("test result: FAILED");
                        let bad_err =
                            t.starts_with("error:") || t.starts_with("error[");
                        if bad_fail || bad_err {
                            violations.push(Violation {
                                receipt: receipt_rel.into(),
                                line: c.line,
                                msg: format!(
                                    "RECEIPT-DRIFT: log '{}' claimed EXIT=0 but line {} carries a failure tail: '{}'",
                                    c.log_rel,
                                    i + 1,
                                    t.chars().take(80).collect::<String>()
                                ),
                            });
                        }
                    }
                }
            }
        }
    }
    (citations.len(), violations)
}

fn audit_tree(root: &Path) -> (usize, usize, usize, Vec<Violation>) {
    let receipts_dir = root.join(RECEIPTS_REL);
    let mut receipts: Vec<_> = fs::read_dir(&receipts_dir)
        .expect("receipts/ dir readable")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
        .collect();
    receipts.sort();
    let mut files_scanned = 0;
    let mut files_citing = 0;
    let mut citations = 0;
    let mut violations = Vec::new();
    for p in receipts {
        let text = fs::read_to_string(&p).expect("receipt readable");
        files_scanned += 1;
        let rel = format!(
            "{RECEIPTS_REL}/{}",
            p.file_name().unwrap().to_string_lossy()
        );
        let (n, mut v) = audit_receipt(&rel, &text, root);
        if n > 0 {
            files_citing += 1;
            citations += n;
        }
        violations.append(&mut v);
    }
    (files_scanned, files_citing, citations, violations)
}

// ------------------------------------------------------------------- Leg 1

/// Inline fixture (temp root): a receipt claiming EXIT=0 over a log ending
/// EXIT=1 MUST be flagged. Real files in a real temp dir — no mocks.
#[test]
fn leg1_fixture_exit_mismatch_flags() {
    let dir = std::env::temp_dir().join(format!(
        "g1-fixture-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(dir.join("receipts/logs")).expect("temp receipts/logs");
    fs::write(
        dir.join("receipts/logs/fake-run.log"),
        "running 3 courts...\nEXIT=1\n",
    )
    .expect("write fixture log");
    let receipt_text =
        "court run over receipts/logs/fake-run.log EXIT=0 (all green)\n";

    let (_, violations) = audit_receipt("<fixture>", receipt_text, &dir);
    assert!(
        !violations.is_empty(),
        "VACUOUS COURT: EXIT=0 claim over an EXIT=1 log was NOT flagged"
    );
    let v = &violations[0];
    assert!(
        v.msg.contains("RECEIPT-DRIFT") && v.msg.contains("EXIT=0") && v.msg.contains("EXIT=1"),
        "flag must name the drift, got: {v:?}"
    );
    eprintln!("G1 leg1: flagged {}", v.msg);
    fs::remove_dir_all(&dir).ok();
}

/// Positive control: the matching claim (EXIT=1 over an EXIT=1 log) passes.
#[test]
fn leg1b_positive_control_matching_claim_passes() {
    let dir = std::env::temp_dir().join(format!(
        "g1-fixture-b-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(dir.join("receipts/logs")).expect("temp receipts/logs");
    fs::write(
        dir.join("receipts/logs/ok.log"),
        "test result: ok. 3 passed; 0 failed\nEXIT=1\n",
    )
    .expect("write fixture log");
    let receipt_text = "court run over receipts/logs/ok.log EXIT=1\n";
    let (_, violations) = audit_receipt("<fixture-b>", receipt_text, &dir);
    assert!(
        violations.is_empty(),
        "positive control must pass, got: {violations:?}"
    );
    eprintln!("G1 leg1b: matching claim over nonzero log passes");
    fs::remove_dir_all(&dir).ok();
}

/// No-tail control: a log without any EXIT= line proves nothing and must flag.
#[test]
fn leg1c_log_without_exit_tail_flags() {
    let dir = std::env::temp_dir().join(format!(
        "g1-fixture-c-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(dir.join("receipts/logs")).expect("temp receipts/logs");
    fs::write(dir.join("receipts/logs/tail-less.log"), "all output, no tail\n")
        .expect("write fixture log");
    let receipt_text = "court run over receipts/logs/tail-less.log EXIT=0\n";
    let (_, violations) = audit_receipt("<fixture-c>", receipt_text, &dir);
    assert!(
        violations
            .iter()
            .any(|v| v.msg.contains("no EXIT=<n> tail")),
        "no-tail log must be flagged, got: {violations:?}"
    );
    eprintln!("G1 leg1c: log without EXIT tail flagged");
    fs::remove_dir_all(&dir).ok();
}

// ------------------------------------------------------------------- Leg 2

/// On-tree scan: every receipts/*.md citing logs must agree with the real
/// chain-attested log tails. Receipts citing no logs are out of scope.
#[test]
fn leg2_tree_receipts_match_cited_logs() {
    let (scanned, citing, citations, violations) = audit_tree(Path::new(ROOT));
    for v in &violations {
        eprintln!("G1 VIOLATION {}:{}: {}", v.receipt, v.line, v.msg);
    }
    eprintln!(
        "G1 leg2: scanned {scanned} receipts/*.md; {citing} citing logs; {citations} citation(s) audited"
    );
    assert!(
        violations.is_empty(),
        "{} receipt-drift violation(s) on the tree",
        violations.len()
    );
}

// ------------------------------------------------------------------- Leg 3

/// Count floor: at least one receipt must cite a log (the convention exists;
/// this court's own receipt starts it).
#[test]
fn leg3_at_least_one_citing_receipt() {
    let (_, citing, citations, _) = audit_tree(Path::new(ROOT));
    eprintln!("G1 leg3: {citing} receipt(s) citing {citations} log citation(s)");
    assert!(
        citing >= 1,
        "count floor: >=1 receipts/*.md must cite a receipts/logs path with \
         EXIT=<n> (capture convention is not optional)"
    );
}
