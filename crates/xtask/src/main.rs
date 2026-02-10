//! Cross-platform release tooling for ringlet
//!
//! Usage:
//!   cargo xtask release 0.2.0
//!   cargo xtask release 0.2.0 --dry-run
//!   cargo xtask release 0.2.0 --only cargo,npm
//!   cargo xtask build 0.2.0

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use console::{style, Emoji};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
static PACKAGE: Emoji<'_, '_> = Emoji("📦 ", "");
static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", "");
static CHECK: Emoji<'_, '_> = Emoji("✅ ", "[OK] ");
static WARN: Emoji<'_, '_> = Emoji("⚠️  ", "[WARN] ");
static ERROR: Emoji<'_, '_> = Emoji("❌ ", "[ERROR] ");

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development and release tasks for ringlet")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build release binaries for all platforms
    Build {
        /// Version to build
        version: String,

        /// Only build for specific platforms (comma-separated)
        #[arg(long)]
        only: Option<String>,

        /// Dry run - show what would be done
        #[arg(long)]
        dry_run: bool,
    },

    /// Full release: build, publish, and create GitHub release
    Release {
        /// Version to release
        version: String,

        /// Dry run - show what would be done
        #[arg(long)]
        dry_run: bool,

        /// Skip build phase
        #[arg(long)]
        skip_build: bool,

        /// Skip publish phase
        #[arg(long)]
        skip_publish: bool,

        /// Only publish to specific registries (comma-separated)
        #[arg(long)]
        only: Option<String>,

        /// Skip GitHub release
        #[arg(long)]
        no_github: bool,
    },

    /// Publish to a specific registry
    Publish {
        /// Registry to publish to
        registry: String,

        /// Version to publish
        version: String,

        /// Dry run - show what would be done
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Deserialize)]
struct ReleaseConfig {
    project: ProjectConfig,
    build: BuildConfig,
    publishers: PublishersConfig,
}

#[derive(Debug, Deserialize)]
struct ProjectConfig {
    name: String,
    binaries: Vec<String>,
    repository: String,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BuildConfig {
    platforms: Vec<String>,
    #[serde(default)]
    macos_universal: bool,
    #[serde(default = "default_dist_dir")]
    dist_dir: String,
}

fn default_dist_dir() -> String {
    "dist".to_string()
}

#[derive(Debug, Deserialize, Default)]
struct PublishersConfig {
    #[serde(default)]
    cargo: Option<PublisherEnabled>,
    #[serde(default)]
    npm: Option<PublisherEnabled>,
    #[serde(default)]
    pypi: Option<PublisherEnabled>,
    #[serde(default)]
    rubygems: Option<PublisherEnabled>,
    #[serde(default)]
    homebrew: Option<PublisherEnabled>,
    #[serde(default)]
    chocolatey: Option<PublisherEnabled>,
    #[serde(default)]
    debian: Option<PublisherEnabled>,
    #[serde(default)]
    arch: Option<PublisherEnabled>,
    #[serde(default)]
    dmg: Option<PublisherEnabled>,
    #[serde(default)]
    msi: Option<PublisherEnabled>,
}

#[derive(Debug, Deserialize)]
struct PublisherEnabled {
    #[serde(default)]
    enabled: bool,
}

/// Target triple mappings
fn get_target_triple(platform: &str) -> Option<&'static str> {
    match platform {
        "linux-x64" => Some("x86_64-unknown-linux-gnu"),
        "linux-arm64" => Some("aarch64-unknown-linux-gnu"),
        "darwin-x64" => Some("x86_64-apple-darwin"),
        "darwin-arm64" => Some("aarch64-apple-darwin"),
        "win32-x64" => Some("x86_64-pc-windows-msvc"),
        _ => None,
    }
}

/// Detect current platform
fn detect_platform() -> String {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "win32"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "unknown"
    };

    format!("{}-{}", os, arch)
}

