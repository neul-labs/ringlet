#!/usr/bin/env bash
#
# Bump version across all ringlet project files
#
# Usage: scripts/bump-version.sh <new-version>
#        scripts/bump-version.sh patch|minor|major
#        scripts/bump-version.sh <new-version> --commit
#        scripts/bump-version.sh <new-version> --commit --tag
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Version files
CARGO_TOML="$ROOT_DIR/Cargo.toml"
PACKAGE_JSON="$ROOT_DIR/ringlet-ui/package.json"
TAURI_CONF="$ROOT_DIR/crates/ringlet-app/tauri.conf.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }
debug() { echo -e "${BLUE}[DEBUG]${NC} $1"; }

usage() {
    cat << 'EOF'
Bump version across all ringlet project files.

Usage: scripts/bump-version.sh <new-version> [options]
       scripts/bump-version.sh patch|minor|major [options]

Arguments:
    new-version    Explicit semantic version (e.g., 0.2.0)
    patch          Bump patch version (0.1.0 → 0.1.1)
    minor          Bump minor version (0.1.0 → 0.2.0)
    major          Bump major version (0.1.0 → 1.0.0)

Options:
    --commit       Create a git commit with the version changes
    --tag          Create a git tag (implies --commit)
    --help, -h     Show this help message

Files updated:
    Cargo.toml                          [workspace.package] version + dependency versions
    ringlet-ui/package.json             "version" field
    crates/ringlet-app/tauri.conf.json  "version" field

Examples:
    scripts/bump-version.sh 0.2.0             # Set explicit version
    scripts/bump-version.sh patch             # 0.1.0 → 0.1.1
    scripts/bump-version.sh minor --commit    # 0.1.0 → 0.2.0 + git commit
    scripts/bump-version.sh major --tag       # 0.1.0 → 1.0.0 + git commit + tag
EOF
}

get_current_version() {
    grep -m1 '^version = "' "$CARGO_TOML" | sed 's/.*version = "\([^"]*\)".*/\1/'
}

validate_semver() {
    local version="$1"
    if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        error "Invalid semantic version: $version (expected X.Y.Z)"
    fi
}

compute_bump() {
    local current="$1"
    local bump_type="$2"

    local major minor patch
    IFS='.' read -r major minor patch <<< "$current"

    case "$bump_type" in
        major) major=$((major + 1)); minor=0; patch=0 ;;
        minor) minor=$((minor + 1)); patch=0 ;;
        patch) patch=$((patch + 1)) ;;
        *) error "Unknown bump type: $bump_type" ;;
    esac

    echo "${major}.${minor}.${patch}"
}

update_cargo_toml() {
    local version="$1"
    local file="$CARGO_TOML"

    # Update [workspace.package] version
    sed -i.bak "s/^version = \"[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\"/version = \"$version\"/" "$file"

    # Update internal crate dependency versions
    sed -i.bak "s/\(ringlet-core = { path = \"[^\"]*\", version = \)\"[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\"/\1\"$version\"/" "$file"
    sed -i.bak "s/\(ringlet-scripting = { path = \"[^\"]*\", version = \)\"[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\"/\1\"$version\"/" "$file"

    rm -f "$file.bak"
}

update_package_json() {
    local version="$1"
    local file="$PACKAGE_JSON"

    if [[ ! -f "$file" ]]; then
        warn "File not found: $file (skipping)"
        return
    fi

    sed -i.bak "s/\"version\": \"[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\"/\"version\": \"$version\"/" "$file"
    rm -f "$file.bak"
}

update_tauri_conf() {
    local version="$1"
    local file="$TAURI_CONF"

    if [[ ! -f "$file" ]]; then
        warn "File not found: $file (skipping)"
        return
    fi

    sed -i.bak "s/\"version\": \"[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\"/\"version\": \"$version\"/" "$file"
    rm -f "$file.bak"
}

main() {
    local new_version=""
    local do_commit=false
    local do_tag=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --commit) do_commit=true; shift ;;
            --tag)    do_tag=true; do_commit=true; shift ;;
            --help|-h) usage; exit 0 ;;
            -*) error "Unknown option: $1. Use --help for usage." ;;
            *)
                if [[ -z "$new_version" ]]; then
                    new_version="$1"
                else
                    error "Unexpected argument: $1"
                fi
                shift
                ;;
        esac
    done

    if [[ -z "$new_version" ]]; then
        usage
        exit 1
    fi

    # Read current version
    local current_version
    current_version="$(get_current_version)"
    if [[ -z "$current_version" ]]; then
        error "Could not read current version from $CARGO_TOML"
    fi

    # Resolve bump type to explicit version
    case "$new_version" in
        patch|minor|major)
            new_version="$(compute_bump "$current_version" "$new_version")"
            ;;
    esac

    # Validate
    validate_semver "$new_version"

    if [[ "$current_version" == "$new_version" ]]; then
        warn "Version is already $new_version"
        exit 0
    fi

    echo ""
    info "Bumping version: $current_version → $new_version"
    echo ""

    # Update all files
    info "Updating $CARGO_TOML"
    update_cargo_toml "$new_version"

    info "Updating $PACKAGE_JSON"
    update_package_json "$new_version"

    info "Updating $TAURI_CONF"
    update_tauri_conf "$new_version"

    # Print summary
    echo ""
    info "Version updated in all files:"
    debug "  Cargo.toml:          $(grep -m1 '^version = "' "$CARGO_TOML" | sed 's/.*"\(.*\)".*/\1/')"
    debug "  ringlet-core dep:    $(grep 'ringlet-core.*version' "$CARGO_TOML" | sed 's/.*version = "\([^"]*\)".*/\1/')"
    debug "  ringlet-scripting:   $(grep 'ringlet-scripting.*version' "$CARGO_TOML" | sed 's/.*version = "\([^"]*\)".*/\1/')"
    if [[ -f "$PACKAGE_JSON" ]]; then
        debug "  package.json:        $(grep '"version"' "$PACKAGE_JSON" | sed 's/.*"\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\)".*/\1/')"
    fi
    if [[ -f "$TAURI_CONF" ]]; then
        debug "  tauri.conf.json:     $(grep '"version"' "$TAURI_CONF" | sed 's/.*"\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\)".*/\1/')"
    fi

    # Git operations
    if $do_commit; then
        echo ""
        info "Creating git commit..."
        git -C "$ROOT_DIR" add "$CARGO_TOML" "$PACKAGE_JSON" "$TAURI_CONF"
        git -C "$ROOT_DIR" commit -m "Bump version to $new_version"
        info "Committed: Bump version to $new_version"
    fi

    if $do_tag; then
        local tag="v$new_version"
        info "Creating git tag: $tag"
        git -C "$ROOT_DIR" tag -a "$tag" -m "Release $new_version"
        info "Tagged: $tag"
    fi

    echo ""
    info "Done!"
}

main "$@"
