// path: the_element/examples/governance_turn_demo.rs

use the_element::*;

fn main() -> Result<(), String> {
    let mut element = default_element();

    let agent = AgentId("did:aln:player:neo".into());
    let gov_turn = GovernanceTurnId("turn:2026:new-year-protocol".into());

    // Agent explicitly opts into two enhancements.
    element.request_enable(
        &agent,
        &CapabilityId("cognitive:pattern_assist".into()),
        true,
    )?;
    element.request_enable(
        &agent,
        &CapabilityId("sensory:xr_overlay_competitive".into()),
        true,
    )?;

    // Governance-turn attempts to *add* one new experimental ability
    // and *restrict* none (pure expansion turn).
    let mut unlock = HashSet::new();
    unlock.insert(CapabilityId("cognitive:focus_enhancer".into()));

    let restrict = HashSet::new();

    element.governance_turn(&gov_turn, &agent, &restrict, &unlock)?;

    let profile = element.get_profile(&agent).unwrap();
    println!("Enabled capabilities for {}:", (agent.0).as_str());
    for cap in &profile.enabled_capabilities {
        println!(" - {}", cap.0);
    }

    // If a later governance-turn tried to *restrict* more than 33% of these,
    // or touch baseline abilities, it would be rejected automatically. [web:21][web:26][web:29]

    Ok(())
}
