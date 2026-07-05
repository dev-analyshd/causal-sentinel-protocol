#!/usr/bin/env python3
"""
Coherence Engine Main Entry Point

Computes the five-plane coherence score Ψ(t) for all registered agents
and emits SILENCE events when coherence gates close.
"""

import asyncio
import json
import math
import os
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple
from datetime import datetime

import numpy as np
import websockets
from rich.console import Console
from rich.live import Live
from rich.table import Table
from rich.panel import Panel
from rich.layout import Layout
import structlog

from .perceptual_plane import PerceptualPlane
from .inferential_plane import InferentialPlane
from .consensus_plane import ConsensusPlane
from .self_reflection_plane import SelfReflectionPlane
from .world_model_plane import WorldModelPlane
from .moat import MoatCompounder
from .silence_emitter import SilenceEmitter

logger = structlog.get_logger()
console = Console()


@dataclass
class CoherenceConfig:
    """Configuration for the coherence engine."""
    alpha: float = 0.25  # Perceptual weight
    beta: float = 0.30   # Inferential weight
    gamma: float = 0.20  # Consensus weight
    delta: float = 0.15  # Self-reflection weight
    epsilon: float = 0.10 # World model weight

    base_threshold: float = 0.57
    volatility_coeff: float = 0.20
    moat_coeff: float = 0.15

    kappa: float = 0.01  # Moat compounding rate

    l0_websocket: str = "ws://localhost:9001"
    casper_node: str = "http://localhost:7777"
    registry_contract: str = "hash-...a3f2"

    evaluation_interval: float = 8.0  # seconds (matches Casper block time)

    # FAISS memory config
    faiss_index_path: str = "./faiss_index"
    behavioral_vector_dim: int = 128

    # World model anomaly detection
    anomaly_z_threshold: float = 3.0


@dataclass
class CoherenceState:
    """Current coherence state for an agent."""
    agent_id: str
    psi: float = 0.0
    p_t: float = 0.0
    i_t: float = 0.0
    c_t: float = 0.0
    s_t: float = 0.0
    w_t: float = 0.0
    delta_t: float = 0.57
    lambda_t: float = 0.0
    regime: str = "Normal"
    gate_open: bool = False
    last_eval: datetime = field(default_factory=datetime.now)
    history: List[Tuple[float, float]] = field(default_factory=list)


class CoherenceEngine:
    """
    Five-Plane Behavioral Coherence Engine.

    Computes Ψ(t) every block and manages the compounding moat Λ(t).
    """

    def __init__(self, config: CoherenceConfig):
        self.config = config
        self.states: Dict[str, CoherenceState] = {}

        # Initialize five planes
        self.perceptual = PerceptualPlane(config)
        self.inferential = InferentialPlane(config)
        self.consensus = ConsensusPlane(config)
        self.self_reflection = SelfReflectionPlane(config)
        self.world_model = WorldModelPlane(config)

        # Moat compounding
        self.moat = MoatCompounder(config)

        # Silence emitter
        self.silence = SilenceEmitter(config)

        # Market volatility (mock — in production from oracle)
        self.volatility_index: float = 0.0

        # Regime factor
        self.regime_factor: float = 1.0

        logger.info("coherence_engine_initialized", 
                   alpha=config.alpha, beta=config.beta, gamma=config.gamma,
                   delta=config.delta, epsilon=config.epsilon)

    async def run(self):
        """Main evaluation loop."""
        logger.info("coherence_engine_started")

        # Connect to L0 daemon
        async with websockets.connect(self.config.l0_websocket) as ws:
            logger.info("connected_to_l0_daemon", url=self.config.l0_websocket)

            while True:
                try:
                    # Receive event from L0 daemon
                    message = await asyncio.wait_for(ws.recv(), timeout=1.0)
                    event = json.loads(message)

                    if event.get("type") == "update":
                        await self.evaluate_all_agents()

                except asyncio.TimeoutError:
                    # Evaluate on timeout even without events
                    await self.evaluate_all_agents()
                except Exception as e:
                    logger.error("evaluation_error", error=str(e))

    async def evaluate_all_agents(self):
        """Evaluate coherence for all registered agents."""
        # In production: fetch agent list from SentinelRegistry
        agent_ids = self.get_registered_agents()

        for agent_id in agent_ids:
            await self.evaluate_agent(agent_id)

    async def evaluate_agent(self, agent_id: str):
        """Compute Ψ(t) for a single agent."""
        state = self.states.get(agent_id, CoherenceState(agent_id=agent_id))

        # Compute five planes
        p_t = await self.perceptual.compute(agent_id)
        i_t = await self.inferential.compute(agent_id)
        c_t = await self.consensus.compute(agent_id)
        s_t = await self.self_reflection.compute(agent_id)
        w_t = await self.world_model.compute(agent_id)

        # Compute Ψ(t)
        psi = (
            self.config.alpha * p_t +
            self.config.beta * i_t +
            self.config.gamma * c_t +
            self.config.delta * s_t +
            self.config.epsilon * w_t
        )

        # Get current moat
        lambda_t = self.moat.get_moat(agent_id)

        # Compute dynamic threshold Δ(t)
        delta_t = self.compute_threshold(lambda_t)

        # Determine regime
        regime = self.determine_regime(psi, delta_t, w_t)
        self.regime_factor = self.get_regime_factor(regime)

        # Action gate
        gate_open = psi >= delta_t

        if not gate_open:
            # Emit SILENCE
            await self.silence.emit(agent_id, psi, delta_t, {
                "p_t": p_t, "i_t": i_t, "c_t": c_t, "s_t": s_t, "w_t": w_t
            })
        else:
            # Compound moat
            self.moat.compound(agent_id, psi)

        # Update state
        state.psi = psi
        state.p_t = p_t
        state.i_t = i_t
        state.c_t = c_t
        state.s_t = s_t
        state.w_t = w_t
        state.delta_t = delta_t
        state.lambda_t = self.moat.get_moat(agent_id)
        state.regime = regime
        state.gate_open = gate_open
        state.last_eval = datetime.now()
        state.history.append((psi, delta_t))

        if len(state.history) > 1000:
            state.history.pop(0)

        self.states[agent_id] = state

        logger.info("coherence_evaluated",
                   agent_id=agent_id, psi=round(psi, 4), delta=round(delta_t, 4),
                   gate_open=gate_open, lambda=round(state.lambda_t, 4),
                   regime=regime)

    def compute_threshold(self, lambda_t: float) -> float:
        """
        Δ(t) = 0.57 · (1 + 0.20·V(t)) · (1 - 0.15·Λ(t)) · regime_factor
        """
        base = self.config.base_threshold
        v_t = self.volatility_index

        threshold = base * (1 + self.config.volatility_coeff * v_t)
        threshold *= (1 - self.config.moat_coeff * min(lambda_t, 5.0))  # Cap lambda effect
        threshold *= self.regime_factor

        # Ensure threshold stays in valid range [0.3, 0.9]
        return max(0.3, min(0.9, threshold))

    def determine_regime(self, psi: float, delta: float, w_t: float) -> str:
        """Determine regime based on coherence and world model."""
        if w_t == 0.0:  # Hard zero from anomaly detection
            return "Silence"
        elif psi < delta * 0.5:
            return "Critical"
        elif psi < delta * 0.8:
            return "Alert"
        else:
            return "Normal"

    def get_regime_factor(self, regime: str) -> float:
        """Get regime factor for threshold modulation."""
        factors = {
            "Normal": 1.0,
            "Alert": 1.2,
            "Critical": 1.5,
            "Silence": 10.0,
        }
        return factors.get(regime, 1.0)

    def get_registered_agents(self) -> List[str]:
        """In production: query SentinelRegistry. Mock for now."""
        return ["agent_001", "agent_002", "agent_003"]

    def get_dashboard_data(self) -> Dict:
        """Export data for real-time dashboard."""
        return {
            "agents": {
                aid: {
                    "psi": round(s.psi, 4),
                    "delta": round(s.delta_t, 4),
                    "lambda": round(s.lambda_t, 4),
                    "gate_open": s.gate_open,
                    "regime": s.regime,
                    "planes": {
                        "perceptual": round(s.p_t, 4),
                        "inferential": round(s.i_t, 4),
                        "consensus": round(s.c_t, 4),
                        "self_reflection": round(s.s_t, 4),
                        "world_model": round(s.w_t, 4),
                    }
                }
                for aid, s in self.states.items()
            },
            "global": {
                "volatility": self.volatility_index,
                "regime_factor": self.regime_factor,
                "total_agents": len(self.states),
            }
        }