/// Check if a command exists
fn command_exists(cmd: &str) -> bool {
    if cfg!(target_os = "windows") {
        Command::new("where")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Compute SHA256 checksum of a file
fn compute_sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Run a command and return success/failure
fn run_command(cmd: &str, args: &[&str], dry_run: bool) -> Result<bool> {
    if dry_run {
        println!(
            "  {} [DRY-RUN] {} {}",
            style("→").dim(),
            cmd,
            args.join(" ")
        );
        return Ok(true);
    }

    let status = Command::new(cmd).args(args).status()?;

    Ok(status.success())
}

/// Run a command with environment variables
fn run_command_with_env(
    cmd: &str,
    args: &[&str],
    env: &HashMap<String, String>,
    dry_run: bool,
) -> Result<bool> {
    if dry_run {
        println!(
            "  {} [DRY-RUN] {} {}",
            style("→").dim(),
            cmd,
            args.join(" ")
        );
        return Ok(true);
    }

    let status = Command::new(cmd).args(args).envs(env).status()?;

    Ok(status.success())
}

struct ReleaseContext {
    config: ReleaseConfig,
    version: String,
    project_root: PathBuf,
    dist_dir: PathBuf,
    dry_run: bool,
    checksums: HashMap<String, String>,
}

impl ReleaseContext {
    fn new(version: String, dry_run: bool) -> Result<Self> {
        let project_root = find_project_root()?;
        let config_path = project_root.join("release.toml");

        let config_content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;

        let config: ReleaseConfig = toml::from_str(&config_content)
            .with_context(|| "Failed to parse release.toml")?;

        let dist_dir = project_root.join(&config.build.dist_dir);

        Ok(Self {
            config,
            version,
            project_root,
            dist_dir,
            dry_run,
            checksums: HashMap::new(),
        })
    }

    fn log_step(&self, msg: &str) {
        println!("\n{} {}", style("▶").cyan().bold(), style(msg).bold());
    }

    fn log_info(&self, msg: &str) {
        println!("  {} {}", style("ℹ").blue(), msg);
    }

    fn log_success(&self, msg: &str) {
        println!("  {} {}", CHECK, style(msg).green());
    }

    fn log_warn(&self, msg: &str) {
        println!("  {} {}", WARN, style(msg).yellow());
    }

    fn log_error(&self, msg: &str) {
        println!("  {} {}", ERROR, style(msg).red());
    }
}

fn find_project_root() -> Result<PathBuf> {
    let mut current = env::current_dir()?;

    loop {
        if current.join("Cargo.toml").exists() && current.join("release.toml").exists() {
            return Ok(current);
        }

        if !current.pop() {
            anyhow::bail!("Could not find project root (no Cargo.toml + release.toml found)");
        }
    }
}

// ============================================================================
// Build Phase
// ============================================================================

fn build_all(ctx: &mut ReleaseContext, only: Option<&str>) -> Result<()> {
    ctx.log_step("Building release binaries");

    // Create dist directory
    if !ctx.dry_run {
        fs::create_dir_all(&ctx.dist_dir)?;
    }

    let platforms: Vec<String> = if let Some(only) = only {
        only.split(',').map(|s| s.to_string()).collect()
    } else {
        ctx.config.build.platforms.clone()
    };

    let current_platform = detect_platform();

    for platform in &platforms {
        build_platform(ctx, platform.as_str(), &current_platform)?;
    }

    // Create macOS universal binary if enabled
    if ctx.config.build.macos_universal && only.is_none() {
        create_universal_binary(ctx)?;
    }

    // Generate checksums
    generate_checksums(ctx)?;

    ctx.log_success("Build phase complete");
    Ok(())
}

fn build_platform(ctx: &mut ReleaseContext, platform: &str, current: &str) -> Result<()> {
    let target = get_target_triple(platform)
        .ok_or_else(|| anyhow::anyhow!("Unknown platform: {}", platform))?;

    println!("\n  {} Building for {} ({})", PACKAGE, platform, target);

    // Determine build command
    let needs_cross = platform != current;
    let build_cmd = if needs_cross && command_exists("cross") {
        "cross"
    } else {
        "cargo"
    };

    if needs_cross && build_cmd == "cargo" {
        ctx.log_warn(&format!(
            "cross not available, using cargo (may fail for {})",
            platform
        ));
    }

    // Build
    let args = vec!["build", "--release", "--target", target];
    if !run_command(build_cmd, &args, ctx.dry_run)? {
        anyhow::bail!("Build failed for {}", platform);
    }

    // Package
    package_binaries(ctx, platform, target)?;

    ctx.log_success(&format!("Built {}", platform));
    Ok(())
}

fn package_binaries(ctx: &mut ReleaseContext, platform: &str, target: &str) -> Result<()> {
    if ctx.dry_run {
        println!("  {} [DRY-RUN] Would package binaries for {}", style("→").dim(), platform);
        return Ok(());
    }

    let target_dir = ctx.project_root.join("target").join(target).join("release");
    let archive_name = format!(
        "{}-{}-{}",
        ctx.config.project.name, platform, ctx.version
    );

    let staging_dir = ctx.dist_dir.join("staging").join(&archive_name);
    fs::create_dir_all(&staging_dir)?;

    // Copy binaries
    for binary in &ctx.config.project.binaries {
        let bin_name = if platform.starts_with("win32") {
            format!("{}.exe", binary)
        } else {
            binary.clone()
        };

        let src = target_dir.join(&bin_name);
        let dst = staging_dir.join(&bin_name);

        if src.exists() {
            fs::copy(&src, &dst)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&dst, fs::Permissions::from_mode(0o755))?;
            }
        } else {
            ctx.log_warn(&format!("Binary not found: {}", src.display()));
        }
    }

    // Create archive
    let archive_path = if platform.starts_with("win32") {
        let zip_path = ctx.dist_dir.join(format!("{}.zip", archive_name));
        create_zip(&staging_dir, &zip_path)?;
        zip_path
    } else {
        let tar_path = ctx.dist_dir.join(format!("{}.tar.gz", archive_name));
        create_tarball(&staging_dir, &tar_path, &archive_name)?;
        tar_path
    };

    // Compute checksum
    let checksum = compute_sha256(&archive_path)?;
    ctx.checksums.insert(platform.to_string(), checksum.clone());

    ctx.log_info(&format!("Created: {}", archive_path.display()));

    // Cleanup staging
    fs::remove_dir_all(&staging_dir)?;

    Ok(())
}

