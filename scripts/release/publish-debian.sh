#!/usr/bin/env bash
# Build and publish Debian packages
# This file is sourced by release.sh

publish_debian() {
    local version="$1"

    log_info "Building Debian packages..."

    local maintainer
    maintainer=$(parse_toml "publishers.debian.maintainer" "$CONFIG_FILE")
    local section
    section=$(parse_toml "publishers.debian.section" "$CONFIG_FILE")
    section="${section:-devel}"

    if is_dry_run; then
        log_debug "[DRY-RUN] Would build Debian packages"
        return 0
    fi

    # Build for each Linux architecture
    local architectures=("amd64:linux-x64" "arm64:linux-arm64")

    for arch_map in "${architectures[@]}"; do
        local deb_arch="${arch_map%%:*}"
        local platform="${arch_map##*:}"

        log_info "Building ${PROJECT_NAME}_${version}_${deb_arch}.deb..."

        # Check if binary archive exists
        local archive="$DIST_DIR/${PROJECT_NAME}-${platform}-${version}.tar.gz"
        if [[ ! -f "$archive" ]]; then
            log_warn "Binary archive not found: $archive (skipping $deb_arch)"
            continue
        fi

        build_deb_package "$version" "$deb_arch" "$platform" "$maintainer" "$section"
    done

    log_success "Debian packages built in $DIST_DIR"

    # Note: Actual publishing to a Debian repo would require additional setup
    # (e.g., aptly, reprepro, or a PPA)
    log_info "Debian packages are ready for distribution"
    return 0
}

build_deb_package() {
    local version="$1"
    local deb_arch="$2"
    local platform="$3"
    local maintainer="$4"
    local section="$5"

    local pkg_name="${PROJECT_NAME}_${version}_${deb_arch}"
    local pkg_dir="$DIST_DIR/deb-staging/$pkg_name"
    local archive="$DIST_DIR/${PROJECT_NAME}-${platform}-${version}.tar.gz"

    # Create package structure
    mkdir -p "$pkg_dir/DEBIAN"
    mkdir -p "$pkg_dir/usr/bin"

    # Extract binaries
    tar -xzf "$archive" -C "$pkg_dir/usr/bin" --strip-components=1

    # Make binaries executable
    chmod 755 "$pkg_dir/usr/bin/"*

    # Create control file
    cat > "$pkg_dir/DEBIAN/control" << EOF
Package: ${PROJECT_NAME}
Version: ${version}
Section: ${section}
Priority: optional
Architecture: ${deb_arch}
Maintainer: ${maintainer}
Description: CLI orchestrator for coding agents
 Ringlet is a cross-platform orchestrator for CLI-based coding
 agents. It provides profile management, usage tracking, and
 intelligent request routing across multiple AI providers.
Homepage: https://github.com/${REPOSITORY}
EOF

    # Create postinst script (optional)
    local template_postinst="$SCRIPT_DIR/packaging/debian/postinst"
    if [[ -f "$template_postinst" ]]; then
        cp "$template_postinst" "$pkg_dir/DEBIAN/postinst"
        chmod 755 "$pkg_dir/DEBIAN/postinst"
    fi

    # Create prerm script (optional)
    local template_prerm="$SCRIPT_DIR/packaging/debian/prerm"
    if [[ -f "$template_prerm" ]]; then
        cp "$template_prerm" "$pkg_dir/DEBIAN/prerm"
        chmod 755 "$pkg_dir/DEBIAN/prerm"
    fi

    # Build the package
    if command -v dpkg-deb &>/dev/null; then
        dpkg-deb --build --root-owner-group "$pkg_dir" "$DIST_DIR/${pkg_name}.deb"
    elif command -v docker &>/dev/null; then
        # Use Docker if dpkg-deb not available
        docker run --rm \
            -v "$DIST_DIR:/dist" \
            -w /dist \
            debian:bookworm-slim \
            dpkg-deb --build --root-owner-group "deb-staging/$pkg_name" "${pkg_name}.deb"
    else
        log_error "Neither dpkg-deb nor docker available"
        return 1
    fi

    # Cleanup staging
    rm -rf "$pkg_dir"

    log_info "Created: $DIST_DIR/${pkg_name}.deb"
}
