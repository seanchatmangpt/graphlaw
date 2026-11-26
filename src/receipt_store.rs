//! Out-of-subject receipt store (composition C21).
//!
//! Receipts for a subject (e.g. a commit SHA) live in a directory *outside* the
//! subject's tree: `<dir>/<subject>/<blake3-of-file>.json`. A commit cannot
//! contain its own hash, so receipts keyed by subject are the only way exact-head
//! standing becomes reachable. No dependency on git or the subject tree.
//!
//! Files are canonical JSON, content-addressed by the BLAKE3 of their bytes;
//! [`ReceiptStore::verify`] refuses a file whose bytes no longer hash to its
//! name, that is not in canonical form, or a set of receipts that does not form
//! one linear `parent -> child` chain.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::attest::{AttestError, Attestation, TrustedKeys, verify_receipt};
use crate::law::Receipt;
use crate::{BACKEND_AUTHORITIES, BackendAuthority};

/// Typed store refusal.
///
/// This enum is `#[non_exhaustive]`: variants may be added in a minor release, so
/// downstream `match` expressions need a wildcard arm.
///
/// ```
/// use graphlaw::receipt_store::StoreError;
///
/// let e = StoreError::Io("disk full".into());
/// let transient = match e {
///     StoreError::Io(_) => true,
///     _ => false, // required: new variants may be added
/// };
/// assert!(transient);
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub enum StoreError {
    /// An I/O error, rendered as text.
    Io(String),
    /// Subject key is empty or contains path separators / dots-only segments.
    InvalidSubject(String),
    /// A stored file is not a canonical receipt document.
    Malformed {
        /// File the error concerns.
        file: String,
    },
    /// A stored file's bytes do not hash to its content address.
    DigestMismatch {
        /// File whose bytes are wrong.
        file: String,
        /// Digest the file name promises.
        expected: String,
        /// Digest the bytes actually have.
        actual: String,
    },
    /// A receipt has no `<digest>.sig` attestation file.
    Unattested {
        /// File without attestation.
        file: String,
    },
    /// A receipt's attestation is malformed, untrusted, or does not verify.
    Attestation {
        /// File whose attestation was refused.
        file: String,
        /// Why the attestation was refused.
        error: AttestError,
    },
    /// A different attestation already exists for this receipt digest.
    SignatureConflict {
        /// The `<digest>.sig` file that already holds different bytes.
        file: String,
    },
    /// The stored receipts do not form one linear parent -> child chain.
    BrokenChain {
        /// Description of where the chain breaks.
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
            StoreError::Unattested { file } => write!(f, "receipt `{file}` has no attestation"),
            StoreError::Attestation { file, error } => {
                write!(f, "receipt `{file}` attestation refused: {error}")
            }
            StoreError::SignatureConflict { file } => {
                write!(f, "receipt attestation `{file}` already exists with different bytes")
            }
            StoreError::BrokenChain { detail } => write!(f, "receipt chain broken: {detail}"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Attestation { error, .. } => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e.to_string())
    }
}

/// A directory of receipts keyed by subject.
///
/// ```
/// use graphlaw::{dialect::Dialect, law::{LawState, Step}, receipt_store::ReceiptStore};
///
/// let dir = std::env::temp_dir().join(format!("graphlaw-doc-store-{}", std::process::id()));
/// let store = ReceiptStore::open(&dir)?;
/// let s = LawState::parse(b"<urn:a> <urn:p> <urn:b> .\n", Dialect::NTriples, None)?;
/// let (_child, receipt) = s.transition(&Step::EntailRdfs)?;
/// store.put(&receipt, "subject1")?;
/// assert_eq!(store.verify("subject1")?, vec![receipt]);
/// std::fs::remove_dir_all(&dir)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct ReceiptStore {
    dir: PathBuf,
}

fn hex(bytes: &[u8]) -> String {
    const T: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(T[(b >> 4) as usize] as char);
        s.push(T[(b & 15) as usize] as char);
    }
    s
}

