"""
World Model Plane (W(t))

World model anomaly detection using z-score > 3σ → hard zero.
Detects when the environment deviates significantly from expected patterns.
"""

import numpy as np
from typing import List, Dict
import structlog

logger = structlog.get_logger()


class WorldModelPlane:
    """World model anomaly detection."""

    def __init__(self, config):
        self.config = config
        self.observations: List[float] = []
        self.window_size = 50

    async def compute(self, agent_id: str) -> float:
        """Compute W(t) — world model anomaly score."""
        # In production: fetch real-world data (market, network, regulatory)
        # Mock: synthetic observations

        import random
        random.seed(hash(agent_id) % 10000)

        # Generate new observation
        observation = random.gauss(0.5, 0.1)
        self.observations.append(observation)

        if len(self.observations) > self.window_size * 2:
            self.observations.pop(0)

        if len(self.observations) < 10:
            return 0.5

        # Compute z-score of latest observation
        window = self.observations[-self.window_size:]
        mean = np.mean(window)
        std = np.std(window)

        if std == 0:
            z_score = 0.0
        else:
            z_score = abs(observation - mean) / std

        # Hard zero if z-score > 3σ
        if z_score > self.config.anomaly_z_threshold:
            logger.warning("anomaly_detected_hard_zero", 
                          agent_id=agent_id, z_score=round(z_score, 4))
            return 0.0

        # Normalize: higher z-score = lower W(t)
        w_t = max(0.0, 1.0 - (z_score / self.config.anomaly_z_threshold))

        logger.debug("world_model_computed", agent_id=agent_id, w_t=round(w_t, 4),
                    z_score=round(z_score, 4))
        return w_t