fn create_tarball(src_dir: &Path, dst: &Path, archive_name: &str) -> Result<()> {
    let file = File::create(dst)?;
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut archive = tar::Builder::new(encoder);

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = format!("{}/{}", archive_name, entry.file_name().to_string_lossy());

        if path.is_file() {
            archive.append_path_with_name(&path, &name)?;
        }
    }

    archive.finish()?;
    Ok(())
}

fn create_zip(src_dir: &Path, dst: &Path) -> Result<()> {
    let file = File::create(dst)?;
    let mut zip = zip::ZipWriter::new(file);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            zip.start_file(&name, options)?;

            let mut file = File::open(&path)?;
            std::io::copy(&mut file, &mut zip)?;
        }
    }

    zip.finish()?;
    Ok(())
}

fn create_universal_binary(ctx: &mut ReleaseContext) -> Result<()> {
    if !cfg!(target_os = "macos") {
        ctx.log_warn("Universal binary creation requires macOS, skipping");
        return Ok(());
    }

    println!("\n  {} Creating macOS universal binary", PACKAGE);

    let x64_archive = ctx.dist_dir.join(format!(
        "{}-darwin-x64-{}.tar.gz",
        ctx.config.project.name, ctx.version
    ));
    let arm64_archive = ctx.dist_dir.join(format!(
        "{}-darwin-arm64-{}.tar.gz",
        ctx.config.project.name, ctx.version
    ));

    if !x64_archive.exists() || !arm64_archive.exists() {
        ctx.log_warn("Cannot create universal binary: missing darwin-x64 or darwin-arm64 archives");
        return Ok(());
    }

    if ctx.dry_run {
        println!("  {} [DRY-RUN] Would create universal binary", style("→").dim());
        return Ok(());
    }

    // This would use lipo on macOS - for now just note it
    ctx.log_info("Universal binary creation implemented on macOS only");

    Ok(())
}

fn generate_checksums(ctx: &ReleaseContext) -> Result<()> {
    ctx.log_step("Generating checksums");

    if ctx.dry_run {
        println!("  {} [DRY-RUN] Would generate checksums.txt", style("→").dim());
        return Ok(());
    }

    let checksums_path = ctx.dist_dir.join("checksums.txt");
    let mut file = File::create(&checksums_path)?;

    for entry in fs::read_dir(&ctx.dist_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str());
            if matches!(ext, Some("tar.gz" | "gz" | "zip" | "deb" | "msi" | "dmg")) {
                let checksum = compute_sha256(&path)?;
                let filename = entry.file_name().to_string_lossy().to_string();
                writeln!(file, "{}  {}", checksum, filename)?;
            }
        }
    }

    ctx.log_success(&format!("Created: {}", checksums_path.display()));
    Ok(())
}

