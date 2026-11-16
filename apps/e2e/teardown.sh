#!/usr/bin/env bash
# Modified by ACRYL, 2026
# Reverse of bootstrap.sh — stop services, remove systemd units, drop
# the registry container, optionally wipe the database and box home.
#
# Default: stops services + removes unit files + drops the docker
# registry container. Leaves Postgres / Redis installed (cheap to keep,
# and probably used by other things on the host).
#
# With --wipe-data: ALSO drops the acrylmicrovm_dev database AND deletes
# /var/lib/acrylmicrovm (box state + qcow2 disks). Use this when you
# want a clean reset for the next bootstrap.
#
# With --full: --wipe-data PLUS removes /etc/acrylmicrovm-secrets.env (the
# stable secrets file). After this, the next bootstrap mints new
# secrets — any data in the DB encrypted under the old key becomes
# unreadable, which is fine because --wipe-data already dropped the DB.

set -euo pipefail

REPO="${REPO:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
ENV_FILE="${ENV_FILE:-/etc/acrylmicrovm-api.env}"
SECRETS_FILE="${SECRETS_FILE:-/etc/acrylmicrovm-secrets.env}"

mode=basic
for arg in "$@"; do
    case "$arg" in
        --wipe-data) mode=wipe ;;
        --full)      mode=full ;;
        --help|-h)
            sed -n '2,18p' "$0"
            exit 0
            ;;
        *) echo "unknown flag: $arg" >&2; exit 2 ;;
    esac
done

echo "=== stop + disable services ==="
sudo systemctl stop acrylmicrovm-runner 2>/dev/null || true
sudo systemctl stop acrylmicrovm-api 2>/dev/null || true
sudo systemctl disable acrylmicrovm-runner 2>/dev/null || true
sudo systemctl disable acrylmicrovm-api 2>/dev/null || true

echo "=== remove systemd unit files ==="
sudo rm -f /etc/systemd/system/acrylmicrovm-api.service \
           /etc/systemd/system/acrylmicrovm-runner.service
sudo systemctl daemon-reload

echo "=== remove API env file ==="
sudo rm -f "$ENV_FILE"

echo "=== remove deployed runner binary ==="
sudo rm -f /usr/local/bin/acrylmicrovm-runner

echo "=== remove docker registry container ==="
sudo docker rm -f acrylmicrovm-registry 2>/dev/null || true

echo "=== remove apps/apps self-symlink ==="
[[ -L "$REPO/apps/apps" ]] && rm -f "$REPO/apps/apps"

if [[ "$mode" == "wipe" || "$mode" == "full" ]]; then
    echo "=== --wipe-data: drop acrylmicrovm_dev + /var/lib/acrylmicrovm ==="
    sudo -u postgres psql -c "DROP DATABASE IF EXISTS acrylmicrovm_dev" 2>&1 \
        | grep -v "NOTICE:" || true
    sudo rm -rf /var/lib/acrylmicrovm
fi

if [[ "$mode" == "full" ]]; then
    echo "=== --full: drop secrets file ==="
    sudo rm -f "$SECRETS_FILE"
fi

echo ""
echo "=== teardown complete ($mode) ==="
echo ""
case "$mode" in
    basic)
        echo "Postgres role/db, /var/lib/acrylmicrovm, and secrets file are KEPT."
        echo "Re-run apps/e2e/bootstrap.sh to bring things back up."
        echo "Use --wipe-data to also drop DB + box state."
        ;;
    wipe)
        echo "DB and /var/lib/acrylmicrovm are GONE. Secrets file kept; next"
        echo "bootstrap reuses the same encryption key + admin token."
        ;;
    full)
        echo "Everything gone. Next bootstrap mints fresh secrets."
        ;;
esac
