#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
tmp_dir=$(mktemp -d "${TMPDIR:-/tmp}/rustzen-contract-verify.XXXXXX")
trap 'rm -rf "$tmp_dir"' EXIT
mkdir -p "$tmp_dir/input" "$tmp_dir/output"

cd "$repo_root"
cp apps/web/src/api/request.ts "$tmp_dir/request.ts"
cargo run --quiet -p rustzen-admin -- openapi > "$tmp_dir/input/admin-contract.json"

if ! cmp -s openapi/baselines/contract-admin-current.json "$tmp_dir/input/admin-contract.json"; then
    echo "contract-verify: OpenAPI baseline drift" >&2
    exit 1
fi
if ! cmp -s openapi/admin-contract.json "$tmp_dir/input/admin-contract.json"; then
    echo "contract-verify: tracked OpenAPI artifact drift" >&2
    exit 1
fi

CONTRACT_OPENAPI_INPUT="$tmp_dir/input/admin-contract.json" \
CONTRACT_CLIENT_OUTPUT="$tmp_dir/output/admin-contract.ts" \
CONTRACT_MUTATOR_PATH="$tmp_dir/request.ts" \
    just contract-client

if ! cmp -s apps/web/src/api/generated/admin-contract.ts "$tmp_dir/output/admin-contract.ts"; then
    echo "contract-verify: tracked generated client drift" >&2
    exit 1
fi