def create_dashboard(engine: CoherenceEngine) -> Layout:
    """Create Rich dashboard layout."""
    layout = Layout()
    layout.split_column(
        Layout(name="header", size=3),
        Layout(name="main"),
        Layout(name="footer", size=3)
    )
    layout["main"].split_row(
        Layout(name="agents"),
        Layout(name="planes")
    )

    header = Panel(
        "[bold cyan]🔱 Causal Sentinel Coherence Engine[/bold cyan] | "
        f"Agents: {len(engine.states)} | Regime: {engine.regime_factor}",
        style="on blue"
    )
    layout["header"].update(header)

    # Agent table
    table = Table(title="Agent States")
    table.add_column("Agent", style="cyan")
    table.add_column("Ψ", justify="right")
    table.add_column("Δ", justify="right")
    table.add_column("Λ", justify="right")
    table.add_column("Gate", justify="center")
    table.add_column("Regime", justify="center")

    for aid, state in engine.states.items():
        gate = "[green]OPEN[/green]" if state.gate_open else "[red]SILENCE[/red]"
        regime_color = {
            "Normal": "green",
            "Alert": "yellow",
            "Critical": "red",
            "Silence": "white on red"
        }.get(state.regime, "white")

        table.add_row(
            aid,
            f"{state.psi:.4f}",
            f"{state.delta_t:.4f}",
            f"{state.lambda_t:.4f}",
            gate,
            f"[{regime_color}]{state.regime}[/{regime_color}]"
        )

    layout["agents"].update(Panel(table, title="[bold]Agents[/bold]"))

    footer = Panel(
        "[dim]Ψ(t) = 0.25·P(t) + 0.30·I(t) + 0.20·C(t) + 0.15·S(t) + 0.10·W(t)[/dim]",
        style="on black"
    )
    layout["footer"].update(footer)

    return layout


async def main():
    """Main entry point."""
    config = CoherenceConfig()
    engine = CoherenceEngine(config)

    # Start dashboard in background
    with Live(create_dashboard(engine), refresh_per_second=4) as live:
        async def update_dashboard():
            while True:
                live.update(create_dashboard(engine))
                await asyncio.sleep(0.25)

        # Run engine and dashboard
        await asyncio.gather(
            engine.run(),
            update_dashboard()
        )


if __name__ == "__main__":
    asyncio.run(main())
