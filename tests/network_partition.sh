#!/bin/bash
set -e

echo "[TEST] Initiating Network Partition Simulation..."

# 1. Start the agent
./target/release/workforce-agent &
AGENT_PID=$!
sleep 2

# 2. Simulate Network Blackhole (Drop outbound packets to AWS API Gateway)
echo "[TEST] Dropping outbound TLS traffic to AWS (Simulating Offline State)..."
if [ "$(uname)" == "Darwin" ]; then
    # macOS pfctl
    echo "block drop out proto tcp to any port 443" | sudo pfctl -f - -e
else
    # Linux/CI iptables
    sudo iptables -A OUTPUT -p tcp --dport 443 -j DROP
fi

# 3. Feed 100 mock events into the agent
echo "[TEST] Generating 100 telemetry events while offline..."
for i in {1..100}; do
    curl -s -X POST -d "MockEvent_$i" http://localhost:9091/mock-os-event > /dev/null
done
sleep 3

# 4. Verify SQLite WAL Cache Growth
CACHE_COUNT=$(sqlite3 C:/ProgramData/WorkforceOS/telemetry_cache.db "SELECT COUNT(*) FROM queued_payloads;")
echo "[TEST] Offline Cache Count: $CACHE_COUNT"

if [ "$CACHE_COUNT" -ne 100 ]; then
    echo "[FATAL] Cache mismatch. Expected 100, got $CACHE_COUNT. Data lost during partition."
    kill $AGENT_PID
    exit 1
fi

# 5. Restore Network Connectivity
echo "[TEST] Restoring network connectivity..."
if [ "$(uname)" == "Darwin" ]; then
    sudo pfctl -d
else
    sudo iptables -D OUTPUT -p tcp --dport 443 -j DROP
fi

# Allow time for the agent's reconnect heuristic to fire and flush the DB
sleep 10 

# 6. Verify SQLite DB is empty (Flushed)
FINAL_COUNT=$(sqlite3 C:/ProgramData/WorkforceOS/telemetry_cache.db "SELECT COUNT(*) FROM queued_payloads;")
echo "[TEST] Post-Flush Cache Count: $FINAL_COUNT"

if [ "$FINAL_COUNT" -ne 0 ]; then
    echo "[FATAL] Cache flush failed. Expected 0, got $FINAL_COUNT."
    kill $AGENT_PID
    exit 1
fi

echo "[SUCCESS] Network partition resiliency and zero-data-loss cache verified."
kill $AGENT_PID
exit 0
