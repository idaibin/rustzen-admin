#!/usr/bin/env bash
# OE-ISOLATION-GUARD-01: fail-closed isolation gate for local
# acceptance launches.
#
# Incident basis (2026-10-12): a local-acceptance launch omitted the
# isolation-root environment variable from the actual command while a
# side-car meta file claimed it, so the server fell back to the
# default real runtime root and opened the real local database. This
# gate makes that class of divergence impossible: the isolation root
# comes ONLY from the explicit --isolation-root argument, every
# path-affecting contract variable already present in the caller
# environment is rejected, and the target program runs under a
# rebuilt allowlisted environment built from the SAME validated data
# that produces the on-disk receipt.
#
# Contract variables that can move data paths (crates/config):
# RUSTZEN_RUNTIME_ROOT (default "." — the real in-repo runtime root)
# and the four SQLite override variables. Their presence in the
# caller environment is a stale-env/ambiguity hazard and is rejected;
# relative SQLite overrides would resolve inside the root, but
# absolute ones escape it, so all of them are rejected here.
#
# Input rejections happen before any directory or file is created.
# After admission, setup or child failures retain the newly owned root
# for inspection; the launcher never removes it or reuses an existing
# root. The caller owns cleanup after confirming child termination and
# the exact root identity. The receipt records launch inputs only.
# Validation: absolute root; no `.rustzen-admin` component
# anywhere in the chain; no literal `..` path component (v2: literal
# prefix matching alone allowed `allowed-base/../..` escapes); root
# inside one of the two allowed bases (the repository's target/rz
# subtree, or the system temp dir — each matched in lexical and
# physical form); no symlinked path components below the matched base
# (the base itself is trusted, which also tolerates macOS
# /var → /private/var); the root must not already exist — this
# launcher creates it fresh together with its data subpaths and
# re-verifies physical equality afterwards; and the NORMALIZED real
# parent of the root (resolved before any creation) must still be
# inside an allowed base (v2: the authoritative anti-escape check).
#
# v2 dotenv containment: the child additionally receives
# RUSTZEN_SKIP_DOTENV=1 so the real Admin entry
# (crates/config load_dotenv_if_present) does not re-inject data-path
# variables from a .env found at or above the working directory.
#
# Usage:
#   local-acceptance-launch.sh --isolation-root <abs-path> \
#       [--port <1024-65535>] -- <command> [args...]
set -euo pipefail

readonly CONTRACT_PATH_VARS=(
    RUSTZEN_RUNTIME_ROOT
    RUSTZEN_ADMIN_SQLITE_PATH
    RUSTZEN_MONITOR_SQLITE_PATH
    RUSTZEN_INSIGHTS_SQLITE_PATH
    RUSTZEN_REPORTS_SQLITE_PATH
)

die() {
    printf 'local-acceptance-launch: %s\n' "$1" >&2
    exit 2
}

root=''
port=''
while [ "$#" -gt 0 ]; do
    case "$1" in
        --isolation-root)
            [ "$#" -ge 2 ] || die '--isolation-root requires a value'
            root="$2"
            shift 2
            ;;
        --port)
            [ "$#" -ge 2 ] || die '--port requires a value'
            port="$2"
            shift 2
            ;;
        --)
            shift
            break
            ;;
        *)
            die "unknown argument: $1"
            ;;
    esac
done
[ -n "$root" ] || die 'missing --isolation-root: an explicit isolation root is mandatory (a launch falling back to the default root is the 2026-10-12 incident)'
[ "$#" -ge 1 ] || die 'missing target command after --'

for v in "${CONTRACT_PATH_VARS[@]}"; do
    if [ "${!v+x}" = x ]; then
        die "environment variable $v is already set; unset it and pass --isolation-root (single source; a side-car value diverging from the real launch is the incident being guarded against)"
    fi
done

