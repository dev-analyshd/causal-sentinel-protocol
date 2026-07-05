# Causal Sentinel Protocol — Architecture Documentation

## Overview

The Causal Sentinel Protocol (CSP) is a three-layer autonomous intelligence system
combining behavioral coherence gating, zero-knowledge compliance credentials, and
epistatic contract evolution on Casper Network.

## Layer 1: Behavioral Coherence Engine (BCE)

### Five-Plane Coherence Score

```
Ψ(t) = 0.25·P(t) + 0.30·I(t) + 0.20·C(t) + 0.15·S(t) + 0.10·W(t)
```

| Plane | Component | Weight | Description |
|-------|-----------|--------|-------------|
| P(t) | Perceptual entropy | 0.25 | Casper event stream entropy |
| I(t) | Inferential consistency | 0.30 | 5-chain reasoning consensus |
| C(t) | Diversity-weighted consensus | 0.20 | Validator diversity scoring |
| S(t) | Self-reflection | 0.15 | FAISS behavioral memory density |
| W(t) | World model anomaly | 0.10 | z-score > 3σ → hard zero |

### Dynamic Threshold

```
Δ(t) = 0.57 · (1 + 0.20·V(t)) · (1 - 0.15·Λ(t)) · regime_factor
```

### Moat Compounding

```
Λ(t) = Λ(t-1) + κ·Ψ(t)
```

Λ NEVER decreases. Monotonically increasing reputation.

## Layer 2: Zero-Knowledge Behavioral Credentials (ZK-BC)

### Circuit 1: Behavioral Integrity Credential (BIC)
- 12,000 constraints
- Proves: operating duration, coherence threshold, manipulation-free, moat threshold
- Output: nullifier, compliance_tier, reputation_commitment

### Circuit 2: Causal Identity Proof (CIP)
- 8,500 constraints
- Proves: behavioral signature match, temporal challenge, DNA code timing
- Enables: seedless identity recovery

### Circuit 3: Sentinel Compliance
- 6,000 constraints
- Proves: tier limit compliance, jurisdiction rules, geographic HHI
- Output: nullifier, compliance boolean, allowed amount

## Layer 3: Epistatic Contract Evolution (ECE)

### Epistatic State Function

```
EL_state(t) = σ(Threat_level · w_T + Validator_health · w_V + Network_entropy · w_N)
```

### Regimes
| Regime | Condition | Expression |
|--------|-----------|------------|
| Normal | threat < 30 | Relaxed thresholds |
| Alert | 30 <= threat < 60 | Tightened thresholds |
| Critical | 60 <= threat < 80 | ZK-only, restricted |
| Silence | threat >= 80 | All non-essential ops paused |

## Casper Native Advantages

1. Upgradeable contracts natively -> Epistatic evolution without proxy hacks
2. Deterministic finality -> Agent actions irreversible in one block
3. Fixed gas costs -> Precise agent budgeting
4. Account/contract unification -> Agents as first-class entities
5. Protocol-level compliance hooks -> Native speed enforcement
6. x402 live on mainnet -> WASM-native micropayments
7. Fee delegation -> Third-party transaction sponsorship
8. 8-second blocks -> Real-time agent decisions
9. WASM execution -> Direct ZK circuit compilation
10. Quantum safety roadmap -> ML-DSA-44 hybrid accounts
