//! Admit a FOND strong-cyclic retry-loop policy (needs `--features abi`).
#[cfg(feature = "abi")]
fn main() {
    use graphlaw::policy::admit;
    let problem = r#"{"states":[{"id":"s0"},{"id":"g","facts":["done"]}],"initial_states":["s0"],
      "goal":{"facts":["done"]},"transitions":[
      {"action":"flip","from":"s0","to":"g","probability_ppm":500000},
      {"action":"flip","from":"s0","to":"s0","probability_ppm":500000}]}"#;
    let policy = r#"{"solved":true,"policy":[{"state":"s0","action":"flip","outcomes":[
      {"state":"g","probability_ppm":500000},{"state":"s0","probability_ppm":500000}]}]}"#;
    match admit(problem, policy) {
        Ok(a) => println!(
            "ADMITTED reachable={:?} goals={:?} entries={:?}",
            a.reachable, a.goal_states, a.entries
        ),
        Err(e) => println!(
            "REFUSED {:?} at {}/{}: {}",
            e.kind, e.state, e.action, e.message
        ),
    }
}

#[cfg(not(feature = "abi"))]
fn main() {
    eprintln!("policy_admission needs: cargo run --example policy_admission --features abi");
}
