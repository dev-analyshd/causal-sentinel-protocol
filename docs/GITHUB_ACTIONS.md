# GitHub Actions Guide — Causal Sentinel Protocol

Two workflow files cover everything from PR checks to testnet deployment.

```
.github/workflows/
├── ci.yml                 — runs on every push/PR (lint, build, test)
└── deploy-contracts.yml   — builds wasm + optionally deploys to Casper Testnet
```

---

## Workflow 1: CI (`ci.yml`)

**Triggers:** every push to `main`/`develop`, every PR targeting `main`.

### Jobs at a glance

| Job | What it checks | Fails PR? |
|---|---|---|
| `rust-build-test` | Agent core + economy (stable Rust, host target) | Yes |
| `contracts-build` | Odra contracts → wasm32 (nightly Rust) | Soft (continue-on-error) |
| `circuits-check` | Noir ZK circuits via `nargo check` | Soft |
| `python-coherence` | Coherence engine imports + Poetry deps | Soft |
| `frontend-build` | TypeScript type-check + Vite production build | Yes |

> Soft jobs use `continue-on-error: true` because they need optional tooling (Nargo, full Casper toolchain). Hard failures in `rust-build-test` and `frontend-build` block merges.

### What you need to add on GitHub

**No secrets required for CI.** Everything runs without credentials. Just push and it runs.

**Optional:** add a branch protection rule on `main` requiring these status checks to pass before merge:
- `Rust Build & Test`
- `Frontend Build (React + Vite)`

_(Settings → Branches → Add rule → "Require status checks to pass")_

---

## Workflow 2: Build & Deploy (`deploy-contracts.yml`)

**Triggers:**
- **Push to `main`** touching `contracts/**` — builds `.wasm` files and uploads them as artifacts (no deploy)
- **Pull request** touching `contracts/**` — same, build-only
- **Manual trigger** (`workflow_dispatch`) — build + optionally deploy to Casper Testnet

### Jobs

```
build-wasm   ──────────────────────────────────────────► (always)
                                                         upload .wasm artifacts
                                                         (30-day retention)

deploy-testnet ◄── needs: build-wasm                    (only when manually
                   only if deploy input = true)          triggered with deploy=true)
```

### First-time setup: secrets and variables

Go to your repo → **Settings → Secrets and variables → Actions**.

#### Repository secret (required for deploy)

| Secret name | Value |
|---|---|
| `CASPER_SECRET_KEY_PEM` | Your EC private key PEM block (the full `-----BEGIN EC PRIVATE KEY-----` block) |

> This is the same key stored as `CASPER_PRIVATE_KEY` in Replit Secrets. Paste the same value here.

#### Repository variables (optional overrides)

| Variable name | Default | When to set |
|---|---|---|
| `CASPER_NODE` | `https://rpc.testnet.casper.network` | Custom node or sidecar |
| `CASPER_CHAIN` | `casper-testnet` | Mainnet: `casper` |

Variables live under the same **Secrets and variables** page, on the **Variables** tab.

#### GitHub Environment (recommended)

The deploy job uses `environment: casper-testnet`. Create this environment to add an approval gate:

1. Settings → Environments → New environment → name it `casper-testnet`
2. Enable **Required reviewers** — add yourself (or your team)
3. Now every deploy needs a human to click Approve before contracts go live

---

## Running a deploy

1. Go to **Actions → Build & Deploy Contracts (Casper Testnet)**
2. Click **Run workflow**
3. Select branch `main`
4. Set **"Deploy the freshly built .wasm files to Casper Testnet"** → `true`
5. Click **Run workflow**
6. If you set up the `casper-testnet` environment with reviewers, approve when prompted

After a successful deploy:
- `.wasm` artifacts are uploaded (Actions → the run → Artifacts)
- `config/testnet_addresses.json` is auto-committed back to `main` with the deployed contract hashes
- A deployment summary table appears on the workflow run page

---

## The nightly Rust requirement

Odra macros (`odra-macros 2.8.2`) use `#![feature(box_patterns)]`, which only compiles on nightly Rust.

All contract jobs in both workflow files use:
```yaml
- uses: dtolnay/rust-toolchain@nightly
  with:
    targets: wasm32-unknown-unknown
```

`contracts/rust-toolchain.toml` pins `channel = "nightly"` so local builds also pick up nightly automatically — you don't need `+nightly` flags.

The agent core and economy (non-contract Rust) still compile on stable and use `dtolnay/rust-toolchain@stable`.

---

## Caching strategy

Each job has its own cache keyed to the relevant lockfile:

| Job | Cache key | Cache path |
|---|---|---|
| `rust-build-test` | `Cargo.lock` | `~/.cargo/registry`, `~/.cargo/git`, `target/` |
| `contracts-build` | `contracts/Cargo.lock` | `~/.cargo/registry`, `~/.cargo/git`, `contracts/target` |
| `deploy-testnet` | `contracts/Cargo.lock` | `~/.cargo/registry`, `~/.cargo/bin` (casper-client binary) |
| `python-coherence` | `poetry.lock` | `agent-core/python/coherence_engine/.venv` |
| `frontend-build` | `package.json` | npm cache (via `setup-node cache: 'npm'`) |

Caches restore on exact key match, then fall back to prefix (`runner.os-cargo-`). First run will be slow; subsequent runs typically cut 2–4 minutes off build times.

---

## Estimated run times

| Workflow | Cold (no cache) | Warm (cached) |
|---|---|---|
| CI — rust + frontend only | ~8 min | ~3 min |
| CI — all jobs | ~12 min | ~5 min |
| Deploy — build-wasm | ~15 min | ~6 min |
| Deploy — full (build + deploy) | ~25 min | ~14 min |

The `casper-client` binary is compiled from source on first deploy (~8 min alone) and then cached.

---

## Troubleshooting

### `box_patterns` error
```
error[E0554]: `#![feature]` may not be used on the stable release channel
```
**Fix:** the workflow must use `dtolnay/rust-toolchain@nightly`, not `@stable`. Both workflow files are already corrected. If this appears, check that no workflow is overriding the toolchain with `toolchain: stable`.

### `CASPER_SECRET_KEY_PEM is not set`
The deploy job will print `::error::` and exit. Add the secret under Settings → Secrets and variables → Actions → Repository secrets.

### Contract hash not resolved after deploy
The script tries to read the contract hash from the deployer account's named keys. If the named key format from your Odra version differs, the hash will show as `_pending_` in the summary and `config/testnet_addresses.json` won't be updated automatically. In that case:
1. Find the deploy hash in the workflow logs
2. Look up the contract hash manually: `casper-client query-global-state --key <account-hash> ...`
3. Update `config/testnet_addresses.json` by hand and commit it

### `casper-client` build fails in CI
Casper client compiles from source on first run. If it times out (default 6h GitHub limit is usually fine), check for upstream breakage on `crates.io/crates/casper-client`. Pin to a known-good version:
```yaml
run: cargo install casper-client --version =X.Y.Z --locked
```

### Frontend cache miss every run
The cache key uses `package.json` content. If you're seeing constant misses, switch to `package-lock.json`:
```yaml
cache-dependency-path: frontend/dashboard/package-lock.json
```

---

## Adding mainnet deployment

When ready for Casper Mainnet, add a second `workflow_dispatch` input and a second deploy job:

```yaml
inputs:
  deploy_target:
    description: "Target network"
    type: choice
    options: [testnet, mainnet, none]
    default: none
```

Create a separate `casper-mainnet` GitHub Environment with stricter reviewer requirements and a separate `CASPER_MAINNET_KEY_PEM` secret. Never reuse the testnet key on mainnet.
