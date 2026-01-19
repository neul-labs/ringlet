"""clown - CLI orchestrator for coding agents."""

import os
import platform
import subprocess
import sys
import urllib.request
import tarfile
import zipfile
import tempfile
from pathlib import Path

__version__ = "0.1.0"

GITHUB_REPO = "neul-labs/ccswitch"


def get_platform_suffix():
    """Get platform-specific artifact suffix."""
    system = platform.system().lower()
    machine = platform.machine().lower()

    if system == "darwin":
        arch = "arm64" if machine in ("arm64", "aarch64") else "x64"
        return f"darwin-{arch}"
    elif system == "linux":
        arch = "arm64" if machine in ("arm64", "aarch64") else "x64"
        return f"linux-{arch}"
    elif system == "windows":
        return "win32-x64"
    else:
        raise RuntimeError(f"Unsupported platform: {system} {machine}")


def get_binary_dir():
    """Get the directory for storing binaries."""
    return Path(__file__).parent / "bin"


def ensure_binary(name: str) -> Path:
    """Ensure the binary exists, downloading if necessary."""
    binary_dir = get_binary_dir()
    ext = ".exe" if platform.system() == "Windows" else ""
    binary_path = binary_dir / f"{name}{ext}"

    if binary_path.exists():
        return binary_path

    # Download from GitHub releases
    suffix = get_platform_suffix()
    ext_archive = "zip" if platform.system() == "Windows" else "tar.gz"
    url = f"https://github.com/{GITHUB_REPO}/releases/download/v{__version__}/clown-{suffix}-{__version__}.{ext_archive}"

    binary_dir.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory() as tmpdir:
        archive_path = Path(tmpdir) / f"clown.{ext_archive}"

        print(f"Downloading clown v{__version__}...", file=sys.stderr)
        urllib.request.urlretrieve(url, archive_path)

        if ext_archive == "zip":
            with zipfile.ZipFile(archive_path, 'r') as zf:
                zf.extractall(binary_dir)
        else:
            with tarfile.open(archive_path, 'r:gz') as tf:
                tf.extractall(binary_dir)

    # Make executable on Unix
    if platform.system() != "Windows":
        os.chmod(binary_path, 0o755)

    return binary_path


def main():
    """Entry point for clown CLI."""
    binary = ensure_binary("clown")
    sys.exit(subprocess.call([str(binary)] + sys.argv[1:]))


def main_daemon():
    """Entry point for clownd daemon."""
    binary = ensure_binary("clownd")
    sys.exit(subprocess.call([str(binary)] + sys.argv[1:]))
