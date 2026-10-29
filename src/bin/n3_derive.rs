//! Derive the forward-chained closure of an N3 document and print the derived
//! triples as prefixed N3 text (`:s :p :o .` per line, @prefix headers included).
//!
//! Replaces the retired `graphlaw-eyeron` bin as the oracle runner for
//! `scripts/gen_wasm4pm_graphlaw_vectors.py` (which parses exactly these
//! `:holds` lines) and any other consumer that wants the raw derived set.
//!
//! Usage: `n3_derive <document.n3>` — exit 0 on complete fixpoint, exit 1 with
//! the reason on stderr for parse failures, budget/limit refusals, or
//! incomplete fixpoints (no partial output on incomplete: a partial answer is
//! not the answer).

use graphlaw::n3::{
    Document, EyeronError, ReasonerOptions, reason_document, triples_to_n3,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: n3_derive <document.n3>");
        std::process::exit(2);
    }
    let text = match std::fs::read_to_string(&args[1]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("read failed: {e}");
            std::process::exit(2);
        }
    };
    if let Err(e) = run(&text) {
        eprintln!("n3_derive: {e}");
        std::process::exit(1);
    }
}

fn run(text: &str) -> Result<(), String> {
    let doc: Document = graphlaw::n3::parse_n3(text, None)
        .map_err(|e: EyeronError| format!("parse: {} (offset {:?})", e.message, e.offset))?;
    let options = ReasonerOptions {
        proof: false,
        ..ReasonerOptions::default()
    };
    let result = reason_document(&doc, &options);
    let result = match result.status {
        graphlaw::n3::CompletionStatus::Complete => result,
        other => {
            return Err(format!(
                "incomplete fixpoint ({other:?}): a partial answer is not the answer"
            ));
        }
    };
    print!("{}", triples_to_n3(&doc.prefixes, &result.derived));
    Ok(())
}
