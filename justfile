
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

# Build everything in release mode
build:
    cargo build --release --workspace

# Run all tests
test:
    cargo test --workspace

# Check for compile errors without building
check:
    cargo check --workspace

# Format all code
fmt:
    cargo fmt --all

# Run clippy linter — Gaspar will appreciate this
lint:
    cargo clippy --workspace -- -D warnings

# Check for dependency vulnerabilities
audit:
    cargo audit

# Clean build artifacts
clean:
    cargo clean

# Generate and open documentation
docs:
    cargo doc --workspace --no-deps --open

# SSH deploy to VPS (set VPS_HOST env var)
deploy:
    ./deploy.sh

# Check VPS service status
status:
    ssh ${VPS_USER:-bearings}@${VPS_HOST:?VPS_HOST required} \
        "sudo systemctl status bearings-backend bearings-agent --no-pager"

# Tail VPS logs
logs:
    ssh ${VPS_USER:-bearings}@${VPS_HOST:?VPS_HOST required} \
        "sudo journalctl -u bearings-backend -u bearings-agent -f"
