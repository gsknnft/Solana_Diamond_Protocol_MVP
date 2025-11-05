#!/usr/bin/env bash
#
# Native Diamond Protocol - Build Script
# Builds router and facet programs for Solana
#

set -e

echo "🔨 Building Native Diamond Protocol..."
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust."
    exit 1
fi

# Build workspace
echo "📦 Building workspace..."
cargo build --release

echo "✅ Native Diamond Protocol built successfully!"
echo ""
echo "Artifacts:"
echo "  - Router: target/release/libdiamond_router_native.so"
echo "  - Facet:  target/release/libexample_facet_native.so"
echo ""

# If solana-cli is available, also build for BPF
if command -v cargo-build-sbf &> /dev/null || command -v cargo-build-bpf &> /dev/null; then
    echo "🚀 Building for Solana BPF..."
    cargo build-sbf || cargo build-bpf
    echo ""
    echo "✅ BPF programs built!"
    echo "  - target/deploy/diamond_router_native.so"
    echo "  - target/deploy/example_facet_native.so"
else
    echo "ℹ️  Solana CLI not found. Skipping BPF build."
    echo "   Install with: sh -c \"\$(curl -sSfL https://release.solana.com/stable/install)\""
fi

echo ""
echo "💎 Done! Native diamond protocol is ready."