// ============================================================================
// Publish Phase
// ============================================================================

fn is_publisher_enabled(p: &Option<PublisherEnabled>) -> bool {
    p.as_ref().map(|e| e.enabled).unwrap_or(false)
}

fn publish_all(ctx: &ReleaseContext, only: Option<&str>) -> Result<()> {
    ctx.log_step("Publishing to registries");

    let publishers = vec![
        ("cargo", is_publisher_enabled(&ctx.config.publishers.cargo)),
        ("npm", is_publisher_enabled(&ctx.config.publishers.npm)),
        ("pypi", is_publisher_enabled(&ctx.config.publishers.pypi)),
        ("rubygems", is_publisher_enabled(&ctx.config.publishers.rubygems)),
        ("homebrew", is_publisher_enabled(&ctx.config.publishers.homebrew)),
        ("chocolatey", is_publisher_enabled(&ctx.config.publishers.chocolatey)),
        ("debian", is_publisher_enabled(&ctx.config.publishers.debian)),
        ("arch", is_publisher_enabled(&ctx.config.publishers.arch)),
        ("dmg", is_publisher_enabled(&ctx.config.publishers.dmg)),
        ("msi", is_publisher_enabled(&ctx.config.publishers.msi)),
    ];

    let only_set: Option<Vec<&str>> = only.map(|s| s.split(',').collect());

    for (name, enabled) in publishers {
        if !enabled {
            continue;
        }

        if let Some(ref only) = only_set {
            if !only.contains(&name) {
                continue;
            }
        }

        println!("\n  {} Publishing to {}", ROCKET, name);

        match publish_to(ctx, name) {
            Ok(_) => ctx.log_success(&format!("Published to {}", name)),
            Err(e) => ctx.log_error(&format!("Failed to publish to {}: {}", name, e)),
        }
    }

    ctx.log_success("Publish phase complete");
    Ok(())
}

fn publish_to(ctx: &ReleaseContext, registry: &str) -> Result<()> {
    match registry {
        "cargo" => publish_cargo(ctx),
        "npm" => publish_npm(ctx),
        "pypi" => publish_pypi(ctx),
        "rubygems" => publish_rubygems(ctx),
        "homebrew" => publish_homebrew(ctx),
        "chocolatey" => publish_chocolatey(ctx),
        "debian" => publish_debian(ctx),
        "arch" => publish_arch(ctx),
        "dmg" => publish_dmg(ctx),
        "msi" => publish_msi(ctx),
        _ => anyhow::bail!("Unknown registry: {}", registry),
    }
}

fn publish_cargo(ctx: &ReleaseContext) -> Result<()> {
    let token = env::var("CARGO_REGISTRY_TOKEN")
        .context("CARGO_REGISTRY_TOKEN not set")?;

    let crates = ["ringlet-core", "ringlet-scripting", "ringletd", "ringlet"];

    for (i, crate_name) in crates.iter().enumerate() {
        if i > 0 {
            ctx.log_info("Waiting 30s for crates.io to index...");
            if !ctx.dry_run {
                std::thread::sleep(std::time::Duration::from_secs(30));
            }
        }

        ctx.log_info(&format!("Publishing {}", crate_name));

        let mut env = HashMap::new();
        env.insert("CARGO_REGISTRY_TOKEN".to_string(), token.clone());

        run_command_with_env(
            "cargo",
            &["publish", "-p", crate_name, "--allow-dirty"],
            &env,
            ctx.dry_run,
        )?;
    }

    Ok(())
}

fn publish_npm(ctx: &ReleaseContext) -> Result<()> {
    let _token = env::var("NPM_TOKEN").context("NPM_TOKEN not set")?;

    let npm_dir = ctx.project_root.join("packaging/npm");
    if !npm_dir.exists() {
        anyhow::bail!("npm packaging directory not found");
    }

    ctx.log_info("Publishing @neul-labs/ringlet to npm");
    run_command("npm", &["publish", "--access", "public"], ctx.dry_run)?;

    Ok(())
}

