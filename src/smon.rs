//! Self-monitoring pack tooling: transcript capture and the counterfactual
//! topic-broadening experiment. All RDF is read and written by PurRDF; this
//! module only decides *which facts* to assert.
//!
//! The classification below is a disclosed, complete, auditable pattern list
//! over turn text. It is a capture heuristic, not an NLU classifier (see
//! `packs/self-monitoring-pack/ontology.ttl`, CLASSIFICATION-IS-INPUT FENCE).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, LazyLock};

use purrdf::{RdfDataset, RdfDatasetBuilder, RdfLiteral, TermValue};
use regex::Regex;
use serde_json::Value;

/// Namespace of the self-monitoring pack vocabulary.
pub const SMON: &str = "http://seanchatmangpt.github.io/packs/self-monitoring#";
/// Namespace of the dogfood-lifecycle pack vocabulary.
pub const DFL: &str = "http://seanchatmangpt.github.io/packs/dogfood-lifecycle#";
/// Dublin Core terms namespace.
pub const DCTERMS: &str = "http://purl.org/dc/terms/";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

/// Error type for the tools: a message naming what failed.
pub type ToolError = String;

fn rx(pattern: &str) -> Regex {
    Regex::new(&format!("(?i){pattern}")).expect("static pattern")
}

fn rxs(patterns: &[&str]) -> Vec<Regex> {
    patterns.iter().map(|p| rx(p)).collect()
}

static RUN_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    rxs(&[
        r"test result:\s*ok",
        r"\d+\s+passed;\s*\d+\s+failed",
        r"->\s*exit\s+\d",
        r"\bexit\s+code\s*0\b",
        r"\bexit 0\b",
        r"\$\s+(cargo|just|python3?|ggen|git|pytest|npm)\s+\S+.*\n.*\S",
        r"shapes_conform.*true",
        r"\brunning \d+ tests?\b",
    ])
});

static BLOCKER_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    rxs(&[
        r"\bblocked (on|by)\b",
        r"\bblocker:",
        r"\bcannot proceed (until|because|without)\b",
        r"\brequires? (a |an )?(credential|external approval|manual approval|access grant)\b",
        r"\bthe blocking hop is\b",
        r"\bwaiting on (a |an )?(human|approval|credential)\b",
        r"\bgenuinely underdetermined\b",
    ])
});

static SURVEY_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    rxs(&[
        r"real_edge",
        r"partial_real_edge",
        r"```mermaid",
        r"crown[- ]witness",
        r"capability[- ]fence",
        r"\|[^\n]+\|\s*\n\s*\|[-:\s|]+\|",
        r"\barchitecture (survey|map|overview)\b",
        r"\bthis project has moved through\b.{0,40}\bstages?\b",
    ])
});

static GROUNDING_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    rxs(&[
        r"does (this|it|the \w+).{0,40}\bwork\b",
        r"how far has this (evolved|progressed|come)",
        r"what'?s the status\b",
        r"\bwhat is the status\b",
        r"\bis this (actually |really )?(done|working|complete)\b",
        r"\bdoes the cli\b",
        r"\bend[- ]to[- ]end\?",
        r"\be2e\?",
        r"\bdoes it work\b",
        r"\bcan it go from\b",
        r"\bi want to know if it works\b",
    ])
});

static TOPIC_KEYWORDS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    [
        ("arazzo", r"\barr?azzo\b"),
        ("swarm", r"\bswarm\b"),
        ("cli", r"\bcli\b"),
        ("workflow", r"\bworkflow\b"),
        ("atomvm", r"\batomvm\b"),
        ("erlang", r"\berlang\b"),
        ("crown", r"\bcrown\b"),
        ("receipt", r"\breceipts?\b"),
        ("hook", r"\bhooks?\b"),
        ("soc2", r"\bsoc-?2\b"),
        ("mfact", r"\bmfact\b"),
        ("bribery", r"\bbribery\b"),
        ("powl", r"\bpowl\b"),
        ("pddl", r"\bpddl\b"),
        ("ocel", r"\bocel\b"),
        ("dogfood", r"\bdogfood\w*\b"),
        ("togaf", r"\btogaf\b"),
        ("progress", r"\b(evolved|evolution|progress(ed)?)\b"),
        ("status", r"\bstatus\b"),
        ("e2e", r"\bend[- ]to[- ]end\b|\be2e\b"),
        ("cng", r"\bcng\b"),
        ("swarm-agents", r"\bagents?\b"),
    ]
    .into_iter()
    .map(|(k, p)| (k, rx(p)))
    .collect()
});

static WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[a-zA-Z][a-zA-Z0-9']{2,}").unwrap());

const STOPWORDS: &[&str] = &[
    "the", "a", "an", "is", "it", "this", "that", "does", "do", "did", "to", "of", "and", "or",
    "for", "on", "in", "at", "be", "can", "will", "i", "you", "we", "what", "how", "so", "not",
    "no", "yes", "with", "from", "has", "have", "had", "was", "were", "are", "as", "if", "but",
    "want", "know", "go", "get", "let", "me", "my", "your", "actually",
];

/// Priority: RunResponse > BlockerResponse > SurveyResponse > GroundingQuestion > Other.
/// Returns the kind and the exact patterns that matched.
pub fn classify_turn(text: &str) -> (&'static str, Vec<String>) {
    for (kind, patterns) in [
        ("RunResponse", &*RUN_PATTERNS),
        ("BlockerResponse", &*BLOCKER_PATTERNS),
        ("SurveyResponse", &*SURVEY_PATTERNS),
        ("GroundingQuestion", &*GROUNDING_PATTERNS),
    ] {
        let hits: Vec<String> = patterns
            .iter()
            .filter(|p| p.is_match(text))
            .map(|p| p.as_str().to_string())
            .collect();
        if !hits.is_empty() {
            return (kind, hits);
        }
    }
    ("Other", Vec::new())
}

/// Sorted, hyphen-joined canonical keywords; else the first three significant tokens.
pub fn extract_topic(text: &str) -> String {
    let mut hits: Vec<&str> = TOPIC_KEYWORDS
        .iter()
        .filter(|(_, p)| p.is_match(text))
        .map(|(k, _)| *k)
        .collect();
    hits.sort_unstable();
    if !hits.is_empty() {
        return hits.join("-");
    }
    let lower = text.to_lowercase();
    let significant: Vec<&str> = WORD
        .find_iter(&lower)
        .map(|m| m.as_str())
        .filter(|w| !STOPWORDS.contains(w))
        .take(3)
        .collect();
    if significant.is_empty() {
        "unclassified-topic".into()
    } else {
        format!("kw-{}", significant.join("-"))
    }
}

/// One extracted conversational turn.
#[derive(Debug, Clone)]
pub struct Turn {
    /// Sequence number of the turn.
    pub seq: usize,
    /// Speaker role.
    pub role: &'static str,
    /// Turn text.
    pub text: String,
    /// Turn kind.
    pub kind: &'static str,
    /// Evidence references cited by the turn.
    pub evidence: Vec<String>,
    /// Topic label.
    pub topic: String,
}

/// A human-typed chat turn: string content and `origin.kind == "human"`.
/// Excludes tool-result carriers, task notifications and injected summaries.
fn human_text(rec: &Value) -> Option<&str> {
    let content = rec.get("message")?.get("content")?.as_str()?;
    let origin_kind = rec.get("origin")?.get("kind")?.as_str()?;
    (origin_kind == "human" && !content.trim().is_empty()).then_some(content)
}

/// Extract turns from a Claude Code session transcript (`.jsonl`). Consecutive
/// assistant lines sharing a `message.id` are one logical turn; only `text`
/// blocks count (never thinking or tool_use).
pub fn turns_from_jsonl(raw: &str, assistant_only: bool) -> Vec<Turn> {
    let mut texts: Vec<(&'static str, String)> = Vec::new();
    let mut pending: Option<(Option<String>, Vec<String>)> = None;

    let flush = |pending: &mut Option<(Option<String>, Vec<String>)>,
                 out: &mut Vec<(&'static str, String)>| {
        if let Some((_, parts)) = pending.take() {
            let text = parts
                .iter()
                .filter(|t| !t.trim().is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join("\n");
            if !text.trim().is_empty() {
                out.push(("assistant", text));
            }
        }
    };

    for line in raw.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let Ok(rec) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let rtype = rec.get("type").and_then(Value::as_str);
        if rtype == Some("assistant") {
            let msg = rec.get("message");
            let msgid = msg
                .and_then(|m| m.get("id"))
                .and_then(Value::as_str)
                .map(String::from);
            if pending.as_ref().is_none_or(|(id, _)| *id != msgid) {
                flush(&mut pending, &mut texts);
                pending = Some((msgid, Vec::new()));
            }
            if let Some(blocks) = msg.and_then(|m| m.get("content")).and_then(Value::as_array) {
                for b in blocks {
                    if b.get("type").and_then(Value::as_str) == Some("text") {
                        let t = b.get("text").and_then(Value::as_str).unwrap_or("");
                        pending.as_mut().expect("set above").1.push(t.to_string());
                    }
                }
            }
            continue;
        }
        flush(&mut pending, &mut texts);
        if rtype == Some("user")
            && !assistant_only
            && let Some(t) = human_text(&rec)
        {
            texts.push(("user", t.to_string()));
        }
    }
    flush(&mut pending, &mut texts);

    texts
        .into_iter()
        .enumerate()
        .map(|(i, (role, text))| {
            let (kind, evidence) = classify_turn(&text);
            let topic = extract_topic(&text);
            Turn {
                seq: i + 1,
                role,
                text,
                kind,
                evidence,
                topic,
            }
        })
        .collect()
}

fn smon(name: &str) -> String {
    format!("{SMON}{name}")
}

/// Assert the smon: facts for `turns` into a PurRDF dataset.
pub fn build_dataset(
    turns: &[Turn],
    session_id: &str,
    base: &str,
) -> Result<Arc<RdfDataset>, ToolError> {
    let ns = if base.ends_with('#') {
        base.to_string()
    } else {
        format!("{base}#")
    };
    let mut b = RdfDatasetBuilder::new();
    let ty = b.intern_iri(RDF_TYPE);
    let is_part_of = b.intern_iri(&format!("{DCTERMS}isPartOf"));
    let subject = b.intern_iri(&format!("{DCTERMS}subject"));
    let description = b.intern_iri(&format!("{DCTERMS}description"));
    let identifier = b.intern_iri(&format!("{DCTERMS}identifier"));
    let seq_idx = b.intern_iri(&smon("sequenceIndex"));
    let turn_kind = b.intern_iri(&smon("turnKind"));
    let follows = b.intern_iri(&smon("immediatelyFollows"));
    let turn_class = b.intern_iri(&smon("Turn"));

    let session = b.intern_iri(&format!("{ns}session"));
    let session_class = b.intern_iri(&format!("{DFL}Session"));
    b.push_quad(session, ty, session_class, None);
    let id_lit = b.intern_literal(RdfLiteral::simple(session_id));
    b.push_quad(session, identifier, id_lit, None);
    let desc = b.intern_literal(RdfLiteral::simple(
        "Real Claude Code session transcript, captured by smon-transcript-to-turtle.",
    ));
    b.push_quad(session, description, desc, None);

    let mut prior_grounding = None;
    for t in turns {
        let turn = b.intern_iri(&format!("{ns}turn-{}", t.seq));
        b.push_quad(turn, ty, turn_class, None);
        b.push_quad(turn, is_part_of, session, None);
        let topic = b.intern_literal(RdfLiteral::simple(t.topic.clone()));
        b.push_quad(turn, subject, topic, None);
        let seq = b.intern_literal(RdfLiteral::typed(t.seq.to_string(), XSD_INTEGER));
        b.push_quad(turn, seq_idx, seq, None);
        let kind = b.intern_iri(&smon(t.kind));
        b.push_quad(turn, turn_kind, kind, None);
        let mut quote = t.text.trim().replace('\n', " ");
        if quote.chars().count() > 220 {
            quote = quote.chars().take(220).collect::<String>() + "...";
        }
        let d = b.intern_literal(RdfLiteral::simple(format!("[{}] {quote}", t.role)));
        b.push_quad(turn, description, d, None);
        // Only a turn directly after a GroundingQuestion carries immediatelyFollows.
        if let Some(prev) = prior_grounding {
            b.push_quad(turn, follows, prev, None);
        }
        prior_grounding = (t.kind == "GroundingQuestion").then_some(turn);
    }
    b.freeze().map_err(|e| format!("{e:?}"))
}

/// Serialize a dataset as Turtle bytes.
pub fn to_turtle(ds: &RdfDataset) -> Result<Vec<u8>, ToolError> {
    purrdf::serialize_dataset(ds, "text/turtle", purrdf::SerializeGraph::Dataset)
        .map_err(|e| format!("{e:?}"))
}

/// Write a dataset to `out` as Turtle.
pub fn write_turtle(ds: &RdfDataset, out: &Path) -> Result<(), ToolError> {
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(out, to_turtle(ds)?).map_err(|e| e.to_string())
}

/// Read a Turtle file into a dataset.
pub fn read_turtle(path: &Path) -> Result<Arc<RdfDataset>, ToolError> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    purrdf::parse_dataset(&bytes, "text/turtle", None)
        .map_err(|e| format!("{}: {e:?}", path.display()))
}

/// What the broadening experiment changed, for the audit trail.
#[derive(Debug, Default)]
pub struct Broadened {
    /// Audit lines describing each change.
    pub audit: Vec<String>,
    /// Number of first-pass rewrites.
    pub rewrite1: usize,
    /// Number of second-pass rewrites.
    pub rewrite2: usize,
}

fn iri_of(ds: &RdfDataset, id: purrdf::TermId) -> Option<String> {
    match ds.term_value(id) {
        TermValue::Iri(i) => Some(i),
        _ => None,
    }
}

/// The two disclosed counterfactual rewrites over an already-emitted graph.
///
/// 1. A turn immediately following a GroundingQuestion whose own kind is `Other`
///    becomes a `SurveyResponse` (it already failed the run and blocker checks).
/// 2. Every GroundingQuestion turn's `dcterms:subject` is replaced by one
///    canonical literal.
pub fn broaden(
    ds: &RdfDataset,
    canonical_topic: &str,
) -> Result<(Arc<RdfDataset>, Broadened), ToolError> {
    let p_kind = format!("{SMON}turnKind");
    let p_follows = format!("{SMON}immediatelyFollows");
    let p_subject = format!("{DCTERMS}subject");
    let grounding = smon("GroundingQuestion");
    let other = smon("Other");

    let mut kind: BTreeMap<String, String> = BTreeMap::new();
    let mut subj: BTreeMap<String, TermValue> = BTreeMap::new();
    let mut follow: BTreeMap<String, String> = BTreeMap::new();
    for q in ds.quads() {
        let (Some(s), Some(p)) = (iri_of(ds, q.s), iri_of(ds, q.p)) else {
            continue;
        };
        if p == p_kind {
            if let Some(o) = iri_of(ds, q.o) {
                kind.insert(s, o);
            }
        } else if p == p_subject {
            subj.insert(s, ds.term_value(q.o));
        } else if p == p_follows
            && let Some(o) = iri_of(ds, q.o)
        {
            follow.insert(s, o);
        }
    }

    let mut report = Broadened::default();
    let mut drop_kind: BTreeMap<String, String> = BTreeMap::new();
    for (turn, prior) in &follow {
        if kind.get(prior) != Some(&grounding) {
            continue;
        }
        if kind.get(turn) == Some(&other) {
            drop_kind.insert(turn.clone(), other.clone());
            report.audit.push(format!(
                "REWRITE 1: {turn} turnKind Other -> SurveyResponse (follows GroundingQuestion {prior})"
            ));
            report.rewrite1 += 1;
        }
    }
    let mut regrounded: BTreeMap<String, Option<TermValue>> = BTreeMap::new();
    for (turn, k) in &kind {
        if *k == grounding {
            let old = subj.get(turn).cloned();
            report.audit.push(format!(
                "REWRITE 2: {turn} dcterms:subject {} -> \"{canonical_topic}\"",
                old.as_ref().map_or("None".into(), |v| format!("{v:?}"))
            ));
            regrounded.insert(turn.clone(), old);
            report.rewrite2 += 1;
        }
    }

    let mut b = RdfDatasetBuilder::new();
    for q in ds.quads() {
        let (s, p) = (iri_of(ds, q.s), iri_of(ds, q.p));
        if let (Some(s), Some(p)) = (&s, &p) {
            if *p == p_kind
                && drop_kind.contains_key(s)
                && iri_of(ds, q.o).as_deref() == Some(other.as_str())
            {
                continue;
            }
            if *p == p_subject
                && regrounded
                    .get(s)
                    .is_some_and(|old| old.as_ref() == Some(&ds.term_value(q.o)))
            {
                continue;
            }
        }
        let (s, p, o) = (
            b.intern_owned_term(&ds.to_owned_term(q.s)),
            b.intern_owned_term(&ds.to_owned_term(q.p)),
            b.intern_owned_term(&ds.to_owned_term(q.o)),
        );
        let g = q.g.map(|g| b.intern_owned_term(&ds.to_owned_term(g)));
        b.push_quad(s, p, o, g);
    }
    let kind_p = b.intern_iri(&p_kind);
    let subj_p = b.intern_iri(&p_subject);
    let survey = b.intern_iri(&smon("SurveyResponse"));
    for turn in drop_kind.keys() {
        let t = b.intern_iri(turn);
        b.push_quad(t, kind_p, survey, None);
    }
    let canonical = b.intern_literal(RdfLiteral::simple(canonical_topic));
    for turn in regrounded.keys() {
        let t = b.intern_iri(turn);
        b.push_quad(t, subj_p, canonical, None);
    }
    Ok((b.freeze().map_err(|e| format!("{e:?}"))?, report))
}

/// Minimal `--flag value` / `--switch` parsing shared by both tools.
pub struct Args(Vec<String>);

impl Args {
    /// Parse the process arguments.
    pub fn from_env() -> Self {
        Args(std::env::args().skip(1).collect())
    }

    /// The value following `flag`, if present.
    pub fn value(&self, flag: &str) -> Option<&str> {
        self.0
            .iter()
            .position(|a| a == flag)
            .and_then(|i| self.0.get(i + 1))
            .map(String::as_str)
    }

    /// The value following `flag`, or an error when absent.
    pub fn required(&self, flag: &str) -> Result<&str, ToolError> {
        self.value(flag)
            .ok_or_else(|| format!("missing required {flag}"))
    }

    /// True when `flag` is present.
    pub fn switch(&self, flag: &str) -> bool {
        self.0.iter().any(|a| a == flag)
    }
}
