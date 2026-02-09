#!/usr/bin/env bash
# Publish to RubyGems
# This file is sourced by release.sh

publish_rubygems() {
    local version="$1"

    log_info "Publishing to RubyGems..."

    # Check for RubyGems API key
    if [[ -z "${RUBYGEMS_API_KEY:-}" ]]; then
        log_error "RUBYGEMS_API_KEY not set"
        return 1
    fi

    local package_name
    package_name=$(parse_toml "publishers.rubygems.package_name" "$CONFIG_FILE")

    log_info "Publishing $package_name@$version to RubyGems"

    local gem_dir="$SCRIPT_DIR/packaging/rubygems"
    if [[ ! -d "$gem_dir" ]]; then
        log_error "RubyGems packaging directory not found: $gem_dir"
        return 1
    fi

    if is_dry_run; then
        log_debug "[DRY-RUN] Would publish $package_name to RubyGems"
        return 0
    fi

    cd "$gem_dir"

    # Update version in gemspec
    local gemspec_file="${package_name}.gemspec"
    if [[ -f "$gemspec_file" ]]; then
        sed -i "s/spec.version *= *\".*\"/spec.version = \"$version\"/" "$gemspec_file"
        sed -i "s/spec.name *= *\".*\"/spec.name = \"$package_name\"/" "$gemspec_file"
    fi

    # Update version in lib file
    local lib_file="lib/ringlet.rb"
    if [[ -f "$lib_file" ]]; then
        sed -i "s/VERSION *= *\".*\"/VERSION = \"$version\"/" "$lib_file"
    fi

    # Configure credentials
    mkdir -p ~/.gem
    echo "---" > ~/.gem/credentials
    echo ":rubygems_api_key: $RUBYGEMS_API_KEY" >> ~/.gem/credentials
    chmod 0600 ~/.gem/credentials

    # Build gem
    log_info "Building gem..."
    gem build "$gemspec_file"

    # Push to RubyGems
    log_info "Pushing to RubyGems..."
    gem push "${package_name}-${version}.gem" 2>&1 || {
        log_warn "RubyGems push failed (may already be published)"
    }

    # Cleanup
    rm -f *.gem
    cd "$SCRIPT_DIR"

    log_success "RubyGems publishing complete"
    return 0
}