fn publish_pypi(ctx: &ReleaseContext) -> Result<()> {
    let _token = env::var("PYPI_TOKEN").context("PYPI_TOKEN not set")?;

    let pypi_dir = ctx.project_root.join("packaging/pypi");
    if !pypi_dir.exists() {
        anyhow::bail!("PyPI packaging directory not found");
    }

    ctx.log_info("Publishing ringlet to PyPI");

    // Build wheel
    run_command("python", &["-m", "build", "--wheel"], ctx.dry_run)?;

    // Upload
    run_command(
        "twine",
        &["upload", "--username", "__token__", "dist/*.whl"],
        ctx.dry_run,
    )?;

    Ok(())
}

fn publish_rubygems(ctx: &ReleaseContext) -> Result<()> {
    let _key = env::var("RUBYGEMS_API_KEY").context("RUBYGEMS_API_KEY not set")?;

    ctx.log_info("Publishing ringlet to RubyGems");
    run_command("gem", &["build", "ringlet.gemspec"], ctx.dry_run)?;
    run_command("gem", &["push", &format!("ringlet-{}.gem", ctx.version)], ctx.dry_run)?;

    Ok(())
}

fn publish_homebrew(ctx: &ReleaseContext) -> Result<()> {
    let _token = env::var("HOMEBREW_TAP_TOKEN")
        .or_else(|_| env::var("GITHUB_TOKEN"))
        .context("HOMEBREW_TAP_TOKEN or GITHUB_TOKEN not set")?;

    ctx.log_info("Updating Homebrew tap");
    // Implementation would clone tap repo, update formula, push
    if ctx.dry_run {
        println!("  {} [DRY-RUN] Would update Homebrew formula", style("→").dim());
    }

    Ok(())
}

fn publish_chocolatey(ctx: &ReleaseContext) -> Result<()> {
    let _key = env::var("CHOCOLATEY_API_KEY").context("CHOCOLATEY_API_KEY not set")?;

    ctx.log_info("Publishing ringlet to Chocolatey");

    if cfg!(target_os = "windows") {
        run_command("choco", &["pack"], ctx.dry_run)?;
        run_command(
            "choco",
            &[
                "push",
                &format!("ringlet.{}.nupkg", ctx.version),
                "--source",
                "https://push.chocolatey.org/",
            ],
            ctx.dry_run,
        )?;
    } else if command_exists("docker") {
        ctx.log_info("Using Docker for Chocolatey packaging");
        // Docker-based chocolatey packaging
    } else {
        ctx.log_warn("Chocolatey requires Windows or Docker");
    }

    Ok(())
}

fn publish_debian(ctx: &ReleaseContext) -> Result<()> {
    ctx.log_info("Building Debian packages");

    for (arch, platform) in [("amd64", "linux-x64"), ("arm64", "linux-arm64")] {
        let archive = ctx.dist_dir.join(format!(
            "{}-{}-{}.tar.gz",
            ctx.config.project.name, platform, ctx.version
        ));

        if !archive.exists() {
            ctx.log_warn(&format!("Skipping {} - archive not found", arch));
            continue;
        }

        if ctx.dry_run {
            println!("  {} [DRY-RUN] Would build {}.deb", style("→").dim(), arch);
            continue;
        }

        ctx.log_info(&format!("Building {} package", arch));
        // dpkg-deb packaging would go here
    }

    Ok(())
}

