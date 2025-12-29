// path: planetary_stewardship_runtime/src/lib.rs

//! Runtime kernel for `planetary_stewardship` ecosystem.SAI
//! - Enforces SAEP ethics, KSCP consent, and karma-safe attestations.
//! - Provides hooks for AI-chat governance-turns and automation loops
//!   without allowing restrictive / extractive policy overreach.
//!
//! This crate is designed to sit under ALN / XR / BCI / biomechanical
//! modules as a shared policy + attestation engine. [web:6][web:11][web:17]

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

/// ---------------------------------------------------------------------
/// CORE IDS / ENUMS
/// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MissionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttestationId(pub String);

/// Core modules enumerated for binding enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StewardModule {
    PLGA,
    MME,
    VET,
    OCG,
    DCCN,
    REBL,
    PSM,
    CSC,
}

/// ---------------------------------------------------------------------
/// ETHICS KERNEL: SAEP
/// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsContext {
    pub actor: Did,
    pub affected_parties: Vec<Did>,
    pub module: StewardModule,
    pub description: String,
    pub estimated_impact: serde_json::Value, // arbitrary impact model JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsDecision {
    pub allowed: bool,
    pub reasons: Vec<String>,
    pub require_rollback_plan: bool,
    pub require_public_intent_log: bool,
    pub require_consent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaepConfig {
    pub enforce_non_harm: bool,
    pub enforce_transparency: bool,
    pub enforce_reversibility: bool,
    pub enforce_informed_consent: bool,
    pub enforce_commons_benefit: bool,
    // Karma safety.
    pub forbid_punitive_scoring: bool,
}

impl Default for SaepConfig {
    fn default() -> Self {
        SaepConfig {
            enforce_non_harm: true,
            enforce_transparency: true,
            enforce_reversibility: true,
            enforce_informed_consent: true,
            enforce_commons_benefit: true,
            forbid_punitive_scoring: true,
        }
    }
}

/// Ethics engine: in practice you plug your risk models in here.[web:17]
pub struct SaepEngine {
    config: SaepConfig,
}

impl SaepEngine {
    pub fn new(config: SaepConfig) -> Self {
        Self { config }
    }

    /// Evaluate a proposed action in any module (missions, simulations, guild ops, etc.).
    pub fn evaluate(&self, ctx: &EthicsContext) -> EthicsDecision {
        let mut allowed = true;
        let mut reasons = Vec::new();
        let mut require_rollback_plan = false;
        let mut require_public_intent_log = false;
        let mut require_consent = false;

        if self.config.enforce_non_harm {
            // Placeholder: wire real risk analysis models here (e.g., env harm, psych load).
            let maybe_risky = ctx.description.to_lowercase().contains("weapon")
                || ctx.description.to_lowercase().contains("coercive");
            if maybe_risky {
                allowed = false;
                reasons.push("non_harm: detected potential harmful or coercive intent".into());
            }
        }

        if self.config.enforce_transparency {
            require_public_intent_log = true;
        }

        if self.config.enforce_reversibility {
            require_rollback_plan = true;
        }

        if self.config.enforce_informed_consent {
            require_consent = true;
        }

        if self.config.enforce_commons_benefit {
            // Block explicit private-hoarding keywords.
            if ctx.description.to_lowercase().contains("exclusive monetization") {
                allowed = false;
                reasons.push("commons_benefit: private hoarding flagged".into());
            }
        }

        EthicsDecision {
            allowed,
            reasons,
            require_rollback_plan,
            require_public_intent_log,
            require_consent,
        }
    }
}

/// ---------------------------------------------------------------------
/// CONSENT: KSCP
/// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub participant: Did,
    pub module: StewardModule,
    pub mission: Option<MissionId>,
    pub consent_given: bool,
    pub timestamp_ms: u64,
    pub evidence_uri: Option<String>,
}

pub struct ConsentRegistry {
    records: HashMap<(Did, StewardModule, Option<MissionId>), ConsentRecord>,
}

impl ConsentRegistry {
    pub fn new() -> Self {
        Self { records: HashMap::new() }
    }

    pub fn upsert_consent(&mut self, record: ConsentRecord) {
        let key = (record.participant.clone(), record.module, record.mission.clone());
        self.records.insert(key, record);
    }

    pub fn has_valid_consent(&self, did: &Did, module: StewardModule, mission: Option<&MissionId>) -> bool {
        let key = (did.clone(), module, mission.cloned());
        self.records
            .get(&key)
            .map(|r| r.consent_given)
            .unwrap_or(false)
    }
}

