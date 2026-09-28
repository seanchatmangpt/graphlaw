//! Out-of-subject receipt store (composition C21).
//!
//! Receipts for a subject (e.g. a commit SHA) live in a directory *outside* the
//! subject's tree: `<dir>/<subject>/<sha256-of-file>.json`. A commit cannot
//! contain its own hash, so receipts keyed by subject are the only way exact-head
//! standing becomes reachable. No dependency on git or the subject tree.
//!
//! Files are canonical JSON, content-addressed by the SHA-256 of their bytes;
//! [`ReceiptStore::verify`] refuses a file whose bytes no longer hash to its
//! name, that is not in canonical form, or a set of receipts that does not form
//! one linear `parent -> child` chain.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::law::Receipt;
use crate::{BACKEND_AUTHORITIES, BackendAuthority};

/// Typed store refusal.
#[derive(Debug)]
pub enum StoreError {
    Io(String),
    /// Subject key is empty or contains path separators / dots-only segments.
    InvalidSubject(String),
    /// A stored file is not a canonical receipt document.
    Malformed {
        file: String,
    },
    /// A stored file's bytes do not hash to its content address.
    DigestMismatch {
        file: String,
        expected: String,
        actual: String,
    },
    /// The stored receipts do not form one linear parent -> child chain.
    BrokenChain {
        detail: String,
    },
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "receipt store io: {e}"),
            StoreError::InvalidSubject(s) => write!(f, "invalid subject key `{s}`"),
            StoreError::Malformed { file } => write!(f, "malformed receipt file `{file}`"),
            StoreError::DigestMismatch {
                file,
                expected,
                actual,
            } => write!(
                f,
                "receipt file `{file}` tampered: address {expected}, content hashes to {actual}"
            ),
            StoreError::BrokenChain { detail } => write!(f, "receipt chain broken: {detail}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e.to_string())
    }
}

/// A directory of receipts keyed by subject.
#[derive(Debug, Clone)]
pub struct ReceiptStore {
    dir: PathBuf,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn digest_of(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn esc(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Canonical serialization: fixed key order, no whitespace.
fn encode(r: &Receipt) -> String {
    let lease = r.lease_id.as_deref().map_or("null".to_string(), esc);
    format!(
        "{{\"parent\":{},\"child\":{},\"step\":{},\"authority\":{},\"revision\":{},\"added\":{},\"lease_id\":{}}}",
        esc(&r.parent),
        esc(&r.child),
        esc(r.step),
        esc(r.authority.authority),
        esc(r.authority.revision),
        r.added,
        lease
    )
}

/// Receipt digest: SHA-256 of the canonical serialization.
pub fn receipt_digest(r: &Receipt) -> String {
    digest_of(encode(r).as_bytes())
}

struct Cur<'a> {
    b: &'a [u8],
    i: usize,
}

impl Cur<'_> {
    fn eat(&mut self, lit: &str) -> Option<()> {
        if self.b[self.i..].starts_with(lit.as_bytes()) {
            self.i += lit.len();
            Some(())
        } else {
            None
        }
    }

    fn string(&mut self) -> Option<String> {
        self.eat("\"")?;
        let mut out = String::new();
        loop {
            let rest = std::str::from_utf8(&self.b[self.i..]).ok()?;
            let c = rest.chars().next()?;
            self.i += c.len_utf8();
            match c {
                '"' => return Some(out),
                '\\' => {
                    let e = *self.b.get(self.i)?;
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let h = std::str::from_utf8(self.b.get(self.i..self.i + 4)?).ok()?;
                            self.i += 4;
                            out.push(char::from_u32(u32::from_str_radix(h, 16).ok()?)?);
                        }
                        _ => return None,
                    }
                }
                c => out.push(c),
            }
        }
    }

    fn key(&mut self, name: &str) -> Option<()> {
        self.eat(&format!("\"{name}\":"))
    }
}

fn authority_of(name: &str, revision: &str) -> Option<BackendAuthority> {
    BACKEND_AUTHORITIES
        .iter()
        .find(|a| a.authority == name && a.revision == revision)
        .copied()
}

fn step_name(name: &str) -> Option<&'static str> {
    [
        "admit:shacl",
        "derive:n3",
        "derive:hooks",
        "admit:plan",
        "admit:require-receipt",
        "derive:rdfs",
        "derive:owl-rl",
        "plan-action",
    ]
    .into_iter()
    .find(|s| *s == name)
}

