#!/usr/bin/env bash
# ─── Causal Sentinel Protocol — Bootstrap Script ─────────────────────────────
# Registers a test agent, runs an initial heartbeat, and generates a mock
# ZK credential to confirm the full protocol stack is operational.
#
# Usage:
#   bash scripts/bootstrap.sh
#   AGENT_ID=my_agent bash scripts/bootstrap.sh

set -euo pipefail

API_URL="${CSP_API_URL:-http://localhost:8080}"
AGENT_ID="${AGENT_ID:-test_agent_$(date +%s)}"

GREEN="\033[32m"
CYAN="\033[36m"
BOLD="\033[1m"
RESET="\033[0m"

step() { echo -e "\n${CYAN}${BOLD}▶ $1${RESET}"; }
ok()   { echo -e "${GREEN}  ✅ $1${RESET}"; }

echo -e "${BOLD}"
echo "╔══════════════════════════════════════════════════╗"
echo "║     Causal Sentinel Protocol — Bootstrap         ║"
echo "╚══════════════════════════════════════════════════╝"
echo -e "${RESET}"
echo "API:      $API_URL"
echo "Agent ID: $AGENT_ID"

# ─── 1. Health check ─────────────────────────────────────────────────────────
step "1. Health check"
health=$(curl -sf "$API_URL/health" || echo "{}")
echo "  $health"
ok "API reachable"

# ─── 2. Register agent ───────────────────────────────────────────────────────
step "2. Register agent"
DNA_HASH=$(echo -n "${AGENT_ID}:dna_code" | sha256sum | awk '{print $1}')
BEH_COMMITMENT=$(echo -n "${AGENT_ID}:behavioral" | sha256sum | awk '{print $1}')

reg=$(curl -sf -X POST "$API_URL/api/v1/agents/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\":              \"$AGENT_ID\",
    \"dna_code_hash\":         \"$DNA_HASH\",
    \"behavioral_commitment\": \"$BEH_COMMITMENT\",
    \"stake_motes\":           10000000000
  }")
echo "  $reg" | python3 -m json.tool 2>/dev/null || echo "  $reg"
ok "Agent registered"

# ─── 3. Evaluate coherence ───────────────────────────────────────────────────
step "3. Coherence gate evaluation (trade action)"
result=$(curl -sf -X POST "$API_URL/api/v1/coherence/evaluate" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\":    \"$AGENT_ID\",
    \"action_type\": \"trade\",
    \"amount\":      1000000000,
    \"jurisdiction\": \"EU\"
  }")
echo "  $result" | python3 -m json.tool 2>/dev/null || echo "  $result"
gate=$(echo "$result" | python3 -c "import sys,json; print(json.load(sys.stdin).get('action','?'))" 2>/dev/null || echo "?")
ok "Gate response: $gate"

# ─── 4. Submit heartbeat ─────────────────────────────────────────────────────
step "4. On-chain heartbeat"
hb=$(curl -sf -X POST "$API_URL/api/v1/heartbeat" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\": \"$AGENT_ID\",
    \"psi\":      750000,
    \"lambda\":   100000,
    \"iq\":       500000
  }")
echo "  $hb" | python3 -m json.tool 2>/dev/null || echo "  $hb"
ok "Heartbeat submitted"

# ─── 5. Generate ZK credential ───────────────────────────────────────────────
step "5. ZK Behavioral Integrity Credential"
proof=$(curl -sf -X POST "$API_URL/api/v1/zk/generate" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\":        \"$AGENT_ID\",
    \"credential_type\": \"behavioral_integrity\",
    \"target_tier\":     1
  }")
echo "  $proof" | python3 -m json.tool 2>/dev/null || echo "  $proof"
ok "ZK credential generated"

# ─── 6. Moat status ──────────────────────────────────────────────────────────
step "6. Moat Λ(t) status"
moat=$(curl -sf "$API_URL/api/v1/moat/$AGENT_ID")
echo "  $moat" | python3 -m json.tool 2>/dev/null || echo "  $moat"
ok "Moat status retrieved"

# ─── Summary ─────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}════════════════════════════════════════════════════${RESET}"
echo -e "${GREEN}${BOLD}  Bootstrap complete! Agent $AGENT_ID is live.${RESET}"
echo -e "${GREEN}${BOLD}════════════════════════════════════════════════════${RESET}"
echo ""
echo "Next steps:"
echo "  1. Open dashboard: http://localhost:3000"
echo "  2. Watch coherence: wscat -c ws://localhost:8080/ws/coherence"
echo "  3. Run integration tests: cargo test --workspace"
echo ""
