//! Disclosed counterfactual post-processor over an already-emitted smon: graph.
//! Prints a full audit of every triple it changes.
//!
//! smon-broaden-topic --in-ttl IN.ttl --out-ttl OUT.ttl [--canonical-topic T]

use std::path::Path;
use std::process::ExitCode;

use graphlaw::smon::{Args, ToolError, broaden, read_turtle, write_turtle};

const HELP: &str = "\
smon-broaden-topic — disclosed counterfactual post-processor over an already-emitted smon: graph.
Prints a full audit of every triple it changes.

Usage:
  smon-broaden-topic --in-ttl IN.ttl --out-ttl OUT.ttl [--canonical-topic T]

Flags:
  --in-ttl <path>         Input Turtle graph (required)
  --out-ttl <path>        Output Turtle graph (required)
  --canonical-topic <t>   Topic to collapse GroundingQuestion topics to
                          (default: e2e-capability-grounding)

Exit codes: 0 ok, 1 usage, I/O or parse error.";

/// Validate argv before handing it to `Args`: every flag must be known, every
/// value flag must be followed by a value that is not itself a flag.
fn check_args() -> Result<(), ToolError> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        println!("{HELP}");
        std::process::exit(0);
    }
    const VALUE_FLAGS: [&str; 3] = ["--in-ttl", "--out-ttl", "--canonical-topic"];
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
        } else {
            return Err(format!("unknown argument `{a}` (see --help)"));
        }
    }
    Ok(())
}

fn run() -> Result<(), ToolError> {
    check_args()?;
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
    write_turtle(&out, output).map_err(|e| format!("{}: {e}", output.display()))?;
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
