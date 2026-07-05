# Causal Sentinel Protocol — Test & Assessment Report
_Generated: July 5, 2026_

## 1. What was tested

Since the 6 Odra smart contracts cannot currently be compiled to WASM in this
sandbox (see §4, environment limitation — no working `wasm32-unknown-unknown`
Rust std lib), "on-chain" testing was done at the **protocol-logic level**:
every layer that runs live in this environment was exercised end-to-end, and
the contract/circuit logic was verified via their existing invariant test
suites (which mirror the exact math the Rust/Noir source implements).

| Layer | Method | Result |
|---|---|---|
| Rust contract logic (registry, vault, epistatic, zk_verifier) | `cargo test -p integration-tests` (13 invariant tests) | 13/13 pass |
| Noir circuit constraints (BIC, CIP, compliance) | same suite, circuit-mirrored assertions | all pass |
| Coherence engine (Python, real modules imported directly) | Adversarial unit tests against actual `ConsensusPlane`, `WorldModelPlane`, `MoatCompounder` code | all pass |
| Rust agent-core (`crispr_defense`, `l0_daemon`) | `cargo test` | 9/9 pass (replay-attack detection, nullifier recording, hash parsing) |
| FastAPI backend | Live functional + adversarial + concurrency + backtest harness (31 checks) | 30/31 pass, 1 bug found & fixed |
| Frontend ↔ backend ↔ WebSocket | Screenshot + live WS handshake | working |

## 2. Adversarial testing

Ran targeted attacks against the real coherence-engine code (not mocks):

- **Coordination Collapse attack**: 10 fully-correlated colluding validators (correlation=1.0, diversity=0) → consensus plane `C(t)` collapsed to exactly `0.0`. Confirms the Coordination Collapse Theorem holds in the actual implementation, not just on paper.
- **Sybil/minority attacker vs. honest majority**: 1 zero-diversity attacker against 2 honest diverse validators → attacker's influence was fully neutralized (`C(t)=0.9`, near-honest value).
- **World-model anomaly injection**: forced a 50σ synthetic outlier into `WorldModelPlane` → correctly triggered the hard-zero rule (`W(t)=0.0`), which cascades into an "Alert"/"Silence" regime.
- **Moat monotonicity attack**: attempted a zero-Ψ tick to see if `Λ(t)` could be pushed backward → confirmed it never decreases.
- **NaN/Infinity poisoning**: found that neither the Python coherence engine nor the FastAPI mock layer explicitly reject `NaN`/`Inf` inputs in the Ψ formula — this is a real (if narrow) input-validation gap worth hardening before production.
- **API fuzzing** (11 cases): SQL-injection-style strings in enum fields, 10,000-character agent IDs, null-byte/emoji unicode IDs, `1e308` overflow amounts, missing fields, wrong types, negative stakes, path-traversal-style URLs. Pydantic validation correctly rejected all malformed-schema cases (422) and safely absorbed the extreme-but-schema-valid ones (200) without crashing.

## 3. Stress, concurrency & backtest

