"""
Causal Sentinel Protocol — Python SDK

Type-safe client for CSP agent operations.
Supports both sync and async usage.
"""

import hashlib
import hmac
import json
import time
from dataclasses import dataclass, field
from enum import IntEnum
from typing import Any, Callable, Dict, List, Literal, Optional, Tuple, Union
import urllib.request
import urllib.error


# ─── Enums & constants ────────────────────────────────────────────────────────

class ComplianceTier(IntEnum):
    BASIC    = 1
    BRONZE   = 2
    SILVER   = 3
    GOLD     = 4
    PLATINUM = 5

TIER_NAMES = {
    ComplianceTier.BASIC:    "Basic",
    ComplianceTier.BRONZE:   "Bronze",
    ComplianceTier.SILVER:   "Silver",
    ComplianceTier.GOLD:     "Gold",
    ComplianceTier.PLATINUM: "Platinum",
}

TIER_LIMITS_CSPR = {
    ComplianceTier.BASIC:    200_000,
    ComplianceTier.BRONZE:   400_000,
    ComplianceTier.SILVER:   600_000,
    ComplianceTier.GOLD:     800_000,
    ComplianceTier.PLATINUM: 1_000_000,
}

CoherencePlane = Literal["perceptual", "inferential", "consensus", "self_reflection", "world_model"]
Regime         = Literal["Normal", "Alert", "Critical", "Silence"]
ActionType     = Literal["transfer", "trade", "stake", "unstake", "bridge", "credential_renewal"]


# ─── Signal types ─────────────────────────────────────────────────────────────

@dataclass(frozen=True)
class ValuationSignal:
    """Coherence gate is OPEN — action is cleared for execution."""
    action: Literal["EXECUTE"]
    value: float
    ci_95: Tuple[float, float]   # 95% confidence interval (never None)
    psi: float
    delta: float
    lambda_t: float
    tier: ComplianceTier
    sentinel_score: float

    @property
    def is_execute(self) -> bool:
        return True


@dataclass(frozen=True)
class SilenceSignal:
    """Coherence gate is CLOSED — action is suppressed."""
    action: Literal["SILENCE", "EMERGENCY_HALT"]
    psi: float
    delta: float
    gap: float                   # delta - psi: magnitude of shortfall
    limiting_plane: CoherencePlane
    trend: Literal["improving", "stable", "degrading"]
    eta_seconds: Optional[float] # None = unknown
    reason: str

    @property
    def is_execute(self) -> bool:
        return False


CoherenceResult = Union[ValuationSignal, SilenceSignal]


def is_execute(result: CoherenceResult) -> bool:
    """Type-safe gate check. Always use this before acting on a result."""
    return result.is_execute


# ─── Request types ────────────────────────────────────────────────────────────

@dataclass
class CoherenceRequest:
    agent_id: str
    action_type: ActionType
    amount: Optional[float] = None     # CSPR motes
    target: Optional[str]  = None     # Destination address
    jurisdiction: Optional[str] = None # ISO code


@dataclass
class ZKProofRequest:
    agent_id: str
    credential_type: Literal["behavioral_integrity", "causal_identity", "sentinel_compliance"]
    target_tier: Optional[ComplianceTier] = None


@dataclass
class ZKProof:
    circuit_type: str
    proof_bytes: bytes
    public_inputs: bytes
    nullifier: str             # Hex-encoded [u8; 32]
    compliance_tier: ComplianceTier
    expiry_block: int
    generated_at: float        # Unix timestamp


@dataclass
class MoatStatus:
    agent_id: str
    lambda_t: float            # Current Λ(t)
    tier: ComplianceTier
    registration_block: int
    age_days: float
    moat_growth_rate: float    # κ × Ψ(t) per cycle
    next_tier_eta_days: Optional[float]


# ─── SDK client ───────────────────────────────────────────────────────────────

