#!/usr/bin/env python3
"""
Causal Sentinel Protocol — FastAPI Backend

Endpoints:
  POST /api/v1/coherence/evaluate   — Coherence gate evaluation
  POST /api/v1/zk/generate          — ZK proof generation (mock)
  GET  /api/v1/moat/:agent_id       — Moat status
  POST /api/v1/heartbeat            — On-chain heartbeat relay
  POST /api/v1/agents/register      — Agent registration relay
  GET  /api/v1/silence/:agent_id    — Silence log
  WS   /ws/coherence                — Real-time coherence events
  GET  /health                      — Health check
"""

import asyncio
import hashlib
import json
import math
import os
import time
from dataclasses import dataclass, field, asdict
from typing import Any, Dict, List, Literal, Optional, Tuple

from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect, Depends
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field as PydanticField

app = FastAPI(
    title="Causal Sentinel Protocol API",
    description="Behavioral-ZK Agentic Infrastructure for Casper Network",
    version="1.0.0",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

# ─── In-memory state (replace with TimescaleDB in production) ─────────────────

AGENTS: Dict[str, Dict[str, Any]] = {}
SILENCE_LOG: List[Dict[str, Any]] = []
WS_CLIENTS: List[WebSocket] = []


# ─── Request/response models ──────────────────────────────────────────────────

class CoherenceEvalRequest(BaseModel):
    agent_id: str
    action_type: str
    amount: Optional[float] = None
    target: Optional[str] = None
    jurisdiction: Optional[str] = "US"


class ZKProofRequest(BaseModel):
    agent_id: str
    credential_type: Literal["behavioral_integrity", "causal_identity", "sentinel_compliance"]
    target_tier: Optional[int] = 1


class HeartbeatRequest(BaseModel):
    agent_id: str
    psi: int    # Fixed-point (× 1,000,000)
    lambda_: int = PydanticField(alias="lambda")
    iq: int

    class Config:
        populate_by_name = True


class RegisterRequest(BaseModel):
    agent_id: str
    dna_code_hash: str
    behavioral_commitment: str
    stake_motes: int


# ─── Coherence math ───────────────────────────────────────────────────────────

def compute_psi(agent_id: str) -> Tuple[float, Dict[str, float]]:
    """Compute five-plane Ψ(t) for an agent. Mock planes for testnet demo."""
    state = AGENTS.get(agent_id, {})
    history = state.get("psi_history", [])
    lambda_t = state.get("lambda", 0.0)

    # Testnet mock: compute stable values with slight noise
    base = 0.75 + math.sin(time.time() / 60) * 0.05
    planes = {
        "perceptual":     min(1.0, base + 0.05),
        "inferential":    min(1.0, base + 0.02),
        "consensus":      min(1.0, base - 0.02),
        "self_reflection": min(1.0, base - 0.05),
        "world_model":    min(1.0, base + 0.01),
    }
    psi = (
        0.25 * planes["perceptual"] +
        0.30 * planes["inferential"] +
        0.20 * planes["consensus"] +
        0.15 * planes["self_reflection"] +
        0.10 * planes["world_model"]
    )
    return psi, planes


def compute_threshold(lambda_t: float, volatility: float = 0.0) -> float:
    """Δ(t) = 0.57 · (1 + 0.20·V(t)) · (1 - 0.15·Λ(t)) · regime_factor"""
    threshold = 0.57 * (1 + 0.20 * volatility)
    threshold *= max(0.0, 1 - 0.15 * min(lambda_t, 5.0))
    return max(0.30, min(0.90, threshold))


def compound_moat(agent_id: str, psi: float) -> float:
    """Λ(t) = Λ(t-1) + κ·Ψ(t). Moat never decreases."""
    state = AGENTS.setdefault(agent_id, {})
    lambda_t = state.get("lambda", 0.0) + 0.01 * max(0.0, psi)
    state["lambda"] = lambda_t
    return lambda_t


# ─── Endpoints ────────────────────────────────────────────────────────────────

@app.get("/health")
async def health():
    return {"status": "ok", "service": "CSP API", "agents": len(AGENTS)}


@app.post("/api/v1/coherence/evaluate")
async def evaluate_coherence(req: CoherenceEvalRequest):
    """
    Coherence gate evaluation.
    Returns ValuationSignal (EXECUTE) or SilenceSignal (SILENCE).
    """
    agent_id = req.agent_id
    state = AGENTS.setdefault(agent_id, {"lambda": 0.0, "tier": 1, "psi_history": []})
    state.setdefault("lambda", 0.0)
    state.setdefault("tier", 1)
    state.setdefault("psi_history", [])

    psi, planes = compute_psi(agent_id)
    lambda_t = state.get("lambda", 0.0)
    delta = compute_threshold(lambda_t)

    gate_open = psi >= delta

    if gate_open:
        # Compound moat
        new_lambda = compound_moat(agent_id, psi)
        state["psi_history"].append(psi)
        if len(state["psi_history"]) > 1000:
            state["psi_history"].pop(0)

        result = {
            "action": "EXECUTE",
            "value": psi,
            "ci_95": [psi - 0.05, psi + 0.05],
            "psi": psi,
            "threshold": delta,
            "lambda": new_lambda,
            "tier": state.get("tier", 1),
            "sentinel_score": psi * math.exp(new_lambda * 1.0),
            "planes": planes,
        }
    else:
        # Gate closed — emit SILENCE
        gap = delta - psi
        limiting_plane = min(planes, key=planes.get)
        trend = "improving" if len(state.get("psi_history", [])) > 1 and psi > state["psi_history"][-1] else "stable"

        silence_entry = {
            "agent_id": agent_id,
            "psi": psi,
            "delta": delta,
            "gap": gap,
            "timestamp": time.time(),
            "planes": planes,
        }
        SILENCE_LOG.append(silence_entry)
        if len(SILENCE_LOG) > 10_000:
            SILENCE_LOG.pop(0)

        result = {
            "action": "SILENCE",
            "psi": psi,
            "threshold": delta,
            "gap": gap,
            "limiting_plane": limiting_plane,
            "trend": trend,
            "eta_seconds": gap / 0.01 * 8.0 if gap < 0.5 else None,
            "reason": f"Ψ={psi:.4f} < Δ={delta:.4f} (gap={gap:.4f})",
            "planes": planes,
        }

    # Broadcast to WebSocket subscribers
    asyncio.create_task(broadcast_event({"agent_id": agent_id, **result}))

    return JSONResponse(result)


@app.post("/api/v1/zk/generate")
async def generate_zk_proof(req: ZKProofRequest):
    """
    ZK proof generation (testnet mock — production uses Nargo/Barretenberg).
    Returns a mock proof that matches circuit output structure.
    """
    agent_id = req.agent_id
    state = AGENTS.get(agent_id, {"lambda": 0.0, "tier": 1})
    current_block = int(time.time() // 8)  # Mock block number (8s blocks)
    expiry_block = current_block + 972_000  # 90-day TTL

    # Generate deterministic nullifier: hash(secret_key_mock || current_block)
    nullifier = hashlib.sha3_256(
        f"{agent_id}:{current_block}".encode()
    ).hexdigest()

    # Mock proof bytes (in production: actual Barretenberg UltraHonk proof)
    proof_bytes = hashlib.sha3_256(
        f"proof:{req.credential_type}:{agent_id}:{current_block}".encode()
    ).digest() * 4  # 128 bytes for behavioral_integrity circuit

    tier = req.target_tier or state.get("tier", 1)

    return {
        "circuit_type":      req.credential_type,
        "proof_bytes_hex":   proof_bytes.hex(),
        "public_inputs_hex": f"0{tier:02x}" + "00" * 31,  # First byte = tier
        "nullifier":         nullifier,
        "compliance_tier":   tier,
        "expiry_block":      expiry_block,
        "generated_at":      time.time(),
        "note":              "Testnet mock proof — production requires Nargo + Barretenberg",
    }


@app.get("/api/v1/moat/{agent_id}")
async def get_moat_status(agent_id: str):
    """Get current Λ(t) and tier projection for an agent."""
    state = AGENTS.get(agent_id)
    if not state:
        # Return initial state for unregistered agent
        state = {"lambda": 0.0, "tier": 1, "registration_block": int(time.time() // 8)}

    lambda_t = state.get("lambda", 0.0)
    reg_block = state.get("registration_block", int(time.time() // 8))
    age_blocks = int(time.time() // 8) - reg_block
    age_days = age_blocks * 8 / 86400

    # Estimate days to next tier
    tier = state.get("tier", 1)
    lambda_for_next = [0.1, 0.5, 1.0, 1.5, 2.0]
    if tier < 5:
        lambda_needed = lambda_for_next[tier] - lambda_t
        growth_rate = 0.01 * 0.75  # κ × avg_psi
        eta_days = (lambda_needed / growth_rate) * 8 / 86400 if lambda_needed > 0 else 0
    else:
        eta_days = None

    return {
        "agent_id":           agent_id,
        "lambda":             lambda_t,
        "tier":               tier,
        "registration_block": reg_block,
        "age_days":           age_days,
        "moat_growth_rate":   0.01 * 0.75,
        "next_tier_eta_days": eta_days,
    }


@app.post("/api/v1/heartbeat")
async def submit_heartbeat(req: HeartbeatRequest):
    """Relay heartbeat to SentinelVault on-chain."""
    state = AGENTS.setdefault(req.agent_id, {"lambda": 0.0, "tier": 1})
    state["last_heartbeat"] = time.time()

    # Update tier from lambda
    lam = req.lambda_ / 1_000_000
    state["lambda"] = lam
    if lam >= 2.0:
        state["tier"] = 5
    elif lam >= 1.5:
        state["tier"] = 4
    elif lam >= 1.0:
        state["tier"] = 3
    elif lam >= 0.5:
        state["tier"] = 2

    return {
        "agent_id": req.agent_id,
        "psi":      req.psi / 1_000_000,
        "lambda":   lam,
        "iq":       req.iq / 1_000_000,
        "block":    int(time.time() // 8),
        "note":     "Testnet: heartbeat recorded locally; production sends to SentinelVault",
    }


@app.post("/api/v1/agents/register")
async def register_agent(req: RegisterRequest):
    """Register agent and relay to SentinelRegistry on-chain."""
    if req.agent_id in AGENTS:
        raise HTTPException(status_code=409, detail="Agent already registered")

    reg_block = int(time.time() // 8)
    AGENTS[req.agent_id] = {
        "lambda": 0.0,
        "tier": 1,
        "registration_block": reg_block,
        "dna_code_hash": req.dna_code_hash,
        "stake_motes": req.stake_motes,
    }

    return {
        "agent_address":      req.agent_id,
        "registration_block": reg_block,
        "stake_amount":       req.stake_motes,
        "initial_tier":       1,
        "tx_hash":            hashlib.sha3_256(f"reg:{req.agent_id}:{reg_block}".encode()).hexdigest(),
        "note":               "Testnet: registration recorded locally; production deploys to SentinelRegistry",
    }


@app.get("/api/v1/silence/{agent_id}")
async def get_silence_log(agent_id: str, limit: int = 50):
    """Get recent SILENCE events for an agent."""
    agent_silences = [e for e in SILENCE_LOG if e.get("agent_id") == agent_id]
    return agent_silences[-limit:]


# ─── WebSocket ────────────────────────────────────────────────────────────────

@app.websocket("/ws/coherence")
async def websocket_coherence(ws: WebSocket):
    """Real-time coherence event stream."""
    await ws.accept()
    WS_CLIENTS.append(ws)
    try:
        while True:
            await asyncio.sleep(8.0)  # Send update every Casper block
            for agent_id in list(AGENTS.keys()):
                psi, planes = compute_psi(agent_id)
                state = AGENTS[agent_id]
                delta = compute_threshold(state.get("lambda", 0.0))
                await ws.send_json({
                    "type":     "coherence_update",
                    "agent_id": agent_id,
                    "psi":      psi,
                    "delta":    delta,
                    "lambda":   state.get("lambda", 0.0),
                    "tier":     state.get("tier", 1),
                    "gate":     "OPEN" if psi >= delta else "SILENCE",
                    "planes":   planes,
                })
    except WebSocketDisconnect:
        WS_CLIENTS.remove(ws)


async def broadcast_event(event: Dict[str, Any]):
    """Broadcast to all connected WebSocket clients."""
    dead = []
    for ws in WS_CLIENTS:
        try:
            await ws.send_json(event)
        except Exception:
            dead.append(ws)
    for ws in dead:
        WS_CLIENTS.remove(ws)


if __name__ == "__main__":
    import uvicorn
    port = int(os.environ.get("PORT", 8000))
    uvicorn.run(app, host="0.0.0.0", port=port, log_level="info")
