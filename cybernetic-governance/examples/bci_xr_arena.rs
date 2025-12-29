// path: cybernetic-governance/examples/bci_xr_arena.rs

//! Example: BCI/XR competitive arena with pre-emptive governance guardrails.
//! - Capabilities = cybernetic “moves” (neuro-commands, biomech actuations).
//! - Governance turns cannot fully disable experimentation or safe-exit moves.

use cybernetic_governance::*;
use std::collections::HashSet;

fn cap(id: &str) -> CapabilityId { CapabilityId(id.to_string()) }

fn main() {
    // Global constitution: define what governance can *never* take away. [web:9]
    let mut nonrestrictable = HashSet::new();
    nonrestrictable.insert(cap("safety:emergency_stop"));
    nonrestrictable.insert(cap("safety:session_exit"));
    nonrestrictable.insert(cap("access:baseline_play"));
    nonrestrictable.insert(cap("research:noninvasive_bci"));

    let constitution = GovernanceConstitution {
        global_min_capability_floor: 4,
        max_restriction_fraction_per_turn: 0.40,
        min_supermajority_floor: 0.67,
        hard_protect_safety_capabilities: true,
        globally_nonrestrictable: nonrestrictable,
    };

    let mut gov = CapabilityGovernance::new(constitution);

    // Define a competitive BCI/XR domain. [web:6]
    let mut allowed = HashSet::new();
    allowed.insert(cap("safety:emergency_stop"));
    allowed.insert(cap("safety:session_exit"));
    allowed.insert(cap("access:baseline_play"));
    allowed.insert(cap("research:noninvasive_bci"));
    allowed.insert(cap("move:bci_push"));
    allowed.insert(cap("move:bci_pull"));
    allowed.insert(cap("move:bci_shield"));

    let domain = CompetitiveDomain {
        id: "arena:phoenix:bci_xr_championship".into(),
        description: "Phoenix BCI/XR competitive cybernetic arena".into(),
        allowed_capabilities: allowed,
        min_capability_count: 5,
    };

    gov.upsert_domain(domain);

    // Governance proposal tries to heavily restrict gameplay.
    let proposal = GovernanceProposal {
        proposal_id: "prop-2026-01-lockdown".into(),
        domain_id: "arena:phoenix:bci_xr_championship".into(),
        restrict_capabilities: vec![
            cap("move:bci_push"),
            cap("move:bci_pull"),
            cap("move:bci_shield"),
            cap("research:noninvasive_bci"), // will be blocked by constitution
        ].into_iter().collect(),
        protect_capabilities: HashSet::new(),
        required_supermajority: 0.75,
        activation_height: 1_000,
    };

    let outcome = GovernanceVoteOutcome {
        proposal_id: "prop-2026-01-lockdown".into(),
        yes_weight: 800,
        no_weight: 200,
        finalized_height: 1_005,
    };

    match gov.evaluate_proposal(&proposal, &outcome, 1_010) {
        Ok(Some(new_state)) => {
            println!(
                "New disabled capabilities in {}:",
                new_state.domain.id
            );
            for cap in &new_state.disabled_capabilities {
                println!(" - {}", cap.0);
            }
            println!(
                "Enabled capabilities remain: {}",
                new_state.domain.allowed_capabilities.len() - new_state.disabled_capabilities.len()
            );
        }
        Ok(None) => {
            println!("Proposal did not meet thresholds; no change.");
        }
        Err(e) => {
            println!("Proposal rejected by constitution: {e}");
        }
    }
}
