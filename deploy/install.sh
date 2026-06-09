#!/usr/bin/env sh
set -eu

ROOT="${RUSTZEN_INSTALL_ROOT:-$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)}"

cd "$ROOT"
mkdir -p data/uploads data/avatars logs
chmod +x bin/rustzen-admin

if command -v systemctl >/dev/null 2>&1 && [ "$(id -u)" -eq 0 ]; then
    cp systemd/rustzen-admin.service /etc/systemd/system/rustzen-admin.service
    systemctl daemon-reload
    systemctl enable rustzen-admin
    systemctl restart rustzen-admin
else
    echo "Installed rustzen-admin at $ROOT"
    echo "Run bin/rustzen-admin manually or install systemd/rustzen-admin.service as root."
fi
