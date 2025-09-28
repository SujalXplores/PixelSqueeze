#!/bin/bash
# PixelSqueeze Installation Script for macOS/Linux
# Run with: curl -sSL https://raw.githubusercontent.com/SujalXplores/PixelSqueeze/main/install.sh | bash

echo "🎨 Installing PixelSqueeze..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust found, installing PixelSqueeze..."

# Install from crates.io
if cargo install pixelsqueeze; then
    echo "🎉 PixelSqueeze installed successfully!"
    echo ""
    echo "Try it out:"
    echo "  pixelsqueeze --help"
    echo "  pixelsqueeze image.jpg"
    echo "  pixelsqueeze photos/ --recursive"
else
    echo "❌ Installation failed. Try installing manually:"
    echo "   cargo install pixelsqueeze"
    exit 1
fi