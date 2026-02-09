#!/usr/bin/env bash
# Common utilities for release scripts

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Logging functions
log_info()    { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1"; }
log_debug()   { echo -e "${BLUE}[DEBUG]${NC} $1"; }
log_step()    { echo -e "${CYAN}[STEP]${NC} ${BOLD}$1${NC}"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} ${BOLD}$1${NC}"; }

die() {
    log_error "$1"
    exit 1
}

# Check if a command exists
require_cmd() {
    local cmd="$1"
    if ! command -v "$cmd" &> /dev/null; then
        die "Required command not found: $cmd"
    fi
}

# Check if running in dry-run mode
is_dry_run() {
    [[ "${DRY_RUN:-false}" == "true" ]]
}

# Run a command, or just print it in dry-run mode
run_cmd() {
    if is_dry_run; then
        log_debug "[DRY-RUN] $*"
    else
        "$@"
    fi
}

# Parse a simple TOML file (basic implementation for our needs)
# Usage: parse_toml "key.subkey" "file.toml"
# Supports: "project.name", "publishers.msi.upgrade_guid"
parse_toml() {
    local key="$1"
    local file="$2"

    # Count dots to determine nesting level
    local dot_count="${key//[^.]/}"
    dot_count="${#dot_count}"

    if [[ "$dot_count" -eq 0 ]]; then
        # Top-level field (no dots)
        awk -v field="$key" '
            /^\[/ { exit }
            $0 ~ "^" field " *= *" {
                gsub(/^[^=]*= *"?/, "")
                gsub(/"? *$/, "")
                print
                exit
            }
        ' "$file"
    elif [[ "$dot_count" -eq 1 ]]; then
        # Single section: "section.field"
        local section="${key%%.*}"
        local field="${key#*.}"
        awk -v section="$section" -v field="$field" '
            /^\[/ { in_section = ($0 ~ "\\[" section "\\]$") }
            in_section && $0 ~ "^" field " *= *" {
                gsub(/^[^=]*= *"?/, "")
                gsub(/"? *$/, "")
                print
                exit
            }
        ' "$file"
    else
        # Nested section: "section.subsection.field" -> look for [section.subsection]
        local field="${key##*.}"
        local section="${key%.*}"
        awk -v section="$section" -v field="$field" '
            /^\[/ { in_section = ($0 ~ "\\[" section "\\]$") }
            in_section && $0 ~ "^" field " *= *" {
                gsub(/^[^=]*= *"?/, "")
                gsub(/"? *$/, "")
                print
                exit
            }
        ' "$file"
    fi
}

# Parse TOML array (returns newline-separated values)
parse_toml_array() {
    local key="$1"
    local file="$2"

    local section=""
    local field="$key"

    if [[ "$key" == *.* ]]; then
        section="${key%%.*}"
        field="${key#*.}"
    fi

    if [[ -n "$section" ]]; then
        awk -v section="$section" -v field="$field" '
            /^\[/ { in_section = ($0 ~ "\\[" section "\\]") }
            in_section && $0 ~ "^" field " *= *\\[" {
                gsub(/^[^=]*= *\[/, "")
                gsub(/\] *$/, "")
                gsub(/"/, "")
                gsub(/, */, "\n")
                print
                exit
            }
        ' "$file"
    else
        awk -v field="$field" '
            /^\[/ { exit }
            $0 ~ "^" field " *= *\\[" {
                gsub(/^[^=]*= *\[/, "")
                gsub(/\] *$/, "")
                gsub(/"/, "")
                gsub(/, */, "\n")
                print
                exit
            }
        ' "$file"
    fi
}

# Parse TOML boolean
parse_toml_bool() {
    local key="$1"
    local file="$2"
    local value
    value=$(parse_toml "$key" "$file")
    [[ "$value" == "true" ]]
}

# Validate semantic version
validate_version() {
    local version="$1"
    if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$ ]]; then
        die "Invalid version format: $version (expected: X.Y.Z or X.Y.Z-suffix)"
    fi
}

# Get the project root directory
get_project_root() {
    local dir="${BASH_SOURCE[0]}"
    dir="$(cd "$(dirname "$dir")/../../.." && pwd)"
    echo "$dir"
}

# Compute SHA256 checksum
compute_sha256() {
    local file="$1"
    if command -v sha256sum &> /dev/null; then
        sha256sum "$file" | cut -d' ' -f1
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "$file" | cut -d' ' -f1
    else
        die "No SHA256 tool found (sha256sum or shasum)"
    fi
}

# Check if we should run a specific publisher
should_run_publisher() {
    local publisher="$1"
    local only_publish="${ONLY_PUBLISH:-}"

    if [[ -z "$only_publish" ]]; then
        return 0  # Run all publishers
    fi

    # Check if publisher is in the comma-separated list
    [[ ",$only_publish," == *",$publisher,"* ]]
}

# Target triple mappings
declare -A TARGET_TRIPLES=(
    ["linux-x64"]="x86_64-unknown-linux-gnu"
    ["linux-arm64"]="aarch64-unknown-linux-gnu"
    ["darwin-x64"]="x86_64-apple-darwin"
    ["darwin-arm64"]="aarch64-apple-darwin"
    ["win32-x64"]="x86_64-pc-windows-gnu"
)

get_target_triple() {
    local platform="$1"
    echo "${TARGET_TRIPLES[$platform]:-}"
}

# Detect current platform
detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)  os="linux" ;;
        Darwin) os="darwin" ;;
        *)      die "Unsupported OS: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="x64" ;;
        aarch64|arm64) arch="arm64" ;;
        *)             die "Unsupported architecture: $arch" ;;
    esac

    echo "${os}-${arch}"
}
