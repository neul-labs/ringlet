#!/usr/bin/env bash
# Build macOS DMG installer
# This file is sourced by release.sh

publish_dmg() {
    local version="$1"

    log_info "Building macOS DMG installer..."

    # Check if we're on macOS
    if [[ "$(uname -s)" != "Darwin" ]]; then
        log_warn "DMG creation requires macOS. Skipping."
        log_info "Run this step on a macOS machine to create the DMG."
        return 0
    fi

    local volume_name
    volume_name=$(parse_toml "publishers.dmg.volume_name" "$CONFIG_FILE")
    volume_name="${volume_name:-Ringlet}"

    if is_dry_run; then
        log_debug "[DRY-RUN] Would create DMG: ${PROJECT_NAME}-${version}.dmg"
        return 0
    fi

    # Check for create-dmg tool
    if ! command -v create-dmg &>/dev/null; then
        log_info "Installing create-dmg..."
        if command -v brew &>/dev/null; then
            brew install create-dmg
        else
            log_error "create-dmg not found and brew not available"
            log_info "Install with: brew install create-dmg"
            return 1
        fi
    fi

    # Prefer universal binary, fall back to native arch
    local source_archive=""
    local universal="$DIST_DIR/${PROJECT_NAME}-darwin-universal-${version}.tar.gz"
    local native_arch
    native_arch=$(uname -m)

    if [[ -f "$universal" ]]; then
        source_archive="$universal"
        log_info "Using universal binary"
    elif [[ "$native_arch" == "arm64" ]] && [[ -f "$DIST_DIR/${PROJECT_NAME}-darwin-arm64-${version}.tar.gz" ]]; then
        source_archive="$DIST_DIR/${PROJECT_NAME}-darwin-arm64-${version}.tar.gz"
        log_info "Using ARM64 binary"
    elif [[ -f "$DIST_DIR/${PROJECT_NAME}-darwin-x64-${version}.tar.gz" ]]; then
        source_archive="$DIST_DIR/${PROJECT_NAME}-darwin-x64-${version}.tar.gz"
        log_info "Using x64 binary"
    else
        log_error "No macOS binary archive found"
        return 1
    fi

    # Create staging directory
    local staging_dir="$DIST_DIR/dmg-staging"
    mkdir -p "$staging_dir"

    # Extract binaries
    tar -xzf "$source_archive" -C "$staging_dir" --strip-components=1

    # Make binaries executable
    chmod 755 "$staging_dir/"*

    # Create DMG
    local dmg_path="$DIST_DIR/${PROJECT_NAME}-${version}.dmg"

    log_info "Creating DMG..."

    # Check for custom background/icon
    local dmg_config_dir="$SCRIPT_DIR/packaging/dmg"
    local extra_args=()

    if [[ -f "$dmg_config_dir/icon.icns" ]]; then
        extra_args+=(--volicon "$dmg_config_dir/icon.icns")
    fi

    if [[ -f "$dmg_config_dir/background.png" ]]; then
        extra_args+=(--background "$dmg_config_dir/background.png")
    fi

    create-dmg \
        --volname "$volume_name $version" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 100 \
        --icon "ringlet" 150 190 \
        --icon "ringletd" 350 190 \
        --hide-extension "ringlet" \
        --hide-extension "ringletd" \
        "${extra_args[@]}" \
        "$dmg_path" \
        "$staging_dir" 2>&1 || {
        # create-dmg returns non-zero even on success sometimes
        if [[ -f "$dmg_path" ]]; then
            log_info "DMG created (with warnings)"
        else
            log_error "Failed to create DMG"
            rm -rf "$staging_dir"
            return 1
        fi
    }

    # Compute checksum
    local checksum
    checksum=$(compute_sha256 "$dmg_path")
    save_checksum "$VERSION" "darwin-dmg" "$checksum"

    # Cleanup
    rm -rf "$staging_dir"

    log_success "Created: $dmg_path"
    log_info "Checksum: $checksum"

    # Note about signing
    log_info ""
    log_info "To distribute via Gatekeeper, sign and notarize the DMG:"
    log_info "  codesign --sign 'Developer ID' $dmg_path"
    log_info "  xcrun notarytool submit $dmg_path --apple-id ... --team-id ..."

    return 0
}
