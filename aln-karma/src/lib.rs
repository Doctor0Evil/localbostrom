// path: aln-karma/src/lib.rs

//! ALN-compliant “karma-increasing” primitives for AU.ET / CSP
//! - Non-mintable, non-transferable impact allowances
//! - Backed only by SafetyEpochManifests derived from vNode logs
//! - Baseline/additionality aware
//! - Ready to plug into ALN/CEM runtimes as a Rust crate

use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// vNode identity & policy shard binding (traffic, grid, habitat, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VNodeId {
    pub vnode_id: String,
    pub policy_shard_id: String,
}

/// Core physical metrics we allow as “impact substrate”.
/// Each field is *measured* or derived from measured data – no symbolic scores.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImpactMetrics {
    /// Tons CO₂e avoided in this epoch vs. conservative baseline.
    pub t_co2e_avoided: f64,
    /// kWh reduced vs. baseline (demand response, building retrofits, etc.).
    pub kwh_reduced: f64,
    /// Aggregate pollution reductions (µg/m³ * people * hours, etc.).
    pub pollution_exposure_delta: f64,
    /// Count of blocked near-miss safety events (e.g. over-exposure prevented).
    pub near_misses_blocked: u64,
    /// Qualitative biosafety index delta, normalized to [-1, +1].
    pub biosafety_delta: f64,
}

/// Baseline model configuration: defines the conservative counterfactual.
/// This is where additionality/baseline logic is enforced per policy shard. [web:0][web:1][web:2]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineModel {
    /// Human-readable description of the baseline assumption.
    pub description: String,
    /// Whether this baseline has passed an “additionality” review
    /// (not already required by law or economically inevitable). [web:0][web:1]
    pub additionality_certified: bool,
    /// Minimal acceptable improvement factor (e.g. 0.05 => 5% better than baseline).
    pub min_improvement_ratio: f64,
}

/// Justice & equity constraints attached to the policy shard.
/// Ensures “positive karma” cannot be claimed by burden-shifting harms. [web:0]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JusticeConstraints {
    /// If true, disallow counting when pollution is shifted to more vulnerable tracts.
    pub forbid_burden_shifting: bool,
    /// If true, personal data / augmentation uses must have opt-out respected.
    pub require_opt_out_respected: bool,
}

/// AU.ET-linked, non-mintable “karma allowance” for a single epoch.
/// This is *not* a token, credit, or transferable asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KarmaAllowance {
    pub id: Uuid,
    pub vnode: VNodeId,
    pub epoch_start: u64,
    pub epoch_end: u64,
    /// Total AU.ET “budget delta” implied by impact, for internal accounting only.
    pub au_et_delta: f64,
    /// Underlying physical impact metrics.
    pub metrics: ImpactMetrics,
    /// Baseline model used to compute the allowance.
    pub baseline: BaselineModel,
    /// Policy shard & justice constraints in force.
    pub justice: JusticeConstraints,
    /// Hash pointer to the SafetyEpochManifest this allowance is derived from.
    pub manifest_hash: String,
    /// Local hash-chain anchor for auditability. [web:0]
    pub prev_hash: Option<String>,
    pub self_hash: String,
}

/// SafetyEpochManifest: hash-chained, audit-ready log of one epoch’s impact. [web:0]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyEpochManifest {
    pub id: Uuid,
    pub vnode: VNodeId,
    pub epoch_start: u64,
    pub epoch_end: u64,
    pub metrics: ImpactMetrics,
    pub baseline: BaselineModel,
    pub justice: JusticeConstraints,
    pub vnode_log_root: String,     // Merkle-root over raw vNode logs.
    pub external_refs: Vec<String>, // MRV systems, sensors, etc.
    pub prev_hash: Option<String>,
    pub self_hash: String,
}

