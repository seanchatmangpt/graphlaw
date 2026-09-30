//! Receipt feedback: a gate refuses until the receipt is recorded into the state.
use graphlaw::{
    dialect::Dialect,
    law::{LawState, Step},
    receipt,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s0 = LawState::parse(
        b"<urn:a:C> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <urn:a:D> .\n\
          <urn:a:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:a:C> .\n",
        Dialect::NTriples,
        None,
    )?;
    let (s1, r) = s0.transition(&Step::EntailRdfs)?;
    let gate = Step::RequireReceipt {
        step: "derive:rdfs",
    };
    println!("before record: {}", s1.transition(&gate).unwrap_err());
    let recorded = receipt::record(&s1, &r)?;
    let (_, gate_receipt) = recorded.transition(&gate)?;
    println!("after record:  admitted by `{}`", gate_receipt.step);
    Ok(())
}