/// ---------------------------------------------------------------------
/// PLANETARY LEDGER OF GOOD ACTIONS (PLGA) – NON-COMPETITIVE ATTESTATIONS
/// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactMetrics {
    pub co2eq_reduced: f64,
    pub biodiversity_index_delta: f64,
    pub restored_area_m2: f64,
    pub avoided_emissions_co2eq: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StewardshipAttestation {
    pub id: AttestationId,
    pub actor_did: Did,
    pub mission_id: Option<MissionId>,
    pub timestamp_ms: u64,
    pub description: String,
    pub impact_metrics: ImpactMetrics,
    pub evidence_uri: String,
    pub verifier_dids: Vec<Did>,
    /// Non-transferable, non-speculative “badge” view.
    pub visible_symbol: String, // "STWD"
}

pub struct PlanetaryLedger {
    saep: SaepEngine,
    consent: ConsentRegistry,
    attestations: HashMap<AttestationId, StewardshipAttestation>,
}

impl PlanetaryLedger {
    pub fn new(saep: SaepEngine, consent: ConsentRegistry) -> Self {
        Self {
            saep,
            consent,
            attestations: HashMap::new(),
        }
    }

    /// Karma-safe: no scores, no ranks, just per-actor, per-mission attestations.[web:16]
    pub fn issue_attestation(
        &mut self,
        actor_did: Did,
        mission_id: Option<MissionId>,
        description: String,
        impact_metrics: ImpactMetrics,
        evidence_uri: String,
        verifier_dids: Vec<Did>,
        timestamp_ms: u64,
    ) -> Result<StewardshipAttestation, String> {
        let ctx = EthicsContext {
            actor: actor_did.clone(),
            affected_parties: vec![],
            module: StewardModule::PLGA,
            description: description.clone(),
            estimated_impact: serde_json::json!({
                "co2eq_reduced": impact_metrics.co2eq_reduced,
                "biodiversity_index_delta": impact_metrics.biodiversity_index_delta,
            }),
        };

        let decision = self.saep.evaluate(&ctx);
        if !decision.allowed {
            return Err(format!("SAEP blocked attestation: {:?}", decision.reasons));
        }

        // KSCP: require explicit consent for logging under PLGA.
        if decision.require_consent &&
            !self.consent.has_valid_consent(&actor_did, StewardModule::PLGA, mission_id.as_ref())
        {
            return Err("No valid KSCP consent for PLGA attestation".into());
        }

        let att_id = AttestationId(uuid::Uuid::new_v4().to_string());
        let att = StewardshipAttestation {
            id: att_id.clone(),
            actor_did,
            mission_id,
            timestamp_ms,
            description,
            impact_metrics,
            evidence_uri,
            verifier_dids,
            visible_symbol: "STWD".into(),
        };

        self.attestations.insert(att_id.clone(), att.clone());
        Ok(att)
    }

    pub fn get_attestations_for_actor(&self, actor: &Did) -> Vec<&StewardshipAttestation> {
        self.attestations
            .values()
            .filter(|a| &a.actor_did == actor)
            .collect()
    }

    /// No transfer operation by design.
    pub fn forbid_transfer(&self, _attestation_id: &AttestationId, _to: &Did) -> Result<(), String> {
        Err("Stewardship attestations are non-transferable and non-speculative by design.")
    }
}

/// ---------------------------------------------------------------------
/// MICRO-MISSIONS ENGINE (MME) – WITH ETHICS + CONSENT CHECKS
/// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTemplate {
    pub id: MissionId,
    pub title: String,
    pub description: String, // markdown
    pub difficulty: String,  // XS,S,M,L,XL
    pub expected_impact: serde_json::Value,
    pub location_hint: String, // "geo" or "virtual"
    pub required_skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignedMission {
    pub mission: MissionTemplate,
    pub assignee: Did,
    pub assigned_ts_ms: u64,
}

pub struct MicroMissionsEngine {
    saep: SaepEngine,
    consent: ConsentRegistry,
    templates: HashMap<MissionId, MissionTemplate>,
    active_assignments: Vec<AssignedMission>,
}

impl MicroMissionsEngine {
    pub fn new(saep: SaepEngine, consent: ConsentRegistry) -> Self {
        Self {
            saep,
            consent,
            templates: HashMap::new(),
            active_assignments: Vec::new(),
        }
    }

    pub fn add_template(&mut self, tpl: MissionTemplate) {
        self.templates.insert(tpl.id.clone(), tpl);
    }

