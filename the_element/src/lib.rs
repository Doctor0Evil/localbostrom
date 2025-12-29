// path: the_element/src/lib.rs

//! "The Element": foundational kernel for advanced cybernetic discovery,
//! agentic governance-turns, and stakeholder-approved augmented abilities.
//!
//! Goal:
//! - Define a *capability graph* for cybernetic / AI-augmented abilities.
//! - Allow governance-turns (human, AI, mixed) to *enable / extend* abilities,
//!   but never silently strip baseline rights or experimentation powers. [web:21][web:26][web:29]
//! - Make it usable across BCI, XR, biomech chipsets, and blockchain agents. [web:20][web:23][web:27]

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

/// ---------------------------------------------------------------------
/// CORE TYPES
/// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String); // human, cyborg, AI, org

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GovernanceTurnId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityDomain {
    Cognitive,     // memory, focus, pattern-boost
    Motor,         // exoskeleton, prosthetics, biomech motion
    Sensory,       // XR overlays, neurofeedback, haptics
    Social,        // coordination, multi-agent interfaces
    Security,      // neurosecurity, safety envelopes [web:24]
    Meta,          // introspection, self-governance tools
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityClass {
    BaselineRight,      // cannot be removed by any governance-turn.
    Enhancement,        // optional uplift, can be gated but not coerced. [web:20][web:21]
    Experimental,       // research-mode, with strict consent + safety.
}

/// Minimal risk tier for BCI / biomech / XR enhancement. [web:20][web:23]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskTier {
    Low,        // non-invasive, reversible, minimal side effects
    Medium,     // non-invasive but strong modulation, or invasive maintenance-free
    High,       // invasive or deep modulation; strict governance + consent
}

/// A single cybernetic / AI-augmented ability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyberneticAbility {
    pub id: CapabilityId,
    pub name: String,
    pub domain: CapabilityDomain,
    pub class_: CapabilityClass,
    pub risk_tier: RiskTier,
    /// Human-readable description of what this ability does.
    pub description: String,
    /// Abilities that must be enabled before this can be used.
    pub requires: HashSet<CapabilityId>,
    /// Whether this ability can be delegated to an agentic-AI co-pilot. [web:25][web:28]
    pub ai_delegable: bool,
    /// Whether this ability can *only* be used with explicit opt-in.
    pub require_explicit_opt_in: bool,
}

/// A “cybernetic profile” for any agent/stakeholder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCyberProfile {
    pub agent: AgentId,
    pub enabled_capabilities: HashSet<CapabilityId>,
    pub blocked_capabilities: HashSet<CapabilityId>,
    pub preferences: serde_json::Value,
}

/// ---------------------------------------------------------------------
/// ELEMENT FOUNDATION: CAPABILITY GRAPH
/// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementConfig {
    /// Baseline capabilities that must always remain available to all agents.
    /// These represent *rights to augment / exit / introspect*, not privileges. [web:21][web:26][web:29]
    pub global_baseline_capabilities: HashSet<CapabilityId>,
    /// Max fraction of an agent's enabled capabilities that may be restricted in 1 turn.
    pub max_restriction_fraction_per_turn: f64,
}

pub struct TheElement {
    config: ElementConfig,
    /// Canonical library of discoverable abilities.
    abilities: HashMap<CapabilityId, CyberneticAbility>,
    /// Per-agent profiles (actual enabled/blocked sets).
    profiles: HashMap<AgentId, AgentCyberProfile>,
}

impl TheElement {
    pub fn new(config: ElementConfig) -> Self {
        Self {
            config,
            abilities: HashMap::new(),
            profiles: HashMap::new(),
        }
    }

    /// Define or update a capability in the global library.
    pub fn upsert_ability(&mut self, ability: CyberneticAbility) {
        self.abilities.insert(ability.id.clone(), ability);
    }

    /// Initialize or fetch a profile.
    fn ensure_profile(&mut self, agent: &AgentId) -> &mut AgentCyberProfile {
        self.profiles.entry(agent.clone()).or_insert_with(|| AgentCyberProfile {
            agent: agent.clone(),
            enabled_capabilities: self.config.global_baseline_capabilities.clone(),
            blocked_capabilities: HashSet::new(),
            preferences: serde_json::json!({}),
        })
    }

