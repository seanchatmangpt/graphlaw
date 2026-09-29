//! Admit a 3-action plan over an N-Triples state and print the receipt chain.
use graphlaw::{
    dialect::Dialect,
    law::LawState,
    plan::{Action, Plan},
};

fn act(name: &str, pre: &str, add: &str, del: &str) -> Action {
    Action {
        name: name.into(),
        pre: pre.into(),
        add: add.into(),
        del: del.into(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = LawState::parse(
        b"<urn:d:door> <urn:p:is> <urn:v:closed> .\n",
        Dialect::NTriples,
        None,
    )?;
    let plan = Plan {
        goal: "<urn:d:door> <urn:p:is> <urn:v:locked> .".into(),
        actions: vec![
            act(
                "open",
                "<urn:d:door> <urn:p:is> <urn:v:closed> .",
                "<urn:d:door> <urn:p:is> <urn:v:open> .",
                "<urn:d:door> <urn:p:is> <urn:v:closed> .",
            ),
            act(
                "close",
                "<urn:d:door> <urn:p:is> <urn:v:open> .",
                "<urn:d:door> <urn:p:is> <urn:v:closed> .",
                "<urn:d:door> <urn:p:is> <urn:v:open> .",
            ),
            act(
                "lock",
                "<urn:d:door> <urn:p:is> <urn:v:closed> .",
                "<urn:d:door> <urn:p:is> <urn:v:locked> .",
                "<urn:d:door> <urn:p:is> <urn:v:closed> .",
            ),
        ],
    };
    let admitted = plan.admit(&start)?; // the core: 1 line
    for r in &admitted.receipts {
        println!("{} -> {}", r.step, r.child);
    }
    println!("final state: {}", admitted.state.id());
    Ok(())
}
