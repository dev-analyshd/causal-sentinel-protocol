"""
Perceptual Plane (P(t))

Computes perceptual entropy from Casper event streams via MCP.
Measures the information content of observed blockchain events.
"""

import math
import numpy as np
from typing import Dict, List
import structlog

logger = structlog.get_logger()


class PerceptualPlane:
    """Perceptual entropy from Casper event streams."""

    def __init__(self, config):
        self.config = config
        self.event_history: List[Dict] = []
        self.entropy_window = 100

    async def compute(self, agent_id: str) -> float:
        """Compute P(t) — perceptual entropy from recent events."""
        # In production: fetch from L0 daemon WebSocket
        # Mock: generate synthetic event entropy

        if not self.event_history:
            # Bootstrap with synthetic data
            self.event_history = self._generate_synthetic_events()

        # Compute Shannon entropy of event distribution
        entropy = self._compute_shannon_entropy()

        # Normalize to [0, 1]
        p_t = min(1.0, entropy / 2.0)

        logger.debug("perceptual_computed", agent_id=agent_id, p_t=round(p_t, 4))
        return p_t

    def _compute_shannon_entropy(self) -> float:
        """Compute Shannon entropy of event type distribution."""
        if not self.event_history:
            return 0.0

        # Count event types
        type_counts = {}
        for event in self.event_history[-self.entropy_window:]:
            event_type = event.get("type", "unknown")
            type_counts[event_type] = type_counts.get(event_type, 0) + 1

        # Compute entropy
        total = sum(type_counts.values())
        entropy = 0.0
        for count in type_counts.values():
            p = count / total
            if p > 0:
                entropy -= p * math.log2(p)

        return entropy

    def _generate_synthetic_events(self) -> List[Dict]:
        """Generate synthetic events for bootstrapping."""
        import random
        event_types = ["deploy", "transfer", "stake", "vote", "bridge"]
        events = []
        for _ in range(200):
            events.append({
                "type": random.choice(event_types),
                "entropy": random.random()
            })
        return events

    def ingest_event(self, event: Dict):
        """Ingest a new event from the L0 daemon."""
        self.event_history.append(event)
        if len(self.event_history) > 1000:
            self.event_history.pop(0)