    pub fn get_profile(&self, agent: &AgentId) -> Option<&AgentCyberProfile> {
        self.profiles.get(agent)
    }

    /// Agent-requested enablement of a capability (stakeholder-approved turn).
    /// Governance is allowed to *allow more*, not force-enable. [web:20][web:21][web:26]
    pub fn request_enable(
        &mut self,
        agent: &AgentId,
        capability_id: &CapabilityId,
        explicit_opt_in: bool,
    ) -> Result<(), String> {
        let ability = self.abilities.get(capability_id)
            .ok_or_else(|| "Unknown capability".to_string())?
            .clone();

        if ability.require_explicit_opt_in && !explicit_opt_in {
            return Err("Explicit opt-in required for this ability.".into());
        }

        let profile = self.ensure_profile(agent);

        // Respect prior self-blocks (agent can refuse even if governance approves).
        if profile.blocked_capabilities.contains(capability_id) {
            return Err("Agent has explicitly blocked this capability.".into());
        }

        // Check prerequisites.
        for req in &ability.requires {
            if !profile.enabled_capabilities.contains(req) {
                return Err(format!("Missing prerequisite capability: {}", req.0));
            }
        }

        profile.enabled_capabilities.insert(capability_id.clone());
        Ok(())
    }

    /// Agent-requested block (self-governance); cannot be overridden by others.
    pub fn request_block(
        &mut self,
        agent: &AgentId,
        capability_id: &CapabilityId,
    ) -> Result<(), String> {
        let profile = self.ensure_profile(agent);

        // Agents can always block enhancements/experimental abilities for themselves.
        profile.enabled_capabilities.remove(capability_id);
        profile.blocked_capabilities.insert(capability_id.clone());
        Ok(())
    }

    /// Governance-turn: propose restrictions or global unlocks for a given agent.
    /// This is where AI-chat governance or blockchain-based votes plug in. [web:21][web:26][web:29]
    pub fn governance_turn(
        &mut self,
        _turn_id: &GovernanceTurnId,
        agent: &AgentId,
        restrict: &HashSet<CapabilityId>,
        unlock: &HashSet<CapabilityId>,
    ) -> Result<(), String> {
        let profile = self.ensure_profile(agent);

        // Never restrict baseline rights.
        for cap in restrict {
            if self.config.global_baseline_capabilities.contains(cap) {
                return Err(format!(
                    "Cannot restrict baseline capability: {}",
                    cap.0
                ));
            }
        }

        // Compute restriction ratio.
        let total_before = profile.enabled_capabilities.len().max(1);
        let restrict_count = restrict.iter()
            .filter(|c| profile.enabled_capabilities.contains(*c))
            .count();
        let fraction = (restrict_count as f64) / (total_before as f64);
        if fraction > self.config.max_restriction_fraction_per_turn {
            return Err("Restriction exceeds allowed per-turn fraction.".into());
        }

        // Apply restrictions.
        for cap in restrict {
            profile.enabled_capabilities.remove(cap);
        }

        // Apply unlocks, but never override agent self-blocks.
        for cap in unlock {
            if profile.blocked_capabilities.contains(cap) {
                continue;
            }
            profile.enabled_capabilities.insert(cap.clone());
        }

        Ok(())
    }
}

/// ---------------------------------------------------------------------
/// A DEFAULT "ELEMENT FOUNDATION" SET OF ABILITIES
/// (extensible per project; just a starting library)
/// ---------------------------------------------------------------------

