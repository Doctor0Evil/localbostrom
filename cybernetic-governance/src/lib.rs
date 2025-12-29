// path: cybernetic-governance/src/lib.rs

//! Cybernetic gameplay + governance-turn guardrails for ALN / XR / BCI-integrated chains.
//! - Competitive “moves” defined as capabilities on cybernetic / biomech modules
//! - Governance-turns can *propose* restrictions but cannot auto-enforce them
//!   unless pre-defined constitutional rules are satisfied.
//! - Designed for integration with BCI / neuromorphic and cybernetic-chipset vNodes. [web:6][web:9]

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

/// Core module or capability IDs in the cybernetic / biomechanical system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(pub String);

/// “Competitive domain” describes a game / sport / XR grid where cybernetic moves occur.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitiveDomain {
    pub id: String,
    /// Human readable description of the game / sport / xr-grid.
    pub description: String,
    /// Capabilities that define the legal move-space in this domain.
    pub allowed_capabilities: HashSet<CapabilityId>,
    /// A minimal “freedom budget” – number of capabilities that must remain
    /// enabled; governance cannot drop below this. [web:9]
    pub min_capability_count: usize,
}

/// A governance-turn proposal about capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub proposal_id: String,
    /// Domain this proposal applies to.
    pub domain_id: String,
    /// Capabilities to restrict (disable) if the proposal passes.
    pub restrict_capabilities: HashSet<CapabilityId>,
    /// Capabilities to explicitly protect (whitelist) regardless of other rules.
    pub protect_capabilities: HashSet<CapabilityId>,
    /// Required supermajority threshold (0.0–1.0) for this proposal to apply.
    pub required_supermajority: f64,
    /// Epoch height or block number at which this proposal becomes eligible.
    pub activation_height: u64,
}

/// Result of a governance vote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceVoteOutcome {
    pub proposal_id: String,
    pub yes_weight: u128,
    pub no_weight: u128,
    /// Block or epoch height where tally was finalized.
    pub finalized_height: u64,
}

/// Immutable “constitutional” parameters that governance cannot bypass. [web:2][web:8]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConstitution {
    /// Absolute floor for min_capability_count in any domain.
    pub global_min_capability_floor: usize,
    /// Absolute maximum fraction of capabilities that can be restricted in a single turn.
    pub max_restriction_fraction_per_turn: f64,
    /// Minimal required_supermajority to *ever* restrict a capability.
    pub min_supermajority_floor: f64,
    /// Whether BCI/biomech safety capabilities are *hard protected*.
    pub hard_protect_safety_capabilities: bool,
    /// Capabilities that are globally non-restrictable (e.g., safety & access). [web:9]
    pub globally_nonrestrictable: HashSet<CapabilityId>,
}

/// Runtime state for a domain (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainState {
    pub domain: CompetitiveDomain,
    /// Currently disabled capabilities (after prior governance-turns).
    pub disabled_capabilities: HashSet<CapabilityId>,
}

/// Governance engine for capability changes.
pub struct CapabilityGovernance {
    constitution: GovernanceConstitution,
    /// Domain states indexed by domain_id.
    domains: HashMap<String, DomainState>,
}

impl CapabilityGovernance {
    pub fn new(constitution: GovernanceConstitution) -> Self {
        Self {
            constitution,
            domains: HashMap::new(),
        }
    }

    pub fn upsert_domain(&mut self, domain: CompetitiveDomain) {
        let entry = self.domains.entry(domain.id.clone()).or_insert(DomainState {
            domain: domain.clone(),
            disabled_capabilities: HashSet::new(),
        });
        entry.domain = domain;
    }

    /// Core logic: check if a governance proposal *may* apply, and if so,
    /// compute the new DomainState after restrictions.
    pub fn evaluate_proposal(
        &self,
        proposal: &GovernanceProposal,
        vote_outcome: &GovernanceVoteOutcome,
        current_height: u64,
    ) -> Result<Option<DomainState>, String> {
        let state = match self.domains.get(&proposal.domain_id) {
            Some(s) => s,
            None => return Err("Unknown domain_id".into()),
        };

        // 1. Check height / timing: proposal cannot auto-apply before activation. [web:8]
        if current_height < proposal.activation_height || vote_outcome.finalized_height < proposal.activation_height {
            return Ok(None);
        }

        // 2. Check supermajority threshold.
        let total = vote_outcome.yes_weight + vote_outcome.no_weight;
        if total == 0 {
            return Ok(None);
        }
        let yes_ratio = (vote_outcome.yes_weight as f64) / (total as f64);
        if yes_ratio < proposal.required_supermajority ||
           yes_ratio < self.constitution.min_supermajority_floor {
            // Proposal fails; no change.
            return Ok(None);
        }

        // 3. Compute tentative restricted set.
        let mut disabled = state.disabled_capabilities.clone();
        for cap in &proposal.restrict_capabilities {
            // Apply constitutional non-restrictable list. [web:9]
            if self.constitution.globally_nonrestrictable.contains(cap) {
                continue;
            }
            disabled.insert(cap.clone());
        }

        // 4. Enforce domain and global capability floors.
        let total_caps = state.domain.allowed_capabilities.len();
        let disabled_count = disabled.len().min(total_caps);
        let enabled_count = total_caps - disabled_count;

        // Per-domain floor:
        if enabled_count < state.domain.min_capability_count {
            return Err("Proposal would violate domain.min_capability_count; rejected".into());
        }

        // Global floor:
        if enabled_count < self.constitution.global_min_capability_floor {
            return Err("Proposal would violate global_min_capability_floor; rejected".into());
        }

        // Per-turn maximum restriction fraction:
        let restrict_fraction = (disabled_count as f64) / (total_caps as f64);
        if restrict_fraction > self.constitution.max_restriction_fraction_per_turn {
            return Err("Proposal over max_restriction_fraction_per_turn; rejected".into());
        }

        // 5. Hard protection for safety capabilities (e.g., fail-safes, safe-exit, pause). [web:9]
        let mut final_disabled = HashSet::new();
        for cap in disabled {
            if self.constitution.hard_protect_safety_capabilities
                && self.constitution.globally_nonrestrictable.contains(&cap)
            {
                // Skip disabling this safety capability.
                continue;
            }
            final_disabled.insert(cap);
        }

        let mut new_state = state.clone();
        new_state.disabled_capabilities = final_disabled;
        Ok(Some(new_state))
    }

    pub fn get_domain_state(&self, domain_id: &str) -> Option<&DomainState> {
        self.domains.get(domain_id)
    }
}
