# ZK Circuit Documentation

Causal Sentinel Protocol uses three Noir circuits targeting Barretenberg's
UltraHonk proof system for WASM-native verification on Casper.

Total constraints: **26,500** across all three circuits.

---

## Circuit 1: Behavioral Integrity Credential (BIC)

**File:** `circuits/behavioral_integrity/src/main.nr`  
**Constraints:** ~12,000  
**Proving time (estimated):** ~2–4 seconds on modern hardware

### Purpose

Proves in zero-knowledge that an agent has:
- Operated for ≥ D_min blocks with Ψ ≥ threshold (sustained coherence)
- No manipulation fingerprints beyond tier tolerance
- Diversity-weighted consensus score ≥ 0.5 (Coordination Collapse protection)
- Moat Λ(t) exceeding the reputation threshold for the requested tier
- Valid credential window (90-day TTL)

### Private Inputs

| Field | Type | Description |
|-------|------|-------------|
| `behavioral_history` | `[u64; 100]` | Last 100 coherence scores Ψ (fixed-point ×10⁶) |
| `manipulation_scores` | `[u8; 100]` | 0=clean, 1=manipulation detected per window |
| `lambda_trace` | `[u64; 100]` | Λ(t) trace over 100 evaluation cycles |
| `diversity_weights` | `[u64; 50]` | Validator diversity weights (fixed-point ×10⁶) |
| `secret_key` | `Field` | Agent's secret key (BN254 field element) |

### Public Inputs

| Field | Type | Description |
|-------|------|-------------|
| `current_block` | `u64` | Current Casper block height |
| `registration_block` | `u64` | Block agent registered (determines age) |
| `target_tier` | `u8` | Requested compliance tier (1–5) |
| `expiry_block` | `u64` | Credential expiry (≤ current + 972,000) |

### Public Outputs

| Field | Type | Description |
|-------|------|-------------|
| `nullifier` | `Field` | Pedersen hash(secret, block) — prevents double-use |
| `computed_tier` | `u8` | Verified tier ∈ {1..5} |
| `reputation_commitment` | `[u8; 32]` | Commitment to behavioral history |

### Compliance Tiers

| Tier | Name | Λ min | Max manipulations | Min history | Limit |
|------|------|-------|-------------------|-------------|-------|
| 1 | Basic | 0.1 | 3 | 10 days | 200K CSPR |
| 2 | Bronze | 0.5 | 2 | 30 days | 400K CSPR |
| 3 | Silver | 1.0 | 1 | 60 days | 600K CSPR |
| 4 | Gold | 1.5 | 0 | 90 days | 800K CSPR |
| 5 | Platinum | 2.0 | 0 | 120 days | 1M CSPR |

### Key Constraint: Diversity Score

The diversity score prevents Coordination Collapse (see Proposal §Novel):
```
diversity = 1 - HHI
HHI = Σ(w_i / Σw)²
```
When agents coordinate, their weight correlation increases → HHI → 1 → diversity → 0 → circuit rejects.  
Byzantine coordination is **structurally self-defeating**.

---

## Circuit 2: Causal Identity Proof (CIP)

**File:** `circuits/causal_identity/src/main.nr`  
**Constraints:** ~8,500  
**Proving time (estimated):** ~1–2 seconds

### Purpose

Enables **identity recovery without seed phrases**. Proves that the agent's
current behavioral signature matches their historical baseline — using three
years of on-chain behavioral history as the cryptographic identity root.

"What you have lived" replaces "what you know."

### Private Inputs

| Field | Type | Description |
|-------|------|-------------|
| `historical_baseline` | `[u64; 50]` | Historical behavioral signature (50-dim) |
| `current_signature` | `[u64; 50]` | Current behavioral signature (50-dim) |
| `temporal_challenge` | `u64` | Random block number issued by contract |
| `temporal_response` | `u64` | Agent's response (must be within ±100 blocks) |
| `dna_code_schedule` | `[u64; 20]` | User-defined change schedule |
| `secret_key` | `Field` | Agent's private key |

