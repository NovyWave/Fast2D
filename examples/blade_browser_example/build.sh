#!/bin/bash
# Build script for blade_browser_example without mzoon
# This is a workaround for testing when mzoon is not available

set -e

echo "🔪 Building Fast2D Blade Browser Example..."

# Set required MoonZoon environment variables
export FRONTEND_BUILD_ID="blade_browser_example_123"
export CACHE_BUSTING="false"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Run this script from the blade_browser_example directory"
    exit 1
fi

# Build backend
echo "📦 Building backend..."
cd backend
cargo build --release
cd ..

# Build frontend WASM
echo "🕸️ Building frontend WASM..."
cd frontend

# Add wasm32 target if not already added
rustup target add wasm32-unknown-unknown

# Build the WASM library
cargo build --release --target wasm32-unknown-unknown

# Generate JS bindings with wasm-bindgen (if available)
if command -v wasm-bindgen &> /dev/null; then
    echo "🔗 Generating JS bindings..."
    wasm-bindgen --out-dir pkg --target web \
        target/wasm32-unknown-unknown/release/frontend.wasm
    
    echo "✅ Build completed!"
    echo "📁 WASM output: frontend/pkg/"
    echo "🏃 Backend binary: backend/target/release/backend"
    echo ""
    echo "To run:"
    echo "1. Start backend: ./backend/target/release/backend"
    echo "2. Open browser to: http://localhost:8085"
else
    echo "⚠️ wasm-bindgen not found. Install with:"
    echo "cargo install wasm-bindgen-cli"
fi

cd ..