#!/bin/bash
set -e

echo "🔱 Causal Sentinel Protocol — Testnet Deployment"
echo "================================================"

# Configuration
NODE_URL="${CASPER_NODE_URL:-http://localhost:7777}"
CHAIN_NAME="${CASPER_CHAIN:-casper-testnet}"
KEY_PATH="${CASPER_KEY:-./keys/sentinel.pem}"

# Build contracts
echo "Building contracts..."
cd contracts/
cargo build --release --target wasm32-unknown-unknown

# Deploy each contract
deploy_contract() {
    local name=$1
    local wasm_path="target/wasm32-unknown-unknown/release/${name}.wasm"

    echo "Deploying ${name}..."
    casper-client put-deploy \
        --node-address "$NODE_URL" \
        --chain-name "$CHAIN_NAME" \
        --secret-key "$KEY_PATH" \
        --payment-amount 10000000000 \
        --session-path "$wasm_path"

    echo "✅ ${name} deployed"
}

deploy_contract "sentinel_registry"
deploy_contract "sentinel_vault"
deploy_contract "sentinel_learner"
deploy_contract "compliance_engine"
deploy_contract "epistatic_controller"
deploy_contract "zk_verifier"

echo ""
echo "All contracts deployed successfully!"
echo "Update contract hashes in config files."