case "$root" in
    /*) ;;
    *) die "isolation root must be an absolute path: $root" ;;
esac
case "$root" in
    .rustzen-admin|*/.rustzen-admin|*/.rustzen-admin/*)
        die "default runtime-root name rejected anywhere in the chain: $root"
        ;;
esac
case "/$root/" in
    */../*)
        die "dot-dot path component rejected (literal-prefix escapes are not normalized away): $root"
        ;;
esac

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
repo_dir="$(CDPATH= cd -- "$script_dir/.." && pwd -P)"
bound=''
bound_phys=''
# A missing unrelated base must not block an existing permitted base.
# Never create a base merely to validate a launch.
for base in "$repo_dir/target/rz" "${TMPDIR:-/tmp}"; do
    [ -d "$base" ] || continue
    physical="$(CDPATH= cd -- "$base" && pwd -P)"
    for candidate in "$base" "$physical"; do
        case "$root" in
            "$candidate"/*) bound="$candidate"; bound_phys="$physical"; break ;;
        esac
    done
    [ -z "$bound" ] || break
done
[ -n "$bound" ] || die "isolation root is outside the allowed bases (repo target/rz or the system temp dir): $root"

p="${root%/*}"
while [ "$p" != "$bound" ] && [ "$p" != "/" ]; do
    [ -L "$p" ] && die "symlinked path component below the allowed base: $p"
    p="${p%/*}"
    [ -z "$p" ] && p='/'
done
if [ -e "$root" ] || [ -L "$root" ]; then
    die "isolation root already exists; this launcher must create it fresh: $root"
fi
parent="${root%/*}"
[ -n "$parent" ] || parent='/'
[ -d "$parent" ] || die "isolation-root parent does not exist: $parent"
parent_real="$(CDPATH= cd -- "$parent" && pwd -P)"
case "$parent_real" in
    "$bound_phys"|"$bound_phys"/*) ;;
    *) die "normalized parent escapes the allowed base (checked before any creation): $parent_real" ;;
esac

if [ -n "$port" ]; then
    case "$port" in
        ''|*[!0-9]*) die "invalid --port (expected integer 1024-65535): $port" ;;
    esac
    if [ "$port" -lt 1024 ] || [ "$port" -gt 65535 ]; then
        die "invalid --port (expected integer 1024-65535): $port"
    fi
fi

mkdir "$root"
mkdir "$root/data" "$root/data/db" "$root/logs" "$root/logs/admin"
# Physical-equality check scoped below the trusted base: resolve the
# parent physically (components below the base were symlink-checked;
# the base itself is trusted, e.g. macOS /var -> /private/var) and
# require the freshly created root to match parent+basename exactly.
expected_real="$(CDPATH= cd -- "${root%/*}" && pwd -P)/$(basename "$root")"
real_root="$(CDPATH= cd -- "$root" && pwd -P)"
[ "$real_root" = "$expected_real" ] || die "created root resolves elsewhere: $real_root (expected $expected_real)"

receipt="$root/LAUNCH-RECEIPT.txt"
i=0
{
    printf 'generated_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'gate=%s\n' "$0"
    printf 'gate_sha256=%s\n' "$(shasum -a 256 "$0" | awk '{print $1}')"
    printf 'isolation_root=%s\n' "$root"
    printf 'isolation_root_real=%s\n' "$real_root"
    printf 'port=%s\n' "${port:-<unset>}"
    printf 'child_env=RUSTZEN_RUNTIME_ROOT=<%s>\n' "$root"
    printf 'child_env=RUSTZEN_SKIP_DOTENV=<1>\n'
    if [ -n "$port" ]; then
        printf 'child_env=RUSTZEN_ADMIN_PORT=<%s>\n' "$port"
    fi
    printf 'child_argv_count=%d\n' "$#"
    for a in "$@"; do
        i=$((i + 1))
        printf 'child_argv[%d]=<%s>\n' "$i" "$a"
    done
    printf 'created_paths=%s/data %s/data/db %s/logs %s/logs/admin\n' \
        "$root" "$root" "$root" "$root"
    printf 'note=child runs under env -i with PATH HOME TMPDIR LANG TZ plus the child_env entries above and nothing else\n'
} > "$receipt"

if [ -n "$port" ]; then
    exec env -i \
        PATH="$PATH" HOME="$HOME" TMPDIR="${TMPDIR:-/tmp}" \
        LANG="${LANG:-en_US.UTF-8}" TZ="${TZ:-UTC}" \
        RUSTZEN_RUNTIME_ROOT="$root" RUSTZEN_SKIP_DOTENV=1 RUSTZEN_ADMIN_PORT="$port" \
        "$@"
fi
exec env -i \
    PATH="$PATH" HOME="$HOME" TMPDIR="${TMPDIR:-/tmp}" \
    LANG="${LANG:-en_US.UTF-8}" TZ="${TZ:-UTC}" \
    RUSTZEN_RUNTIME_ROOT="$root" RUSTZEN_SKIP_DOTENV=1 \
    "$@"
