"""
Causal Sentinel — Coherence Engine

Five-Plane Coherence Computation:
  Ψ(t) = 0.25·P(t) + 0.30·I(t) + 0.20·C(t) + 0.15·S(t) + 0.10·W(t)

Dynamic Threshold:
  Δ(t) = 0.57 · (1 + 0.20·V(t)) · (1 - 0.15·Λ(t)) · regime_factor

Action Gate: [Ψ(t) ≥ Δ(t)] → execute, else emit structured SILENCE

The moat Λ(t) compounds every coherent cycle: Λ(t) = Λ(t-1) + κ·Ψ(t)
Λ NEVER decreases.
"""

__version__ = "0.1.0"
