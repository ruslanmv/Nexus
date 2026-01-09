#!/usr/bin/env bash
#
# Nexus Installation Script
#
# This script installs Nexus and its dependencies.
# Usage: ./install.sh [OPTIONS]
#
# Options:
#   --prefix PATH    Installation prefix (default: $HOME/.local)
#   --system         Install system-wide (requires sudo)
#   --dev            Install in development mode
#   --help           Show this help message

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
PREFIX="${HOME}/.local"
SYSTEM_INSTALL=false
DEV_MODE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --system)
            SYSTEM_INSTALL=true
            PREFIX="/usr/local"
            shift
            ;;
        --dev)
            DEV_MODE=true
            shift
            ;;
        --help)
            sed -n '2,13p' "$0" | sed 's/^# //'
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

INSTALL_DIR="${PREFIX}/bin"
CONFIG_DIR="${HOME}/.config/nexus"
DATA_DIR="${HOME}/.local/share/nexus"

if [ "$SYSTEM_INSTALL" = true ]; then
    CONFIG_DIR="/etc/nexus"
    DATA_DIR="/var/lib/nexus"
fi

echo -e "${BLUE}╔═══════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         Nexus Installation Script                ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo "  Installation directory: ${INSTALL_DIR}"
echo "  Configuration directory: ${CONFIG_DIR}"
echo "  Data directory: ${DATA_DIR}"
echo "  System install: ${SYSTEM_INSTALL}"
echo "  Development mode: ${DEV_MODE}"
echo ""

# Check for Rust installation
echo -e "${BLUE}[1/5]${NC} Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Rust not found. Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "${HOME}/.cargo/env"
    echo -e "${GREEN}✓ Rust installed successfully${NC}"
else
    RUST_VERSION=$(rustc --version)
    echo -e "${GREEN}✓ Rust already installed: ${RUST_VERSION}${NC}"
fi

# Build Nexus
echo ""
echo -e "${BLUE}[2/5]${NC} Building Nexus..."
if [ "$DEV_MODE" = true ]; then
    echo "Building in debug mode..."
    cargo build
    BUILD_DIR="target/debug"
else
    echo "Building in release mode..."
    cargo build --release
    BUILD_DIR="target/release"
fi
echo -e "${GREEN}✓ Build complete${NC}"

# Run tests
echo ""
echo -e "${BLUE}[3/5]${NC} Running tests..."
cargo test --quiet
echo -e "${GREEN}✓ All tests passed${NC}"

# Create directories
echo ""
echo -e "${BLUE}[4/5]${NC} Creating directories..."
if [ "$SYSTEM_INSTALL" = true ]; then
    sudo mkdir -p "${INSTALL_DIR}"
    sudo mkdir -p "${CONFIG_DIR}"
    sudo mkdir -p "${DATA_DIR}/logs"
    sudo mkdir -p "${DATA_DIR}/metrics"
else
    mkdir -p "${INSTALL_DIR}"
    mkdir -p "${CONFIG_DIR}"
    mkdir -p "${DATA_DIR}/logs"
    mkdir -p "${DATA_DIR}/metrics"
fi
echo -e "${GREEN}✓ Directories created${NC}"

# Install binary
echo ""
echo -e "${BLUE}[5/5]${NC} Installing binary..."
BINARY_NAME="nexus-runtime"
if [ "$SYSTEM_INSTALL" = true ]; then
    sudo cp "${BUILD_DIR}/nexus-demo" "${INSTALL_DIR}/${BINARY_NAME}"
    sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
else
    cp "${BUILD_DIR}/nexus-demo" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
fi
echo -e "${GREEN}✓ Binary installed to ${INSTALL_DIR}/${BINARY_NAME}${NC}"

# Create default configuration
if [ ! -f "${CONFIG_DIR}/config.toml" ]; then
    echo ""
    echo -e "${BLUE}Creating default configuration...${NC}"

    CONFIG_CONTENT="# Nexus Runtime Configuration

[kernel]
mailbox_capacity = 1024
send_timeout_ms = 500
handle_timeout_ms = 30000

[logging]
level = \"info\"
format = \"compact\"

[runtime]
worker_threads = 0  # 0 = auto-detect CPU count
max_blocking_threads = 512
"

    if [ "$SYSTEM_INSTALL" = true ]; then
        echo "${CONFIG_CONTENT}" | sudo tee "${CONFIG_DIR}/config.toml" > /dev/null
    else
        echo "${CONFIG_CONTENT}" > "${CONFIG_DIR}/config.toml"
    fi
    echo -e "${GREEN}✓ Default configuration created at ${CONFIG_DIR}/config.toml${NC}"
fi

# Installation complete
echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  Installation Complete!                           ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"

# Check if install dir is in PATH
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo ""
    echo -e "${YELLOW}⚠  ${INSTALL_DIR} is not in your PATH${NC}"
    echo "   Add it to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo ""
    echo "   export PATH=\"${INSTALL_DIR}:\$PATH\""
    echo ""
    echo "   Then reload your shell:"
    echo "   source ~/.bashrc  # or ~/.zshrc"
fi

echo ""
echo -e "${BLUE}Usage:${NC}"
echo "  ${BINARY_NAME}                    # Run with default config"
echo "  ${BINARY_NAME} --help             # Show all options"
echo "  ${BINARY_NAME} -n 10 -d 30        # Run 10 agents for 30 seconds"
echo "  ${BINARY_NAME} --config custom.toml  # Use custom config"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo "  Edit: ${CONFIG_DIR}/config.toml"
echo ""
echo -e "${BLUE}Logs:${NC}"
echo "  Directory: ${DATA_DIR}/logs"
echo ""
echo -e "${GREEN}Happy agent orchestration! 🚀${NC}"
