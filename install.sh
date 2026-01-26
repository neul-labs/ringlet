#!/usr/bin/env bash
set -euo pipefail

# ringlet installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/neul-labs/ccswitch/main/install.sh | bash

VERSION="${RINGLET_VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
FROM_SOURCE="${FROM_SOURCE:-false}"
LOCAL_BUILD="${LOCAL_BUILD:-false}"
GITHUB_REPO="neul-labs/ccswitch"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }
debug() { echo -e "${BLUE}[DEBUG]${NC} $1"; }

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version|-v)
            VERSION="$2"
            shift 2
            ;;
        --install-dir|-d)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --from-source|-s)
            FROM_SOURCE="true"
            shift
            ;;
        --local|-l)
            LOCAL_BUILD="true"
            shift
            ;;
        --help|-h)
            cat << EOF
ringlet installer

Usage: install.sh [OPTIONS]

Options:
    --version, -v VERSION    Install specific version (default: latest)
    --install-dir, -d DIR    Installation directory (default: ~/.local/bin)
    --from-source, -s        Build from source instead of downloading binary
    --local, -l              Build from current directory (when inside ringlet repo)
    --help, -h               Show this help message

Environment variables:
    RINGLET_VERSION           Same as --version
    INSTALL_DIR             Same as --install-dir
    FROM_SOURCE             Set to 'true' for source build

Examples:
    # Install latest version
    curl -fsSL https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.sh | bash

    # Install specific version
    curl -fsSL https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.sh | bash -s -- --version 0.2.0

    # Install to custom directory
    curl -fsSL https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.sh | bash -s -- --install-dir /usr/local/bin

    # Build from source
    curl -fsSL https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.sh | bash -s -- --from-source

    # Build from local clone
    cd /path/to/ringlet && ./install.sh --local
EOF
            exit 0
            ;;
        *)
            error "Unknown option: $1. Use --help for usage information."
            ;;
    esac
done

detect_platform() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)   os="linux" ;;
        Darwin)  os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="win32" ;;
        *)       error "Unsupported OS: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="x64" ;;
        aarch64|arm64) arch="arm64" ;;
        *)             error "Unsupported architecture: $arch" ;;
    esac

    echo "${os}-${arch}"
}

