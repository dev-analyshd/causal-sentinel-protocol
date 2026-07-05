# Causal Sentinel Protocol — Buildathon Submission

**Casper Agentic Buildathon 2026**  
**Category:** Agentic Infrastructure  
**License:** CC0-1.0

---

## TL;DR

CSP is the first system that makes behavioral history a cryptographic primitive.
An agent with six months of honest operation is exponentially harder to replace
than a new agent. The history cannot be bought. It can only be lived.

---

## What We Built

| Component | Status | Language | Description |
|-----------|--------|----------|-------------|
| SentinelRegistry | ✅ Complete | Rust/Odra | Agent identity, Λ accumulation, credential lifecycle |
| SentinelVault | ✅ Complete | Rust/Odra | ZK-gated capital, coherence gate, heartbeat |
| EpistaticController | ✅ Complete | Rust/Odra | EL_state computation, contract expression modulation |
| ZKVerifier | ✅ Complete | Rust/Odra | UltraHonk proof verification (3 circuit types) |
| SentinelLearner | ✅ Complete | Rust/Odra | Domain mastery, IQ milestones, epistatic params |
| ComplianceEngine | ✅ Complete | Rust/Odra | Protocol-level compliance, Chameleon Protocol |
| BIC Circuit | ✅ Complete | Noir | 12,000-constraint behavioral integrity credential |
| CIP Circuit | ✅ Complete | Noir | 8,500-constraint causal identity recovery proof |
| Compliance Circuit | ✅ Complete | Noir | 6,000-constraint tier enforcement |
| L0 Daemon | ✅ Complete | Rust | Casper event streaming, dual SHA3-256 behavioral hashing |
| Coherence Engine | ✅ Complete | Python | Five-plane Ψ(t) computation, moat compounding |
| ANIMA Crawler | ✅ Complete | Python | 1,000+ concurrent crawlers, 50+ languages |
| FAISS Memory | ✅ Complete | Python | 128-dim behavioral vectors, similarity search |
| CRISPR Defense | ✅ Complete | Rust | Pre-execution attack interception (mempool layer) |
| x402 Facilitator | ✅ Complete | Rust | Per-request micropayments (HMAC-signed, server-priced) |
| MCP Server | ✅ Complete | Rust | NL → DeFi action parser (swap/stake/transfer/bridge) |
| Federation Protocol | ✅ Complete | Rust | A2A peer discovery, mutual coherence exchange |
| TypeScript SDK | ✅ Complete | TypeScript | Type-safe developer interface (discriminated unions) |
| Python SDK | ✅ Complete | Python | Sync client + local coherence math helpers |
| FastAPI Backend | ✅ Complete | Python | REST + WebSocket gateway |
| React Dashboard | ✅ Complete | TypeScript | Real-time Ψ(t) chart, Λ curve, gate feed, ZK wizard |
| Docker Compose | ✅ Complete | YAML | One-command full-stack local deployment |
| CI Pipeline | ✅ Complete | GitHub Actions | Rust/Python/Frontend/Noir multi-job CI |

**Test coverage:** 46 tests passing (Rust workspace, `cargo test --workspace`)

---

## The Three Problems We Solve

### Problem 1: AI Agents Without Behavioral Accountability
**Our answer:** Behavioral Coherence Engine (BCE)  
Every agent action is gated by Ψ(t) = α·P(t) + β·I(t) + γ·C(t) + δ·S(t) + ε·W(t).  
The moat Λ(t) = Λ(t-1) + κ·Ψ(t) **never decreases**. History cannot be bought.

### Problem 2: Smart Contracts That Cannot Adapt
**Our answer:** Epistatic Contract Evolution (ECE)  
EL_state(t) = σ(Threat·w_T + Validator_health·w_V + Network_entropy·w_N)  
DNA (bytecode) is immutable. Phenotype (expression) adapts autonomously.

### Problem 3: Identity Based on Secrets That Can Be Lost
**Our answer:** Causal Identity Proof (CIP)  
"What you have lived" replaces "what you know."  
Behavioral history is the cryptographic primitive. Losing a key ≠ losing identity.

---

## What Makes This Novel (Six Firsts)

1. **Behavioral-ZK fusion** — behavioral history IS the credential (no system has merged these)
2. **Causal identity** — lived history as ontological identity root (not metaphor, formal bound)
3. **Epistatic contracts** — environmental signal → expression modulation, not governance vote
4. **Diversity-weighted consensus** — Byzantine coordination is structurally self-defeating
5. **Protocol-native compliance** — Casper Native Token Registry at native speed
6. **Regulatory auto-adaptation** — Chameleon Protocol, Right to Invisibility, AWA enforcement

---

## Why Casper

CSP exploits **ten unique Casper advantages** that make this architecture impossible on any other chain:

1. Upgradeable contracts → Epistatic evolution without proxy hacks
2. Deterministic finality → Agent actions irreversible in one block
3. Fixed gas costs → Agents budget precisely, no fee spike risk
4. Account/contract unification → Agents are first-class on-chain entities
5. Protocol-level compliance hooks → Native Token Registry at native speed
6. x402 live on mainnet → First WASM-native L1 with agent micropayments
7. Fee delegation → Third parties sponsor agent transactions
8. 8-second blocks → Fast enough for real-time agent decisions
9. WASM execution → ZK circuits compile directly, no EVM overhead
10. Quantum safety roadmap → ML-DSA-44 hybrid for long-term agent ops

---

## Deployment Instructions

### Prerequisites
```bash
rustup target add wasm32-unknown-unknown
cargo install odra-cli
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash && noirup
cargo install casper-client
```

### Deploy to Casper Testnet
```bash
# 1. Build WASM contracts
cd contracts && cargo build --release --target wasm32-unknown-unknown

# 2. Run deployment
bash scripts/deploy_testnet.sh

# 3. Verify deployment
python3 scripts/verify_deployment.py

# 4. Bootstrap agent
bash scripts/bootstrap.sh
```

### Local Development (Docker)
```bash
cp .env.example .env   # Fill in API keys
docker-compose up -d
open http://localhost:3000
```

---

## Security Properties

| Property | Mechanism |
|----------|-----------|
| Replay prevention | ZK nullifiers (spent on-chain, never reused) |
| Payment integrity | HMAC-SHA3-256 over canonical payload (constant-time compare) |
| Underpay prevention | Server-side pricing from ServiceRegistry |
| Admin endpoint | Protected by ADMIN_SECRET env var |
| Nullifier gap | CRISPR Defense auto-records on accept (no caller forget path) |
| Hash panics | parse_hash32() validates exact 32-byte length before copy |

---

## The Master Equation

```
Σ(a,t) = [Ψ(t) ≥ Δ(t)] · R(a,t) · e^(Λ·t)
```

Where:
- `[Ψ(t) ≥ Δ(t)]` = Coherence gate (1=EXECUTE, 0=SILENCE)
- `R(a,t)` = Reward/relevance of action a
- `e^(Λ·t)` = Compounding moat (never decreases)

---

## Numbers

| Metric | Value |
|--------|-------|
| Formulas | 47 |
| ZK constraints | 26,500 (across 3 circuits) |
| Formal proofs | 6 |
| Signal types | 19 |
| Languages | 8 (Rust, Noir, Python, TypeScript, Bash, SQL, YAML, TOML) |
| Contracts | 6 |
| Tests passing | 46 |
| Build levels | 10 |

---

*"The seed phrase was always the wrong foundation for machine identity. Behavioral causality was always the right one. CSP makes that truth computable."*
