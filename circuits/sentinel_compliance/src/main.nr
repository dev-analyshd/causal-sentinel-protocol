// sentinel_compliance.nr
// Sentinel Compliance Tier Enforcement Circuit
// 6,000 constraints
//
// Enforces compliance tier limits in zero-knowledge:
//   ✓ Transaction amount within tier limit
//   ✓ Agent credential tier valid and not expired
//   ✓ Jurisdiction restrictions satisfied
//   ✓ Geographic HHI within acceptable bounds (decentralization check)

use dep::std;

global SCALE: u64 = 1_000_000;

// Tier limits in CSPR motes (testnet values; mainnet uses $-denominated limits)
global LIMIT_BASIC: u64 = 200_000_000_000;      // ~200,000 CSPR equivalent
global LIMIT_BRONZE: u64 = 400_000_000_000;
global LIMIT_SILVER: u64 = 600_000_000_000;
global LIMIT_GOLD: u64 = 800_000_000_000;
global LIMIT_PLATINUM: u64 = 1_000_000_000_000;

// Geographic HHI: < 2500 = healthy decentralization
global MAX_HHI: u64 = 2500;

fn main(
    // Private inputs
    agent_secret: Field,
    credential_expiry: u64,

    // Public inputs
    transaction_amount: pub u64,
    agent_tier: pub u8,
    current_block: pub u64,
    jurisdiction_code: pub u8,   // 1=US, 2=EU, 3=SG, 0=unrestricted
    geographic_hhi: pub u64,     // 0-10000 (Herfindahl-Hirschman Index × 10000)
) -> pub (Field, bool, u64) {   // (nullifier, compliant, allowed_amount)

    // 1. Verify credential is not expired
    assert(current_block <= credential_expiry, "Credential expired");

    // 2. Verify tier is valid (1-5)
    assert(agent_tier >= 1, "Tier must be at least 1");
    assert(agent_tier <= 5, "Tier cannot exceed 5");

    // 3. Lookup tier limit
    let tier_limit = get_tier_limit(agent_tier);

    // 4. Verify transaction within tier limit
    assert(transaction_amount <= tier_limit, "Transaction exceeds tier limit");

    // 5. Verify jurisdiction restrictions
    let jurisdiction_ok = verify_jurisdiction(jurisdiction_code, agent_tier);
    assert(jurisdiction_ok, "Jurisdiction restriction violated for this tier");

    // 6. Verify geographic decentralization (HHI < 2500 = healthy network)
    assert(geographic_hhi < MAX_HHI, "Network geographic centralization too high");

    // 7. Compute nullifier (binds secret + block + amount → unique proof)
    let nullifier = std::hash::pedersen_hash([
        agent_secret,
        current_block as Field,
        transaction_amount as Field
    ]);

    // 8. Allowed amount = transaction_amount (already verified within limit)
    (nullifier, true, transaction_amount)
}

fn get_tier_limit(tier: u8) -> u64 {
    if tier == 1 { LIMIT_BASIC }
    else if tier == 2 { LIMIT_BRONZE }
    else if tier == 3 { LIMIT_SILVER }
    else if tier == 4 { LIMIT_GOLD }
    else { LIMIT_PLATINUM }
}

fn verify_jurisdiction(code: u8, tier: u8) -> bool {
    // US (code=1): Basic tier (1) requires enhanced KYC - restricted here
    // All other jurisdictions allow all tiers
    // In production: full jurisdiction matrix with MiCA, SEC, MAS rules
    if (tier == 1) & (code == 1) {
        false  // US requires at least Bronze for regulated operations
    } else {
        true
    }
}

// Test vectors
#[test]
fn test_basic_tier_within_limit() {
    let limit = get_tier_limit(1);
    assert(limit == LIMIT_BASIC, "Basic tier limit mismatch");
    assert(LIMIT_BASIC < LIMIT_BRONZE, "Basic < Bronze");
}

#[test]
fn test_platinum_tier_max_limit() {
    let limit = get_tier_limit(5);
    assert(limit == LIMIT_PLATINUM, "Platinum tier limit mismatch");
}

#[test]
fn test_us_jurisdiction_rejects_basic() {
    assert(!verify_jurisdiction(1, 1), "US must reject Basic tier");
}

#[test]
fn test_us_jurisdiction_allows_silver() {
    assert(verify_jurisdiction(1, 3), "US must allow Silver tier");
}

#[test]
fn test_eu_jurisdiction_allows_basic() {
    assert(verify_jurisdiction(2, 1), "EU allows Basic tier");
}
