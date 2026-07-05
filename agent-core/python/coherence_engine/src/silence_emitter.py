"""
Silence Emitter

Emits structured SILENCE events when coherence gates close.
"""

import json
from typing import Dict
import structlog

logger = structlog.get_logger()


class SilenceEmitter:
    """Emits structured SILENCE events."""

    def __init__(self, config):
        self.config = config
        self.silence_count = 0

    async def emit(self, agent_id: str, psi: float, delta: float, planes: Dict):
        """Emit a SILENCE event."""
        self.silence_count += 1

        event = {
            "type": "SILENCE",
            "agent_id": agent_id,
            "psi": psi,
            "delta": delta,
            "gap": delta - psi,
            "planes": planes,
            "timestamp": "",  # Would use datetime
            "sequence": self.silence_count,
        }

        logger.warning("silence_emitted", 
                      agent_id=agent_id, psi=round(psi, 4), delta=round(delta, 4),
                      gap=round(delta - psi, 4))

        # In production: push to SentinelRegistry.record_silence
        # and broadcast via WebSocket to dashboard
