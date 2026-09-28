//! Read a Claude Code session transcript (.jsonl) and emit smon: Turtle facts.
//!
//! smon-transcript-to-turtle --transcript T.jsonl --out OUT.ttl
//!     [--session-id ID] [--base IRI] [--assistant-only] [--topic-filter S]

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

use praxis_graphlaw::smon::{Args, ToolError, build_dataset, turns_from_jsonl, write_turtle};

fn run() -> Result<(), ToolError> {
    let args = Args::from_env();
    let transcript = Path::new(args.required("--transcript")?);
    let out = Path::new(args.required("--out")?);
    let session_id = args
        .value("--session-id")
        .map(String::from)
        .or_else(|| {
            transcript
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .ok_or("cannot derive a session id")?;
    let base = args.value("--base").map(String::from).unwrap_or_else(|| {
        format!("http://seanchatmangpt.github.io/packs/self-monitoring/sessions/{session_id}")
    });

    let raw = String::from_utf8_lossy(&std::fs::read(transcript).map_err(|e| e.to_string())?)
        .into_owned();
    let turns = turns_from_jsonl(&raw, args.switch("--assistant-only"));
    let ds = build_dataset(&turns, &session_id, &base)?;
    write_turtle(&ds, out)?;

    let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
    let mut topics: BTreeMap<&str, usize> = BTreeMap::new();
    for t in &turns {
        *kinds.entry(t.kind).or_default() += 1;
        if t.kind == "GroundingQuestion" {
            *topics.entry(t.topic.as_str()).or_default() += 1;
        }
    }
    let repeated: Vec<_> = topics
        .iter()
        .filter(|(_, c)| **c >= 2)
        .map(|(t, _)| *t)
        .collect();
    println!("transcript: {}", transcript.display());
    println!("session_id: {session_id}");
    println!("total extracted turns: {}", turns.len());
    println!("turnKind counts: {kinds:?}");
    println!(
        "distinct grounding_topics (GroundingQuestion turns only): {}",
        topics.len()
    );
    println!("grounding_topics: {:?}", topics.keys().collect::<Vec<_>>());
    println!(
        "repeated grounding_topics (>=2 GroundingQuestion turns, candidate escalation pairs): {repeated:?}"
    );
    println!("wrote: {} ({} quads)", out.display(), ds.quad_count());

    if let Some(filter) = args.value("--topic-filter") {
        println!("\n--- turns matching topic filter '{filter}' ---");
        for t in turns.iter().filter(|t| t.topic.contains(filter)) {
            let q: String = t.text.trim().replace('\n', " ").chars().take(160).collect();
            println!(
                "turn-{} [{}] kind={} topic={} :: {q}",
                t.seq, t.role, t.kind, t.topic
            );
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ERROR: {e}");
            ExitCode::FAILURE
        }
    }
}
