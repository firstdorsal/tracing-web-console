#!/bin/bash
# Development script that watches for changes and auto-restarts the server
#
# Prerequisites:
#   cargo install cargo-watch
#
# Usage:
#   ./dev.sh

set -e

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "========================================="
    echo "cargo-watch is not installed."
    echo "Installing cargo-watch (this may take a few minutes)..."
    echo "========================================="
    echo ""

    if cargo install cargo-watch; then
        echo ""
        echo "========================================="
        echo "✓ cargo-watch installed successfully!"
        echo "========================================="
        echo ""
    else
        echo ""
        echo "========================================="
        echo "✗ Failed to install cargo-watch"
        echo "Please install it manually: cargo install cargo-watch"
        echo "========================================="
        exit 1
    fi
fi

echo "✓ cargo-watch is available"
echo ""

# Cleanup function to kill background processes
cleanup() {
    echo ""
    echo "========================================="
    echo "Shutting down..."
    echo "========================================="
    if [ ! -z "$CARGO_WATCH_PID" ]; then
        kill $CARGO_WATCH_PID 2>/dev/null || true
    fi
    if [ ! -z "$LOG_GENERATOR_PID" ]; then
        kill $LOG_GENERATOR_PID 2>/dev/null || true
    fi
    exit 0
}

# Set up trap to cleanup on exit
trap cleanup SIGINT SIGTERM EXIT

echo "Starting development server with auto-reload..."
echo "The server will automatically rebuild and restart when files change."
echo "Log events will be generated automatically every 5-15 seconds."
echo ""
echo "Press Ctrl+C to stop"
echo "========================================="
echo ""

# Start cargo watch in the background
cargo watch -x "run --package example-server" \
    -w ../example-server/src \
    -w src \
    -w frontend/src \
    -w frontend/package.json \
    -w frontend/vite.config.ts \
    -w frontend/tailwind.config.js \
    -w Cargo.toml &
CARGO_WATCH_PID=$!

# Wait for server to start
echo "Waiting for server to start..."
sleep 8

# Function to generate log events
generate_logs() {
    local base_url="http://localhost:3000"

    while true; do
        # Random delay between 5 and 15 seconds
        sleep $((5 + RANDOM % 11))

        # Randomly choose an endpoint to call
        case $((RANDOM % 6)) in
            0)
                echo "[$(date +%H:%M:%S)] Generating logs: Creating user..."
                curl -s -X POST "$base_url/api/users" \
                    -H "Content-Type: application/json" \
                    -d '{"username":"User'$RANDOM'","email":"user'$RANDOM'@example.com"}' \
                    > /dev/null 2>&1 || true
                ;;
            1)
                echo "[$(date +%H:%M:%S)] Generating logs: Listing users..."
                curl -s "$base_url/api/users" > /dev/null 2>&1 || true
                ;;
            2)
                echo "[$(date +%H:%M:%S)] Generating logs: Creating product..."
                curl -s -X POST "$base_url/api/products" \
                    -H "Content-Type: application/json" \
                    -d '{"name":"Product'$RANDOM'","price":'$((10 + RANDOM % 90))',"stock":'$((5 + RANDOM % 50))'}' \
                    > /dev/null 2>&1 || true
                ;;
            3)
                echo "[$(date +%H:%M:%S)] Generating logs: Listing products..."
                curl -s "$base_url/api/products" > /dev/null 2>&1 || true
                ;;
            4)
                echo "[$(date +%H:%M:%S)] Generating logs: Creating order..."
                curl -s -X POST "$base_url/api/orders" \
                    -H "Content-Type: application/json" \
                    -d '{"user_id":"user-'$RANDOM'","product_ids":["prod-'$((1 + RANDOM % 100))'"],"total":'$((10 + RANDOM % 200))'.99}' \
                    > /dev/null 2>&1 || true
                ;;
            5)
                echo "[$(date +%H:%M:%S)] Generating logs: Listing orders..."
                curl -s "$base_url/api/orders" > /dev/null 2>&1 || true
                ;;
        esac
    done
}

# Start log generator in the background
generate_logs &
LOG_GENERATOR_PID=$!

# Wait for cargo-watch to finish (it won't unless interrupted)
wait $CARGO_WATCH_PID
