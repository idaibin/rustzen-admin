#!/usr/bin/env sh
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"

if [ -n "${RUSTZEN_ARTIFACT_DIR:-}" ]; then
    ARTIFACT_DIR="$RUSTZEN_ARTIFACT_DIR"
elif [ -f "$SCRIPT_DIR/config/app.env" ]; then
    ARTIFACT_DIR="$SCRIPT_DIR"
else
    ARTIFACT_DIR="$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)/target/rustzen-admin"
fi

if [ -n "${RUSTZEN_INSTALL_ROOT:-}" ]; then
    INSTALL_ROOT="$RUSTZEN_INSTALL_ROOT"
elif [ -x "$ARTIFACT_DIR/bin/rustzen-admin" ] && [ -d "$ARTIFACT_DIR/web/dist" ]; then
    INSTALL_ROOT="$ARTIFACT_DIR"
else
    INSTALL_ROOT="/opt/rustzen-admin"
fi
ARCH="${RUSTZEN_ARCH:-x86_64}"

copy_if_different() {
    src="$1"
    dest="$2"

    if [ -e "$dest" ] && cmp -s "$src" "$dest"; then
        return 0
    fi

    cp "$src" "$dest"
}

copy_if_missing() {
    src="$1"
    dest="$2"

    if [ -e "$dest" ]; then
        return 0
    fi

    cp "$src" "$dest"
}

has_release_jwt_placeholder() {
    env_file="$1"

    grep -Eq '^RUSTZEN_JWT_SECRET=(replace-me|rustzen-admin-release-.*)$' "$env_file"
}

print_jwt_and_restart_instructions() {
    root="$1"

    echo "Next steps:"
    echo "  1. Set a real JWT secret:"
    echo "     jwt_secret=\$(openssl rand -hex 32)"
    echo "     sed -i \"s#^RUSTZEN_JWT_SECRET=.*#RUSTZEN_JWT_SECRET=\${jwt_secret}#\" $root/config/app.env"
    echo "  2. Restart service:"
    echo "     systemctl restart rustzen-admin"
    echo "     systemctl status rustzen-admin --no-pager"
}

print_full_systemd_instructions() {
    root="$1"

    echo "Next steps:"
    echo "  1. Install and enable systemd service:"
    echo "     cp $root/systemd/rustzen-admin.service /etc/systemd/system/rustzen-admin.service"
    echo "     systemctl daemon-reload"
    echo "     systemctl enable rustzen-admin"
    echo "  2. If JWT is still a placeholder, set a real JWT secret:"
    echo "     jwt_secret=\$(openssl rand -hex 32)"
    echo "     sed -i \"s#^RUSTZEN_JWT_SECRET=.*#RUSTZEN_JWT_SECRET=\${jwt_secret}#\" $root/config/app.env"
    echo "  3. Start or restart service:"
    echo "     systemctl restart rustzen-admin"
    echo "     systemctl status rustzen-admin --no-pager"
}

if [ ! -f "$ARTIFACT_DIR/config/app.env" ]; then
    echo "Missing config file: $ARTIFACT_DIR/config/app.env" >&2
    exit 1
fi

if [ ! -f "$ARTIFACT_DIR/systemd/rustzen-admin.service" ]; then
    echo "Missing systemd file: $ARTIFACT_DIR/systemd/rustzen-admin.service" >&2
    exit 1
fi

mkdir -p \
    "$INSTALL_ROOT/bin" \
    "$INSTALL_ROOT/config" \
    "$INSTALL_ROOT/data/db" \
    "$INSTALL_ROOT/data/uploads" \
    "$INSTALL_ROOT/data/avatars" \
    "$INSTALL_ROOT/logs" \
    "$INSTALL_ROOT/systemd" \
    "$INSTALL_ROOT/web"

if [ -x "$ARTIFACT_DIR/bin/rustzen-admin" ] && [ -d "$ARTIFACT_DIR/web/dist" ]; then
    chmod +x "$ARTIFACT_DIR/bin/rustzen-admin"
else
    SERVER_FILE=""
    for file in "$ARTIFACT_DIR"/rustzen-admin-*; do
        if [ -f "$file" ]; then
            SERVER_FILE="$file"
            break
        fi
    done

    if [ -z "$SERVER_FILE" ]; then
        echo "Missing server binary in $ARTIFACT_DIR" >&2
        exit 1
    fi

    VERSION="$(basename "$SERVER_FILE" | sed 's/^rustzen-admin-//')"
    WEB_ZIP="$ARTIFACT_DIR/dist-$VERSION.zip"

    if [ ! -f "$WEB_ZIP" ]; then
        echo "Missing web zip: $WEB_ZIP" >&2
        exit 1
    fi

    SERVER_NAME="rustzen-admin-$VERSION-$ARCH"
    install -m 0755 "$SERVER_FILE" "$INSTALL_ROOT/bin/$SERVER_NAME"

    if [ -e "$INSTALL_ROOT/bin/rustzen-admin" ] && [ ! -L "$INSTALL_ROOT/bin/rustzen-admin" ]; then
        echo "Refusing to replace non-symlink: $INSTALL_ROOT/bin/rustzen-admin" >&2
        exit 1
    fi

    ln -sfn "$SERVER_NAME" "$INSTALL_ROOT/bin/rustzen-admin"
    copy_if_missing "$ARTIFACT_DIR/config/app.env" "$INSTALL_ROOT/config/app.env"
    copy_if_different "$ARTIFACT_DIR/systemd/rustzen-admin.service" "$INSTALL_ROOT/systemd/rustzen-admin.service"
    unzip -oq "$WEB_ZIP" -d "$INSTALL_ROOT/web"
fi

echo "Prepared rustzen-admin layout at $INSTALL_ROOT"

if command -v systemctl >/dev/null 2>&1 && [ "$(id -u)" -eq 0 ]; then
    cp "$INSTALL_ROOT/systemd/rustzen-admin.service" /etc/systemd/system/rustzen-admin.service
    systemctl daemon-reload
    systemctl enable rustzen-admin
    if has_release_jwt_placeholder "$INSTALL_ROOT/config/app.env"; then
        echo "Skipped service start because $INSTALL_ROOT/config/app.env still has a placeholder RUSTZEN_JWT_SECRET."
        print_jwt_and_restart_instructions "$INSTALL_ROOT"
    else
        systemctl restart rustzen-admin
        systemctl --no-pager status rustzen-admin
    fi
else
    print_full_systemd_instructions "$INSTALL_ROOT"
fi
