
#!/bin/bash
# deploy.sh — cross-compile and deploy bearings-rs to Hostinger VPS
# Usage: ./deploy.sh
# Prerequisites: cargo, cross (cargo install cross), SSH key for VPS

set -euo pipefail

VPS_HOST="${VPS_HOST:?Set VPS_HOST to your Hostinger VPS IP}"
VPS_USER="${VPS_USER:-bearings}"

# Detect VPS architecture — Hostinger cloud VPS is almost always x86_64
# Change to aarch64-unknown-linux-gnu for ARM VPS
TARGET="${CROSS_TARGET:-x86_64-unknown-linux-gnu}"

echo "▸ Target:  $TARGET"
echo "▸ Host:    $VPS_USER@$VPS_HOST"
echo ""

# Use 'cross' for cross-compilation (handles musl libc properly)
# Install: cargo install cross
# Alternatively use: cargo build --release if building on the VPS directly

echo "▸ Building bearings-backend..."
cross build --release -p bearings-backend --target "$TARGET"

echo "▸ Building bearings-agent..."
cross build --release -p bearings-agent --target "$TARGET"

echo "▸ Stopping services on VPS..."
ssh "$VPS_USER@$VPS_HOST" "sudo systemctl stop bearings-backend bearings-agent 2>/dev/null || true"

echo "▸ Copying binaries..."
scp "target/$TARGET/release/bearings-backend" "$VPS_USER@$VPS_HOST:/opt/bearings-backend/bearings-backend"
scp "target/$TARGET/release/bearings-agent"   "$VPS_USER@$VPS_HOST:/opt/bearings-agent/bearings-agent"

echo "▸ Starting services..."
ssh "$VPS_USER@$VPS_HOST" "sudo systemctl start bearings-backend bearings-agent"

echo "▸ Status:"
ssh "$VPS_USER@$VPS_HOST" "sudo systemctl status bearings-backend bearings-agent --no-pager -l"

echo ""
echo "✓ Deploy complete."
echo "  Backend: https://$VPS_HOST/health"
echo "  llms.txt: https://$VPS_HOST/llms.txt"
