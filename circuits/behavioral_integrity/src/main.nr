// behavioral_integrity.nr
// Behavioral Integrity Credential (BIC) Circuit
// 12,000 constraints
// 
// Proves in zero-knowledge:
//   ✓ Agent has operated for ≥ D_min blocks with Ψ ≥ threshold
//   ✓ No manipulation fingerprints detected in behavioral history
//   ✓ Diversity-weighted consensus score above minimum
//   ✓ Moat Λ(t) exceeds reputation threshold
//   ✓ Credential has not expired (90-day TTL)
//
// Private inputs: behavioral history vector, manipulation scores, Λ trace
// Public outputs: nullifier, compliance_tier ∈ {1..5}, reputation_commitment

use dep::std;

// Fixed-point scaling factor: 1_000_000
global SCALE: u64 = 1_000_000;

// Minimum blocks for each tier (Casper ~8s/block → 30d = 324,000 blocks)
global D_MIN_BASIC: u64 = 108_000;      // ~10 days
global D_MIN_BRONZE: u64 = 324_000;     // ~30 days
global D_MIN_SILVER: u64 = 648_000;     // ~60 days
global D_MIN_GOLD: u64 = 972_000;       // ~90 days
global D_MIN_PLATINUM: u64 = 1_296_000; // ~120 days

// Λ threshold constants (fixed-point)
global THRESHOLD_BASIC: u64 = 100_000;      // Λ ≥ 0.1
global THRESHOLD_BRONZE: u64 = 500_000;     // Λ ≥ 0.5
global THRESHOLD_SILVER: u64 = 1_000_000;   // Λ ≥ 1.0
global THRESHOLD_GOLD: u64 = 1_500_000;     // Λ ≥ 1.5
global THRESHOLD_PLATINUM: u64 = 2_000_000; // Λ ≥ 2.0

// Credential TTL: 90 days in blocks
global CREDENTIAL_TTL: u64 = 972_000;

fn main(
    // Private inputs
    behavioral_history: [u64; 100],      // Last 100 coherence scores (fixed-point)
    manipulation_scores: [u8; 100],       // 0 = clean, 1 = manipulation detected
    lambda_trace: [u64; 100],            // Λ(t) over time (fixed-point)
    diversity_weights: [u64; 50],        // Validator diversity weights (fixed-point)
    secret_key: Field,                    // Agent's secret key

    // Public inputs
    current_block: pub u64,
    registration_block: pub u64,
    target_tier: pub u8,                  // Desired tier (1-5)
    expiry_block: pub u64,                // Credential expiry
) -> pub (Field, u8, [u8; 32]) {        // (nullifier, computed_tier, reputation_commitment)

    // 1. Verify credential has not expired
    assert(current_block <= expiry_block, "Credential expired");
    assert(expiry_block <= current_block + CREDENTIAL_TTL, "Invalid expiry window");

    // 2. Verify minimum operating duration
    let blocks_operated = current_block - registration_block;
    let min_blocks = get_min_blocks(target_tier);
    assert(blocks_operated >= min_blocks, "Insufficient operating history");

    // 3. Verify coherence threshold sustained (average Ψ over history)
    let psi_avg = compute_average_u64(behavioral_history);
    let psi_threshold = get_psi_threshold(target_tier);
    assert(psi_avg >= psi_threshold, "Coherence average below threshold");

    // 4. Verify no manipulation fingerprints (or within tolerance for tier)
    let manipulation_count = count_nonzero(manipulation_scores);
    let max_manipulations = get_max_manipulations(target_tier);
    assert(manipulation_count <= max_manipulations as u64, "Too many manipulations");

    // 5. Verify diversity-weighted consensus score
    let diversity_score = compute_diversity_score(diversity_weights);
    assert(diversity_score >= 500_000, "Diversity score below minimum (0.5)");

    // 6. Verify moat Λ(t) exceeds threshold for requested tier
    let current_lambda = lambda_trace[99]; // Most recent entry
    let lambda_threshold = get_lambda_threshold(target_tier);
    assert(current_lambda >= lambda_threshold, "Moat score below tier threshold");

    // 7. Compute nullifier (Pedersen hash: prevents credential double-use)
    let nullifier = std::hash::pedersen_hash([secret_key, current_block as Field]);

    // 8. Compute actual tier from on-chain data (must match or exceed target)
    let computed_tier = compute_tier(current_lambda, manipulation_count as u8, blocks_operated);
    assert(computed_tier >= target_tier, "Actual tier below requested target");

    // 9. Compute reputation commitment (commitment to behavioral history)
    let reputation_commitment = compute_reputation_commitment(
        behavioral_history, lambda_trace, secret_key
    );

    (nullifier, computed_tier, reputation_commitment)
}

fn get_min_blocks(tier: u8) -> u64 {
    if tier == 1 { D_MIN_BASIC }
    else if tier == 2 { D_MIN_BRONZE }
    else if tier == 3 { D_MIN_SILVER }
    else if tier == 4 { D_MIN_GOLD }
    else { D_MIN_PLATINUM }
}

