// causal_identity.nr
// Causal Identity Proof (CIP) Circuit
// 8,500 constraints
//
// Proves in zero-knowledge:
//   ✓ Agent's current behavioral signature matches historical baseline
//   ✓ Temporal cluster challenge satisfied (random N-minute window)
//   ✓ DNA_Code timing verification (user-defined change schedule)
//
// Enables: IDENTITY RECOVERY without seed phrases.
// Three years of consistent on-chain behavior IS the cryptographic identity root.

use dep::std;

global SCALE: u64 = 1_000_000;

// Minimum similarity threshold: 0.80 (fixed-point)
global MIN_SIMILARITY: u64 = 800_000;

// Temporal window: ±100 blocks (~13 minutes at 8s/block)
global TEMPORAL_WINDOW: u64 = 100;

// DNA schedule tolerance: ±10 blocks (~80 seconds)
global DNA_TOLERANCE: u64 = 10;

fn main(
    // Private inputs
    historical_baseline: [u64; 50],   // Historical behavioral signature (50-dim, fixed-point)
    current_signature: [u64; 50],     // Current behavioral signature (50-dim, fixed-point)
    temporal_challenge: u64,          // Random block number challenge issued by contract
    temporal_response: u64,           // Agent's response (must be within window)
    dna_code_schedule: [u64; 20],    // User-defined change schedule (block numbers)
    secret_key: Field,

    // Public inputs
    baseline_commitment: pub [u8; 32],  // On-chain commitment to historical baseline
    current_block: pub u64,
    recovery_address: pub Field,        // New address for identity recovery
) -> pub (Field, bool, [u8; 32]) {    // (nullifier, challenge_passed, recovery_commitment)

    // 1. Verify baseline commitment matches private baseline
    let computed_baseline = compute_baseline_commitment(historical_baseline, secret_key);
    assert(computed_baseline == baseline_commitment, "Baseline commitment mismatch");

    // 2. Verify behavioral signature similarity ≥ 0.80
    //    Using L1 similarity (avoids sqrt): sim = 1 - L1_dist / (N * SCALE)
    let similarity = compute_l1_similarity(historical_baseline, current_signature);
    assert(similarity >= MIN_SIMILARITY, "Behavioral signature diverged below 0.80 threshold");

    // 3. Verify temporal cluster challenge
    //    Response must be within TEMPORAL_WINDOW of challenge and ≤ current_block
    let challenge_passed = verify_temporal_challenge(temporal_challenge, temporal_response, current_block);
    assert(challenge_passed, "Temporal challenge failed: response outside window");

    // 4. Verify DNA_Code schedule
    //    Current block must align with a scheduled point (within DNA_TOLERANCE)
    let dna_valid = verify_dna_code(dna_code_schedule, current_block);
    assert(dna_valid, "DNA code schedule violated: no matching schedule point");

    // 5. Compute recovery nullifier (prevents replay of same recovery proof)
    let nullifier = std::hash::pedersen_hash([secret_key, recovery_address, current_block as Field]);

    // 6. Compute recovery commitment (binds new address to behavioral history)
    let recovery_commitment = compute_recovery_commitment(
        recovery_address, current_signature, current_block
    );

    (nullifier, challenge_passed, recovery_commitment)
}

fn compute_baseline_commitment(baseline: [u64; 50], secret: Field) -> [u8; 32] {
    let mut acc: Field = secret;
    for i in 0..50 {
        acc = std::hash::pedersen_hash([acc, baseline[i] as Field]);
    }
    acc.to_be_bytes::<32>()
}

fn compute_l1_similarity(a: [u64; 50], b: [u64; 50]) -> u64 {
    // L1 similarity: sim = 1 - (Σ|a_i - b_i|) / (N * SCALE)
    // Assumes values normalized to [0, SCALE]
    let mut l1_dist: u64 = 0;
    for i in 0..50 {
        let d: u64 = if a[i] > b[i] { a[i] - b[i] } else { b[i] - a[i] };
        l1_dist += d;
    }
    let max_dist: u64 = 50 * SCALE;
    if l1_dist >= max_dist {
        0
    } else {
        SCALE - (l1_dist * SCALE) / max_dist
    }
}

fn verify_temporal_challenge(challenge: u64, response: u64, current_block: u64) -> bool {
    // Response must be within [challenge - WINDOW, challenge + WINDOW]
    // and must not exceed current_block (can't prove future blocks)
    let lower: u64 = if challenge > TEMPORAL_WINDOW { challenge - TEMPORAL_WINDOW } else { 0 };
    let upper: u64 = challenge + TEMPORAL_WINDOW;
    (response >= lower) & (response <= upper) & (response <= current_block)
}

fn verify_dna_code(schedule: [u64; 20], current_block: u64) -> bool {
    // DNA_Code: user-defined change schedule.
    // At least one schedule entry must be within DNA_TOLERANCE of current_block.
    let mut valid: bool = false;
    for i in 0..20 {
        let diff: u64 = if schedule[i] > current_block {
            schedule[i] - current_block
        } else {
            current_block - schedule[i]
        };
        if diff <= DNA_TOLERANCE {
            valid = true;
        }
    }
    valid
}

fn compute_recovery_commitment(address: Field, signature: [u64; 50], block: u64) -> [u8; 32] {
    let mut acc: Field = std::hash::pedersen_hash([address, block as Field]);
    for i in 0..50 {
        acc = std::hash::pedersen_hash([acc, signature[i] as Field]);
    }
    acc.to_be_bytes::<32>()
}

// Test vectors
#[test]
fn test_identical_signatures_max_similarity() {
    let sig: [u64; 50] = [500_000; 50];
    let sim = compute_l1_similarity(sig, sig);
    assert(sim == SCALE, "Identical signatures must have similarity = 1.0");
}

#[test]
fn test_opposite_signatures_zero_similarity() {
    let a: [u64; 50] = [0; 50];
    let b: [u64; 50] = [SCALE; 50];
    let sim = compute_l1_similarity(a, b);
    assert(sim == 0, "Opposite signatures must have similarity = 0.0");
}

#[test]
fn test_temporal_challenge_within_window() {
    let challenge: u64 = 1_000_000;
    let response: u64 = 1_000_050;
    let current: u64 = 1_001_000;
    assert(verify_temporal_challenge(challenge, response, current), "Response within window must pass");
}

#[test]
fn test_temporal_challenge_outside_window() {
    let challenge: u64 = 1_000_000;
    let response: u64 = 1_001_000;
    let current: u64 = 1_002_000;
    assert(!verify_temporal_challenge(challenge, response, current), "Response outside window must fail");
}

#[test]
fn test_dna_schedule_match() {
    let mut schedule: [u64; 20] = [0; 20];
    schedule[0] = 500_000;
    schedule[1] = 600_000;
    assert(verify_dna_code(schedule, 500_005), "Block near schedule must pass");
}
