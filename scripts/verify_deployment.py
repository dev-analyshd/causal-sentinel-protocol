#!/usr/bin/env python3
"""
Causal Sentinel Protocol — Deployment Verification

Queries Casper testnet to confirm each contract is deployed and responsive.
"""

import json
import os
import sys
import urllib.request
import urllib.error
from typing import Any, Dict, Optional

NODE_URL = os.environ.get("CASPER_NODE", "https://rpc.testnet.casper.network")
CONFIG_PATH = os.path.join(os.path.dirname(__file__), "..", "config", "testnet_addresses.json")

ANSI_GREEN = "\033[32m"
ANSI_RED   = "\033[31m"
ANSI_RESET = "\033[0m"
ANSI_BOLD  = "\033[1m"

ok  = lambda s: print(f"  {ANSI_GREEN}✅{ANSI_RESET} {s}")
err = lambda s: print(f"  {ANSI_RED}❌{ANSI_RESET} {s}")
hdr = lambda s: print(f"\n{ANSI_BOLD}{s}{ANSI_RESET}")


def rpc(method: str, params: Any) -> Optional[Dict]:
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode()
    req = urllib.request.Request(
        f"{NODE_URL}/rpc",
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            data = json.loads(resp.read())
            return data.get("result")
    except urllib.error.HTTPError as e:
        err(f"HTTP {e.code}: {e.read().decode()}")
        return None
    except Exception as e:
        err(str(e))
        return None


def check_node() -> bool:
    hdr("1. Node connectivity")
    result = rpc("info_get_status", {})
    if result:
        chain = result.get("chainspec_name", "?")
        block = result.get("last_added_block_info", {}).get("height", "?")
        ok(f"Node reachable — chain={chain}, last_block={block}")
        return True
    err("Node unreachable")
    return False


def check_contract(name: str, hash_value: Optional[str]) -> bool:
    if not hash_value:
        err(f"{name}: no hash in config (not yet deployed)")
        return False
    result = rpc("chain_get_state_root_hash", {})
    if not result:
        err(f"{name}: could not get state root hash")
        return False
    state_root = result.get("state_root_hash")
    result2 = rpc("state_get_item", {
        "state_root_hash": state_root,
        "key": hash_value,
        "path": [],
    })
    if result2 and result2.get("stored_value"):
        ok(f"{name}: deployed at {hash_value[:20]}...")
        return True
    err(f"{name}: not found at {hash_value[:20]}...")
    return False


def check_service(name: str, url: str) -> bool:
    try:
        req = urllib.request.Request(url + "/health")
        with urllib.request.urlopen(req, timeout=5) as resp:
            data = json.loads(resp.read())
            if data.get("status") == "ok":
                ok(f"{name}: healthy at {url}")
                return True
            err(f"{name}: unhealthy — {data}")
            return False
    except Exception as e:
        err(f"{name}: unreachable ({e})")
        return False


def main():
    print(f"\n{ANSI_BOLD}{'='*60}{ANSI_RESET}")
    print(f"{ANSI_BOLD}  Causal Sentinel Protocol — Deployment Verification{ANSI_RESET}")
    print(f"{ANSI_BOLD}{'='*60}{ANSI_RESET}")

    with open(CONFIG_PATH) as f:
        config = json.load(f)

    contracts = config.get("contracts", {})

    passed = 0
    total  = 0

    # 1. Node check
    total += 1
    if check_node():
        passed += 1

    # 2. Contracts
    hdr("2. Contract verification")
    order = [
        "zk_verifier",
        "sentinel_registry",
        "sentinel_vault",
        "sentinel_learner",
        "compliance_engine",
        "epistatic_controller",
    ]
    for name in order:
        total += 1
        h = contracts.get(name, {}).get("hash")
        if check_contract(name, h):
            passed += 1

    # 3. Off-chain services
    hdr("3. Off-chain services")
    services = [
        ("CSP API",         "http://localhost:8080"),
        ("x402 Facilitator","http://localhost:8081"),
        ("MCP Server",      "http://localhost:8082"),
        ("Federation",      "http://localhost:8083"),
    ]
    for svc_name, url in services:
        total += 1
        if check_service(svc_name, url):
            passed += 1

    # Summary
    print(f"\n{ANSI_BOLD}{'='*60}{ANSI_RESET}")
    color = ANSI_GREEN if passed == total else ANSI_RED
    print(f"{color}{ANSI_BOLD}  Result: {passed}/{total} checks passed{ANSI_RESET}")
    print(f"{ANSI_BOLD}{'='*60}{ANSI_RESET}\n")

    sys.exit(0 if passed == total else 1)


if __name__ == "__main__":
    main()
