#!/usr/bin/env bash
# ─── Write CASPER_PRIVATE_KEY secret to keys/sentinel.pem ────────────────────
#
# Run this before deploy_testnet.sh if keys/sentinel.pem doesn't exist.
# The key is read from the CASPER_PRIVATE_KEY environment variable (Replit Secret).
#
# Usage: bash scripts/setup_key.sh

set -euo pipefail

mkdir -p keys

if [[ -z "${CASPER_PRIVATE_KEY:-}" ]]; then
    echo "❌ CASPER_PRIVATE_KEY environment variable is not set."
    echo "   Add it as a Replit Secret (the PEM block of your EC private key)."
    exit 1
fi

echo "$CASPER_PRIVATE_KEY" > keys/sentinel.pem
chmod 600 keys/sentinel.pem
echo "✅ keys/sentinel.pem written ($(wc -c < keys/sentinel.pem) bytes, mode 600)"
