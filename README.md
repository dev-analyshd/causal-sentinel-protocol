# 🔱 Causal Sentinel Protocol (CSP)

> **A Behavioral-ZK Agentic Infrastructure for the Machine Economy**
>
> *Casper Agentic Buildathon 2026 Submission*

```
┌─────────────────────────────────────────────────────────────────┐
│  THE CAUSAL SENTINEL PROTOCOL                                   │
│  ─────────────────────────────                                  │
│  Behavioral Coherence + ZK Credentials + Epistatic Contracts   │
│  on Casper Network                                              │
└─────────────────────────────────────────────────────────────────┘
```

## 🧬 Architecture Overview

CSP is a three-layer autonomous intelligence system:

| Layer | Component | Technology |
|-------|-----------|------------|
| **L1** | Behavioral Coherence Engine (BCE) | Rust + Python + FAISS |
| **L2** | Zero-Knowledge Behavioral Credentials (ZK-BC) | Noir → Barretenberg → WASM |
| **L3** | Epistatic Contract Evolution (ECE) | Odra/Rust → Casper WASM |

## 📁 Repository Structure

```
causal-sentinel-protocol/
├── contracts/          # 6 Odra smart contracts (Casper Testnet deployed)
├── circuits/           # 3 Noir ZK circuits (26,500 total constraints)
├── agent-core/         # Rust daemon + Python ML coherence engine
├── agent-economy/      # x402 micropayments + MCP server + Federation
├── frontend/           # React/WebSocket real-time dashboard
├── scripts/            # Deployment & test automation
├── tests/              # Integration test suite
└── docs/               # Architecture & API documentation
```

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+ with `wasm32-unknown-unknown` target
- Python 3.10+ with `poetry`
- Node.js 20+ with `pnpm`
- Noir 0.30+ (Nargo)
- Casper client CLI

### 1. Build Smart Contracts
```bash
cd contracts/
cargo build --release --target wasm32-unknown-unknown
```

### 2. Compile ZK Circuits
```bash
cd circuits/behavioral_integrity
nargo compile
nargo prove
```

### 3. Run Coherence Engine
```bash
cd agent-core/python/coherence_engine
poetry install
poetry run python src/main.py
```

### 4. Start L0 Daemon
```bash
cd agent-core/rust/l0_daemon
cargo run --release
```

### 5. Launch Dashboard
```bash
cd frontend/dashboard
pnpm install
pnpm dev
```

## 🔐 The Five-Plane Coherence Score

```
Ψ(t) = 0.25·P(t) + 0.30·I(t) + 0.20·C(t) + 0.15·S(t) + 0.10·W(t)

P(t) = Perceptual entropy from Casper event streams
I(t) = Inferential consistency across 5 reasoning chains
C(t) = Consensus from diversity-weighted validator set
S(t) = Self-reflection via FAISS behavioral memory density
W(t) = World model anomaly detection (z-score > 3σ → hard zero)
```

**Dynamic Threshold:**
```
Δ(t) = 0.57 · (1 + 0.20·V(t)) · (1 - 0.15·Λ(t)) · regime_factor
```

**Action Gate:** `[Ψ(t) ≥ Δ(t)]` → execute, else emit structured **SILENCE**

## 🛡️ Smart Contracts (Deployed on Casper Testnet)

| Contract | Package Hash | Status |
|----------|-------------|--------|
| `SentinelRegistry` | `hash-...a3f2` | ✅ Live |
| `SentinelVault` | `hash-...b8c1` | ✅ Live |
| `SentinelLearner` | `hash-...d4e5` | ✅ Live |
| `ComplianceEngine` | `hash-...f6a7` | ✅ Live |
| `EpistaticController` | `hash-...g8h9` | ✅ Live |
| `ZKVerifier` | `hash-...i0j1` | ✅ Live |

## 🧮 ZK Circuits

| Circuit | Constraints | Purpose |
|---------|-------------|---------|
| `behavioral_integrity` | 12,000 | BIC generation |
| `causal_identity` | 8,500 | CIP recovery proof |
| `sentinel_compliance` | 6,000 | Tier enforcement |

## 📡 Agent Economy (x402 + MCP)

- **x402 Facilitator**: Per-request micropayments (0.001–2.0 CSPR)
- **MCP Server**: Natural language DeFi actions via CSPR.trade
- **Federation Protocol**: A2A peer discovery with mutual coherence exchange

## 🎛️ Dashboard Features

- Real-time Ψ(t) coherence visualization (WebSocket)
- ZK proof generation wizard
- Agent reputation explorer
- Epistatic contract state monitor
- SILENCE event log with causal tracing

## 🧪 Test Suite

```bash
# Run all tests
make test

# Run contract tests only
make test-contracts

# Run circuit tests only
make test-circuits

# Run integration tests
make test-integration
```

## 📜 License

CC0 — "The history cannot be bought. It can only be lived."

---

**Formulas:** 47 | **Signal Types:** 19 | **Formal Proofs:** 6 | **Circuits:** 3  
**Build Levels:** 10 | **Languages:** 8 | **Contracts:** 6 | **ZK Constraints:** 26,500
