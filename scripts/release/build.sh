#!/usr/bin/env bash
# Build script for cross-platform compilation
# This file is sourced by release.sh

# Ensure we have the common library loaded
if ! type log_info &>/dev/null; then
    source "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"
fi

# Build a single platform
build_platform() {
    local version="$1"
    local platform="$2"
    local target
    target=$(get_target_triple "$platform")

    if [[ -z "$target" ]]; then
        log_error "Unknown platform: $platform"
        return 1
    fi

    local current_platform
    current_platform=$(detect_platform)

    log_info "Building for $platform (target: $target)..."

    # Determine if we need cross-compilation
    local build_cmd="cargo"
    local needs_cross=false

    case "$platform" in
        linux-arm64)
            needs_cross=true
            ;;
        darwin-*)
            if [[ "$current_platform" != darwin-* ]]; then
                needs_cross=true
            fi
            ;;
        win32-*)
            needs_cross=true
            ;;
        linux-x64)
            if [[ "$current_platform" != "linux-x64" ]]; then
                needs_cross=true
            fi
            ;;
    esac

    if $needs_cross; then
        if command -v cross &>/dev/null; then
            build_cmd="cross"
            log_debug "Using cross for $platform"
        else
            log_warn "cross not available, attempting with cargo (may fail)"
        fi
    fi

    # Run the build
    if is_dry_run; then
        log_debug "[DRY-RUN] $build_cmd build --release --target $target"
        return 0
    fi

    if ! $build_cmd build --release --target "$target"; then
        log_error "Build failed for $platform"
        return 1
    fi

    # Package the binaries
    package_binaries "$version" "$platform" "$target"
}

# Package binaries into distributable archive
package_binaries() {
    local version="$1"
    local platform="$2"
    local target="$3"

    local target_dir="$SCRIPT_DIR/target/$target/release"
    local archive_name="${PROJECT_NAME}-${platform}-${version}"
    local staging_dir="$DIST_DIR/staging/$archive_name"

    log_info "Packaging $platform binaries..."

    # Create staging directory
    mkdir -p "$staging_dir"

    # Copy binaries
    for binary in "${BINARIES[@]}"; do
        local bin_name="$binary"
        if [[ "$platform" == win32-* ]]; then
            bin_name="${binary}.exe"
        fi

        local src="$target_dir/$bin_name"
        if [[ -f "$src" ]]; then
            cp "$src" "$staging_dir/"
            log_debug "Copied $bin_name"
        else
            log_warn "Binary not found: $src"
        fi
    done

    # Create archive
    local archive_path
    if [[ "$platform" == win32-* ]]; then
        archive_path="$DIST_DIR/${archive_name}.zip"
        (cd "$DIST_DIR/staging" && zip -r "../${archive_name}.zip" "$archive_name")
    else
        archive_path="$DIST_DIR/${archive_name}.tar.gz"
        (cd "$DIST_DIR/staging" && tar -czvf "../${archive_name}.tar.gz" "$archive_name")
    fi

    # Compute and save checksum
    local checksum
    checksum=$(compute_sha256 "$archive_path")
    save_checksum "$VERSION" "$platform" "$checksum"

    log_info "Created: $archive_path"
    log_debug "Checksum: $checksum"

    # Cleanup staging
    rm -rf "$staging_dir"
}

# Create macOS universal binary
create_universal_binary() {
    local version="$1"

    local x64_archive="$DIST_DIR/${PROJECT_NAME}-darwin-x64-${version}.tar.gz"
    local arm64_archive="$DIST_DIR/${PROJECT_NAME}-darwin-arm64-${version}.tar.gz"
    local universal_name="${PROJECT_NAME}-darwin-universal-${version}"
    local staging_dir="$DIST_DIR/staging/$universal_name"

    # Check if both architectures exist
    if [[ ! -f "$x64_archive" ]] || [[ ! -f "$arm64_archive" ]]; then
        log_warn "Cannot create universal binary: missing darwin-x64 or darwin-arm64 archives"
        return 1
    fi

    # Check if lipo is available (macOS only)
    if ! command -v lipo &>/dev/null; then
        log_warn "lipo not available (requires macOS). Skipping universal binary."
        return 1
    fi

    log_info "Creating macOS universal binary..."

    if is_dry_run; then
        log_debug "[DRY-RUN] Would create universal binary"
        return 0
    fi

    # Extract both archives
    local x64_dir="$DIST_DIR/staging/darwin-x64"
    local arm64_dir="$DIST_DIR/staging/darwin-arm64"
    mkdir -p "$x64_dir" "$arm64_dir" "$staging_dir"

    tar -xzf "$x64_archive" -C "$x64_dir" --strip-components=1
    tar -xzf "$arm64_archive" -C "$arm64_dir" --strip-components=1

    # Create universal binaries with lipo
    for binary in "${BINARIES[@]}"; do
        if [[ -f "$x64_dir/$binary" ]] && [[ -f "$arm64_dir/$binary" ]]; then
            lipo -create -output "$staging_dir/$binary" "$x64_dir/$binary" "$arm64_dir/$binary"
            log_debug "Created universal: $binary"
        fi
    done

    # Package
    local archive_path="$DIST_DIR/${universal_name}.tar.gz"
    (cd "$DIST_DIR/staging" && tar -czvf "../${universal_name}.tar.gz" "$universal_name")

    # Compute checksum
    local checksum
    checksum=$(compute_sha256 "$archive_path")
    save_checksum "$VERSION" "darwin-universal" "$checksum"

    log_info "Created: $archive_path"

    # Cleanup
    rm -rf "$x64_dir" "$arm64_dir" "$staging_dir"
}

# Generate checksums file
generate_checksums() {
    local version="$1"
    local checksums_file="$DIST_DIR/checksums.txt"

    log_info "Generating checksums file..."

    if is_dry_run; then
        log_debug "[DRY-RUN] Would generate $checksums_file"
        return 0
    fi

    # Clear existing
    : > "$checksums_file"

    # Add checksums for all archives
    for file in "$DIST_DIR"/*.tar.gz "$DIST_DIR"/*.zip; do
        if [[ -f "$file" ]]; then
            local filename
            filename=$(basename "$file")
            local checksum
            checksum=$(compute_sha256 "$file")
            echo "$checksum  $filename" >> "$checksums_file"
        fi
    done

    log_info "Created: $checksums_file"
}