fn digest_of(bytes: &[u8]) -> String {
    hex(blake3::hash(bytes).as_bytes())
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
    let body = format!(
        "{{\"parent\":{},\"child\":{},\"step\":{},\"authority\":{},\"revision\":{},\"added\":{},\"lease_id\":{}}}",
        esc(&r.parent),
        esc(&r.child),
        esc(r.step),
        esc(r.authority.authority),
        esc(r.authority.revision),
        r.added,
        lease
    );
    // Optional bindings are emitted only when present, so receipts without them
    // keep their historical bytes (and content addresses).
    let mut body = body;
    body.pop(); // closing brace
    if let Some(p) = &r.plan_blake3 {
        body.push_str(&format!(",\"plan_blake3\":{}", esc(p)));
    }
    if let Some(p) = &r.subject_blake3 {
        body.push_str(&format!(",\"subject_blake3\":{}", esc(p)));
    }
    body.push('}');
    body
}

/// Receipt digest: BLAKE3 of the canonical serialization.
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
            // One scalar is at most 4 bytes: validate that window, not the whole
            // remainder (which made each string O(n^2) in the input length).
            let window = &self.b[self.i..self.b.len().min(self.i + 4)];
            let rest = match std::str::from_utf8(window) {
                Ok(s) => s,
                Err(e) if e.valid_up_to() > 0 => {
                    std::str::from_utf8(&window[..e.valid_up_to()]).ok()?
                }
                Err(_) => return None,
            };
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
        let n = name.as_bytes();
        let rest = self.b.get(self.i..)?;
        if rest.len() >= n.len() + 3
            && rest[0] == b'"'
            && &rest[1..1 + n.len()] == n
            && rest[1 + n.len()] == b'"'
            && rest[2 + n.len()] == b':'
        {
            self.i += n.len() + 3;
            Some(())
        } else {
            None
        }
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
        "admit:require-signed-receipt",
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
    let mut plan_blake3 = None;
    if c.eat(",\"plan_blake3\":").is_some() {
        plan_blake3 = Some(c.string()?);
    }
    let mut subject_blake3 = None;
    if c.eat(",\"subject_blake3\":").is_some() {
        subject_blake3 = Some(c.string()?);
    }
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
        plan_blake3,
        subject_blake3,
    })
}

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// Write `bytes` to a unique temp name, fsync it, rename to `<digest>.<ext>`,
/// then fsync the parent directory.
fn publish(dir: &Path, digest: &str, ext: &str, bytes: &[u8]) -> Result<(), StoreError> {
    use std::io::Write;
    let seq = TMP_SEQ.fetch_add(1, Ordering::SeqCst);
    let tmp = dir.join(format!(".tmp-{digest}-{ext}-{}-{seq}", std::process::id()));
    let dest = dir.join(format!("{digest}.{ext}"));
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, &dest)?;
    fs::File::open(dir)?.sync_all()?;
    Ok(())
}

