#!/usr/bin/env bash
# Publish to npm
# This file is sourced by release.sh

publish_npm() {
    local version="$1"

    log_info "Publishing to npm..."

    # Check for npm token
    if [[ -z "${NPM_TOKEN:-}" ]]; then
        log_error "NPM_TOKEN not set"
        return 1
    fi

    # Get npm configuration
    local scope
    scope=$(parse_toml "publishers.npm.scope" "$CONFIG_FILE")
    local package_name
    package_name=$(parse_toml "publishers.npm.package_name" "$CONFIG_FILE")

    local full_name="${scope}/${package_name}"
    log_info "Publishing $full_name@$version"

    local npm_dir="$SCRIPT_DIR/packaging/npm"
    if [[ ! -d "$npm_dir" ]]; then
        log_error "npm packaging directory not found: $npm_dir"
        return 1
    fi

    if is_dry_run; then
        log_debug "[DRY-RUN] Would publish $full_name to npm"
        return 0
    fi

    # Configure npm auth
    echo "//registry.npmjs.org/:_authToken=${NPM_TOKEN}" > "$npm_dir/.npmrc"

    # Update version in package.json
    cd "$npm_dir"

    # Update main package.json
    if [[ -f "package.json" ]]; then
        # Use node/jq to update version
        if command -v node &>/dev/null; then
            node -e "
                const fs = require('fs');
                const pkg = JSON.parse(fs.readFileSync('package.json'));
                pkg.version = '$version';
                pkg.name = '$full_name';
                fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2));
            "
        else
            # Fallback to sed
            sed -i "s/\"version\": \".*\"/\"version\": \"$version\"/" package.json
            sed -i "s/\"name\": \".*\"/\"name\": \"$full_name\"/" package.json
        fi
    fi

    # Publish
    npm publish --access public 2>&1 || {
        log_warn "npm publish failed (may already be published)"
    }

    # Cleanup
    rm -f "$npm_dir/.npmrc"
    cd "$SCRIPT_DIR"

    log_success "npm publishing complete"
    return 0
}
