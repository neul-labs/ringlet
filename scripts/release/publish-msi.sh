#!/usr/bin/env bash
# Build Windows MSI installer
# This file is sourced by release.sh

publish_msi() {
    local version="$1"

    log_info "Building Windows MSI installer..."

    local manufacturer
    manufacturer=$(parse_toml "publishers.msi.manufacturer" "$CONFIG_FILE")
    manufacturer="${manufacturer:-Neul Labs}"

    local upgrade_guid
    upgrade_guid=$(parse_toml "publishers.msi.upgrade_guid" "$CONFIG_FILE")
    if [[ -z "$upgrade_guid" ]]; then
        log_error "publishers.msi.upgrade_guid not set in release.toml"
        log_info "Generate a GUID and add it to your config (keep it forever for upgrades)"
        return 1
    fi

    if is_dry_run; then
        log_debug "[DRY-RUN] Would create MSI: ${PROJECT_NAME}-${version}.msi"
        return 0
    fi

    # Check for Windows binary
    local win_archive="$DIST_DIR/${PROJECT_NAME}-win32-x64-${version}.zip"
    if [[ ! -f "$win_archive" ]]; then
        log_error "Windows binary archive not found: $win_archive"
        return 1
    fi

    local msi_dir="$SCRIPT_DIR/packaging/msi"
    local staging_dir="$DIST_DIR/msi-staging"
    mkdir -p "$staging_dir"

    # Extract Windows binaries
    unzip -o "$win_archive" -d "$staging_dir"

    # Move binaries to staging root (remove nested directory)
    if [[ -d "$staging_dir/${PROJECT_NAME}-win32-x64-${version}" ]]; then
        mv "$staging_dir/${PROJECT_NAME}-win32-x64-${version}"/* "$staging_dir/"
        rmdir "$staging_dir/${PROJECT_NAME}-win32-x64-${version}"
    fi

    # Generate WiX variables file
    cat > "$staging_dir/variables.wxi" << EOF
<?xml version="1.0" encoding="utf-8"?>
<Include>
    <?define Version="${version}" ?>
    <?define Manufacturer="${manufacturer}" ?>
    <?define UpgradeCode="${upgrade_guid}" ?>
    <?define ProductName="Ringlet" ?>
</Include>
EOF

    # Check for WiX template or generate
    local wxs_file="$staging_dir/main.wxs"
    if [[ -f "$msi_dir/main.wxs.template" ]]; then
        sed -e "s/{{VERSION}}/$version/g" \
            -e "s/{{MANUFACTURER}}/$manufacturer/g" \
            -e "s/{{UPGRADE_GUID}}/$upgrade_guid/g" \
            "$msi_dir/main.wxs.template" > "$wxs_file"
    else
        # Generate a basic WiX source file
        cat > "$wxs_file" << 'WXSEOF'
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <?include variables.wxi ?>

    <Product Id="*"
             Name="$(var.ProductName)"
             Version="$(var.Version)"
             Manufacturer="$(var.Manufacturer)"
             Language="1033"
             Codepage="1252"
             UpgradeCode="$(var.UpgradeCode)">

        <Package Id="*"
                 InstallerVersion="500"
                 Compressed="yes"
                 InstallScope="perMachine"
                 Description="Ringlet $(var.Version) Installer"
                 Manufacturer="$(var.Manufacturer)" />

        <MajorUpgrade DowngradeErrorMessage="A newer version of Ringlet is already installed." />
        <MediaTemplate EmbedCab="yes" />

        <Feature Id="ProductFeature" Title="Ringlet" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
            <ComponentRef Id="PathEnvironment" />
        </Feature>

        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="ProgramFiles64Folder">
                <Directory Id="INSTALLFOLDER" Name="Ringlet" />
            </Directory>
        </Directory>

        <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
            <Component Id="RingletExe" Guid="*">
                <File Id="ringlet.exe" Source="ringlet.exe" KeyPath="yes" />
            </Component>
            <Component Id="RingletdExe" Guid="*">
                <File Id="ringletd.exe" Source="ringletd.exe" />
            </Component>
        </ComponentGroup>

        <Component Id="PathEnvironment" Directory="INSTALLFOLDER" Guid="*">
            <Environment Id="PATH" Name="PATH" Value="[INSTALLFOLDER]"
                        Permanent="no" Part="last" Action="set" System="yes" />
        </Component>

        <UI>
            <UIRef Id="WixUI_Minimal" />
        </UI>
    </Product>
</Wix>
WXSEOF
    fi

    # Copy license if exists
    if [[ -f "$SCRIPT_DIR/LICENSE" ]]; then
        # Convert to RTF for WiX (simple conversion)
        echo "{\rtf1\ansi\deff0 {\fonttbl {\f0 Courier;}}" > "$staging_dir/license.rtf"
        echo "\f0\fs20" >> "$staging_dir/license.rtf"
        sed 's/$/\\par/' "$SCRIPT_DIR/LICENSE" >> "$staging_dir/license.rtf"
        echo "}" >> "$staging_dir/license.rtf"
    fi

    local msi_output="$DIST_DIR/${PROJECT_NAME}-${version}.msi"

    # Build MSI with WiX
    if command -v candle &>/dev/null && command -v light &>/dev/null; then
        # Native WiX (Windows)
        log_info "Building MSI with native WiX..."
        cd "$staging_dir"
        candle -nologo main.wxs -o main.wixobj
        light -nologo -ext WixUIExtension main.wixobj -o "$msi_output"
        cd "$SCRIPT_DIR"
    elif command -v docker &>/dev/null; then
        # Use Docker with WiX image
        log_info "Building MSI with Docker (WiX)..."
        docker run --rm \
            -v "$staging_dir:/src" \
            -v "$DIST_DIR:/out" \
            -w /src \
            dactiv/wix:latest \
            bash -c "
                candle -nologo main.wxs -o main.wixobj && \
                light -nologo -ext WixUIExtension main.wixobj -o /out/${PROJECT_NAME}-${version}.msi
            " 2>&1
    else
        log_error "Neither WiX tools nor Docker available"
        log_info "Install Docker or WiX Toolset to build MSI"
        rm -rf "$staging_dir"
        return 1
    fi

    if [[ -f "$msi_output" ]]; then
        local checksum
        checksum=$(compute_sha256 "$msi_output")
        save_checksum "$VERSION" "win32-msi" "$checksum"

        log_success "Created: $msi_output"
        log_info "Checksum: $checksum"

        # Note about signing
        log_info ""
        log_info "To sign the MSI for distribution:"
        log_info "  signtool sign /f cert.pfx /p password /t http://timestamp.digicert.com $msi_output"
    else
        log_error "MSI creation failed"
        rm -rf "$staging_dir"
        return 1
    fi

    # Cleanup
    rm -rf "$staging_dir"

    return 0
}