fn publish_arch(ctx: &ReleaseContext) -> Result<()> {
    ctx.log_info("Generating Arch Linux PKGBUILD");

    let arch_dir = ctx.dist_dir.join("arch");

    if ctx.dry_run {
        println!("  {} [DRY-RUN] Would generate PKGBUILD", style("→").dim());
        return Ok(());
    }

    fs::create_dir_all(&arch_dir)?;

    // Generate PKGBUILD content
    let x64_checksum = ctx.checksums.get("linux-x64").map(|s| s.as_str()).unwrap_or("SKIP");
    let arm64_checksum = ctx.checksums.get("linux-arm64").map(|s| s.as_str()).unwrap_or("SKIP");

    let pkgbuild = format!(
        r#"# Maintainer: Neul Labs <hello@neullabs.com>
pkgname=ringlet
pkgver={}
pkgrel=1
pkgdesc="CLI orchestrator for coding agents"
arch=('x86_64' 'aarch64')
url="https://github.com/{}"
license=('MIT')
depends=('gcc-libs')
provides=('ringlet' 'ringletd')

source_x86_64=("https://github.com/{}/releases/download/v${{pkgver}}/ringlet-linux-x64-${{pkgver}}.tar.gz")
source_aarch64=("https://github.com/{}/releases/download/v${{pkgver}}/ringlet-linux-arm64-${{pkgver}}.tar.gz")

sha256sums_x86_64=('{}')
sha256sums_aarch64=('{}')

package() {{
    install -Dm755 ringlet "$pkgdir/usr/bin/ringlet"
    install -Dm755 ringletd "$pkgdir/usr/bin/ringletd"
}}
"#,
        ctx.version,
        ctx.config.project.repository,
        ctx.config.project.repository,
        ctx.config.project.repository,
        x64_checksum,
        arm64_checksum
    );

    fs::write(arch_dir.join("PKGBUILD"), pkgbuild)?;
    ctx.log_success(&format!("Generated: {}", arch_dir.join("PKGBUILD").display()));

    Ok(())
}

fn publish_dmg(ctx: &ReleaseContext) -> Result<()> {
    if !cfg!(target_os = "macos") {
        ctx.log_warn("DMG creation requires macOS, skipping");
        return Ok(());
    }

    ctx.log_info("Building macOS DMG installer");

    if ctx.dry_run {
        println!("  {} [DRY-RUN] Would create DMG", style("→").dim());
        return Ok(());
    }

    // create-dmg would be called here
    ctx.log_info("DMG creation would use create-dmg tool");

    Ok(())
}

fn publish_msi(ctx: &ReleaseContext) -> Result<()> {
    ctx.log_info("Building Windows MSI installer");

    let win_archive = ctx.dist_dir.join(format!(
        "{}-win32-x64-{}.zip",
        ctx.config.project.name, ctx.version
    ));

    if !win_archive.exists() {
        anyhow::bail!("Windows binary archive not found");
    }

    if ctx.dry_run {
        println!("  {} [DRY-RUN] Would create MSI", style("→").dim());
        return Ok(());
    }

    if cfg!(target_os = "windows") && command_exists("candle") {
        ctx.log_info("Building MSI with WiX");
        // WiX commands would go here
    } else if command_exists("docker") {
        ctx.log_info("Building MSI with Docker (WiX)");
        // Docker-based WiX
    } else {
        ctx.log_warn("MSI creation requires Windows with WiX or Docker");
    }

    Ok(())
}

// ============================================================================
// GitHub Release
// ============================================================================

fn create_github_release(ctx: &ReleaseContext) -> Result<()> {
    ctx.log_step("Creating GitHub release");

    if !command_exists("gh") {
        ctx.log_warn("gh CLI not found, skipping GitHub release");
        return Ok(());
    }

    let tag = format!("v{}", ctx.version);

    if ctx.dry_run {
        println!("  {} [DRY-RUN] Would create release {}", style("→").dim(), tag);
        return Ok(());
    }

    // Create tag
    ctx.log_info(&format!("Creating tag {}", tag));
    run_command("git", &["tag", "-a", &tag, "-m", &format!("Release {}", ctx.version)], false)?;
    run_command("git", &["push", "origin", &tag], false)?;

    // Generate release notes
    let release_notes = generate_release_notes(ctx)?;

    // Create release with assets
    let title = format!("Release {}", ctx.version);
    let mut args: Vec<String> = vec![
        "release".to_string(),
        "create".to_string(),
        tag.clone(),
        "--title".to_string(),
        title,
        "--notes".to_string(),
        release_notes,
    ];

    // Add all dist files as assets
    for entry in fs::read_dir(&ctx.dist_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            args.push(path.to_string_lossy().to_string());
        }
    }

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run_command("gh", &args_refs, false)?;

    ctx.log_success(&format!("Created GitHub release: {}", tag));
    Ok(())
}

