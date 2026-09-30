//! Admit a 3-action plan over an N-Triples state and print the receipt chain.
//! No hand-written N-Triples: `Triple` validates IRIs and escapes literals.
use graphlaw::{
    dialect::Dialect,
    law::LawState,
    plan::{Plan, Triple},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- core ---
    let is = |v| Triple::iri("urn:d:door", "urn:p:is", v);
    #[rustfmt::skip]
    let start = LawState::parse(b"<urn:d:door> <urn:p:is> <urn:v:closed> .", Dialect::NTriples, None)?;
    #[rustfmt::skip]
    let plan = Plan::builder()
        .action("open").requires(is("urn:v:closed")?).adds(is("urn:v:open")?).deletes(is("urn:v:closed")?)
        .action("close").requires(is("urn:v:open")?).adds(is("urn:v:closed")?).deletes(is("urn:v:open")?)
        .action("lock").requires(is("urn:v:closed")?).adds(is("urn:v:locked")?).deletes(is("urn:v:closed")?)
        .goal(is("urn:v:locked")?).build()?;
    let admitted = plan.admit(&start)?;
    // --- end core ---
    for r in &admitted.receipts {
        println!("{} -> {}", r.step, r.child);
    }
    println!("final state: {}", admitted.state.id());
    Ok(())
}