/// Publish `<digest>.<ext>` only if it does not exist yet: the temp file is
/// hard-linked to the destination (`link(2)` fails with `AlreadyExists` instead of
/// replacing). `Ok(true)` when this call created it, `Ok(false)` when a file was
/// already there (left untouched).
fn publish_new(dir: &Path, digest: &str, ext: &str, bytes: &[u8]) -> Result<bool, StoreError> {
    use std::io::Write;
    let seq = TMP_SEQ.fetch_add(1, Ordering::SeqCst);
    let tmp = dir.join(format!(".tmp-{digest}-{ext}-{}-{seq}", std::process::id()));
    let dest = dir.join(format!("{digest}.{ext}"));
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    let linked = fs::hard_link(&tmp, &dest);
    let _ = fs::remove_file(&tmp);
    match linked {
        Ok(()) => {
            fs::File::open(dir)?.sync_all()?;
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(e.into()),
    }
}

/// Read `path`; `None` when it does not exist.
fn read_existing(path: &Path) -> Result<Option<Vec<u8>>, StoreError> {
    match fs::read(path) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// [`publish`] unless `<digest>.<ext>` already holds exactly `bytes` (true no-op:
/// no temp file, no rename, no fsync). A file with different bytes is replaced.
fn publish_idempotent(
    dir: &Path,
    digest: &str,
    ext: &str,
    bytes: &[u8],
) -> Result<(), StoreError> {
    if read_existing(&dir.join(format!("{digest}.{ext}")))?.as_deref() == Some(bytes) {
        return Ok(());
    }
    publish(dir, digest, ext, bytes)
}

/// Orphaned `.tmp-*` files younger than this are presumed to belong to a live
/// writer (possibly in another process) and are left alone by [`ReceiptStore::open`].
const TMP_GRACE: std::time::Duration = std::time::Duration::from_secs(30);

fn sweep_dir(dir: &Path, min_age: std::time::Duration) -> Result<usize, StoreError> {
    let mut removed = 0;
    for e in fs::read_dir(dir)? {
        let e = e?;
        if !e.file_name().to_string_lossy().starts_with(".tmp-") {
            continue;
        }
        // A live writer may rename/remove its temp between read_dir and stat.
        let meta = match e.metadata() {
            Ok(m) => m,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err.into()),
        };
        if !meta.is_file() {
            continue;
        }
        let age = meta
            .modified()
            .ok()
            .and_then(|m| m.elapsed().ok())
            .unwrap_or_default();
        if age >= min_age {
            match fs::remove_file(e.path()) {
                Ok(()) => removed += 1,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => return Err(err.into()),
            }
        }
    }
    Ok(removed)
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
        let store = Self {
            dir: dir.as_ref().to_path_buf(),
        };
        store.sweep_orphans(TMP_GRACE)?;
        Ok(store)
    }

    /// Remove orphaned `.tmp-*` files (left by a crash between create and rename)
    /// at least `min_age` old, across the store root and every subject directory.
    /// [`ReceiptStore::open`] calls this with a 30 s grace so an in-flight writer
    /// in another process is not disturbed; pass `Duration::ZERO` only when no
    /// writer is live. Returns the number of files removed.
    pub fn sweep_orphans(&self, min_age: std::time::Duration) -> Result<usize, StoreError> {
        let mut removed = sweep_dir(&self.dir, min_age)?;
        for e in fs::read_dir(&self.dir)? {
            let e = e?;
            if e.file_type()?.is_dir() {
                removed += sweep_dir(&e.path(), min_age)?;
            }
        }
        Ok(removed)
    }

    fn subject_dir(&self, subject: &str) -> Result<PathBuf, StoreError> {
        check_subject(subject)?;
        Ok(self.dir.join(subject))
    }

    /// Store `receipt` under `subject`; returns its digest. Atomic (temp + rename);
    /// a true no-op when the file already holds identical bytes.
    pub fn put(&self, receipt: &Receipt, subject_sha: &str) -> Result<String, StoreError> {
        let sd = self.subject_dir(subject_sha)?;
        fs::create_dir_all(&sd)?;
        let body = encode(receipt);
        let digest = digest_of(body.as_bytes());
        publish_idempotent(&sd, &digest, "json", body.as_bytes())?;
        Ok(digest)
    }

    /// [`ReceiptStore::put`] plus the receipt's attestation, stored beside it as
    /// `<digest>.sig` (canonical attestation JSON). The `.sig` is durable before
    /// the `.json` becomes visible, so a `.json` is never observed without its
    /// `.sig`. Returns the receipt digest.
    pub fn put_signed(
        &self,
        receipt: &Receipt,
        subject_sha: &str,
        attestation: &Attestation,
    ) -> Result<String, StoreError> {
        let sd = self.subject_dir(subject_sha)?;
        fs::create_dir_all(&sd)?;
        let body = encode(receipt);
        let digest = digest_of(body.as_bytes());
        let sig = attestation.to_json();
        match read_existing(&sd.join(format!("{digest}.sig")))? {
            Some(existing) if existing == sig.as_bytes() => {}
            Some(_) => {
                return Err(StoreError::SignatureConflict {
                    file: format!("{digest}.sig"),
                });
            }
            None => {
                // Exclusive create: a racing writer with a different attestation
                // loses with SignatureConflict instead of being overwritten.
                if !publish_new(&sd, &digest, "sig", sig.as_bytes())? {
                    let file = format!("{digest}.sig");
                    match read_existing(&sd.join(&file))? {
                        Some(existing) if existing == sig.as_bytes() => {}
                        _ => return Err(StoreError::SignatureConflict { file }),
                    }
                }
            }
        }
        publish_idempotent(&sd, &digest, "json", body.as_bytes())?;
        Ok(digest)
    }

    /// [`ReceiptStore::verify`], then require every receipt to carry a
    /// `<digest>.sig` attestation that verifies against `trusted`. Returns the
    /// receipts in chain order.
    pub fn verify_attested(
        &self,
        subject_sha: &str,
        trusted: &TrustedKeys,
    ) -> Result<Vec<Receipt>, StoreError> {
        let rs = self.verify(subject_sha)?;
        let sd = self.subject_dir(subject_sha)?;
        for r in &rs {
            let digest = receipt_digest(r);
            let file = format!("{digest}.sig");
            let text = match fs::read_to_string(sd.join(&file)) {
                Ok(t) => t,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Err(StoreError::Unattested { file });
                }
                Err(e) => return Err(e.into()),
            };
            let att = Attestation::from_json(&text).map_err(|error| StoreError::Attestation {
                file: file.clone(),
                error,
            })?;
            verify_receipt(r, &att, trusted)
                .map_err(|error| StoreError::Attestation { file, error })?;
        }
        Ok(rs)
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
        chain(self.load(subject_sha)?)
    }

    /// Like [`ReceiptStore::verify`] but only the chain reachable from `head` (a
    /// state hash that is some receipt's `parent` or `child`) must be linear:
    /// returns the receipts after `head` in order. A fork elsewhere (e.g. an
    /// abandoned retry branch before `head`) does not poison the subject; a fork
    /// on the path from `head` still refuses. Every stored file is still
    /// integrity-checked (digest, canonical form).
    pub fn verify_from(&self, subject_sha: &str, head: &str) -> Result<Vec<Receipt>, StoreError> {
        chain_from(self.load(subject_sha)?, head)
    }

    fn load(&self, subject_sha: &str) -> Result<Vec<Receipt>, StoreError> {
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
        Ok(rs)
    }
}