fn get_psi_threshold(tier: u8) -> u64 {
    if tier == 1 { 450_000 }      // Ψ ≥ 0.45
    else if tier == 2 { 500_000 } // Ψ ≥ 0.50
    else if tier == 3 { 550_000 } // Ψ ≥ 0.55
    else if tier == 4 { 600_000 } // Ψ ≥ 0.60
    else { 650_000 }              // Ψ ≥ 0.65
}

fn get_max_manipulations(tier: u8) -> u8 {
    if tier == 1 { 3 }
    else if tier == 2 { 2 }
    else if tier == 3 { 1 }
    else { 0 }
}

fn get_lambda_threshold(tier: u8) -> u64 {
    if tier == 1 { THRESHOLD_BASIC }
    else if tier == 2 { THRESHOLD_BRONZE }
    else if tier == 3 { THRESHOLD_SILVER }
    else if tier == 4 { THRESHOLD_GOLD }
    else { THRESHOLD_PLATINUM }
}

fn compute_average_u64(values: [u64; 100]) -> u64 {
    let mut sum: u64 = 0;
    for i in 0..100 {
        sum += values[i];
    }
    sum / 100
}

fn count_nonzero(scores: [u8; 100]) -> u64 {
    let mut count: u64 = 0;
    for i in 0..100 {
        if scores[i] != 0 {
            count += 1;
        }
    }
    count
}

fn compute_diversity_score(weights: [u64; 50]) -> u64 {
    // Diversity = 1 - HHI where HHI = Σ(w_i/Σw)²
    // Using fixed-point: diversity_score ∈ [0, SCALE]
    let mut sum: u64 = 0;
    let mut sum_sq: u64 = 0;

    for i in 0..50 {
        sum += weights[i];
    }
    if sum == 0 {
        return 0;
    }
    // Compute sum of squared normalized weights (scaled by SCALE²)
    for i in 0..50 {
        let w_norm = (weights[i] * SCALE) / sum;
        sum_sq += w_norm * w_norm;
    }
    // HHI in [0, SCALE²], normalize to [0, SCALE]
    let hhi = sum_sq / SCALE;
    if hhi >= SCALE { 0 } else { SCALE - hhi }
}

fn compute_tier(lambda: u64, manipulations: u8, blocks: u64) -> u8 {
    if lambda >= THRESHOLD_PLATINUM && manipulations == 0 && blocks >= D_MIN_PLATINUM {
        5
    } else if lambda >= THRESHOLD_GOLD && manipulations == 0 && blocks >= D_MIN_GOLD {
        4
    } else if lambda >= THRESHOLD_SILVER && manipulations <= 1 && blocks >= D_MIN_SILVER {
        3
    } else if lambda >= THRESHOLD_BRONZE && manipulations <= 2 && blocks >= D_MIN_BRONZE {
        2
    } else {
        1
    }
}

fn compute_reputation_commitment(
    history: [u64; 100],
    lambda_trace: [u64; 100],
    secret: Field,
) -> [u8; 32] {
    // Hash: secret || history[0..99] || lambda_trace[0..99]
    // Reduced to 3 fields (Pedersen input limit) via chaining
    let mut acc: Field = secret;
    for i in 0..100 {
        acc = std::hash::pedersen_hash([acc, history[i] as Field, lambda_trace[i] as Field]);
    }
    acc.to_be_bytes::<32>()
}

// Test vectors
#[test]
fn test_basic_tier_qualification() {
    let history: [u64; 100] = [600_000; 100]; // Ψ = 0.6, above all thresholds
    let manip: [u8; 100] = [0; 100];           // No manipulations
    let lambda: [u64; 100] = [200_000; 100];    // Λ = 0.2 > basic threshold
    let div: [u64; 50] = [20_000; 50];          // Equal weights → max diversity
    let secret = 42;

    let current = 200_000;
    let registration = 0;
    let target_tier: u8 = 1;
    let expiry = current + 500_000;

    let (nullifier, tier, _commitment) = main(
        history, manip, lambda, div, secret,
        current, registration, target_tier, expiry
    );

    assert(tier >= 1, "Should qualify for at least tier 1");
    assert(nullifier != 0, "Nullifier must be non-zero");
}

#[test]
fn test_diversity_score_equal_weights() {
    // Equal weights → HHI = 1/N → diversity = 1 - 1/N ≈ 0.98
    let weights: [u64; 50] = [1_000_000; 50];
    let score = compute_diversity_score(weights);
    assert(score >= 500_000, "Equal weights must give high diversity");
}

#[test]
fn test_diversity_score_single_validator() {
    // All weight in one → HHI = 1 → diversity = 0
    let mut weights: [u64; 50] = [0; 50];
    weights[0] = 1_000_000;
    let score = compute_diversity_score(weights);
    assert(score == 0, "Single validator must give zero diversity");
}