### Public Inputs

| Field | Type | Description |
|-------|------|-------------|
| `baseline_commitment` | `[u8; 32]` | On-chain commitment (SHA3 of baseline) |
| `current_block` | `u64` | Current block height |
| `recovery_address` | `Field` | New address receiving the recovered identity |

### Public Outputs

| Field | Type | Description |
|-------|------|-------------|
| `nullifier` | `Field` | Recovery nullifier (prevents replay) |
| `challenge_passed` | `bool` | Temporal challenge result |
| `recovery_commitment` | `[u8; 32]` | Binds new address to behavioral history |

### Similarity Metric

Uses L1 similarity (circuit-friendly, no sqrt):
```
sim = 1 - (Σ|a_i - b_i|) / (N × SCALE)
```
Required: `sim ≥ 0.80` (at least 80% behavioral continuity).

The security bound:
```
P(break BCK) = P(reproduce causal_history(entity, t0→t))
lim_{t→∞} P(break BCK) = 0  (monotonically decreasing)
```

---

## Circuit 3: Sentinel Compliance

**File:** `circuits/sentinel_compliance/src/main.nr`  
**Constraints:** ~6,000  
**Proving time (estimated):** <1 second

### Purpose

Enforces compliance tier limits at the ZK level — jurisdiction rules, tier
limits, and geographic decentralization checks cannot be bypassed because they
are constraints in the circuit, not checked post-hoc in contract code.

### Inputs / Outputs

| Field | Type | Vis | Description |
|-------|------|-----|-------------|
| `agent_secret` | `Field` | Private | Agent secret key |
| `credential_expiry` | `u64` | Private | Credential expiry block |
| `transaction_amount` | `u64` | Public | Amount in CSPR motes |
| `agent_tier` | `u8` | Public | Declared tier (1–5) |
| `current_block` | `u64` | Public | Current block |
| `jurisdiction_code` | `u8` | Public | 1=US, 2=EU, 3=SG, 0=unrestricted |
| `geographic_hhi` | `u64` | Public | Network HHI (must be < 2500) |
| `nullifier` | `Field` | Output | Proof-of-compliance nullifier |
| `compliant` | `bool` | Output | True iff all checks pass |
| `allowed_amount` | `u64` | Output | Permitted transaction amount |

### Geographic HHI Check

The Herfindahl-Hirschman Index measures validator geographic concentration:
- HHI < 1000: highly decentralized (healthy)
- HHI 1000–2500: moderately concentrated (acceptable)  
- HHI > 2500: **circuit rejects** — network too centralized for compliant operation

This structurally prevents CSP from operating on a captured network.

---

## Building the Circuits

```bash
# Install Nargo (Noir toolchain)
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup

# Compile all circuits
cd circuits/behavioral_integrity && nargo compile
cd circuits/causal_identity      && nargo compile
cd circuits/sentinel_compliance  && nargo compile

# Run circuit tests
nargo test

# Generate proving/verification keys (Barretenberg)
bb write_vk -b target/behavioral_integrity.json
bb write_vk -b target/causal_identity.json
bb write_vk -b target/sentinel_compliance.json
```

## Registering Verification Keys On-Chain

After generating VKs, register them with the `ZKVerifier` contract:

```bash
casper-client put-deploy \
  --secret-key ./keys/sentinel.pem \
  --chain-name casper-testnet \
  --node-address https://rpc.testnet.casper.network \
  --session-hash hash-<ZK_VERIFIER_HASH> \
  --session-entry-point "register_vk" \
  --session-arg "circuit_type:string='behavioral_integrity'" \
  --session-arg "vk:bytes=$(xxd -p -c 0 vk_behavioral_integrity.bin)"
```

Repeat for `causal_identity` and `sentinel_compliance`.
