
# bearings-rs

Rust backend for the Bearings global bear community platform.

## Architecture

Four crates in one Cargo workspace:

| Crate | Purpose | Status |
|-------|---------|--------|
| `bearings-shared` | Database model types. Shared across all crates. | Draft |
| `bearings-backend` | Axum HTTP server. REST API + SSR + llms.txt. | Draft |
| `bearings-agent` | Treasury monitor. Blockfrost + Supabase writer. | Draft |
| `bearings-frontend` | Leptos WASM frontend. Replaces Lovable. | Planned |

## Getting Started

```bash
# Clone
git clone https://github.com/bearings-admin/bearings-rs
cd bearings-rs

# Configure
cp .env.example .env
# Fill in: SUPABASE_URL, SUPABASE_ANON_KEY, SUPABASE_SERVICE_ROLE_KEY,
#          BLOCKFROST_PROJECT_ID, TREASURY_WALLET_ADDRESS, OPERATIONAL_WALLET_ADDRESS

# Run the backend
cargo run -p bearings-backend

# Run the treasury agent
cargo run -p bearings-agent
```

## Where the Code Lives

The code in this repository is mirrored to the Bearings Supabase database in the `code` table.
Any Claude session can query it:

```sql
SELECT file_path, content
FROM code
WHERE crate = 'bearings-backend'
AND active = true
ORDER BY file_path;
```

This means agents can read, understand, and propose changes to the codebase even without
GitHub access. The database is the source of truth.

## For Gaspar

The interesting files for your first review:
- `bearings-shared/src/models.rs` — does the schema mapping look correct?
- `bearings-agent/src/blockfrost.rs` — is the Cardano integration approach right?
- `bearings-agent/src/treasury.rs` — any issues with the monitoring loop?

No maintenance obligation. Review at your own pace.

## Deployment

The backend and agent run as systemd services on a Hostinger VPS.
See `deploy.sh` and `bearings-agent/deploy/bearings-agent.service`.

## Design Philosophy

Follows Gaspar's approach from srv750649.hstgr.cloud:
- Minimal, explicit routing — no magic
- Axum for the web layer, tokio for async
- Direct Supabase REST calls — no ORM
- The shared types crate as the single source of schema truth
- Secrets only in .env, never in code
