#!/usr/bin/env bash
# Publish to PyPI
# This file is sourced by release.sh

publish_pypi() {
    local version="$1"

    log_info "Publishing to PyPI..."

    # Check for PyPI token
    if [[ -z "${PYPI_TOKEN:-}" ]]; then
        log_error "PYPI_TOKEN not set"
        return 1
    fi

    local package_name
    package_name=$(parse_toml "publishers.pypi.package_name" "$CONFIG_FILE")

    log_info "Publishing $package_name@$version to PyPI"

    local pypi_dir="$SCRIPT_DIR/packaging/pypi"
    if [[ ! -d "$pypi_dir" ]]; then
        log_error "PyPI packaging directory not found: $pypi_dir"
        return 1
    fi

    if is_dry_run; then
        log_debug "[DRY-RUN] Would publish $package_name to PyPI"
        return 0
    fi

    cd "$pypi_dir"

    # Update version in pyproject.toml
    if [[ -f "pyproject.toml" ]]; then
        sed -i "s/^version = \".*\"/version = \"$version\"/" pyproject.toml
        sed -i "s/^name = \".*\"/name = \"$package_name\"/" pyproject.toml
    fi

    # Update version in __init__.py if exists
    local init_file="$pypi_dir/ringlet/__init__.py"
    if [[ -f "$init_file" ]]; then
        sed -i "s/__version__ = \".*\"/__version__ = \"$version\"/" "$init_file"
    fi

    # Check for required tools
    if ! command -v python3 &>/dev/null; then
        log_error "python3 not found"
        return 1
    fi

    # Create virtual environment if needed
    if [[ ! -d ".venv" ]]; then
        python3 -m venv .venv
    fi

    # Install build tools
    .venv/bin/pip install --quiet build twine

    # Build wheel
    log_info "Building wheel..."
    .venv/bin/python -m build --wheel

    # Upload to PyPI
    log_info "Uploading to PyPI..."
    .venv/bin/twine upload \
        --username __token__ \
        --password "$PYPI_TOKEN" \
        dist/*.whl 2>&1 || {
        log_warn "PyPI upload failed (may already be published)"
    }

    # Cleanup
    rm -rf dist/ build/ *.egg-info
    cd "$SCRIPT_DIR"

    log_success "PyPI publishing complete"
    return 0
}
