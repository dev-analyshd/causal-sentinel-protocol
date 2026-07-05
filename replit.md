# Causal Sentinel Protocol (CSP)

A behavioral-ZK agentic infrastructure for the Machine Economy on the Casper Network. Combines AI coherence scoring, zero-knowledge proofs, and evolving smart contracts to ensure agentic compliance.

## Running on Replit

Two services run here:

| Workflow | Command | Port |
|---|---|---|
| **Backend API** | `cd api && python3 main.py` | 8000 |
| **Start application** (Frontend) | `cd frontend/dashboard && npm run dev` | 5000 |

The frontend proxies `/api` and `/ws` requests to the backend, so the preview (port 5000) is the primary entry point.

### Install dependencies
```bash
pip install -r api/requirements.txt
cd frontend/dashboard && npm install
```

## Architecture

| Layer | Component | Tech | Runnable on Replit |
|---|---|---|---|
| L1 | Behavioral Coherence Engine | Python + FastAPI | ✅ |
| L1 | Frontend Dashboard | React + Vite + TypeScript | ✅ |
| L1 | L0 Daemon | Rust | ✅ (with Rust toolchain) |
| L2 | ZK Circuits | Noir/Nargo | ⚠️ Needs Nargo toolchain |
| L3 | Smart Contracts | Odra/Rust → Casper wasm32 | ⚠️ Needs Casper toolchain + testnet |

## Contract Deployment (Casper Testnet)

Contracts live in `contracts/` as a Cargo workspace targeting `wasm32-unknown-unknown`. Prerequisites:
- `rustup target add wasm32-unknown-unknown`
- Casper client CLI
- Testnet CSPR for gas

Build: `cd contracts && cargo build --release --target wasm32-unknown-unknown`

See `docs/ARCHITECTURE.md` and `README.md` for full deployment scripts.

## Key Files

- `api/main.py` — FastAPI backend (coherence evaluation, ZK mock proofs, agent registry, WebSocket broadcast)
- `frontend/dashboard/src/` — React dashboard with real-time coherence charts
- `contracts/` — 6 Odra smart contracts (Registry, Vault, Learner, Compliance, Epistatic, ZK Verifier)
- `circuits/` — Noir ZK circuits for behavioral integrity
- `docs/` — Architecture, API, and circuit documentation

## User Preferences

- Keep existing project structure — do not restructure or migrate the monorepo layout.
