//! A plan whose second action needs a triple that no earlier action produced.
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
            },
            Action {
                name: "paint".into(),
                pre: "<urn:d:door> <urn:p:has> <urn:v:key> .".into(),
                add: "<urn:d:door> <urn:p:is> <urn:v:red> .".into(),
                del: String::new(),
            },
        ],
    };
    match plan.admit(&start) {
        Err(LawError::PlanRefused {
            index,
            action,
            missing,
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