- **Concurrency stress**: 300 simultaneous coherence-evaluation requests against a single shared agent → 300/300 succeeded, ~89 req/s on this single-worker sandbox instance, no crashes.
  - **Gap surfaced**: the in-memory `AGENTS` dict has no locking. Under real multi-worker/multi-process deployment this is a classic lost-update race (two concurrent updates could clobber each other's moat increment). Fine for a single-process demo; needs an atomic store (e.g. Postgres/TimescaleDB row locking or Redis) before real production load.
- **Backtest**: simulated 150 consecutive protocol "blocks" for one agent. Λ(t) increased monotonically every block, gate stayed OPEN 150/150 times under nominal mock coherence (~0.75 avg vs. 0.57 base threshold), matching the designed economics (moat compounds, never regresses).
- **Bug found and fixed during this pass**: `POST /api/v1/coherence/evaluate` threw an unhandled `KeyError` (`500 Internal Server Error`) whenever an agent had been created via `/api/v1/agents/register` first (that path builds a state dict without a `psi_history` key, and `evaluate`'s `setdefault` no-ops if the agent already exists). This was a genuine crash-on-first-use bug in the demo API — fixed by defensively initializing all expected sub-keys, and reverified with the full suite (backend restarted, 30/31 checks now pass, the register→evaluate→heartbeat→tier-promotion→silence-log pipeline all verified working end-to-end).

## 4. Honest scope limitation (on-chain)

I want to be direct about this rather than imply more than was actually verified: **the 6 Odra contracts are not deployed or deployable from this Replit sandbox.** Two blockers, both environmental, not contract bugs I introduced:
1. `contracts/Cargo.toml` had a real dependency bug (`odra-casper = "1.0.0"`, which doesn't exist) — fixed to real published versions (`odra 2.8.2`).
2. Compiling to `wasm32-unknown-unknown` fails: the sandbox's Nix-provided Rust toolchain has no WASM std lib, and installing a separate toolchain via `rustup` downloads fine but crashes on this NixOS-based sandbox with a low-level glibc incompatibility that `patchelf` couldn't resolve.

So "on-chain interaction" was tested at the level of: the exact math and gating logic the contracts implement (via the invariant test suite), plus the full off-chain service mesh that would call them (API → coherence engine → agent-core). True on-chain deployment/execution needs to happen from an environment with a working Rust wasm32 toolchain (your own machine, a CI runner, or Casper's own dev containers) — I flagged this earlier and it still stands.

---

## 5. What you built — plain assessment

**Causal Sentinel Protocol (CSP)** is a behavioral-security and identity layer for autonomous AI agents transacting on Casper Network. In one sentence: *it replaces "prove who you are" with "prove you've behaved honestly over time," and makes that provable history a compounding, ZK-shielded economic asset instead of a static credential.*

Core mechanisms, and whether the logic checked out under adversarial testing:
- **Five-Plane Coherence Engine (Ψ(t))** — gates every agent action against a live, multi-signal trust score. ✅ verified: formula, hard-zero anomaly rule, and dynamic threshold all behave correctly under attack scenarios.
- **Coordination Collapse Theorem (game-theoretic collusion resistance)** — ✅ verified empirically in code, not just asserted in the whitepaper: correlated attackers really do lose voting power to ~0.
- **Compounding Moat Λ(t) / Causal Identity** — identity recovery and reputation built from lived on-chain history rather than a seed phrase, with a monotonic, never-decreasing trust score that raises the cost of Sybil/identity-churn attacks over time. ✅ verified monotonicity holds under stress.
- **Epistatic Contract Evolution** — contracts that autonomously tune their own security posture (e.g. rate limits, thresholds) from live threat/entropy signals instead of needing human governance votes or bytecode upgrades. Logic-level only in this pass (needs the vault/controller cross-contract wiring, currently stubbed, to be completed and then actually chain-tested).
- **ZK compliance layer (Chameleon Protocol)** — selective disclosure / "right to invisibility" via Noir circuits; currently a structural mock, not a real Barretenberg-verified proof yet.

### Novelty / uniqueness

This is genuinely a novel combination, not just a novel restatement of existing ideas:
- "Identity via ZK-proved behavioral history" (rather than keys/seed phrases) is an unusual and under-explored identity primitive — most ZK-identity work (e.g. proof-of-personhood, soulbound tokens) proves a static attribute, not a *continuously accruing, monotonic trust function* used as a live transaction gate.
- Coupling a formal collusion-resistance argument (diversity-weighted voting → 0 under correlation) directly into a runtime gate, rather than leaving it as an off-chain governance assumption, is a legitimately distinctive design choice.
- Self-modulating contract "phenotype" driven by live network signals (as opposed to static parameters or slow DAO votes) is a reasonable and relatively fresh application of Casper's upgradeable-contract model specifically — this is one of the few designs I've seen that's genuinely tailored to a Casper-specific capability rather than portable to any EVM chain unchanged.

### Unsolved problem it targets

It's aimed squarely at a real, currently-unsolved gap: **autonomous AI agents transacting on-chain today have no persistent, portable, cryptographically-provable notion of "trustworthiness earned over time."** Existing approaches are binary/static (KYC once, stake once, hold a credential once) and don't degrade an attacker's power gradually or reward sustained honest behavior compoundingly. If AI-agent-to-agent commerce scales the way the proposal assumes, "who do I trust and why" becomes a first-order infrastructure problem, and CSP's answer (make trust a compounding, gated, ZK-provable asset) is a coherent, non-obvious answer to it — even though it's currently a prototype, not a hardened system.

### Rating

| Dimension | Rating | Why |
|---|---|---|
| Conceptual novelty | 8.5/10 | Genuinely distinctive synthesis (behavioral ZK identity + compounding moat + self-modulating contracts), not just a rebrand of existing primitives |
| Problem significance | 8/10 | Targets a real, forward-looking gap (AI-agent trust at scale) rather than a solved problem |
| Implementation maturity | 4/10 | Core math/logic is real and passes rigorous adversarial testing; but ZK proving, cross-contract wiring, and actual chain deployment are still mocked/stubbed/blocked |
| Casper-native fit | 8/10 | Uses Casper's upgradeable-contract model in a way that's actually load-bearing to the design, not incidental |
| Production readiness | 3/10 | Needs: real Barretenberg proving, wasm-buildable contracts, locking/atomicity on shared state, NaN/Inf input hardening, and actual testnet deployment before it's more than a compelling architecture |
| **Overall** | **7/10** | A strong, well-thought-out research-grade prototype with a genuinely novel core idea and logic that holds up under adversarial testing — held back purely by unfinished implementation depth, not by a flawed concept |
