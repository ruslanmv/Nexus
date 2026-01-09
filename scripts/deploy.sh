#!/usr/bin/env bash
#
# Nexus Deployment Script
#
# This script deploys Nexus as a systemd service on production systems.
# Requires: sudo privileges
#
# Usage: ./scripts/deploy.sh

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔═══════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║       Nexus Production Deployment Script         ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════╝${NC}"
echo ""

# Check for sudo
if [[ $EUID -ne 0 ]]; then
    echo -e "${RED}Error: This script must be run with sudo${NC}"
    exit 1
fi

# Check if binary exists
if [ ! -f "target/release/nexus-demo" ]; then
    echo -e "${RED}Error: Binary not found. Run 'make build' first.${NC}"
    exit 1
fi

echo -e "${BLUE}[1/7]${NC} Creating nexus user..."
if id "nexus" &>/dev/null; then
    echo -e "${YELLOW}User 'nexus' already exists${NC}"
else
    useradd --system --no-create-home --shell /usr/sbin/nologin nexus
    echo -e "${GREEN}✓ User 'nexus' created${NC}"
fi

echo ""
echo -e "${BLUE}[2/7]${NC} Creating directories..."
mkdir -p /etc/nexus
mkdir -p /var/lib/nexus/logs
mkdir -p /var/lib/nexus/metrics
mkdir -p /usr/local/bin

chown -R nexus:nexus /var/lib/nexus
echo -e "${GREEN}✓ Directories created${NC}"

echo ""
echo -e "${BLUE}[3/7]${NC} Installing binary..."
cp target/release/nexus-demo /usr/local/bin/nexus-runtime
chmod +x /usr/local/bin/nexus-runtime
echo -e "${GREEN}✓ Binary installed${NC}"

echo ""
echo -e "${BLUE}[4/7]${NC} Installing configuration..."
if [ -f /etc/nexus/config.toml ]; then
    echo -e "${YELLOW}Configuration exists, backing up...${NC}"
    cp /etc/nexus/config.toml /etc/nexus/config.toml.backup.$(date +%Y%m%d_%H%M%S)
fi

cat > /etc/nexus/config.toml << 'EOF'
# Nexus Production Configuration

[kernel]
mailbox_capacity = 2048
send_timeout_ms = 500
handle_timeout_ms = 30000

[logging]
level = "info"
format = "json"

[runtime]
worker_threads = 0
max_blocking_threads = 512
EOF

chown root:nexus /etc/nexus/config.toml
chmod 640 /etc/nexus/config.toml
echo -e "${GREEN}✓ Configuration installed${NC}"

echo ""
echo -e "${BLUE}[5/7]${NC} Installing systemd service..."
cp scripts/nexus.service /etc/systemd/system/nexus.service
chmod 644 /etc/systemd/system/nexus.service
systemctl daemon-reload
echo -e "${GREEN}✓ Systemd service installed${NC}"

echo ""
echo -e "${BLUE}[6/7]${NC} Enabling service..."
systemctl enable nexus.service
echo -e "${GREEN}✓ Service enabled${NC}"

echo ""
echo -e "${BLUE}[7/7]${NC} Starting service..."
systemctl start nexus.service

# Wait for service to start
sleep 2

if systemctl is-active --quiet nexus.service; then
    echo -e "${GREEN}✓ Service started successfully${NC}"
else
    echo -e "${RED}✗ Service failed to start${NC}"
    echo "Check logs with: journalctl -u nexus.service -n 50"
    exit 1
fi

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  Deployment Complete!                             ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Service Management:${NC}"
echo "  Status:  systemctl status nexus"
echo "  Stop:    systemctl stop nexus"
echo "  Start:   systemctl start nexus"
echo "  Restart: systemctl restart nexus"
echo "  Logs:    journalctl -u nexus -f"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo "  File: /etc/nexus/config.toml"
echo "  Edit and restart service to apply changes"
echo ""
echo -e "${BLUE}Data:${NC}"
echo "  Directory: /var/lib/nexus"
echo "  Logs: /var/lib/nexus/logs"
echo "  Metrics: /var/lib/nexus/metrics"
