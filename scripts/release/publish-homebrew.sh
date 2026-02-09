#!/usr/bin/env bash
# Publish to Homebrew tap
# This file is sourced by release.sh

publish_homebrew() {
    local version="$1"

    log_info "Publishing to Homebrew..."

    # Check for GitHub token (for tap repo access)
    if [[ -z "${HOMEBREW_TAP_TOKEN:-}" ]] && [[ -z "${GITHUB_TOKEN:-}" ]]; then
        log_error "HOMEBREW_TAP_TOKEN or GITHUB_TOKEN not set"
        return 1
    fi
    local token="${HOMEBREW_TAP_TOKEN:-$GITHUB_TOKEN}"

    local tap_repo
    tap_repo=$(parse_toml "publishers.homebrew.tap_repo" "$CONFIG_FILE")
    local formula_name
    formula_name=$(parse_toml "publishers.homebrew.formula_name" "$CONFIG_FILE")

    log_info "Updating $tap_repo with $formula_name@$version"

    local template_file="$SCRIPT_DIR/packaging/homebrew/${formula_name}.rb.template"
    if [[ ! -f "$template_file" ]]; then
        log_error "Homebrew formula template not found: $template_file"
        return 1
    fi

    if is_dry_run; then
        log_debug "[DRY-RUN] Would update Homebrew tap $tap_repo"
        return 0
    fi

    # Calculate checksums for each platform
    local darwin_arm64_sha256
    darwin_arm64_sha256=$(get_checksum "$version" "darwin-arm64")
    local darwin_x64_sha256
    darwin_x64_sha256=$(get_checksum "$version" "darwin-x64")
    local linux_arm64_sha256
    linux_arm64_sha256=$(get_checksum "$version" "linux-arm64")
    local linux_x64_sha256
    linux_x64_sha256=$(get_checksum "$version" "linux-x64")

    # If checksums not in state, compute from files
    if [[ -z "$darwin_arm64_sha256" ]]; then
        darwin_arm64_sha256=$(compute_sha256 "$DIST_DIR/${PROJECT_NAME}-darwin-arm64-${version}.tar.gz" 2>/dev/null || echo "")
    fi
    if [[ -z "$darwin_x64_sha256" ]]; then
        darwin_x64_sha256=$(compute_sha256 "$DIST_DIR/${PROJECT_NAME}-darwin-x64-${version}.tar.gz" 2>/dev/null || echo "")
    fi
    if [[ -z "$linux_arm64_sha256" ]]; then
        linux_arm64_sha256=$(compute_sha256 "$DIST_DIR/${PROJECT_NAME}-linux-arm64-${version}.tar.gz" 2>/dev/null || echo "")
    fi
    if [[ -z "$linux_x64_sha256" ]]; then
        linux_x64_sha256=$(compute_sha256 "$DIST_DIR/${PROJECT_NAME}-linux-x64-${version}.tar.gz" 2>/dev/null || echo "")
    fi

    # Generate formula from template
    local formula_content
    formula_content=$(cat "$template_file" | \
        sed "s/{{VERSION}}/$version/g" | \
        sed "s/{{SHA256_DARWIN_ARM64}}/$darwin_arm64_sha256/g" | \
        sed "s/{{SHA256_DARWIN_X64}}/$darwin_x64_sha256/g" | \
        sed "s/{{SHA256_LINUX_ARM64}}/$linux_arm64_sha256/g" | \
        sed "s/{{SHA256_LINUX_X64}}/$linux_x64_sha256/g")

    # Clone tap repo, update formula, push
    local tap_dir
    tap_dir=$(mktemp -d)
    log_info "Cloning tap repo..."

    git clone "https://x-access-token:${token}@github.com/${tap_repo}.git" "$tap_dir" 2>&1

    # Write formula
    echo "$formula_content" > "$tap_dir/${formula_name}.rb"

    # Commit and push
    cd "$tap_dir"
    git config user.name "Release Bot"
    git config user.email "release@neullabs.com"
    git add "${formula_name}.rb"
    git commit -m "Update $formula_name to $version" || {
        log_info "No changes to commit"
        rm -rf "$tap_dir"
        cd "$SCRIPT_DIR"
        return 0
    }
    git push origin main 2>&1

    # Cleanup
    rm -rf "$tap_dir"
    cd "$SCRIPT_DIR"

    log_success "Homebrew tap updated"
    return 0
}
