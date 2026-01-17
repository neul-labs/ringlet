#!/bin/bash
set -e

# Installation script for clown
# Builds and installs clown and clownd binaries

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

echo "Building clown in release mode..."
cargo build --release

echo "Creating install directory: $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"

echo "Installing binaries..."
cp target/release/clown "$INSTALL_DIR/"
cp target/release/clownd "$INSTALL_DIR/"

echo "Setting executable permissions..."
chmod +x "$INSTALL_DIR/clown"
chmod +x "$INSTALL_DIR/clownd"

echo ""
echo "Installation complete!"
echo "Binaries installed to: $INSTALL_DIR"
echo ""

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "Note: $INSTALL_DIR is not in your PATH."
    echo "Add it by adding this line to your shell profile:"
    echo ""
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    echo ""
fi
