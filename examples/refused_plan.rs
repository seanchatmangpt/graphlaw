//! A plan whose second action needs a triple that no earlier action produced.
//!
//! Run: cargo run --example refused_plan
//! Output: typed `REFUSED at index 1, action ` + backticked `paint`, naming the
//! unmet triple `unmet: <urn:d:door> <urn:p:has> <urn:v:key> .` (exit 0 —
//! refusal is the demonstrated value, not a crash).
//! Court: tests/readme_first_ten_minutes.rs::refused_plan_matches_readme_text.
use graphlaw::{
    dialect::Dialect,
    law::{LawError, LawState},
    plan::{Action, Plan},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = LawState::parse(
        b"<urn:d:door> <urn:p:is> <urn:v:closed> .\n",
        Dialect::NTriples,
        None,
    )?;
    let plan = Plan {
        goal: String::new(),
        actions: vec![
            Action {
                name: "lock".into(),
                pre: "<urn:d:door> <urn:p:is> <urn:v:closed> .".into(),
                add: "<urn:d:door> <urn:p:is> <urn:v:locked> .".into(),
                del: String::new(),
                ..Default::default()
            },
            Action {
                name: "paint".into(),
                pre: "<urn:d:door> <urn:p:has> <urn:v:key> .".into(),
                add: "<urn:d:door> <urn:p:is> <urn:v:red> .".into(),
                del: String::new(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    match plan.admit(&start) {
        Err(LawError::PlanRefused {
            index,
            action,
            missing,
            ..
        }) => {
            println!("REFUSED at index {index}, action `{action}`");
            for m in missing {
                println!("  unmet: {m}");
            }
        }
        other => println!("unexpected: {other:?}"),
    }
    Ok(())
}
