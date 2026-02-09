#!/usr/bin/env bash
# Generate Arch Linux PKGBUILD for AUR
# This file is sourced by release.sh

publish_arch() {
    local version="$1"

    log_info "Generating Arch Linux PKGBUILD..."

    local maintainer
    maintainer=$(parse_toml "publishers.arch.maintainer" "$CONFIG_FILE")
    local pkgname
    pkgname=$(parse_toml "publishers.arch.pkgname" "$CONFIG_FILE")

    if is_dry_run; then
        log_debug "[DRY-RUN] Would generate PKGBUILD for $pkgname"
        return 0
    fi

    local arch_dir="$DIST_DIR/arch"
    mkdir -p "$arch_dir"

    # Get checksums
    local x64_sha256
    x64_sha256=$(get_checksum "$version" "linux-x64")
    if [[ -z "$x64_sha256" ]]; then
        x64_sha256=$(compute_sha256 "$DIST_DIR/${PROJECT_NAME}-linux-x64-${version}.tar.gz" 2>/dev/null || echo "SKIP")
    fi

    local arm64_sha256
    arm64_sha256=$(get_checksum "$version" "linux-arm64")
    if [[ -z "$arm64_sha256" ]]; then
        arm64_sha256=$(compute_sha256 "$DIST_DIR/${PROJECT_NAME}-linux-arm64-${version}.tar.gz" 2>/dev/null || echo "SKIP")
    fi

    # Check for template
    local template="$SCRIPT_DIR/packaging/arch/PKGBUILD.template"
    if [[ -f "$template" ]]; then
        # Use template
        sed -e "s/{{VERSION}}/$version/g" \
            -e "s/{{MAINTAINER}}/$maintainer/g" \
            -e "s/{{SHA256_X64}}/$x64_sha256/g" \
            -e "s/{{SHA256_ARM64}}/$arm64_sha256/g" \
            "$template" > "$arch_dir/PKGBUILD"
    else
        # Generate PKGBUILD from scratch
        cat > "$arch_dir/PKGBUILD" << EOF
# Maintainer: ${maintainer}
pkgname=${pkgname}
pkgver=${version}
pkgrel=1
pkgdesc="CLI orchestrator for coding agents"
arch=('x86_64' 'aarch64')
url="https://github.com/${REPOSITORY}"
license=('MIT')
depends=('gcc-libs')
provides=('ringlet' 'ringletd')
conflicts=('ringlet-git')

source_x86_64=("https://github.com/${REPOSITORY}/releases/download/v\${pkgver}/${PROJECT_NAME}-linux-x64-\${pkgver}.tar.gz")
source_aarch64=("https://github.com/${REPOSITORY}/releases/download/v\${pkgver}/${PROJECT_NAME}-linux-arm64-\${pkgver}.tar.gz")

sha256sums_x86_64=('${x64_sha256}')
sha256sums_aarch64=('${arm64_sha256}')

package() {
    install -Dm755 ringlet "\$pkgdir/usr/bin/ringlet"
    install -Dm755 ringletd "\$pkgdir/usr/bin/ringletd"
}
EOF
    fi

    log_info "Generated PKGBUILD at: $arch_dir/PKGBUILD"

    # Generate .SRCINFO
    if command -v makepkg &>/dev/null; then
        cd "$arch_dir"
        makepkg --printsrcinfo > .SRCINFO
        cd "$SCRIPT_DIR"
        log_info "Generated .SRCINFO"
    elif command -v docker &>/dev/null; then
        docker run --rm \
            -v "$arch_dir:/pkg" \
            -w /pkg \
            archlinux:base \
            bash -c "pacman -Sy --noconfirm base-devel && sudo -u nobody makepkg --printsrcinfo > .SRCINFO" 2>/dev/null || {
            log_warn "Could not generate .SRCINFO automatically"
        }
    else
        log_warn "makepkg not available. .SRCINFO not generated."
        log_info "Generate manually with: cd $arch_dir && makepkg --printsrcinfo > .SRCINFO"
    fi

    log_success "Arch Linux package files ready"
    log_info "To publish to AUR:"
    log_info "  1. Clone your AUR package repo"
    log_info "  2. Copy PKGBUILD and .SRCINFO from $arch_dir"
    log_info "  3. Commit and push to AUR"

    return 0
}
