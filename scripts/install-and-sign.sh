#!/usr/bin/env zsh

# Get the directory where this script is located
SCRIPT_DIR="${0:A:h}"

# Build and install the binary
echo "Building and installing arcli-backend..."
cargo install --path .

# Sign the binary
echo "Signing the binary..."
codesign -s - --force --identifier com.agilityrobotics.arcli.backend --entitlements "$SCRIPT_DIR/entitlements.plist" ~/.cargo/bin/backend

echo "✅ Installation and signing complete!"

