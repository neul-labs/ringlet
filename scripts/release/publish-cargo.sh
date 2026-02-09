#!/usr/bin/env bash
# Publish to crates.io
# This file is sourced by release.sh

publish_cargo() {
    local version="$1"

    log_info "Publishing to crates.io..."

    # Check for cargo registry token
    if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
        log_error "CARGO_REGISTRY_TOKEN not set"
        return 1
    fi

    # Get crates to publish in order
    local crates_str
    crates_str=$(parse_toml_array "publishers.cargo.crates" "$CONFIG_FILE")
    local wait_between
    wait_between=$(parse_toml "publishers.cargo.wait_between" "$CONFIG_FILE")
    wait_between="${wait_between:-30}"

    local crates=()
    while IFS= read -r crate; do
        [[ -n "$crate" ]] && crates+=("$crate")
    done <<< "$crates_str"

    if [[ ${#crates[@]} -eq 0 ]]; then
        log_warn "No crates configured for publishing"
        return 0
    fi

    log_info "Publishing ${#crates[@]} crates in order: ${crates[*]}"

    local first=true
    for crate in "${crates[@]}"; do
        if ! $first; then
            log_info "Waiting ${wait_between}s for crates.io to index..."
            sleep "$wait_between"
        fi
        first=false

        log_info "Publishing $crate..."

        if is_dry_run; then
            log_debug "[DRY-RUN] cargo publish -p $crate --allow-dirty"
            continue
        fi

        # Find crate directory
        local crate_dir="$SCRIPT_DIR/crates/$crate"
        if [[ ! -d "$crate_dir" ]]; then
            log_warn "Crate directory not found: $crate_dir"
            continue
        fi

        # Publish (allow-dirty because we may have uncommitted version changes)
        if cargo publish -p "$crate" --allow-dirty 2>&1; then
            log_success "Published $crate"
        else
            log_warn "Failed to publish $crate (may already be published)"
        fi
    done

    log_success "Cargo publishing complete"
    return 0
}
