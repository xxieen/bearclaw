#!/bin/bash
# Check if bear-cli is installed, install if not
if command -v bearclaw &> /dev/null; then
    echo "bearclaw is already installed: $(bearclaw --version)"
    exit 0
fi

echo "bearclaw not found. Installing..."

# Try cargo first
if command -v cargo &> /dev/null; then
    cargo install bearclaw
    exit $?
fi

# Try downloading pre-built binary
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    TARGET="aarch64-apple-darwin"
else
    TARGET="x86_64-apple-darwin"
fi

REPO="xxieen/bearclaw"
URL="https://github.com/${REPO}/releases/latest/download/bearclaw-${TARGET}.tar.gz"

echo "Downloading from ${URL}..."
curl -fsSL "$URL" | tar xz
if [ -f bearclaw ]; then
    sudo mv bearclaw /usr/local/bin/
    echo "bearclaw installed successfully: $(bearclaw --version)"
else
    echo "Failed to install bearclaw"
    exit 1
fi