    /// “Agentic-RAG” placeholder: real system uses profiles + local context. [web:6][web:11]
    pub fn assign_mission(
        &mut self,
        mission_id: &MissionId,
        assignee: Did,
        now_ms: u64,
    ) -> Result<AssignedMission, String> {
        let tpl = self.templates.get(mission_id)
            .ok_or_else(|| "Unknown mission template".to_string())?
            .clone();

        let ctx = EthicsContext {
            actor: assignee.clone(),
            affected_parties: vec![],
            module: StewardModule::MME,
            description: tpl.description.clone(),
            estimated_impact: tpl.expected_impact.clone(),
        };

        let decision = self.saep.evaluate(&ctx);
        if !decision.allowed {
            return Err(format!("SAEP blocked mission assignment: {:?}", decision.reasons));
        }

        if decision.require_consent &&
            !self.consent.has_valid_consent(&assignee, StewardModule::MME, Some(mission_id))
        {
            return Err("No valid KSCP consent for mission assignment".into());
        }

        let assigned = AssignedMission {
            mission: tpl,
            assignee,
            assigned_ts_ms: now_ms,
        };
        self.active_assignments.push(assigned.clone());
        Ok(assigned)
    }
}

/// ---------------------------------------------------------------------
/// GOVERNANCE HOOKS – POLYCENTRIC + QUADRATIC CONSENSUS
/// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceScope {
    Module(ModuleId),
    EcosystemWide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub proposal_id: String,
    pub scope: GovernanceScope,
    pub title: String,
    pub description: String,
    /// JSON patch or domain-specific payload (e.g., config changes).
    pub payload: serde_json::Value,
    /// Only ethics-kernel-triggered veto allowed in spec.
    pub can_introduce_restrictions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadraticVote {
    pub voter: Did,
    /// cost^2 relationship modeled off-chain/on-chain; store effective weight here. [web:15][web:18]
    pub effective_weight: f64,
    pub support: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadraticOutcome {
    pub proposal_id: String,
    pub total_support: f64,
    pub total_opposition: f64,
}

pub struct GovernanceEngine {
    saep: SaepEngine,
    /// modules bound to co-stewardship charter; they may not be weaponized. [web:16]
    charter_bound_modules: HashSet<StewardModule>,
}

impl GovernanceEngine {
    pub fn new(saep: SaepEngine) -> Self {
        let mut bound = HashSet::new();
        bound.insert(StewardModule::PLGA);
        bound.insert(StewardModule::MME);
        bound.insert(StewardModule::VET);
        bound.insert(StewardModule::OCG);
        bound.insert(StewardModule::DCCN);
        bound.insert(StewardModule::REBL);
        bound.insert(StewardModule::PSM);

        Self {
            saep,
            charter_bound_modules: bound,
        }
    }

    pub fn tally_quadratic(&self, proposal_id: &str, votes: &[QuadraticVote]) -> QuadraticOutcome {
        let mut support = 0.0;
        let mut oppose = 0.0;
        for v in votes {
            if v.support {
                support += v.effective_weight;
            } else {
                oppose += v.effective_weight;
            }
        }
        QuadraticOutcome {
            proposal_id: proposal_id.into(),
            total_support: support,
            total_opposition: oppose,
        }
    }

    /// Core guard: even if governance supports a proposal, SAEP + charter must pass.
    pub fn can_apply_proposal(
        &self,
        proposal: &GovernanceProposal,
        outcome: &QuadraticOutcome,
    ) -> Result<bool, String> {
        // Basic quadratic consensus heuristic.
        if outcome.total_support <= outcome.total_opposition {
            return Ok(false);
        }

        // Apply SAEP to the governance action itself.
        let module = match &proposal.scope {
            GovernanceScope::Module(mid) => {
                match mid.0.as_str() {
                    "PLGA" => StewardModule::PLGA,
                    "MME" => StewardModule::MME,
                    "VET" => StewardModule::VET,
                    "OCG" => StewardModule::OCG,
                    "DCCN" => StewardModule::DCCN,
                    "REBL" => StewardModule::REBL,
                    "PSM" => StewardModule::PSM,
                    "CSC" => StewardModule::CSC,
                    _      => StewardModule::CSC,
                }
            }
            GovernanceScope::EcosystemWide => StewardModule::CSC,
        };

        let ctx = EthicsContext {
            actor: Did("did:psv:governance:collective".into()),
            affected_parties: vec![],
            module,
            description: proposal.description.clone(),
            estimated_impact: proposal.payload.clone(),
        };

        let decision = self.saep.evaluate(&ctx);
        if !decision.allowed {
            // This is your “ethics-kernel-triggered veto” – no human kingmaking. [web:18]
            return Err(format!("Ethics-kernel vetoed governance proposal: {:?}", decision.reasons));
        }

        // Co-stewardship charter binding: no weaponization or extractive shifts. [web:16]
        if proposal.can_introduce_restrictions && self.charter_bound_modules.contains(&module) {
            // Require that payload explicitly documents non-military, non-extractive use.
            let text = proposal.description.to_lowercase();
            if text.contains("weapon") || text.contains("military") {
                return Err("CSC: disallows militarization or harmful use in charter-bound modules.".into());
            }
        }

        Ok(true)
    }
}