get_latest_version() {
    local url="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
    local version

    if command -v curl &> /dev/null; then
        version=$(curl -fsSL "$url" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
    elif command -v wget &> /dev/null; then
        version=$(wget -qO- "$url" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
    else
        error "Neither curl nor wget found. Please install one of them."
    fi

    if [[ -z "$version" ]]; then
        error "Failed to fetch latest version from GitHub"
    fi

    echo "$version"
}

download_binary() {
    local version="$1"
    local platform="$2"
    local install_dir="$3"

    local ext="tar.gz"
    [[ "$platform" == "win32-x64" ]] && ext="zip"

    local url="https://github.com/${GITHUB_REPO}/releases/download/v${version}/ringlet-${platform}-${version}.${ext}"
    local tmpdir
    tmpdir="$(mktemp -d)"

    info "Downloading ringlet v${version} for ${platform}..."
    debug "URL: $url"

    if command -v curl &> /dev/null; then
        if ! curl -fsSL "$url" -o "${tmpdir}/archive.${ext}"; then
            rm -rf "$tmpdir"
            return 1
        fi
    elif command -v wget &> /dev/null; then
        if ! wget -q "$url" -O "${tmpdir}/archive.${ext}"; then
            rm -rf "$tmpdir"
            return 1
        fi
    else
        error "Neither curl nor wget found. Please install one of them."
    fi

    info "Extracting..."

    mkdir -p "$install_dir"

    if [[ "$ext" == "zip" ]]; then
        if command -v unzip &> /dev/null; then
            unzip -q "${tmpdir}/archive.zip" -d "$install_dir"
        else
            error "unzip command not found. Please install it."
        fi
    else
        tar -xzf "${tmpdir}/archive.tar.gz" -C "$install_dir"
    fi

    chmod +x "${install_dir}/ringlet"
    chmod +x "${install_dir}/ringletd"

    rm -rf "$tmpdir"
}

build_from_source() {
    local version="$1"
    local install_dir="$2"

    info "Building from source..."

    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo not found. Please install Rust: https://rustup.rs"
    fi

    # Check for git
    if ! command -v git &> /dev/null; then
        error "git not found. Please install git."
    fi

    local tmpdir
    tmpdir="$(mktemp -d)"

    info "Cloning repository..."

    if [[ "$version" == "latest" ]]; then
        git clone --depth 1 "https://github.com/${GITHUB_REPO}.git" "$tmpdir"
    else
        git clone --depth 1 --branch "v${version}" "https://github.com/${GITHUB_REPO}.git" "$tmpdir"
    fi

    info "Building (this may take a few minutes)..."

    cd "$tmpdir"
    cargo build --release

    info "Installing..."

    mkdir -p "$install_dir"
    cp target/release/ringlet "$install_dir/"
    cp target/release/ringletd "$install_dir/"

    chmod +x "${install_dir}/ringlet"
    chmod +x "${install_dir}/ringletd"

    rm -rf "$tmpdir"
}

build_local() {
    local install_dir="$1"

    info "Building from local repository..."

    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo not found. Please install Rust: https://rustup.rs"
    fi

    info "Building (this may take a few minutes)..."

    cargo build --release

    info "Installing..."

    mkdir -p "$install_dir"
    cp target/release/ringlet "$install_dir/"
    cp target/release/ringletd "$install_dir/"

    chmod +x "${install_dir}/ringlet"
    chmod +x "${install_dir}/ringletd"
}

is_ringlet_repo() {
    # Check if we're in a ringlet repository
    if [[ -f "Cargo.toml" ]] && grep -q 'name = "ringlet"' Cargo.toml 2>/dev/null; then
        return 0
    fi
    return 1
}

verify_installation() {
    local install_dir="$1"

    if [[ -x "${install_dir}/ringlet" ]] && [[ -x "${install_dir}/ringletd" ]]; then
        info "Installation successful!"

        local version
        version="$("${install_dir}/ringlet" --version 2>/dev/null || echo "unknown")"
        info "Installed version: $version"
        return 0
    else
        error "Installation failed. Binaries not found in $install_dir"
    fi
}

check_path() {
    local install_dir="$1"

    if [[ ":$PATH:" != *":${install_dir}:"* ]]; then
        echo ""
        warn "$install_dir is not in your PATH."
        echo ""
        echo "Add it by adding this line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "  export PATH=\"\$PATH:${install_dir}\""
        echo ""
        echo "Then restart your shell or run:"
        echo ""
        echo "  source ~/.bashrc  # or ~/.zshrc"
        echo ""
    fi
}

main() {
    echo ""
    echo "  ╭─────────────────────────────────╮"
    echo "  │      ringlet installer            │"
    echo "  │  CLI orchestrator for coding    │"
    echo "  │           agents                │"
    echo "  ╰─────────────────────────────────╯"
    echo ""

    # Check for local build mode (explicit flag or auto-detect)
    if [[ "$LOCAL_BUILD" == "true" ]]; then
        if ! is_ringlet_repo; then
            error "Not in a ringlet repository. Use --local only from within the ringlet source directory."
        fi
        info "Local build mode enabled"
        info "Install directory: $INSTALL_DIR"
        build_local "$INSTALL_DIR"
        verify_installation "$INSTALL_DIR"
        check_path "$INSTALL_DIR"
        echo ""
        info "To get started, run: ringlet --help"
        echo ""
        return
    fi

    # Auto-detect local repository (when running ./install.sh from repo)
    if is_ringlet_repo && [[ "$FROM_SOURCE" != "true" ]]; then
        info "Detected local ringlet repository"
        info "Install directory: $INSTALL_DIR"
        build_local "$INSTALL_DIR"
        verify_installation "$INSTALL_DIR"
        check_path "$INSTALL_DIR"
        echo ""
        info "To get started, run: ringlet --help"
        echo ""
        return
    fi

    # Get version (with graceful network failure handling)
    if [[ "$VERSION" == "latest" ]]; then
        info "Fetching latest version..."
        if ! VERSION="$(get_latest_version 2>/dev/null)"; then
            warn "Could not fetch latest version (network unavailable?)"
            info "Falling back to source build from main branch..."
            FROM_SOURCE="true"
            VERSION="main"
        fi
    fi

    if [[ "$FROM_SOURCE" != "true" ]]; then
        info "Version: v${VERSION}"
    else
        info "Building from: ${VERSION}"
    fi

    # Detect platform
    local platform
    platform="$(detect_platform)"
    info "Platform: $platform"
    info "Install directory: $INSTALL_DIR"

    # Install
    if [[ "$FROM_SOURCE" == "true" ]]; then
        build_from_source "$VERSION" "$INSTALL_DIR"
    else
        if ! download_binary "$VERSION" "$platform" "$INSTALL_DIR"; then
            warn "Binary download failed, falling back to source build..."
            build_from_source "$VERSION" "$INSTALL_DIR"
        fi
    fi

    # Verify
    verify_installation "$INSTALL_DIR"

    # Check PATH
    check_path "$INSTALL_DIR"

    echo ""
    info "To get started, run: ringlet --help"
    echo ""
}

main
