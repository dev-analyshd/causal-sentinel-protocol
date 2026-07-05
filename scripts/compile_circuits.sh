#!/bin/bash
set -e

echo "🔱 Compiling ZK Circuits"
echo "========================="

cd circuits/

for circuit in behavioral_integrity causal_identity sentinel_compliance; do
    echo "Compiling ${circuit}..."
    cd "$circuit"
    nargo compile
    nargo prove
    cd ..
    echo "✅ ${circuit} compiled"
done

echo ""
echo "All circuits compiled. Verification keys generated."
