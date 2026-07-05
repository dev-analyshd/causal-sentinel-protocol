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

> ⚠️ Contract compilation (`wasm32-unknown-unknown`) and Casper testnet RPC are both unavailable in the Replit sandbox due to platform limitations. Deploy from your local machine or CI. See **`DEPLOY.md`** for the full step-by-step guide.

### Quick summary

1. **Private key** is stored as the `CASPER_PRIVATE_KEY` Replit Secret. Write it locally with:
   ```bash
   bash scripts/setup_key.sh
   ```
2. **Build contracts** locally (needs `rustup target add wasm32-unknown-unknown` + `casper-client`):
   ```bash
   cd contracts && cargo build --release --target wasm32-unknown-unknown
   ```
3. **Deploy all 6 contracts** in dependency order:
   ```bash
   export CASPER_KEY=./keys/sentinel.pem
   bash scripts/deploy_testnet.sh
   ```
4. Contract addresses auto-save to `config/testnet_addresses.json`.
5. A **GitHub Actions workflow** template is in `DEPLOY.md` for CI-based deployment.

## Key Files

- `api/main.py` — FastAPI backend (coherence evaluation, ZK mock proofs, agent registry, WebSocket broadcast)
- `frontend/dashboard/src/` — React dashboard with real-time coherence charts
- `contracts/` — 6 Odra smart contracts (Registry, Vault, Learner, Compliance, Epistatic, ZK Verifier)
- `circuits/` — Noir ZK circuits for behavioral integrity
- `docs/` — Architecture, API, and circuit documentation

## User Preferences

- Keep existing project structure — do not restructure or migrate the monorepo layout.
