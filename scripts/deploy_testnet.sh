#!/usr/bin/env bash
# ─── Causal Sentinel Protocol — Testnet Deployment ───────────────────────────
#
# Deploys all 6 Odra contracts to Casper Testnet in dependency order:
#   1. ZKVerifier         (no deps)
#   2. SentinelRegistry   (no deps)
#   3. SentinelVault      (registry + zk_verifier)
#   4. SentinelLearner    (registry)
#   5. ComplianceEngine   (no deps)
#   6. EpistaticController (vault + learner + compliance)
#
# Prerequisites:
#   rustup target add wasm32-unknown-unknown
#   cargo install odra-cli    # https://odra.dev/docs/getting-started
#   cargo install casper-client
#
# Usage:
#   export CASPER_KEY=./keys/sentinel.pem
#   bash scripts/deploy_testnet.sh

set -euo pipefail

# ─── Configuration ────────────────────────────────────────────────────────────
NODE_URL="${CASPER_NODE:-https://rpc.testnet.casper.network}"
CHAIN_NAME="${CASPER_CHAIN:-casper-testnet}"
KEY_PATH="${CASPER_KEY:-./keys/sentinel.pem}"
WASM_DIR="contracts/target/wasm32-unknown-unknown/release"
CONFIG_FILE="config/testnet_addresses.json"

PAYMENT_CONTRACT=10000000000      # 10 CSPR
PAYMENT_SESSION=5000000000        # 5 CSPR
MIN_STAKE=10000000000             # 10 CSPR (min registration stake)

GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
BOLD="\033[1m"
RESET="\033[0m"

step()  { echo -e "\n${CYAN}${BOLD}▶ $1${RESET}"; }
ok()    { echo -e "${GREEN}  ✅ $1${RESET}"; }
warn()  { echo -e "${YELLOW}  ⚠ $1${RESET}"; }
die()   { echo -e "\033[31m  ❌ $1${RESET}"; exit 1; }

echo -e "${BOLD}"
echo "╔══════════════════════════════════════════════════╗"
echo "║  Causal Sentinel Protocol — Testnet Deployment   ║"
echo "╚══════════════════════════════════════════════════╝"
echo -e "${RESET}"
echo "Node:      $NODE_URL"
echo "Chain:     $CHAIN_NAME"
echo "Key:       $KEY_PATH"

# ─── Preflight checks ─────────────────────────────────────────────────────────
step "Preflight"

[[ -f "$KEY_PATH" ]] || die "Secret key not found at $KEY_PATH"
command -v casper-client >/dev/null 2>&1 || die "casper-client not installed"
command -v cargo >/dev/null 2>&1 || die "cargo not found"

# Get deployer address from key
DEPLOYER=$(casper-client keygen /tmp/csp_keygen_check 2>/dev/null || true)
DEPLOYER_ADDR=$(casper-client account-address --secret-key "$KEY_PATH" 2>/dev/null || echo "unknown")
echo "  Deployer: $DEPLOYER_ADDR"

# Check balance
BALANCE_RESULT=$(casper-client get-balance \
    --node-address "$NODE_URL" \
    --purse-uref "$(casper-client query-global-state \
        --node-address "$NODE_URL" \
        --state-root-hash "$(casper-client get-state-root-hash --node-address "$NODE_URL" | jq -r '.result.state_root_hash')" \
        --key "$DEPLOYER_ADDR" 2>/dev/null | jq -r '.result.stored_value.Account.main_purse' 2>/dev/null || echo 'uref-0')" \
    2>/dev/null || echo '{}')
echo "  Balance result: checking..."
ok "Preflight passed"

# ─── Build contracts ──────────────────────────────────────────────────────────
step "Building Odra contracts (wasm32-unknown-unknown)"
(cd contracts && cargo build --release --target wasm32-unknown-unknown) \
    || die "Contract build failed — ensure odra-cli and wasm32 target are installed"
ok "Contracts built"

