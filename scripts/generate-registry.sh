#!/usr/bin/env bash
set -euo pipefail

# Generate registry.json from manifest files
# Usage: ./scripts/generate-registry.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
MANIFESTS_DIR="$ROOT_DIR/manifests"
OUTPUT_FILE="$MANIFESTS_DIR/registry.json"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
debug() { echo -e "${BLUE}[DEBUG]${NC} $1"; }

# Get git commit SHA if available
get_commit_sha() {
    if command -v git &> /dev/null && git rev-parse --git-dir &> /dev/null; then
        git rev-parse HEAD 2>/dev/null || echo ""
    else
        echo ""
    fi
}

# Compute SHA256 checksum of a file
get_checksum() {
    local file="$1"
    if command -v sha256sum &> /dev/null; then
        sha256sum "$file" | cut -d' ' -f1
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "$file" | cut -d' ' -f1
    else
        echo ""
    fi
}

# Extract version from TOML file (if present)
get_version() {
    local file="$1"
    grep -E '^version\s*=' "$file" 2>/dev/null | head -1 | sed -E 's/.*=\s*"([^"]+)".*/\1/' || echo ""
}

# Extract ID from file (TOML id field or filename without extension)
get_id() {
    local file="$1"
    local id=$(grep -E '^id\s*=' "$file" 2>/dev/null | head -1 | sed -E 's/.*=\s*"([^"]+)".*/\1/')
    if [[ -n "$id" ]]; then
        echo "$id"
    else
        # Fallback to filename without extension
        local filename=$(basename "$file")
        echo "${filename%.*}"
    fi
}

# Generate JSON for artifacts in a directory
# Usage: generate_artifacts <dir> <type> [extension]
generate_artifacts() {
    local dir="$1"
    local type="$2"
    local ext="${3:-toml}"
    local first=true

    echo "    \"$type\": {"

    if [[ -d "$dir" ]]; then
        for file in "$dir"/*."$ext"; do
            [[ -f "$file" ]] || continue

            local id=$(get_id "$file")
            local filename=$(basename "$file")
            local path="$type/$filename"
            local checksum=$(get_checksum "$file")
            local version=$(get_version "$file")

            if [[ "$first" == "true" ]]; then
                first=false
            else
                echo ","
            fi

            echo -n "      \"$id\": {"
            echo -n "\"path\": \"$path\""

            if [[ -n "$checksum" ]]; then
                echo -n ", \"checksum\": \"$checksum\""
            fi

            if [[ -n "$version" ]]; then
                echo -n ", \"version\": \"$version\""
            fi

            echo -n "}"
        done
    fi

    echo ""
    echo -n "    }"
}

main() {
    info "Generating registry.json..."

    cd "$ROOT_DIR"

    local commit=$(get_commit_sha)

    {
        echo "{"
        echo "  \"version\": 1,"
        echo "  \"channel\": \"stable\","

        if [[ -n "$commit" ]]; then
            echo "  \"commit\": \"$commit\","
        fi

        generate_artifacts "$MANIFESTS_DIR/agents" "agents"
        echo ","
        generate_artifacts "$MANIFESTS_DIR/providers" "providers"
        echo ","
        generate_artifacts "$MANIFESTS_DIR/scripts" "scripts" "rhai"
        echo "}"
    } > "$OUTPUT_FILE"

    info "Generated: $OUTPUT_FILE"

    # Show summary
    local agent_count=$(find "$MANIFESTS_DIR/agents" -name "*.toml" 2>/dev/null | wc -l)
    local provider_count=$(find "$MANIFESTS_DIR/providers" -name "*.toml" 2>/dev/null | wc -l)
    local script_count=$(find "$MANIFESTS_DIR/scripts" -name "*.rhai" 2>/dev/null | wc -l)

    info "Indexed $agent_count agents, $provider_count providers, and $script_count scripts"

    if [[ -n "$commit" ]]; then
        debug "Commit: ${commit:0:8}"
    fi
}

main "$@"
