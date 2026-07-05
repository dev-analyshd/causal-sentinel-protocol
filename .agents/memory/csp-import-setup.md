---
name: Causal Sentinel Protocol import setup
description: What runs in Replit vs what is out of scope for this multi-language crypto/ZK project
---

This repo (Causal Sentinel Protocol) is a multi-layer system: Rust/Odra Casper smart contracts, Noir ZK circuits, a Python/Rust agent-core daemon, and a FastAPI + React dashboard.

Only two pieces are runnable as a normal web app in the Replit environment:
- `api/main.py` — FastAPI backend (in-memory state, mock ZK/heartbeat endpoints), run on port 8000 (localhost only, not externally mapped).
- `frontend/dashboard` — Vite/React dashboard, run on port 5000 with `host: 0.0.0.0`, `allowedHosts: true`, and dev-server proxy for `/api` and `/ws` to `localhost:8000`.

**Why:** the contracts/circuits require Casper/Odra + Nargo toolchains not present in this environment, and agent-core/agent-economy Rust crates are a separate workspace not meant to serve HTTP directly. The FastAPI backend is the intended dev API layer used by the dashboard.

**How to apply:** if asked to "run the whole protocol", clarify that only the dashboard + API run in Replit; contract/circuit builds are out of scope unless the user explicitly wants to set up the Casper/Noir toolchain. The dashboard's WebSocket client must hit `/ws/coherence` (relative, via proxy) — the original hardcoded `ws://localhost:9001` was dead code and had to be replaced.
