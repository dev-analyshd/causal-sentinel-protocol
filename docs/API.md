# Causal Sentinel Protocol — API Documentation

## Smart Contract APIs

### SentinelRegistry

#### `register_agent(agent_address, dna_code_hash, behavioral_commitment)`
Registers a new sentinel agent with minimum CSPR stake.

**Parameters:**
- `agent_address`: Casper account hash
- `dna_code_hash`: SHA3-256 of DNA code schedule
- `behavioral_commitment`: Initial behavioral history commitment

**Events:** `AgentRegistered`

#### `update_agent_state(agent, psi, lambda, iq, manipulation_detected)`
Updates agent coherence state (called by SentinelVault).

#### `recover_identity(new_address, old_address, causal_proof_hash, temporal_challenge)`
Recovers lost identity via causal history proof.

### SentinelVault

#### `deposit()`
Deposits CSPR into vault. Requires attached value.

#### `execute_action(request)`
Executes action with ZK-gated coherence check.

**Request fields:**
- `agent`: Agent address
- `action_type`: Transfer | Trade | Stake | Unstake | CrossChainBridge | CredentialRenewal
- `amount`: Amount in motes
- `zk_proof`: BIC ZK proof bytes
- `nullifier`: ZK nullifier
- `compliance_tier`: Credential tier (1-5)

#### `heartbeat(psi, lambda, iq)`
On-chain heartbeat. Must be called every 100 blocks.

### EpistaticController

#### `compute_el_state()`
Computes and updates EL_state(t) from environmental signals.

#### `update_signals(threat, health, entropy)`
Updates environmental signals (called by oracle/ANIMA).

#### `calibrate_weights(outcome)`
Online weight calibration via gradient descent.

## x402 Facilitator API

### `GET /services`
Lists all available agent services with pricing.

### `POST /pay`
Processes micropayment for agent service.

### `GET /balance/:agent_id`
Returns agent balance and spending history.

## MCP Server API

### `POST /mcp/parse`
Parses natural language into structured action.

### `POST /mcp/execute`
Executes parsed action with ZK compliance.

## Federation Protocol API

### `POST /federation/register`
Registers agent as federation peer.

### `GET /federation/discover`
Discovers peers by tier and domain.

### `POST /federation/exchange`
Exchanges coherence scores with peer.
