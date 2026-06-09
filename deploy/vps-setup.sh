
#!/bin/bash
# vps-setup.sh — one-time setup for a fresh Hostinger Ubuntu 24.04 VPS
# Run as root: bash vps-setup.sh
# After this script: copy binaries and .env files, then enable services.

set -euo pipefail

echo "▸ Updating system..."
apt-get update && apt-get upgrade -y

echo "▸ Installing dependencies..."
apt-get install -y \
    curl \
    git \
    nginx \
    certbot \
    python3-certbot-nginx \
    ufw \
    fail2ban

echo "▸ Creating bearings system user..."
useradd --system --no-create-home --shell /bin/false bearings 2>/dev/null || true

echo "▸ Creating service directories..."
mkdir -p /opt/bearings-backend/{logs}
mkdir -p /opt/bearings-agent/{logs}
chown -R bearings:bearings /opt/bearings-backend /opt/bearings-agent

echo "▸ Configuring firewall..."
ufw allow OpenSSH
ufw allow 'Nginx Full'
ufw --force enable

echo "▸ Configuring nginx reverse proxy..."
cat > /etc/nginx/sites-available/bearings << 'NGINX'
server {
    listen 80;
    server_name _;  # Replace with your domain

    # Proxy all /api/* and other backend routes to Axum
    location / {
        proxy_pass         http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header   Upgrade $http_upgrade;
        proxy_set_header   Connection 'upgrade';
        proxy_set_header   Host $host;
        proxy_set_header   X-Real-IP $remote_addr;
        proxy_set_header   X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_cache_bypass $http_upgrade;
    }
}
NGINX

ln -sf /etc/nginx/sites-available/bearings /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default
nginx -t && systemctl reload nginx

echo "▸ Installing systemd services..."
# Copy service files (assumes you've already scp'd them)
# cp bearings-backend.service /etc/systemd/system/
# cp bearings-agent.service /etc/systemd/system/
# systemctl daemon-reload
# systemctl enable bearings-backend bearings-agent

echo ""
echo "✓ VPS setup complete."
echo ""
echo "Next steps:"
echo "  1. scp your .env files to /opt/bearings-backend/.env and /opt/bearings-agent/.env"
echo "  2. scp the compiled binaries"
echo "  3. cp deploy/*.service /etc/systemd/system/"
echo "  4. systemctl daemon-reload"
echo "  5. systemctl enable --now bearings-backend bearings-agent"
echo "  6. certbot --nginx  (if you have a domain)"
