
# bearings-rs task runner
# Install: cargo install just
# Usage: just <recipe>

# List available commands
default:
    just --list

# Run the backend server (development)
backend:
    RUST_LOG=debug cargo run -p bearings-backend

# Run the treasury agent (development)
agent:
    RUST_LOG=debug cargo run -p bearings-agent

# Run both in parallel (requires tmux or two terminals)
dev:
    echo "Start backend: just backend"
    echo "Start agent:   just agent"

# Build everything in release mode (parked frontend excluded, matching CI)
build:
    cargo build --release --workspace --exclude bearings-frontend

# Run all tests (needs SUPABASE_* for the integration tests in tests/)
test:
    cargo test --workspace --exclude bearings-frontend

# Run only the unit tests in src/ — no network, mirrors CI
test-unit:
    cargo test --workspace --lib --exclude bearings-frontend

# Check for compile errors without building
check:
    cargo check --workspace --exclude bearings-frontend

# Format all code
fmt:
    cargo fmt --all

# Run clippy linter — Gaspar will appreciate this
lint:
    cargo clippy --workspace --exclude bearings-frontend -- -D warnings

# Check for dependency vulnerabilities
audit:
    cargo audit

# Clean build artifacts
clean:
    cargo clean

# Generate and open documentation
docs:
    cargo doc --workspace --no-deps --open

# Deploy: run deploy.sh ON the VPS (pull main → build → restart). The VPS is
# deploy-only; merge to main via PR first. Override host with VPS_HOST.
deploy:
    ssh root@${VPS_HOST:-srv1744879.hstgr.cloud} 'cd /opt/bearings-rs && ./deploy.sh'

# Check VPS service status
status:
    ssh root@${VPS_HOST:-srv1744879.hstgr.cloud} \
        "systemctl status bearings-backend --no-pager"

# Tail VPS logs
logs:
    ssh root@${VPS_HOST:-srv1744879.hstgr.cloud} \
        "journalctl -u bearings-backend -f"
