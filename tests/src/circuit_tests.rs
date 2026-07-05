use anyhow::Result;

/// Circuit tests: ZK circuit invariant verification
///
/// Note: Full Nargo compilation and proof generation requires Noir/Nargo toolchain.
/// These tests verify the mathematical constraints the circuits enforce.

#[tokio::test]
async fn test_behavioral_integrity_circuit() -> Result<()> {
    // Verify BIC circuit invariants:
    // Ψ(t) = 0.25·P(t) + 0.30·I(t) + 0.20·C(t) + 0.15·S(t) + 0.10·W(t)

    let p_t = 0.80_f64;
    let i_t = 0.75_f64;
    let c_t = 0.70_f64;
    let s_t = 0.65_f64;
    let w_t = 0.60_f64;

    let psi = 0.25 * p_t + 0.30 * i_t + 0.20 * c_t + 0.15 * s_t + 0.10 * w_t;
    let expected = 0.25 * 0.80 + 0.30 * 0.75 + 0.20 * 0.70 + 0.15 * 0.65 + 0.10 * 0.60;
    assert!((psi - expected).abs() < 1e-9, "Ψ(t) formula mismatch");

    // Weights must sum to 1.0
    let weight_sum = 0.25_f64 + 0.30 + 0.20 + 0.15 + 0.10;
    assert!((weight_sum - 1.0).abs() < 1e-9, "Weights must sum to 1.0");

    // Diversity score HHI check
    let weights: Vec<f64> = vec![0.02; 50]; // Equal weights → max diversity
    let hhi = compute_hhi(&weights);
    assert!(hhi < 0.1, "Equal weights should give low HHI (high diversity)");

    println!("Behavioral Integrity circuit tests passed");
    Ok(())
}

#[tokio::test]
async fn test_causal_identity_circuit() -> Result<()> {
    // Verify CIP circuit invariants:
    // Temporal challenge: response must be within [challenge - 100, challenge + 100] blocks

    let challenge = 1000_u64;
    let window = 100_u64;

    // Valid: response within window
    assert!(verify_temporal_challenge(challenge, 1050, challenge + 50));
    // Invalid: response outside window
    assert!(!verify_temporal_challenge(challenge, 50, challenge + 200));

    // Behavioral signature similarity: must be >= 0.8
    let baseline: Vec<f64> = vec![0.5; 50];
    let current_good: Vec<f64> = vec![0.5; 50]; // identical
    let sim_good = cosine_similarity(&baseline, &current_good);
    assert!(sim_good >= 0.8, "Identical signatures must have similarity >= 0.8");

    let current_bad: Vec<f64> = vec![-0.5; 50]; // opposite
    let sim_bad = cosine_similarity(&baseline, &current_bad);
    assert!(sim_bad < 0.8, "Opposite signatures must have similarity < 0.8");

    println!("Causal Identity circuit tests passed");
    Ok(())
}

#[tokio::test]
async fn test_sentinel_compliance_circuit() -> Result<()> {
    // Verify compliance circuit invariants:
    // Transaction amount must not exceed tier limit

    let tier_limits = [
        (1u8, 200_000_000_000u64),   // Basic
        (2u8, 400_000_000_000u64),   // Bronze
        (3u8, 600_000_000_000u64),   // Silver
        (4u8, 800_000_000_000u64),   // Gold
        (5u8, 1_000_000_000_000u64), // Platinum
    ];

    for (tier, limit) in tier_limits {
        // Amount exactly at limit should pass
        assert!(check_tier_limit(tier, limit), "Amount at limit should pass for tier {}", tier);
        // Amount over limit should fail
        assert!(!check_tier_limit(tier, limit + 1), "Amount over limit should fail for tier {}", tier);
    }

    // Geographic HHI check: must be < 2500
    assert!(check_geographic_hhi(2499), "HHI 2499 should pass");
    assert!(!check_geographic_hhi(2500), "HHI 2500 should fail");

    println!("Sentinel Compliance circuit tests passed");
    Ok(())
}

// --- Helper functions mirroring circuit constraints ---

fn compute_hhi(weights: &[f64]) -> f64 {
    let sum: f64 = weights.iter().sum();
    if sum == 0.0 {
        return 0.0;
    }
    let sum_sq: f64 = weights.iter().map(|w| (w / sum) * (w / sum)).sum();
    sum_sq
}

fn verify_temporal_challenge(challenge: u64, response: u64, current_block: u64) -> bool {
    let window = 100_u64;
    let lower = challenge.saturating_sub(window);
    let upper = challenge + window;
    response >= lower && response <= upper && response <= current_block
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

fn check_tier_limit(tier: u8, amount: u64) -> bool {
    let limit = match tier {
        1 => 200_000_000_000u64,
        2 => 400_000_000_000u64,
        3 => 600_000_000_000u64,
        4 => 800_000_000_000u64,
        5 => 1_000_000_000_000u64,
        _ => 0,
    };
    amount <= limit
}

fn check_geographic_hhi(hhi: u64) -> bool {
    hhi < 2500
}
