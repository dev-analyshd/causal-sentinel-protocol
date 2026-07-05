"""
Moat Compounder (Λ(t))

Λ(t) = Λ(t-1) + κ·Ψ(t)
Λ NEVER decreases.
"""

from typing import Dict
import structlog

logger = structlog.get_logger()


class MoatCompounder:
    """Compounding moat score."""

    def __init__(self, config):
        self.config = config
        self.moat_scores: Dict[str, float] = {}

    def get_moat(self, agent_id: str) -> float:
        """Get current moat score for agent."""
        return self.moat_scores.get(agent_id, 0.0)

    def compound(self, agent_id: str, psi: float):
        """Compound moat: Λ(t) = Λ(t-1) + κ·Ψ(t)"""
        current = self.moat_scores.get(agent_id, 0.0)
        new_lambda = current + self.config.kappa * psi
        self.moat_scores[agent_id] = new_lambda

        logger.info("moat_compounded", agent_id=agent_id, 
                   old_lambda=round(current, 4), new_lambda=round(new_lambda, 4))