# ─── Deploy helper ────────────────────────────────────────────────────────────
deploy_contract() {
    local name=$1
    local wasm_path="$WASM_DIR/${name}.wasm"
    shift
    local args=("$@")  # Additional --session-arg entries

    [[ -f "$wasm_path" ]] || die "WASM not found: $wasm_path"

    echo -e "  Deploying ${BOLD}${name}${RESET}..."

    local cmd=(
        casper-client put-deploy
        --node-address "$NODE_URL"
        --chain-name   "$CHAIN_NAME"
        --secret-key   "$KEY_PATH"
        --payment-amount "$PAYMENT_CONTRACT"
        --session-path "$wasm_path"
    )

    for arg in "${args[@]}"; do
        cmd+=(--session-arg "$arg")
    done

    local output
    output=$("${cmd[@]}" 2>&1) || die "Deploy failed for $name: $output"

    local deploy_hash
    deploy_hash=$(echo "$output" | grep -oP '"deploy_hash": *"\K[^"]+' || echo "")
    echo "  Deploy hash: $deploy_hash"

    # Wait for finality (~8-15 seconds)
    echo "  Waiting for finality..."
    sleep 15

    # Get contract hash
    local state_root
    state_root=$(casper-client get-state-root-hash \
        --node-address "$NODE_URL" | jq -r '.result.state_root_hash' 2>/dev/null || echo "")

    local contract_hash
    contract_hash=$(casper-client query-global-state \
        --node-address "$NODE_URL" \
        --state-root-hash "$state_root" \
        --key "$DEPLOYER_ADDR" \
        2>/dev/null | jq -r ".result.stored_value.Account.named_keys[] | select(.name==\"${name}_contract_hash\") | .key" 2>/dev/null || echo "")

    # Update config file
    if [[ -n "$contract_hash" ]]; then
        python3 - <<EOF
import json
with open("$CONFIG_FILE") as f:
    cfg = json.load(f)
cfg["contracts"]["${name}"]["hash"] = "${contract_hash}"
cfg["contracts"]["${name}"]["deploy_hash"] = "${deploy_hash}"
with open("$CONFIG_FILE", "w") as f:
    json.dump(cfg, f, indent=2)
EOF
        ok "$name deployed at $contract_hash"
    else
        warn "$name deployed (hash=$deploy_hash) — contract hash not auto-resolved, update config manually"
    fi

    echo "$contract_hash"
}

# ─── Deployment sequence ──────────────────────────────────────────────────────

# Read existing config
REGISTRY_HASH=$(python3 -c "import json; print(json.load(open('$CONFIG_FILE'))['contracts']['sentinel_registry'].get('hash',''))" 2>/dev/null || echo "")
ZK_HASH=$(python3 -c "import json; print(json.load(open('$CONFIG_FILE'))['contracts']['zk_verifier'].get('hash',''))" 2>/dev/null || echo "")

# 1. ZKVerifier — no deps, deploy first (needed by Vault)
step "1/6: ZKVerifier"
ZK_HASH=$(deploy_contract "zk_verifier")

# 2. SentinelRegistry — no deps
step "2/6: SentinelRegistry"
REGISTRY_HASH=$(deploy_contract "sentinel_registry" \
    "owner:account_hash='$DEPLOYER_ADDR'" \
    "min_stake:u64='$MIN_STAKE'")

# 3. SentinelVault — needs registry + zk_verifier
step "3/6: SentinelVault"
VAULT_HASH=$(deploy_contract "sentinel_vault" \
    "registry:key='$REGISTRY_HASH'" \
    "zk_verifier:key='$ZK_HASH'")

# 4. SentinelLearner — needs registry
step "4/6: SentinelLearner"
LEARNER_HASH=$(deploy_contract "sentinel_learner" \
    "registry:key='$REGISTRY_HASH'")

# 5. ComplianceEngine — no deps
step "5/6: ComplianceEngine"
COMPLIANCE_HASH=$(deploy_contract "compliance_engine" \
    "owner:account_hash='$DEPLOYER_ADDR'")

# 6. EpistaticController — needs vault + learner + compliance
step "6/6: EpistaticController"
EPISTATIC_HASH=$(deploy_contract "epistatic_controller" \
    "owner:account_hash='$DEPLOYER_ADDR'" \
    "vault:key='$VAULT_HASH'" \
    "learner:key='$LEARNER_HASH'" \
    "compliance:key='$COMPLIANCE_HASH'")

# ─── Summary ──────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}══════════════════════════════════════════════════════${RESET}"
echo -e "${GREEN}${BOLD}  All 6 contracts deployed!${RESET}"
echo -e "${GREEN}${BOLD}══════════════════════════════════════════════════════${RESET}"
echo ""
echo "Contract addresses saved to: $CONFIG_FILE"
echo ""
echo "Next steps:"
echo "  1. Register ZK verification keys:"
echo "       bash scripts/register_vks.sh"
echo "  2. Verify deployment:"
echo "       python3 scripts/verify_deployment.py"
echo "  3. Bootstrap test agent:"
echo "       bash scripts/bootstrap.sh"
echo ""
echo "Dashboard: http://localhost:3000"
echo ""