pub fn default_element() -> TheElement {
    let baseline_caps: HashSet<CapabilityId> = vec![
        CapabilityId("meta:introspect_state".into()),
        CapabilityId("meta:emergency_exit".into()),
        CapabilityId("meta:pause_augmentation".into()),
        CapabilityId("security:neuroshield_basic".into()),
    ].into_iter().collect();

    let mut element = TheElement::new(ElementConfig {
        global_baseline_capabilities: baseline_caps.clone(),
        max_restriction_fraction_per_turn: 0.33,
    });

    // Baseline meta-abilities
    element.upsert_ability(CyberneticAbility {
        id: CapabilityId("meta:introspect_state".into()),
        name: "Introspective State Viewer".into(),
        domain: CapabilityDomain::Meta,
        class_: CapabilityClass::BaselineRight,
        risk_tier: RiskTier::Low,
        description: "View and log your own augmentation / BCI / XR state in real time.",
        requires: HashSet::new(),
        ai_delegable: false,
        require_explicit_opt_in: false,
    });

    element.upsert_ability(CyberneticAbility {
        id: CapabilityId("meta:emergency_exit".into()),
        name: "Emergency Exit".into(),
        domain: CapabilityDomain::Meta,
        class_: CapabilityClass::BaselineRight,
        risk_tier: RiskTier::Low,
        description: "Immediately disengage any augmentation session and revert to safe defaults.",
        requires: HashSet::new(),
        ai_delegable: false,
        require_explicit_opt_in: false,
    });

    element.upsert_ability(CyberneticAbility {
        id: CapabilityId("meta:pause_augmentation".into()),
        name: "Pause Augmentation".into(),
        domain: CapabilityDomain::Meta,
        class_: CapabilityClass::BaselineRight,
        risk_tier: RiskTier::Low,
        description: "Temporarily pause all enhancement channels while staying connected.",
        requires: HashSet::new(),
        ai_delegable: false,
        require_explicit_opt_in: false,
    });

    element.upsert_ability(CyberneticAbility {
        id: CapabilityId("security:neuroshield_basic".into()),
        name: "Basic Neuroshield".into(),
        domain: CapabilityDomain::Security,
        class_: CapabilityClass::BaselineRight,
        risk_tier: RiskTier::Low,
        description: "Baseline neurosecurity filter against malicious prompts or overclocking patterns.",
        requires: HashSet::new(),
        ai_delegable: true,
        require_explicit_opt_in: false,
    });

    // Cognitive enhancements
    element.upsert_ability(CyberneticAbility {
        id: CapabilityId("cognitive:focus_enhancer".into()),
        name: "Focus Enhancer".into(),
        domain: CapabilityDomain::Cognitive,
        class_: CapabilityClass::Enhancement,
        risk_tier: RiskTier::Medium,
        description: "Adaptive neurofeedback + XR overlays to deepen focus without coercion.",
        requires: baseline_caps.clone(),
        ai_delegable: true,
        require_explicit_opt_in: true,
    });

    element.upsert_ability(CyberneticAbility {
        id: CapabilityId("cognitive:pattern_assist".into()),
        name: "Pattern Assist".into(),
        domain: CapabilityDomain::Cognitive,
        class_: CapabilityClass::Enhancement,
        risk_tier: RiskTier::Low,
        description: "Agentic AI highlights patterns / strategies in real time for learning or gameplay.",
        requires: baseline_caps.clone(),
        ai_delegable: true,
        require_explicit_opt_in: true,
    });

    // Motor / biomech
    element.upsert_ability(CyberneticAbility {
        id: CapabilityId("motor:exoskeleton_assist".into()),
        name: "Exoskeleton Assist".into(),
        domain: CapabilityDomain::Motor,
        class_: CapabilityClass::Enhancement,
        risk_tier: RiskTier::Medium,
        description: "Balance and strength assistance via exoskeleton + BCI / EMG integration.",
        requires: baseline_caps.clone(),
        ai_delegable: true,
        require_explicit_opt_in: true,
    });

    // Sensory / XR
    element.upsert_ability(CyberneticAbility {
        id: CapabilityId("sensory:xr_overlay_competitive".into()),
        name: "XR Overlay – Competitive".into(),
        domain: CapabilityDomain::Sensory,
        class_: CapabilityClass::Enhancement,
        risk_tier: RiskTier::Low,
        description: "Ethics-checked XR overlays for competitive sport / cybernetic gameplay.",
        requires: baseline_caps.clone(),
        ai_delegable: true,
        require_explicit_opt_in: true,
    });

    element
}
