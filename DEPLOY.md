# Deploying CSP Contracts to Casper Testnet

## Prerequisites (local machine or CI — not needed on Replit)

```bash
# 1. Rust nightly + wasm32 target
# Odra macros require nightly (uses #![feature(box_patterns)])
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
# contracts/rust-toolchain.toml pins nightly automatically when you cd into contracts/

# 2. Casper client
cargo install casper-client

# 3. Clone this repo
git clone <your-repo-url>
cd causal-sentinel-protocol
```

> **Note:** Contract compilation targets `wasm32-unknown-unknown`, which cannot be built
> in the Replit NixOS sandbox. All other steps (key management, API, dashboard) run fine on Replit.

---

## Setup

### 1. Private key

Your `CASPER_PRIVATE_KEY` is stored as a Replit Secret. To use it locally:

```bash
# Export from your environment, then write to file
echo "$CASPER_PRIVATE_KEY" > keys/sentinel.pem
chmod 600 keys/sentinel.pem
```

Or on Replit (after secrets are set):
```bash
bash scripts/setup_key.sh
```

### 2. Get testnet CSPR

Each deployment costs ~60 CSPR total (10 CSPR × 6 contracts).

- Faucet: https://testnet.cspr.live/tools/faucet
- Your public key: run `casper-client account-address --secret-key keys/sentinel.pem`

---

## Build Contracts

```bash
cd contracts
cargo build --release --target wasm32-unknown-unknown
```

Artifacts land in `contracts/target/wasm32-unknown-unknown/release/*.wasm`.

---

## Deploy All 6 Contracts

```bash
export CASPER_KEY=./keys/sentinel.pem
bash scripts/deploy_testnet.sh
```

This deploys in dependency order:
1. `zk_verifier` (no deps)
2. `sentinel_registry` (no deps)
3. `sentinel_vault` (needs registry + zk_verifier)
4. `sentinel_learner` (needs registry)
5. `compliance_engine` (no deps)
6. `epistatic_controller` (needs vault + learner + compliance)

Contract addresses are saved to `config/testnet_addresses.json` automatically.

---

## Post-deployment

```bash
# Verify all contracts are live
python3 scripts/verify_deployment.py

# Bootstrap a test agent
bash scripts/bootstrap.sh
```

Then update the backend API with the deployed contract addresses — copy them from
`config/testnet_addresses.json` into `api/main.py` (search for `CONTRACT_ADDRESSES`).

---

## Using a Custom Node or Chain

```bash
export CASPER_NODE=https://rpc.testnet.casper.network   # default
export CASPER_CHAIN=casper-testnet                       # default
export CASPER_KEY=./keys/sentinel.pem
bash scripts/deploy_testnet.sh
```

---

## GitHub Actions (CI deployment)

Add `CASPER_PRIVATE_KEY` as a GitHub Actions secret, then use this workflow:

```yaml
name: Deploy to Casper Testnet
on:
  workflow_dispatch:

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          targets: wasm32-unknown-unknown
          # Odra macros require nightly (box_patterns feature)
      - name: Install casper-client
        run: cargo install casper-client
      - name: Write key
        run: |
          mkdir -p keys
          echo "${{ secrets.CASPER_PRIVATE_KEY }}" > keys/sentinel.pem
          chmod 600 keys/sentinel.pem
      - name: Deploy contracts
        run: bash scripts/deploy_testnet.sh
      - name: Upload addresses
        uses: actions/upload-artifact@v4
        with:
          name: testnet-addresses
          path: config/testnet_addresses.json
```
