#!/bin/bash
set -e

echo "=== Running PacketCommand Self-Test Suite ==="

echo "Step 1: Checking code compilation..."
cargo check

echo "Step 2: Running cargo test suite..."
cargo test

echo "=== All Tests Passed Successfully! ==="
