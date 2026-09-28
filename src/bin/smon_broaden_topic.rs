//! Disclosed counterfactual post-processor over an already-emitted smon: graph.
//! Prints a full audit of every triple it changes.
//!
//! smon-broaden-topic --in-ttl IN.ttl --out-ttl OUT.ttl [--canonical-topic T]

use std::path::Path;
use std::process::ExitCode;

use graphlaw::smon::{Args, ToolError, broaden, read_turtle, write_turtle};

fn run() -> Result<(), ToolError> {
    let args = Args::from_env();
    let input = Path::new(args.required("--in-ttl")?);
    let output = Path::new(args.required("--out-ttl")?);
    let topic = args
        .value("--canonical-topic")
        .unwrap_or("e2e-capability-grounding");

    let ds = read_turtle(input)?;
    println!("loaded {}: {} quads", input.display(), ds.quad_count());
    let (out, report) = broaden(&ds, topic)?;
    for line in &report.audit {
        println!("{line}");
    }
    write_turtle(&out, output)?;
    println!(
        "REWRITE 1 (Other->SurveyResponse after a GroundingQuestion): {} turns changed",
        report.rewrite1
    );
    println!(
        "REWRITE 2 (GroundingQuestion topic collapsed to '{topic}'): {} turns changed",
        report.rewrite2
    );
    println!("wrote {}: {} quads", output.display(), out.quad_count());
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