fn chain_from(rs: Vec<Receipt>, head: &str) -> Result<Vec<Receipt>, StoreError> {
    let broken = |d: String| StoreError::BrokenChain { detail: d };
    if !rs.iter().any(|r| r.parent == head || r.child == head) {
        return Err(broken(format!("head `{head}` not found in stored receipts")));
    }
    let mut by_parent: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, r) in rs.iter().enumerate() {
        by_parent.entry(r.parent.as_str()).or_default().push(i);
    }
    let mut idx = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut cur = head.to_string();
    loop {
        let next: &[usize] = by_parent.get(cur.as_str()).map_or(&[], |v| v.as_slice());
        match next {
            [] => break,
            [i] => {
                let r = &rs[*i];
                if !seen.insert(r.child.as_str()) {
                    return Err(broken("cycle".to_string()));
                }
                idx.push(*i);
                cur = r.child.clone();
            }
            _ => {
                return Err(broken(format!(
                    "two receipts share parent `{cur}` (fork) on the path from head"
                )));
            }
        }
    }
    Ok(idx.into_iter().map(|i| rs[i].clone()).collect())
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
    let mut visited = vec![false; rs.len()];
    visited[*root] = true;
    while let Some(next) = by_parent
        .get(rs[*order.last().expect("non-empty")].child.as_str())
        .map(|v| v[0])
    {
        if visited[next] {
            return Err(broken("cycle"));
        }
        visited[next] = true;
        order.push(next);
    }
    if order.len() != rs.len() {
        return Err(broken("receipts do not form one linear chain"));
    }
    Ok(order.into_iter().map(|i| rs[i].clone()).collect())
}

#[cfg(test)]
mod hex_table_tests {
    #[test]
    fn hex_matches_format_reference() {
        let bytes: Vec<u8> = (0..=255u8).collect();
        let reference: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(super::hex(&bytes), reference);
        assert_eq!(super::hex(&[]), "");
    }
}
