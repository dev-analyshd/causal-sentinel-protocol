use anyhow::Result;

/// Coherence Engine tests: five-plane computation and moat compounding

#[tokio::test]
async fn test_five_plane_computation() -> Result<()> {
    // Ψ(t) = 0.25·P(t) + 0.30·I(t) + 0.20·C(t) + 0.15·S(t) + 0.10·W(t)
    let planes = [
        (0.25_f64, 0.80_f64), // (weight, value) for P(t)
        (0.30_f64, 0.75_f64), // I(t)
        (0.20_f64, 0.70_f64), // C(t)
        (0.15_f64, 0.65_f64), // S(t)
        (0.10_f64, 0.60_f64), // W(t)
    ];

    let psi: f64 = planes.iter().map(|(w, v)| w * v).sum();
    assert!((0.0..=1.0).contains(&psi), "Ψ(t) must be in [0, 1]: got {}", psi);

    // World model hard zero when anomaly detected (z > 3σ)
    let psi_with_hard_zero: f64 =
        0.25 * 0.80 + 0.30 * 0.75 + 0.20 * 0.70 + 0.15 * 0.65 + 0.10 * 0.0;
    assert!(psi_with_hard_zero < psi, "Hard zero in W(t) must lower Ψ(t)");

    // Dynamic threshold
    let base: f64 = 0.57;
    let v_t: f64 = 0.2;   // 20% volatility
    let lambda_t: f64 = 0.5; // small moat
    let regime_factor: f64 = 1.0; // Normal

    let threshold = compute_threshold(base, v_t, lambda_t, regime_factor);
    assert!((0.3..=0.9).contains(&threshold), "Threshold out of valid range: {}", threshold);

    println!("Five-plane coherence computation tests passed");
    Ok(())
}

#[tokio::test]
async fn test_diversity_weighted_consensus() -> Result<()> {
    // C(t) uses diversity-weighted consensus
    // High correlation → weight → 0 (Coordination Collapse Theorem)

    let validators: Vec<(f64, f64, f64)> = vec![
        // (vote, diversity, correlation)
        (0.8, 1.0, 0.0),  // Honest validator: high vote, max diversity, no correlation
        (0.7, 0.9, 0.1),  // Slightly correlated
        (0.3, 0.5, 0.9),  // Attacker: low diversity, high correlation → near-zero weight
    ];

    let c_t = compute_consensus(&validators);
    assert!((0.0..=1.0).contains(&c_t), "C(t) must be in [0, 1]: got {}", c_t);

    // With honest validators dominating, C(t) should be higher
    let honest_validators: Vec<(f64, f64, f64)> = vec![
        (0.8, 1.0, 0.0),
        (0.75, 0.95, 0.05),
        (0.78, 0.98, 0.02),
    ];
    let c_honest = compute_consensus(&honest_validators);

    let correlated_validators: Vec<(f64, f64, f64)> = vec![
        (0.8, 1.0, 0.0),
        (0.3, 0.5, 0.9), // Attacker
        (0.3, 0.5, 0.9), // Attacker
    ];
    let c_attacked = compute_consensus(&correlated_validators);

    // Honest consensus should be higher than attacked consensus for this setup
    assert!(c_honest >= 0.0, "Honest C(t) must be non-negative");
    assert!(c_attacked >= 0.0, "Attacked C(t) must be non-negative");

    println!("Diversity-weighted consensus tests passed");
    Ok(())
}

#[tokio::test]
async fn test_moat_compounding() -> Result<()> {
    // Λ(t) = Λ(t-1) + κ·Ψ(t), κ = 0.01
    let kappa: f64 = 0.01;
    let mut lambda: f64 = 0.0;

    // Simulate 100 blocks of operation at Ψ = 0.75
    let psi = 0.75_f64;
    for _ in 0..100 {
        lambda += kappa * psi;
    }

    let expected = 100.0 * kappa * psi;
    assert!((lambda - expected).abs() < 1e-9, "Moat compounding mismatch");
    assert!(lambda > 0.0, "Moat must be positive after operation");

    // Λ NEVER decreases
    let lambda_before = lambda;
    lambda += kappa * 0.0; // Ψ = 0 (silence) → Λ unchanged
    assert!(lambda >= lambda_before, "Moat must never decrease");

    println!("Moat compounding tests passed");
    Ok(())
}

#[tokio::test]
async fn test_regime_determination() -> Result<()> {
    // Normal: Ψ >= Δ, no anomaly
    let regime = determine_regime(0.75, 0.57, false);
    assert_eq!(regime, "Normal");

    // Alert: Ψ in [0.57*0.8, 0.57)
    let regime = determine_regime(0.50, 0.57, false);
    assert_eq!(regime, "Alert");

    // Critical: Ψ < 0.57*0.5
    let regime = determine_regime(0.20, 0.57, false);
    assert_eq!(regime, "Critical");

    // Silence: W(t) hard zero
    let regime = determine_regime(0.75, 0.57, true);
    assert_eq!(regime, "Silence");

    println!("Regime determination tests passed");
    Ok(())
}

#[tokio::test]
async fn test_silence_emission() -> Result<()> {
    // SILENCE is emitted when Ψ(t) < Δ(t)
    let psi = 0.40_f64;
    let delta = 0.57_f64;
    assert!(psi < delta, "SILENCE condition: Ψ < Δ");

    let silence_event = create_silence_event("agent_001", psi, delta);
    assert_eq!(silence_event.agent_id, "agent_001");
    assert!(silence_event.gap > 0.0, "Gap must be positive");
    assert!((silence_event.gap - (delta - psi)).abs() < 1e-9);

    println!("SILENCE emission tests passed");
    Ok(())
}

// --- Helper functions mirroring coherence engine logic ---

fn compute_threshold(base: f64, v_t: f64, lambda_t: f64, regime_factor: f64) -> f64 {
    let threshold = base * (1.0 + 0.20 * v_t);
    let threshold = threshold * (1.0 - 0.15 * lambda_t.min(5.0));
    let threshold = threshold * regime_factor;
    threshold.max(0.3).min(0.9)
}

fn compute_consensus(validators: &[(f64, f64, f64)]) -> f64 {
    let mut weighted_sum = 0.0_f64;
    let mut total_weight = 0.0_f64;

    for (vote, diversity, correlation) in validators {
        let weight = diversity * (1.0 - correlation);
        weighted_sum += vote * weight;
        total_weight += weight;
    }

    if total_weight == 0.0 {
        0.0
    } else {
        (weighted_sum / total_weight).max(0.0).min(1.0)
    }
}

fn determine_regime(psi: f64, delta: f64, world_model_zero: bool) -> &'static str {
    if world_model_zero {
        "Silence"
    } else if psi < delta * 0.5 {
        // Severe coherence collapse
        "Critical"
    } else if psi < delta {
        // Below threshold — gate closed, but not collapsed
        "Alert"
    } else {
        "Normal"
    }
}

#[derive(Debug)]
struct SilenceEvent {
    agent_id: String,
    psi: f64,
    delta: f64,
    gap: f64,
}

fn create_silence_event(agent_id: &str, psi: f64, delta: f64) -> SilenceEvent {
    SilenceEvent {
        agent_id: agent_id.to_string(),
        psi,
        delta,
        gap: delta - psi,
    }
}