fn decode(bytes: &[u8]) -> Option<Receipt> {
    let mut c = Cur { b: bytes, i: 0 };
    c.eat("{")?;
    c.key("parent")?;
    let parent = c.string()?;
    c.eat(",")?;
    c.key("child")?;
    let child = c.string()?;
    c.eat(",")?;
    c.key("step")?;
    let step = step_name(&c.string()?)?;
    c.eat(",")?;
    c.key("authority")?;
    let auth = c.string()?;
    c.eat(",")?;
    c.key("revision")?;
    let rev = c.string()?;
    c.eat(",")?;
    c.key("added")?;
    let start = c.i;
    while c.b.get(c.i).is_some_and(u8::is_ascii_digit) {
        c.i += 1;
    }
    let added: usize = std::str::from_utf8(&c.b[start..c.i]).ok()?.parse().ok()?;
    c.eat(",")?;
    c.key("lease_id")?;
    let lease_id = if c.eat("null").is_some() {
        None
    } else {
        Some(c.string()?)
    };
    c.eat("}")?;
    if c.i != c.b.len() {
        return None;
    }
    Some(Receipt {
        parent,
        child,
        step,
        authority: authority_of(&auth, &rev)?,
        added,
        lease_id,
    })
}

fn check_subject(s: &str) -> Result<(), StoreError> {
    let ok = !s.is_empty()
        && s != "."
        && s != ".."
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':' | '.'));
    if ok {
        Ok(())
    } else {
        Err(StoreError::InvalidSubject(s.to_string()))
    }
}

impl ReceiptStore {
    /// Open (creating if needed) a store rooted at `dir`.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, StoreError> {
        fs::create_dir_all(dir.as_ref())?;
        Ok(Self {
            dir: dir.as_ref().to_path_buf(),
        })
    }

    fn subject_dir(&self, subject: &str) -> Result<PathBuf, StoreError> {
        check_subject(subject)?;
        Ok(self.dir.join(subject))
    }

    /// Store `receipt` under `subject`; returns its digest. Atomic (temp + rename), idempotent.
    pub fn put(&self, receipt: &Receipt, subject_sha: &str) -> Result<String, StoreError> {
        let sd = self.subject_dir(subject_sha)?;
        fs::create_dir_all(&sd)?;
        let body = encode(receipt);
        let digest = digest_of(body.as_bytes());
        let dest = sd.join(format!("{digest}.json"));
        let tmp = sd.join(format!(".tmp-{digest}-{}", std::process::id()));
        fs::write(&tmp, body.as_bytes())?;
        fs::rename(&tmp, &dest)?;
        Ok(digest)
    }

    /// Digests of receipts stored for `subject` (sorted); empty when none.
    pub fn list(&self, subject_sha: &str) -> Result<Vec<String>, StoreError> {
        let sd = self.subject_dir(subject_sha)?;
        if !sd.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for e in fs::read_dir(&sd)? {
            let name = e?.file_name().to_string_lossy().into_owned();
            if let Some(d) = name.strip_suffix(".json") {
                out.push(d.to_string());
            }
        }
        out.sort();
        Ok(out)
    }

    /// Re-read every receipt for `subject`, recompute digests, and check they form one
    /// linear `parent -> child` chain. Returns the receipts in chain order.
    pub fn verify(&self, subject_sha: &str) -> Result<Vec<Receipt>, StoreError> {
        let sd = self.subject_dir(subject_sha)?;
        let mut rs = Vec::new();
        for d in self.list(subject_sha)? {
            let file = format!("{d}.json");
            let bytes = fs::read(sd.join(&file))?;
            let actual = digest_of(&bytes);
            if actual != d {
                return Err(StoreError::DigestMismatch {
                    file,
                    expected: d,
                    actual,
                });
            }
            let r = decode(&bytes).ok_or_else(|| StoreError::Malformed { file: file.clone() })?;
            if encode(&r).as_bytes() != bytes.as_slice() {
                return Err(StoreError::Malformed { file });
            }
            rs.push(r);
        }
        chain(rs)
    }
}

fn chain(rs: Vec<Receipt>) -> Result<Vec<Receipt>, StoreError> {
    let broken = |d: &str| StoreError::BrokenChain {
        detail: d.to_string(),
    };
    if rs.is_empty() {
        return Ok(rs);
    }
    let mut by_parent: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, r) in rs.iter().enumerate() {
        by_parent.entry(&r.parent).or_default().push(i);
    }
    if by_parent.values().any(|v| v.len() > 1) {
        return Err(broken("two receipts share a parent (fork)"));
    }
    let children: std::collections::BTreeSet<&str> = rs.iter().map(|r| r.child.as_str()).collect();
    let roots: Vec<usize> = (0..rs.len())
        .filter(|&i| !children.contains(rs[i].parent.as_str()))
        .collect();
    let [root] = roots.as_slice() else {
        return Err(broken("no unique root receipt"));
    };
    let mut order = vec![*root];
    while let Some(next) = by_parent
        .get(rs[*order.last().expect("non-empty")].child.as_str())
        .map(|v| v[0])
    {
        if order.contains(&next) {
            return Err(broken("cycle"));
        }
        order.push(next);
    }
    if order.len() != rs.len() {
        return Err(broken("receipts do not form one linear chain"));
    }
    Ok(order.into_iter().map(|i| rs[i].clone()).collect())
}
