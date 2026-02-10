#!/usr/bin/env bash
#
# ringlet release script
#
# Builds and publishes ringlet to 10 package managers:
#   Cargo, npm, PyPI, RubyGems, Homebrew, Chocolatey, Debian, Arch, DMG, MSI
#
# Usage: ./release.sh <version>
#
# Environment variables:
#   DRY_RUN=true          Show what would be done without executing
#   SKIP_BUILD=true       Skip build phase (use existing artifacts)
#   SKIP_PUBLISH=true     Skip publish phase (build only)
#   ONLY_PUBLISH=x,y      Only publish to specific managers
#   FORCE_FRESH=true      Start fresh, ignore existing state
#   NO_GITHUB=true        Skip GitHub release creation
#
# Examples:
#   ./release.sh 0.2.0                       # Full release
#   DRY_RUN=true ./release.sh 0.2.0          # Dry run
#   SKIP_BUILD=true ./release.sh 0.2.0       # Republish existing artifacts
#   ONLY_PUBLISH=cargo,npm ./release.sh 0.2.0  # Only Cargo and npm

set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_DIR="$SCRIPT_DIR/scripts/release"
DIST_DIR="$SCRIPT_DIR/dist"
CONFIG_FILE="$SCRIPT_DIR/release.toml"

# Load libraries
source "$RELEASE_DIR/lib/common.sh"
source "$RELEASE_DIR/lib/state.sh"

# Configuration
VERSION=""
PROJECT_NAME=""
BINARIES=()
REPOSITORY=""
PLATFORMS=()

usage() {
    cat << 'EOF'
ringlet release script

Usage: ./release.sh <version> [options]

Arguments:
    version     Semantic version (e.g., 0.2.0)

Environment variables:
    DRY_RUN=true          Show what would be done without executing
    SKIP_BUILD=true       Skip build phase (use existing artifacts)
    SKIP_PUBLISH=true     Skip publish phase (build only)
    ONLY_PUBLISH=x,y      Only publish to specific managers
    FORCE_FRESH=true      Start fresh, ignore existing state
    NO_GITHUB=true        Skip GitHub release creation

Publishers:
    cargo, npm, pypi, rubygems, homebrew, chocolatey, debian, arch, dmg, msi

Examples:
    ./release.sh 0.2.0                        # Full release
    DRY_RUN=true ./release.sh 0.2.0           # Dry run
    SKIP_BUILD=true ./release.sh 0.2.0        # Republish existing
    ONLY_PUBLISH=cargo,npm ./release.sh 0.2.0 # Selective publish
EOF
}

load_config() {
    if [[ ! -f "$CONFIG_FILE" ]]; then
        die "Configuration file not found: $CONFIG_FILE"
    fi

    PROJECT_NAME=$(parse_toml "project.name" "$CONFIG_FILE")
    REPOSITORY=$(parse_toml "project.repository" "$CONFIG_FILE")

    # Load binaries array
    while IFS= read -r binary; do
        [[ -n "$binary" ]] && BINARIES+=("$binary")
    done < <(parse_toml_array "project.binaries" "$CONFIG_FILE")

    # Load platforms array
    while IFS= read -r platform; do
        [[ -n "$platform" ]] && PLATFORMS+=("$platform")
    done < <(parse_toml_array "build.platforms" "$CONFIG_FILE")

    log_debug "Loaded config: name=$PROJECT_NAME, binaries=${BINARIES[*]}, platforms=${PLATFORMS[*]}"
}

check_dependencies() {
    log_step "Checking dependencies..."

    require_cmd cargo
    require_cmd git
    require_cmd curl

    # Check for cross (for cross-compilation)
    if ! command -v cross &> /dev/null; then
        log_warn "cross not found. Install with: cargo install cross"
        log_warn "Cross-compilation will fall back to cargo (native only)"
    fi

    # Check for Docker (required for cross and some publishers)
    if ! command -v docker &> /dev/null; then
        log_warn "Docker not found. Some features may not work:"
        log_warn "  - Cross-compilation for non-native platforms"
        log_warn "  - Debian package building"
        log_warn "  - MSI package building"
    fi

    # Check for gh CLI (for GitHub releases)
    if ! command -v gh &> /dev/null; then
        log_warn "gh CLI not found. GitHub releases will be skipped."
        log_warn "Install from: https://cli.github.com/"
    fi

    log_success "Dependency check complete"
}

update_cargo_version() {
    local version="$1"
    local cargo_toml="$SCRIPT_DIR/Cargo.toml"

    log_step "Updating Cargo.toml version to $version..."

    if is_dry_run; then
        log_debug "[DRY-RUN] Would update version in $cargo_toml"
        return
    fi

    # Update workspace version
    sed -i "s/^version = \".*\"/version = \"$version\"/" "$cargo_toml"

    # Verify the change
    local new_version
    new_version=$(grep '^version = ' "$cargo_toml" | head -1 | cut -d'"' -f2)
    if [[ "$new_version" != "$version" ]]; then
        die "Failed to update version in Cargo.toml"
    fi

    log_success "Updated Cargo.toml to version $version"
}

