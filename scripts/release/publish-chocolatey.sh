#!/usr/bin/env bash
# Publish to Chocolatey
# This file is sourced by release.sh

publish_chocolatey() {
    local version="$1"

    log_info "Publishing to Chocolatey..."

    # Check for Chocolatey API key
    if [[ -z "${CHOCOLATEY_API_KEY:-}" ]]; then
        log_error "CHOCOLATEY_API_KEY not set"
        return 1
    fi

    local package_id
    package_id=$(parse_toml "publishers.chocolatey.package_id" "$CONFIG_FILE")

    log_info "Publishing $package_id@$version to Chocolatey"

    local choco_dir="$SCRIPT_DIR/packaging/chocolatey"
    if [[ ! -d "$choco_dir" ]]; then
        log_error "Chocolatey packaging directory not found: $choco_dir"
        return 1
    fi

    if is_dry_run; then
        log_debug "[DRY-RUN] Would publish $package_id to Chocolatey"
        return 0
    fi

    # Get Windows binary checksum
    local win_archive="$DIST_DIR/${PROJECT_NAME}-win32-x64-${version}.zip"
    local win_sha256
    if [[ -f "$win_archive" ]]; then
        win_sha256=$(compute_sha256 "$win_archive")
    else
        win_sha256=$(get_checksum "$version" "win32-x64")
    fi

    if [[ -z "$win_sha256" ]]; then
        log_error "Windows binary checksum not available"
        return 1
    fi

    # Update nuspec version
    local nuspec_file="$choco_dir/${package_id}.nuspec"
    if [[ -f "$nuspec_file" ]]; then
        sed -i "s/<version>.*<\/version>/<version>$version<\/version>/" "$nuspec_file"
    fi

    # Update install script with checksum
    local install_script="$choco_dir/tools/chocolateyinstall.ps1"
    if [[ -f "$install_script" ]]; then
        sed -i "s/\$checksum *= *'.*'/\$checksum = '$win_sha256'/" "$install_script"
        sed -i "s/\$version *= *'.*'/\$version = '$version'/" "$install_script"
    fi

    # Check if we can run choco (needs Windows or Docker)
    if command -v choco &>/dev/null; then
        # Native Windows
        cd "$choco_dir"
        choco pack
        choco push "${package_id}.${version}.nupkg" --source https://push.chocolatey.org/ --api-key "$CHOCOLATEY_API_KEY"
        rm -f *.nupkg
        cd "$SCRIPT_DIR"
    elif command -v docker &>/dev/null; then
        # Use Docker
        log_info "Using Docker for Chocolatey packaging..."
        docker run --rm \
            -v "$choco_dir:/work" \
            -w /work \
            -e CHOCOLATEY_API_KEY="$CHOCOLATEY_API_KEY" \
            chocolatey/choco:latest \
            bash -c "choco pack && choco push *.nupkg --source https://push.chocolatey.org/ --api-key \$CHOCOLATEY_API_KEY"
    else
        log_warn "Neither choco nor docker available. Chocolatey package built but not pushed."
        log_info "Package files prepared in: $choco_dir"
        return 0
    fi

    log_success "Chocolatey publishing complete"
    return 0
}