class CausalSentinelClient:
    """
    Synchronous Python client for the Causal Sentinel Protocol API.

    Usage:
        client = CausalSentinelClient(
            api_url="http://localhost:8080",
            agent_id="agent_001",
            api_key="sk_test_...",
        )

        result = client.evaluate_action(CoherenceRequest(
            agent_id="agent_001",
            action_type="trade",
            amount=100_000_000,
        ))

        if is_execute(result):
            # Gate is open — proceed
            print(f"Execute at Ψ={result.psi:.4f}, tier={result.tier.name}")
        else:
            # Gate is closed — structured silence
            print(f"SILENCE: {result.reason}, ETA={result.eta_seconds}s")
    """

    def __init__(
        self,
        api_url: str,
        agent_id: str,
        api_key: str,
        timeout: float = 10.0,
    ):
        self.api_url   = api_url.rstrip("/")
        self.agent_id  = agent_id
        self.api_key   = api_key
        self.timeout   = timeout

    def evaluate_action(self, request: CoherenceRequest) -> CoherenceResult:
        """Evaluate coherence gate for an action. Returns ValuationSignal or SilenceSignal."""
        data = self._post("/api/v1/coherence/evaluate", {
            "agent_id":    request.agent_id or self.agent_id,
            "action_type": request.action_type,
            "amount":      request.amount,
            "target":      request.target,
            "jurisdiction": request.jurisdiction,
        })
        return self._parse_coherence_result(data)

    def generate_zk_proof(self, request: ZKProofRequest) -> ZKProof:
        """Generate a ZK behavioral credential. Valid for 90 days (972,000 blocks)."""
        data = self._post("/api/v1/zk/generate", {
            "agent_id":        request.agent_id or self.agent_id,
            "credential_type": request.credential_type,
            "target_tier":     request.target_tier,
        })
        return ZKProof(
            circuit_type    = data["circuit_type"],
            proof_bytes     = bytes.fromhex(data["proof_bytes_hex"]),
            public_inputs   = bytes.fromhex(data["public_inputs_hex"]),
            nullifier       = data["nullifier"],
            compliance_tier = ComplianceTier(data["compliance_tier"]),
            expiry_block    = data["expiry_block"],
            generated_at    = data["generated_at"],
        )

    def get_moat_status(self, agent_id: Optional[str] = None) -> MoatStatus:
        """Get current Λ(t) and tier projection."""
        aid = agent_id or self.agent_id
        data = self._get(f"/api/v1/moat/{aid}")
        return MoatStatus(
            agent_id          = data["agent_id"],
            lambda_t          = data["lambda"],
            tier              = ComplianceTier(data["tier"]),
            registration_block = data["registration_block"],
            age_days          = data["age_days"],
            moat_growth_rate  = data["moat_growth_rate"],
            next_tier_eta_days = data.get("next_tier_eta_days"),
        )

    def submit_heartbeat(self, psi: float, lambda_t: float, iq: float) -> Dict[str, Any]:
        """Submit on-chain heartbeat every 100 blocks."""
        return self._post("/api/v1/heartbeat", {
            "agent_id": self.agent_id,
            "psi":      int(psi * 1_000_000),
            "lambda":   int(lambda_t * 1_000_000),
            "iq":       int(iq * 1_000_000),
        })

    def get_silence_log(self, limit: int = 50) -> List[Dict[str, Any]]:
        """Get recent SILENCE events for this agent."""
        return self._get(f"/api/v1/silence/{self.agent_id}?limit={limit}")

    # ─── Private helpers ──────────────────────────────────────────────────────

    def _parse_coherence_result(self, data: Dict[str, Any]) -> CoherenceResult:
        action = data.get("action")
        if action == "EXECUTE":
            return ValuationSignal(
                action         = "EXECUTE",
                value          = data["value"],
                ci_95          = tuple(data["ci_95"]),   # type: ignore[arg-type]
                psi            = data["psi"],
                delta          = data["threshold"],
                lambda_t       = data["lambda"],
                tier           = ComplianceTier(data["tier"]),
                sentinel_score = data["sentinel_score"],
            )
        else:
            return SilenceSignal(
                action          = action,  # type: ignore[arg-type]
                psi             = data["psi"],
                delta           = data["threshold"],
                gap             = data["gap"],
                limiting_plane  = data["limiting_plane"],
                trend           = data["trend"],
                eta_seconds     = data.get("eta_seconds"),
                reason          = data.get("reason", "Coherence below threshold"),
            )

    def _post(self, path: str, payload: Dict[str, Any]) -> Dict[str, Any]:
        body = json.dumps(payload).encode()
        req = urllib.request.Request(
            self.api_url + path,
            data=body,
            method="POST",
            headers={
                "Content-Type":  "application/json",
                "Authorization": f"Bearer {self.api_key}",
            },
        )
        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                return json.loads(resp.read())
        except urllib.error.HTTPError as e:
            raise RuntimeError(f"CSP API {e.code}: {e.read().decode()}") from e

    def _get(self, path: str) -> Any:
        req = urllib.request.Request(
            self.api_url + path,
            headers={"Authorization": f"Bearer {self.api_key}"},
        )
        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                return json.loads(resp.read())
        except urllib.error.HTTPError as e:
            raise RuntimeError(f"CSP API {e.code}: {e.read().decode()}") from e


# ─── Coherence math helpers (local computation) ───────────────────────────────

def compute_psi(
    p: float, i: float, c: float, s: float, w: float,
    alpha: float = 0.25, beta: float = 0.30,
    gamma: float = 0.20, delta: float = 0.15, epsilon: float = 0.10,
) -> float:
    """
    Ψ(t) = α·P(t) + β·I(t) + γ·C(t) + δ·S(t) + ε·W(t)
    All plane values in [0, 1].
    """
    return alpha * p + beta * i + gamma * c + delta * s + epsilon * w


def compute_threshold(
    lambda_t: float,
    volatility: float = 0.0,
    regime_factor: float = 1.0,
    base: float = 0.57,
    v_coeff: float = 0.20,
    m_coeff: float = 0.15,
) -> float:
    """
    Δ(t) = 0.57 · (1 + 0.20·V(t)) · (1 - 0.15·Λ(t)) · regime_factor
    """
    threshold = base * (1 + v_coeff * volatility)
    threshold *= (1 - m_coeff * min(lambda_t, 5.0))
    threshold *= regime_factor
    return max(0.30, min(0.90, threshold))


def compound_moat(lambda_t: float, psi: float, kappa: float = 0.01) -> float:
    """
    Λ(t) = Λ(t-1) + κ·Ψ(t)
    Moat never decreases — compounding is monotonically increasing.
    """
    return lambda_t + kappa * max(0.0, psi)


def sentinel_score(psi: float, reward: float, lambda_t: float, t: float) -> float:
    """
    Σ(a,t) = [Ψ(t) ≥ Δ(t)] · R(a,t) · e^(Λ·t)
    The master equation.
    """
    import math
    gate_open = 1.0 if psi >= compute_threshold(lambda_t) else 0.0
    return gate_open * reward * math.exp(lambda_t * t)


if __name__ == "__main__":
    # Quick smoke test
    psi = compute_psi(0.8, 0.7, 0.75, 0.6, 0.9)
    lam = 0.5
    thr = compute_threshold(lam)
    print(f"Ψ={psi:.4f}  Δ={thr:.4f}  gate={'OPEN' if psi >= thr else 'SILENCE'}")
    for _ in range(5):
        lam = compound_moat(lam, psi)
        thr = compute_threshold(lam)
        print(f"  Λ={lam:.4f}  Δ={thr:.4f}")