run_build_phase() {
    local version="$1"

    if [[ "${SKIP_BUILD:-false}" == "true" ]]; then
        log_info "Skipping build phase (SKIP_BUILD=true)"
        return
    fi

    log_step "=== Phase 1: Building Binaries ==="

    # Source the build script
    source "$RELEASE_DIR/build.sh"

    # Build for each platform
    for platform in "${PLATFORMS[@]}"; do
        local state_key="BUILD_${platform//-/_}"

        if is_completed "$version" "$state_key"; then
            log_info "Skipping $platform (already built)"
            continue
        fi

        log_step "Building for $platform..."
        mark_in_progress "$version" "$state_key"

        if build_platform "$version" "$platform"; then
            mark_completed "$version" "$state_key"
            log_success "Built $platform"
        else
            mark_failed "$version" "$state_key"
            log_error "Failed to build $platform"
            return 1
        fi
    done

    # Create macOS universal binary if enabled
    if parse_toml_bool "build.macos_universal" "$CONFIG_FILE" 2>/dev/null; then
        local state_key="BUILD_darwin_universal"
        if ! is_completed "$version" "$state_key"; then
            log_step "Creating macOS universal binary..."
            mark_in_progress "$version" "$state_key"

            if create_universal_binary "$version"; then
                mark_completed "$version" "$state_key"
                log_success "Created macOS universal binary"
            else
                mark_failed "$version" "$state_key"
                log_warn "Failed to create universal binary (continuing)"
            fi
        fi
    fi

    # Generate checksums
    generate_checksums "$version"

    log_success "Build phase complete"
}

run_publish_phase() {
    local version="$1"

    if [[ "${SKIP_PUBLISH:-false}" == "true" ]]; then
        log_info "Skipping publish phase (SKIP_PUBLISH=true)"
        return
    fi

    log_step "=== Phase 2: Publishing ==="

    local publishers=(cargo npm pypi rubygems homebrew chocolatey debian arch dmg msi)

    for publisher in "${publishers[@]}"; do
        if ! should_run_publisher "$publisher"; then
            log_debug "Skipping $publisher (not in ONLY_PUBLISH)"
            continue
        fi

        # Check if publisher is enabled in config
        local enabled
        enabled=$(parse_toml "publishers.$publisher.enabled" "$CONFIG_FILE" 2>/dev/null)
        if [[ "$enabled" != "true" ]]; then
            log_debug "Skipping $publisher (disabled in config)"
            continue
        fi

        local state_key="PUBLISH_$publisher"

        if is_completed "$version" "$state_key"; then
            log_info "Skipping $publisher (already published)"
            continue
        fi

        local publish_script="$RELEASE_DIR/publish-${publisher}.sh"
        if [[ ! -f "$publish_script" ]]; then
            log_warn "Publisher script not found: $publish_script"
            continue
        fi

        log_step "Publishing to $publisher..."
        mark_in_progress "$version" "$state_key"

        if source "$publish_script" && "publish_${publisher}" "$version"; then
            mark_completed "$version" "$state_key"
            log_success "Published to $publisher"
        else
            mark_failed "$version" "$state_key"
            log_error "Failed to publish to $publisher"
            # Continue with other publishers (soft failure)
        fi
    done

    log_success "Publish phase complete"
}