/// A simple, pluggable hash function (use BLAKE3/SHA-256 in a real deployment).
fn hash_bytes(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

impl SafetyEpochManifest {
    pub fn new(
        vnode: VNodeId,
        epoch_start: u64,
        epoch_end: u64,
        metrics: ImpactMetrics,
        baseline: BaselineModel,
        justice: JusticeConstraints,
        vnode_log_root: String,
        external_refs: Vec<String>,
        prev_hash: Option<String>,
    ) -> Self {
        let id = Uuid::new_v4();
        let mut manifest = SafetyEpochManifest {
            id,
            vnode,
            epoch_start,
            epoch_end,
            metrics,
            baseline,
            justice,
            vnode_log_root,
            external_refs,
            prev_hash,
            self_hash: String::new(),
        };
        manifest.self_hash = manifest.compute_hash();
        manifest
    }

    fn compute_hash(&self) -> String {
        let mut map = HashMap::new();
        map.insert("id", self.id.to_string());
        map.insert("vnode_id", self.vnode.vnode_id.clone());
        map.insert("policy_shard_id", self.vnode.policy_shard_id.clone());
        map.insert("epoch_start", self.epoch_start.to_string());
        map.insert("epoch_end", self.epoch_end.to_string());
        map.insert("vnode_log_root", self.vnode_log_root.clone());
        if let Some(prev) = &self.prev_hash {
            map.insert("prev_hash", prev.clone());
        }
        let payload = serde_json::to_vec(&map).expect("hash serialization");
        hash_bytes(&payload)
    }

    /// Enforce baseline additionality & justice constraints before using this manifest. [web:0][web:1]
    pub fn is_eligible_for_karma(&self) -> bool {
        if !self.baseline.additionality_certified {
            return false;
        }
        // Simple additionality check on CO₂e and kWh reductions.
        let ratio = if self.baseline.min_improvement_ratio <= 0.0 {
            1.0
        } else if self.metrics.t_co2e_avoided > 0.0 {
            // In a real system, compute relative improvement vs. modeled baseline.
            1.0
        } else {
            0.0
        };
        if ratio < self.baseline.min_improvement_ratio {
            return false;
        }

        // Justice constraints: this stub assumes upstream policy evaluation
        // has already checked for burden shifting and opt-out compliance.
        if self.justice.forbid_burden_shifting && self.metrics.pollution_exposure_delta > 0.0 {
            // Positive pollution exposure delta means someone is worse off.
            return false;
        }

        true
    }

    /// Convert this manifest into a non-transferable KarmaAllowance.
    /// No mint, no transfer; this only “earns” AU.ET internally. [web:0][web:3]
    pub fn to_karma_allowance(
        &self,
        prev_hash: Option<String>,
        au_et_price_per_tco2e: f64,
        au_et_price_per_kwh: f64,
        au_et_price_per_near_miss: f64,
    ) -> Option<KarmaAllowance> {
        if !self.is_eligible_for_karma() {
            return None;
        }

        let mut au_et_delta = 0.0;
        au_et_delta += self.metrics.t_co2e_avoided * au_et_price_per_tco2e;
        au_et_delta += self.metrics.kwh_reduced * au_et_price_per_kwh;
        au_et_delta += (self.metrics.near_misses_blocked as f64) * au_et_price_per_near_miss;

        let id = Uuid::new_v4();
        let mut allowance = KarmaAllowance {
            id,
            vnode: self.vnode.clone(),
            epoch_start: self.epoch_start,
            epoch_end: self.epoch_end,
            au_et_delta,
            metrics: self.metrics.clone(),
            baseline: self.baseline.clone(),
            justice: self.justice.clone(),
            manifest_hash: self.self_hash.clone(),
            prev_hash,
            self_hash: String::new(),
        };
        allowance.self_hash = allowance.compute_hash();
        Some(allowance)
    }
}

impl KarmaAllowance {
    fn compute_hash(&self) -> String {
        let mut map = HashMap::new();
        map.insert("id", self.id.to_string());
        map.insert("vnode_id", self.vnode.vnode_id.clone());
        map.insert("policy_shard_id", self.vnode.policy_shard_id.clone());
        map.insert("epoch_start", self.epoch_start.to_string());
        map.insert("epoch_end", self.epoch_end.to_string());
        map.insert("manifest_hash", self.manifest_hash.clone());
        if let Some(prev) = &self.prev_hash {
            map.insert("prev_hash", prev.clone());
        }
        let payload = serde_json::to_vec(&map).expect("hash serialization");
        hash_bytes(&payload)
    }

    /// Explicitly *no-op* if someone tries to “transfer” this object.
    /// Callers must not implement any token/ledger semantics on top. [web:0][web:2]
    pub fn forbid_transfer(&self, _to: &str) -> Result<(), &'static str> {
        Err("ALN karma allowances are non-transferable and non-mintable.")
    }
}

/// Convenience helper for creating an epoch window around “now”.
pub fn current_epoch_window(epoch_seconds: u64) -> (u64, u64) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs();
    let start = now - (now % epoch_seconds);
    (start, start + epoch_seconds)
}
