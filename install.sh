#!/bin/sh
# AgTerm installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/coldwoong-moon/agterm/main/install.sh | sh

set -e

REPO="coldwoong-moon/agterm"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        linux)
            OS="linux"
            ;;
        darwin)
            OS="macos"
            ;;
        mingw*|msys*|cygwin*)
            echo "Error: Please use the Windows installer or download the zip file manually."
            exit 1
            ;;
        *)
            echo "Error: Unsupported operating system: $OS"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            echo "Error: Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    PLATFORM="${OS}-${ARCH}"
}

# Get latest version
get_latest_version() {
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$VERSION" ]; then
        echo "Error: Could not determine latest version"
        exit 1
    fi
}

# Download and install
install() {
    detect_platform
    get_latest_version

    FILENAME="agterm-${PLATFORM}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"

    echo "Installing AgTerm ${VERSION} for ${PLATFORM}..."
    echo "Downloading from: ${URL}"

    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    # Download and extract
    curl -fsSL "$URL" | tar xz -C "$TMP_DIR"

    # Install binary
    if [ -w "$INSTALL_DIR" ]; then
        mv "$TMP_DIR/agterm" "$INSTALL_DIR/"
    else
        echo "Installing to $INSTALL_DIR requires sudo..."
        sudo mv "$TMP_DIR/agterm" "$INSTALL_DIR/"
    fi

    chmod +x "$INSTALL_DIR/agterm"

    echo ""
    echo "AgTerm ${VERSION} installed successfully!"
    echo "Run 'agterm --help' to get started."
}

install
