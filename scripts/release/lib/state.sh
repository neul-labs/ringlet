#!/usr/bin/env bash
# State management for release resume capability

# State file location
STATE_DIR="${STATE_DIR:-$(dirname "${BASH_SOURCE[0]}")/../state}"

# Get state file path for a version
get_state_file() {
    local version="$1"
    echo "${STATE_DIR}/release-${version}.state"
}

# Initialize state file
init_state() {
    local version="$1"
    local state_file
    state_file=$(get_state_file "$version")

    mkdir -p "$STATE_DIR"

    cat > "$state_file" << EOF
# Release state file - DO NOT EDIT MANUALLY
VERSION=${version}
STARTED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Build status
BUILD_linux_x64=pending
BUILD_linux_arm64=pending
BUILD_darwin_x64=pending
BUILD_darwin_arm64=pending
BUILD_win32_x64=pending
BUILD_darwin_universal=pending

# Publisher status
PUBLISH_cargo=pending
PUBLISH_npm=pending
PUBLISH_pypi=pending
PUBLISH_rubygems=pending
PUBLISH_homebrew=pending
PUBLISH_chocolatey=pending
PUBLISH_debian=pending
PUBLISH_arch=pending
PUBLISH_dmg=pending
PUBLISH_msi=pending
PUBLISH_github=pending

# Checksums (populated during build)
EOF
    log_debug "Initialized state file: $state_file"
}

# Load state file
load_state() {
    local version="$1"
    local state_file
    state_file=$(get_state_file "$version")

    if [[ -f "$state_file" ]]; then
        # shellcheck source=/dev/null
        source "$state_file"
        return 0
    fi
    return 1
}

# Update state
set_state() {
    local version="$1"
    local key="$2"
    local value="$3"
    local state_file
    state_file=$(get_state_file "$version")

    if [[ ! -f "$state_file" ]]; then
        die "State file not found: $state_file"
    fi

    # Use sed to update or append the key
    if grep -q "^${key}=" "$state_file"; then
        sed -i "s/^${key}=.*/${key}=${value}/" "$state_file"
    else
        echo "${key}=${value}" >> "$state_file"
    fi
}

# Get state value
get_state() {
    local version="$1"
    local key="$2"
    local state_file
    state_file=$(get_state_file "$version")

    if [[ -f "$state_file" ]]; then
        grep "^${key}=" "$state_file" | cut -d'=' -f2-
    fi
}

# Mark a step as in_progress
mark_in_progress() {
    local version="$1"
    local key="$2"
    set_state "$version" "$key" "in_progress"
    log_debug "Marked $key as in_progress"
}

# Mark a step as completed
mark_completed() {
    local version="$1"
    local key="$2"
    set_state "$version" "$key" "completed"
    log_debug "Marked $key as completed"
}

# Mark a step as failed
mark_failed() {
    local version="$1"
    local key="$2"
    set_state "$version" "$key" "failed"
    log_debug "Marked $key as failed"
}

# Check if a step is completed
is_completed() {
    local version="$1"
    local key="$2"
    local status
    status=$(get_state "$version" "$key")
    [[ "$status" == "completed" ]]
}

# Check if a step is pending
is_pending() {
    local version="$1"
    local key="$2"
    local status
    status=$(get_state "$version" "$key")
    [[ "$status" == "pending" ]]
}

# Check if state file exists for version
has_state() {
    local version="$1"
    local state_file
    state_file=$(get_state_file "$version")
    [[ -f "$state_file" ]]
}

# Prompt to resume or start fresh
check_resume() {
    local version="$1"

    if has_state "$version"; then
        log_warn "Found existing release state for v${version}"

        # Show current state
        echo ""
        echo "Current progress:"
        local state_file
        state_file=$(get_state_file "$version")
        grep -E "^(BUILD_|PUBLISH_)" "$state_file" | while read -r line; do
            local key="${line%%=*}"
            local value="${line#*=}"
            local status_icon="?"
            case "$value" in
                completed)   status_icon="✓" ;;
                in_progress) status_icon="→" ;;
                pending)     status_icon="○" ;;
                failed)      status_icon="✗" ;;
            esac
            echo "  $status_icon $key"
        done
        echo ""

        if [[ "${FORCE_FRESH:-false}" == "true" ]]; then
            log_info "Starting fresh (FORCE_FRESH=true)"
            rm -f "$(get_state_file "$version")"
            return 1
        fi

        read -p "Resume from last checkpoint? [Y/n] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
            load_state "$version"
            return 0  # Resume
        else
            rm -f "$(get_state_file "$version")"
            return 1  # Start fresh
        fi
    fi
    return 1  # No state file
}

# Save checksum to state
save_checksum() {
    local version="$1"
    local platform="$2"
    local checksum="$3"
    local key="CHECKSUM_${platform//-/_}"
    set_state "$version" "$key" "$checksum"
}

# Get checksum from state
get_checksum() {
    local version="$1"
    local platform="$2"
    local key="CHECKSUM_${platform//-/_}"
    get_state "$version" "$key"
}

# Clean up state file after successful release
cleanup_state() {
    local version="$1"
    local state_file
    state_file=$(get_state_file "$version")

    if [[ -f "$state_file" ]]; then
        rm -f "$state_file"
        log_debug "Cleaned up state file: $state_file"
    fi
}