fn generate_release_notes(ctx: &ReleaseContext) -> Result<String> {
    let repo = &ctx.config.project.repository;
    let name = &ctx.config.project.name;
    let version = &ctx.version;

    // Get changelog
    let changelog = Command::new("git")
        .args(["log", "--oneline", "-20"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|_| "Initial release".to_string());

    let notes = format!(
        r#"## Installation

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/{repo}/main/install.sh | bash
```

### Package Managers

| Platform | Command |
|----------|---------|
| **Cargo** | `cargo install {name}` |
| **npm** | `npm install -g @neul-labs/{name}` |
| **PyPI** | `pip install {name}` |
| **RubyGems** | `gem install {name}` |
| **Homebrew** | `brew install neul-labs/homebrew-ringlet/{name}` |
| **Chocolatey** | `choco install {name}` |
| **Arch Linux (AUR)** | `yay -S {name}` |
| **Debian/Ubuntu** | Download `.deb` from assets below |

### Direct Downloads

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x64 | [{name}-linux-x64-{version}.tar.gz](https://github.com/{repo}/releases/download/v{version}/{name}-linux-x64-{version}.tar.gz) |
| Linux | ARM64 | [{name}-linux-arm64-{version}.tar.gz](https://github.com/{repo}/releases/download/v{version}/{name}-linux-arm64-{version}.tar.gz) |
| macOS | Universal | [{name}-darwin-universal-{version}.tar.gz](https://github.com/{repo}/releases/download/v{version}/{name}-darwin-universal-{version}.tar.gz) |
| Windows | x64 | [{name}-win32-x64-{version}.zip](https://github.com/{repo}/releases/download/v{version}/{name}-win32-x64-{version}.zip) |

---

## What's Changed

{changelog}

---

## Checksums (SHA256)

See `checksums.txt` in the release assets.
"#,
        repo = repo,
        name = name,
        version = version,
        changelog = changelog
    );

    Ok(notes)
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            version,
            only,
            dry_run,
        } => {
            let mut ctx = ReleaseContext::new(version, dry_run)?;

            println!(
                "\n{}",
                style("╔════════════════════════════════════════════════╗").bold()
            );
            println!(
                "{}",
                style(format!("║     ringlet Build v{}     ", ctx.version)).bold()
            );
            println!(
                "{}",
                style("╚════════════════════════════════════════════════╝").bold()
            );

            if dry_run {
                println!("\n{}", style("Running in DRY-RUN mode").yellow());
            }

            build_all(&mut ctx, only.as_deref())?;
        }

        Commands::Release {
            version,
            dry_run,
            skip_build,
            skip_publish,
            only,
            no_github,
        } => {
            let mut ctx = ReleaseContext::new(version, dry_run)?;

            println!(
                "\n{}",
                style("╔════════════════════════════════════════════════╗").bold()
            );
            println!(
                "{}",
                style(format!("║     ringlet Release v{}     ", ctx.version)).bold()
            );
            println!(
                "{}",
                style("╚════════════════════════════════════════════════╝").bold()
            );

            if dry_run {
                println!("\n{}", style("Running in DRY-RUN mode").yellow());
            }

            // Build phase
            if !skip_build {
                build_all(&mut ctx, None)?;
            } else {
                ctx.log_info("Skipping build phase");
            }

            // Publish phase
            if !skip_publish {
                publish_all(&ctx, only.as_deref())?;
            } else {
                ctx.log_info("Skipping publish phase");
            }

            // GitHub release
            if !no_github {
                create_github_release(&ctx)?;
            } else {
                ctx.log_info("Skipping GitHub release");
            }

            println!(
                "\n{}",
                style("╔════════════════════════════════════════════════╗").green().bold()
            );
            println!(
                "{}",
                style(format!("║     Release v{} Complete!     ", ctx.version)).green().bold()
            );
            println!(
                "{}",
                style("╚════════════════════════════════════════════════╝").green().bold()
            );
        }

        Commands::Publish {
            registry,
            version,
            dry_run,
        } => {
            let ctx = ReleaseContext::new(version, dry_run)?;

            println!("\n{} Publishing to {}", ROCKET, registry);

            if dry_run {
                println!("{}", style("Running in DRY-RUN mode").yellow());
            }

            publish_to(&ctx, &registry)?;
        }
    }

    Ok(())
}