run_github_release() {
    local version="$1"

    if [[ "${NO_GITHUB:-false}" == "true" ]]; then
        log_info "Skipping GitHub release (NO_GITHUB=true)"
        return
    fi

    if ! command -v gh &> /dev/null; then
        log_warn "Skipping GitHub release (gh CLI not found)"
        return
    fi

    local state_key="PUBLISH_github"

    if is_completed "$version" "$state_key"; then
        log_info "Skipping GitHub release (already created)"
        return
    fi

    log_step "=== Phase 3: Creating GitHub Release ==="
    mark_in_progress "$version" "$state_key"

    # Create and push tag
    local tag="v$version"

    if is_dry_run; then
        log_debug "[DRY-RUN] Would create tag $tag and GitHub release"
        mark_completed "$version" "$state_key"
        return
    fi

    # Check if tag exists
    if git rev-parse "$tag" &>/dev/null; then
        log_info "Tag $tag already exists"
    else
        log_info "Creating tag $tag..."
        git tag -a "$tag" -m "Release $version"
        git push origin "$tag"
    fi

    # Generate changelog
    local changelog
    changelog=$(git log --oneline "$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo '')..HEAD" 2>/dev/null || echo "Initial release")

    # Get package names from config
    local npm_scope
    npm_scope=$(parse_toml "publishers.npm.scope" "$CONFIG_FILE")
    local npm_name
    npm_name=$(parse_toml "publishers.npm.package_name" "$CONFIG_FILE")
    local pypi_name
    pypi_name=$(parse_toml "publishers.pypi.package_name" "$CONFIG_FILE")
    local rubygems_name
    rubygems_name=$(parse_toml "publishers.rubygems.package_name" "$CONFIG_FILE")
    local homebrew_tap
    homebrew_tap=$(parse_toml "publishers.homebrew.tap_repo" "$CONFIG_FILE")
    local homebrew_formula
    homebrew_formula=$(parse_toml "publishers.homebrew.formula_name" "$CONFIG_FILE")
    local choco_id
    choco_id=$(parse_toml "publishers.chocolatey.package_id" "$CONFIG_FILE")

    # Create GitHub release
    log_info "Creating GitHub release..."
    local release_notes="## Installation

### Quick Install (Recommended)

\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/${REPOSITORY}/main/install.sh | bash
\`\`\`

### Package Managers

| Platform | Command |
|----------|---------|
| **Cargo** | \`cargo install ${PROJECT_NAME}\` |
| **npm** | \`npm install -g ${npm_scope}/${npm_name}\` |
| **PyPI** | \`pip install ${pypi_name}\` |
| **RubyGems** | \`gem install ${rubygems_name}\` |
| **Homebrew** | \`brew install ${homebrew_tap}/${homebrew_formula}\` |
| **Chocolatey** | \`choco install ${choco_id}\` |
| **Arch Linux (AUR)** | \`yay -S ${PROJECT_NAME}\` |
| **Debian/Ubuntu** | Download \`.deb\` from assets below |

### Direct Downloads

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x64 | [${PROJECT_NAME}-linux-x64-${version}.tar.gz](https://github.com/${REPOSITORY}/releases/download/v${version}/${PROJECT_NAME}-linux-x64-${version}.tar.gz) |
| Linux | ARM64 | [${PROJECT_NAME}-linux-arm64-${version}.tar.gz](https://github.com/${REPOSITORY}/releases/download/v${version}/${PROJECT_NAME}-linux-arm64-${version}.tar.gz) |
| macOS | Universal | [${PROJECT_NAME}-darwin-universal-${version}.tar.gz](https://github.com/${REPOSITORY}/releases/download/v${version}/${PROJECT_NAME}-darwin-universal-${version}.tar.gz) |
| macOS | Intel | [${PROJECT_NAME}-darwin-x64-${version}.tar.gz](https://github.com/${REPOSITORY}/releases/download/v${version}/${PROJECT_NAME}-darwin-x64-${version}.tar.gz) |
| macOS | Apple Silicon | [${PROJECT_NAME}-darwin-arm64-${version}.tar.gz](https://github.com/${REPOSITORY}/releases/download/v${version}/${PROJECT_NAME}-darwin-arm64-${version}.tar.gz) |
| Windows | x64 | [${PROJECT_NAME}-win32-x64-${version}.zip](https://github.com/${REPOSITORY}/releases/download/v${version}/${PROJECT_NAME}-win32-x64-${version}.zip) |

---

## What's Changed

$changelog

---

## Checksums (SHA256)

\`\`\`
$(cat "$DIST_DIR/checksums.txt" 2>/dev/null || echo "No checksums available")
\`\`\`
"

    # Collect all release assets
    local assets=()
    for file in "$DIST_DIR"/*.tar.gz "$DIST_DIR"/*.zip "$DIST_DIR"/*.deb "$DIST_DIR"/*.msi "$DIST_DIR"/*.dmg; do
        [[ -f "$file" ]] && assets+=("$file")
    done
    [[ -f "$DIST_DIR/checksums.txt" ]] && assets+=("$DIST_DIR/checksums.txt")

    gh release create "$tag" \
        --title "Release $version" \
        --notes "$release_notes" \
        "${assets[@]}" 2>/dev/null || true

    mark_completed "$version" "$state_key"
    log_success "GitHub release created: $tag"
}

main() {
    # Parse arguments
    VERSION="${1:-}"

    if [[ -z "$VERSION" ]] || [[ "$VERSION" == "-h" ]] || [[ "$VERSION" == "--help" ]]; then
        usage
        exit 0
    fi

    # Validate version format
    validate_version "$VERSION"

    echo ""
    echo -e "${BOLD}╔════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║     ringlet Release Script v$VERSION${NC}"
    echo -e "${BOLD}╚════════════════════════════════════════════════╝${NC}"
    echo ""

    if is_dry_run; then
        log_warn "Running in DRY-RUN mode - no changes will be made"
        echo ""
    fi

    # Load configuration
    load_config

    # Check dependencies
    check_dependencies

    # Create directories
    mkdir -p "$DIST_DIR" "$STATE_DIR"

    # Check for resume
    if check_resume "$VERSION"; then
        log_info "Resuming release from checkpoint..."
    else
        # Fresh start
        init_state "$VERSION"
        update_cargo_version "$VERSION"
    fi

    # Run phases
    run_build_phase "$VERSION"
    run_publish_phase "$VERSION"
    run_github_release "$VERSION"

    # Cleanup
    if [[ "${KEEP_STATE:-false}" != "true" ]]; then
        cleanup_state "$VERSION"
    fi

    echo ""
    log_success "╔════════════════════════════════════════════════╗"
    log_success "║     Release v$VERSION Complete!"
    log_success "╚════════════════════════════════════════════════╝"
    echo ""
}

# Run main with all arguments
main "$@"
