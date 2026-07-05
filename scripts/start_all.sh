#!/bin/bash
set -e

echo "🔱 Starting Causal Sentinel Protocol Stack"
echo "============================================="

# Start L0 Daemon
echo "Starting L0 Daemon..."
cd agent-core/rust/l0_daemon
cargo run --release &
L0_PID=$!
cd ../../..

# Start Coherence Engine
echo "Starting Coherence Engine..."
cd agent-core/python/coherence_engine
poetry run python src/main.py &
COH_PID=$!
cd ../../..

# Start ANIMA Crawler
echo "Starting ANIMA Crawler..."
cd agent-core/python/anima_crawler
poetry run python src/main.py &
ANIMA_PID=$!
cd ../../..

# Start x402 Facilitator
echo "Starting x402 Facilitator..."
cd agent-economy/x402_facilitator
cargo run --release &
X402_PID=$!
cd ../..

# Start MCP Server
echo "Starting MCP Server..."
cd agent-economy/mcp_server
cargo run --release &
MCP_PID=$!
cd ../..

# Start Federation Protocol
echo "Starting Federation Protocol..."
cd agent-economy/federation_protocol
cargo run --release &
FED_PID=$!
cd ../..

# Start Frontend
echo "Starting Dashboard..."
cd frontend/dashboard
pnpm dev &
DASH_PID=$!
cd ../..

echo ""
echo "All services started!"
echo "Dashboard: http://localhost:3000"
echo "x402 API: http://localhost:8080"
echo "MCP API: http://localhost:8081"
echo "Federation: http://localhost:8082"
echo "L0 WebSocket: ws://localhost:9001"
echo ""
echo "Press Ctrl+C to stop all services"

# Wait for interrupt
trap "kill $L0_PID $COH_PID $ANIMA_PID $X402_PID $MCP_PID $FED_PID $DASH_PID; exit" INT
wait
