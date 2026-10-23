//! Read a Claude Code session transcript (.jsonl) and emit smon: Turtle facts.
//!
//! smon-transcript-to-turtle --transcript T.jsonl --out OUT.ttl
//!     [--session-id ID] [--base IRI] [--assistant-only] [--topic-filter S]

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

use graphlaw::smon::{Args, ToolError, build_dataset, turns_from_jsonl, write_turtle};

const HELP: &str = "\
smon-transcript-to-turtle — emit smon: Turtle facts from a Claude Code session transcript (.jsonl).

Usage:
  smon-transcript-to-turtle --transcript T.jsonl --out OUT.ttl
      [--session-id ID] [--base IRI] [--assistant-only] [--topic-filter S]

Flags:
  --transcript <path>  Session transcript in JSONL format (required)
  --out <path>         Turtle output file (required)
  --session-id <id>    Session id (default: transcript file stem)
  --base <iri>         Base IRI for emitted subjects
                       (default: .../sessions/<session-id>)
  --assistant-only     Switch: keep only assistant turns
  --topic-filter <s>   Also print turns whose grounding topic contains <s>

Exit codes: 0 ok, 1 usage, I/O or parse error.";

/// Validate argv before handing it to `Args`: every flag must be known, every
/// value flag must be followed by a value that is not itself a flag.
fn check_args() -> Result<(), ToolError> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        println!("{HELP}");
        std::process::exit(0);
    }
    const VALUE_FLAGS: [&str; 5] = [
        "--transcript",
        "--out",
        "--session-id",
        "--base",
        "--topic-filter",
    ];
    const SWITCHES: [&str; 1] = ["--assistant-only"];
    let mut i = 0;
    while i < argv.len() {
        let a = &argv[i];
        if VALUE_FLAGS.contains(&a.as_str()) {
            let v = argv
                .get(i + 1)
                .ok_or_else(|| format!("{a} needs a value (see --help)"))?;
            if v.starts_with("--") {
                return Err(format!(
                    "{a} needs a value; found another flag `{v}` (see --help)"
                ));
            }
            i += 2;
        } else if SWITCHES.contains(&a.as_str()) {
            i += 1;
        } else {
            return Err(format!("unknown argument `{a}` (see --help)"));
        }
    }
    Ok(())
}

fn run() -> Result<(), ToolError> {
    check_args()?;
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

    let raw = String::from_utf8_lossy(
        &std::fs::read(transcript).map_err(|e| format!("{}: {e}", transcript.display()))?,
    )
    .into_owned();
    let turns = turns_from_jsonl(&raw, args.switch("--assistant-only"));
    let ds = build_dataset(&turns, &session_id, &base)?;
    write_turtle(&ds, out).map_err(|e| format!("{}: {e}", out.display()))?;

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
