#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
spec_root="$repo_root/platform-spec"

files='README.md PROJECT.md governance.md architecture.md verification.md domains/role-definition-management.md'
required_readme='## Scope|## Authority|## Files|## Acceptance boundary'
required_project='## Project|## Boundaries|## Evidence vocabulary|## Requirement ledger|## Acceptance'
required_governance='## Authority boundaries|## Owners|## States and evidence|## Change discipline|## Gap ledger'
required_architecture='## Current architecture|## Contract boundaries|## Runtime truth boundary|## Requirement ledger'
required_domain='## Scope|## Fixed basis and owners|## Requirements|## Gaps|## Evidence and changed files|## Tests|## Runtime acceptance'
required_verification='## Commands|## Static acceptance|## Requirement exit matrix|## Runtime acceptance|## Evidence log|## Not verified'

fail() {
    printf '%s\n' "verify-platform-spec: $*" >&2
    exit 1
}

[ -d "$spec_root" ] || fail "missing platform-spec/"

for file in $files; do
    [ -f "$spec_root/$file" ] || fail "missing fixed file platform-spec/$file"
done

extra_markdown=$(find "$spec_root" -type f -name '*.md' \
    ! -path "$spec_root/README.md" \
    ! -path "$spec_root/PROJECT.md" \
    ! -path "$spec_root/governance.md" \
    ! -path "$spec_root/architecture.md" \
    ! -path "$spec_root/verification.md" \
    ! -path "$spec_root/domains/role-definition-management.md" -print)
[ -z "$extra_markdown" ] || fail "unexpected platform-spec Markdown files found: $extra_markdown"

extra=$(find "$spec_root" -type f ! -name '*.md' -print)
[ -z "$extra" ] || fail "non-Markdown platform-spec files found: $extra"

owner_placeholders=$(
    {
        grep -RniE '(owner|owners?)[:=][[:space:]]*(TBD|TODO|UNASSIGNED|UNKNOWN)' "$spec_root" || true
        grep -RniE '\|[^|]*(owner|owners?)[^|]*\|[[:space:]]*(TBD|TODO|UNASSIGNED|UNKNOWN)[[:space:]]*(\||$)' "$spec_root" || true
    }
)
[ -z "$owner_placeholders" ] || fail "owner placeholder found: $owner_placeholders"

validate_owner_columns() {
    spec_file=$1
    awk '
        function trim(value) {
            gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
            gsub(/`/, "", value)
            return value
        }
        function is_placeholder(value, normalized) {
            value = trim(value)
            normalized = tolower(value)
            return value == "" || normalized == "owner" || normalized == "team" ||
                value == "团队" || value == "待定" || normalized ~ /tbd|todo|unassigned|unknown/
        }
        /^\|/ {
            cell_count = split($0, cells, "|")
            header_owner_column = 0
            for (i = 2; i < cell_count; i++) {
                if (tolower(trim(cells[i])) == "owner") {
                    header_owner_column = i
                    break
                }
            }
            if (header_owner_column > 0) {
                owner_column = header_owner_column
                next
            }
            if (owner_column > 0) {
                has_id = 0
                for (i = 2; i < cell_count; i++) {
                    if (cells[i] ~ /(REQ|G)-[A-Z0-9]+-[0-9][0-9][0-9]/) {
                        has_id = 1
                        break
                    }
                }
                if (has_id && is_placeholder(cells[owner_column])) {
                    printf "%s:%d: REQ/GAP definition has empty or placeholder Owner\n", FILENAME, FNR > "/dev/stderr"
                    exit 1
                }
            }
            next
        }
        { owner_column = 0 }
    ' "$spec_file" || fail "REQ/GAP owner validation failed in $spec_file"
}

for file in $files; do
    validate_owner_columns "$spec_root/$file"
done

check_headings() {
    file=$1
    headings=$2
    old_ifs=$IFS
    IFS='|'
    for heading in $headings; do
        grep -Fqx "$heading" "$spec_root/$file" || fail "platform-spec/$file is missing heading: $heading"
    done
    IFS=$old_ifs
}

check_headings README.md "$required_readme"
check_headings PROJECT.md "$required_project"
check_headings governance.md "$required_governance"
check_headings architecture.md "$required_architecture"
check_headings domains/role-definition-management.md "$required_domain"
check_headings verification.md "$required_verification"

check_exact_case_link() {
    source=$1
    target=$2
    source_relative=${source#"$repo_root"/}
    combined="$(dirname -- "$source_relative")/$target"
    normalized=
    old_ifs=$IFS
    IFS=/
    set -f
    for component in $combined; do
        case "$component" in
            ''|.) continue ;;
            ..)
                case "$normalized" in
                    */*) normalized=${normalized%/*} ;;
                    *) normalized= ;;
                esac
                ;;
            *)
                if [ -n "$normalized" ]; then
                    normalized="$normalized/$component"
                else
                    normalized=$component
                fi
                ;;
        esac
    done
    set +f
    IFS=$old_ifs

    current=$repo_root
    IFS=/
    set -f
    for component in $normalized; do
        match=$(find "$current" -mindepth 1 -maxdepth 1 -name "$component" -print -quit)
        [ -n "$match" ] || fail "local link has incorrect path casing in $source: $target"
        current="$current/$component"
    done
    set +f
    IFS=$old_ifs
}

ids=$(grep -RhoE '(REQ|G)-[A-Z0-9]+-[0-9]{3}' "$spec_root" | sort -u)
[ -n "$ids" ] || fail "no REQ/GAP IDs found"
printf '%s\n' "$ids" | grep -q '^REQ-' || fail "no REQ ID found"
printf '%s\n' "$ids" | grep -q '^G-' || fail "no Gap ID found"

definitions=$(grep -RhE '^\|[[:space:]]*`?(REQ|G)-[A-Z0-9]+-[0-9]{3}`?[[:space:]]*\|' "$spec_root" \
    | sed -E 's/^\|[[:space:]]*`?((REQ|G)-[A-Z0-9]+-[0-9]{3})`?[[:space:]]*\|.*/\1/' \
    | sort)
[ -n "$definitions" ] || fail "no REQ/Gap definitions found"
duplicates=$(printf '%s\n' "$definitions" | uniq -d)
[ -z "$duplicates" ] || fail "duplicate REQ/Gap definitions: $duplicates"
for id in $ids; do
    printf '%s\n' "$definitions" | grep -Fqx "$id" || fail "$id is referenced without a definition"
done

for state in Declared Source-resolved Artifact-resolved Automated Runtime-resolved Gap 'Not verified'; do
    grep -Rqw "$state" "$spec_root" || fail "missing status word: $state"
done

# Resolve Markdown links that are local relative paths. External links and
# fragment-only links are intentionally outside this no-dependency check.
find "$spec_root" -type f -name '*.md' -print | while IFS= read -r source; do
    grep -oE '\]\([^)]*\)' "$source" | sed 's/^](//' | sed 's/)$//' | while IFS= read -r target; do
        case "$target" in
            ''|'#'*|'http://'*|'https://'*|'mailto:'*) continue ;;
        esac
        target=${target%%#*}
        [ -n "$target" ] || continue
        candidate=$(CDPATH= cd -- "$(dirname -- "$source")" && printf '%s/%s' "$PWD" "$target") || fail "cannot resolve link $target in $source"
        [ -e "$candidate" ] || fail "broken local link in $source: $target"
        check_exact_case_link "$source" "$target"
    done
done

printf '%s\n' 'verify-platform-spec: PASS (fixed Markdown files, headings, unique IDs, exact-case local links, status words)'
