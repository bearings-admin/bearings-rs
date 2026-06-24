#!/usr/bin/env bash
# deploy.sh — deploy bearings-rs on the VPS.
#
# Run this ON the VPS, in the deploy checkout (/opt/bearings-rs).
# It makes the running server match GitHub `main`:
#   fetch origin/main -> hard reset -> build release -> restart service -> health check
#
# GitHub `main` is the single source of truth; this checkout is deploy-only and
# must never be hand-edited. Develop in the /opt/bearings-dev worktree (or a local
# clone) on a branch, open a PR, let CI merge it, then run this. See CONTRIBUTING.md.
set -euo pipefail

# Ensure cargo is on PATH even when run non-interactively (e.g. ssh host './deploy.sh').
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
command -v cargo >/dev/null 2>&1 || export PATH="$HOME/.cargo/bin:$PATH"

REPO_DIR="${REPO_DIR:-/opt/bearings-rs}"
SERVICE="${SERVICE:-bearings-backend}"
BRANCH="${BRANCH:-main}"
HEALTH_URL="${HEALTH_URL:-http://localhost:3000/health}"

cd "$REPO_DIR"

echo "▸ Fetching origin/$BRANCH ..."
git fetch origin "$BRANCH"

echo "▸ Resetting $REPO_DIR to origin/$BRANCH (deploy checkout is not hand-edited) ..."
git reset --hard "origin/$BRANCH"

echo "▸ Building release (bearings-backend) ..."
cargo build --release -p bearings-backend

echo "▸ Restarting $SERVICE ..."
systemctl restart "$SERVICE"

sleep 1
echo "▸ Health check ($HEALTH_URL):"
curl -fsS "$HEALTH_URL" && echo " ✓"

echo "✓ Deploy complete — now serving $(git rev-parse --short HEAD) ($(git log -1 --pretty=%s))."
