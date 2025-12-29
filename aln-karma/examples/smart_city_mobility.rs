// path: aln-karma/examples/smart_city_mobility.rs

//! Example: smart-city mobility vNode computing AU.ET-priced, “karma-increasing” routes.
//! - Reads per-trip baseline + realized emissions
//! - Emits SafetyEpochManifests
//! - Derives non-mintable KarmaAllowance objects for AU.ET internal budgets. [web:4][web:5][web:6]

use aln_karma::{
    VNodeId, ImpactMetrics, BaselineModel, JusticeConstraints,
    SafetyEpochManifest, current_epoch_window,
};

fn main() {
    // vNode representing a city mobility controller.
    let vnode = VNodeId {
        vnode_id: "city:phoenix:traffic:controller-01".into(),
        policy_shard_id: "policy:aln:mobility:v1".into(),
    };

    // Conservative, additionality-certified baseline: SOV peak-hour driving. [web:1][web:4]
    let baseline = BaselineModel {
        description: "Phoenix SOV baseline, 2018–2020 average, peak hour".into(),
        additionality_certified: true,
        min_improvement_ratio: 0.05,
    };

    let justice = JusticeConstraints {
        forbid_burden_shifting: true,
        require_opt_out_respected: true,
    };

    // Example epoch aggregation for a 15-minute window. [web:4]
    let (epoch_start, epoch_end) = current_epoch_window(900);

    // In a real deployment, these come from vNode logs + grid factors:
    //  - baseline_t_co2e: modeled emissions for trips without ALN routing
    //  - realized_t_co2e: actual emissions with ALN routing applied
    let baseline_t_co2e = 12.5;
    let realized_t_co2e = 9.8;
    let t_co2e_avoided = (baseline_t_co2e - realized_t_co2e).max(0.0);

    let metrics = ImpactMetrics {
        t_co2e_avoided,
        kwh_reduced: 0.0,
        pollution_exposure_delta: -1_500.0, // negative => exposure reduced
        near_misses_blocked: 7,
        biosafety_delta: 0.12,
    };

    let manifest = SafetyEpochManifest::new(
        vnode,
        epoch_start,
        epoch_end,
        metrics,
        baseline,
        justice,
        "merkle-root-vnode-log-0xabc...".into(),
        vec![
            "city_sensors://phoenix/pm25".into(),
            "grid://srp/emissions_factors".into(),
        ],
        None,
    );

    // Convert to AU.ET karma allowance; no mint/transfer semantics. [web:0][web:3]
    let allowance = manifest.to_karma_allowance(
        None,
        10.0,  // AU.ET per tCO₂e
        0.01,  // AU.ET per kWh
        2.5,   // AU.ET per near-miss blocked
    );

    if let Some(allowance) = allowance {
        println!(
            "Epoch karma for {}: AU.ET Δ = {:.3}, tCO₂e avoided = {:.3}, near-misses = {}",
            allowance.vnode.vnode_id,
            allowance.au_et_delta,
            allowance.metrics.t_co2e_avoided,
            allowance.metrics.near_misses_blocked
        );
    } else {
        eprintln!("Manifest did not pass additionality/justice checks; no karma earned.");
    }
}
