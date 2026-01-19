#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const https = require('https');

const PLATFORMS = {
  'linux-x64': '@clown-cli/linux-x64',
  'linux-arm64': '@clown-cli/linux-arm64',
  'darwin-x64': '@clown-cli/darwin-x64',
  'darwin-arm64': '@clown-cli/darwin-arm64',
  'win32-x64': '@clown-cli/win32-x64'
};

function getPlatformKey() {
  const platform = process.platform;
  const arch = process.arch === 'arm64' ? 'arm64' : 'x64';
  return `${platform}-${arch}`;
}

function getPlatformPackage() {
  return PLATFORMS[getPlatformKey()];
}

function findBinary(packageName, binaryName) {
  try {
    const packagePath = require.resolve(`${packageName}/package.json`);
    const packageDir = path.dirname(packagePath);
    const ext = process.platform === 'win32' ? '.exe' : '';
    return path.join(packageDir, 'bin', `${binaryName}${ext}`);
  } catch (e) {
    return null;
  }
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const follow = (url, redirects = 0) => {
      if (redirects > 5) {
        reject(new Error('Too many redirects'));
        return;
      }

      https.get(url, (response) => {
        if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
          follow(response.headers.location, redirects + 1);
          return;
        }

        if (response.statusCode !== 200) {
          reject(new Error(`Download failed with status ${response.statusCode}`));
          return;
        }

        const file = fs.createWriteStream(dest);
        response.pipe(file);
        file.on('finish', () => {
          file.close();
          resolve();
        });
        file.on('error', (err) => {
          fs.unlink(dest, () => {});
          reject(err);
        });
      }).on('error', reject);
    };

    follow(url);
  });
}

async function installFromGitHub() {
  console.log('Platform package not found, downloading from GitHub releases...');

  const pkg = require('./package.json');
  const version = pkg.version;
  const platformKey = getPlatformKey();

  const platformMap = {
    'linux-x64': 'linux-x64',
    'linux-arm64': 'linux-arm64',
    'darwin-x64': 'darwin-x64',
    'darwin-arm64': 'darwin-arm64',
    'win32-x64': 'win32-x64'
  };

  const artifactName = platformMap[platformKey];

  if (!artifactName) {
    console.error(`Unsupported platform: ${platformKey}`);
    process.exit(1);
  }

  const ext = process.platform === 'win32' ? 'zip' : 'tar.gz';
  const url = `https://github.com/neul-labs/ccswitch/releases/download/v${version}/clown-${artifactName}-${version}.${ext}`;

  const binDir = path.join(__dirname, 'bin');
  const tmpDir = path.join(__dirname, 'tmp');

  fs.mkdirSync(binDir, { recursive: true });
  fs.mkdirSync(tmpDir, { recursive: true });

  const archivePath = path.join(tmpDir, `archive.${ext}`);

  console.log(`Downloading from ${url}...`);

  try {
    await downloadFile(url, archivePath);

    console.log('Extracting...');

    if (process.platform === 'win32') {
      // Use PowerShell to extract zip
      execSync(`powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${binDir}' -Force"`, { stdio: 'inherit' });
    } else {
      execSync(`tar -xzf "${archivePath}" -C "${binDir}"`, { stdio: 'inherit' });
      // Make binaries executable
      fs.chmodSync(path.join(binDir, 'clown'), 0o755);
      fs.chmodSync(path.join(binDir, 'clownd'), 0o755);
    }

    console.log('clown binaries installed successfully');
  } finally {
    // Cleanup
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
}

async function main() {
  const platformPackage = getPlatformPackage();
  const binDir = path.join(__dirname, 'bin');

  // Try to find platform-specific package
  const clownPath = findBinary(platformPackage, 'clown');
  const clowndPath = findBinary(platformPackage, 'clownd');

  if (clownPath && clowndPath && fs.existsSync(clownPath) && fs.existsSync(clowndPath)) {
    // Create symlinks/copies in bin directory
    fs.mkdirSync(binDir, { recursive: true });

    const ext = process.platform === 'win32' ? '.exe' : '';
    const clownDest = path.join(binDir, `clown${ext}`);
    const clowndDest = path.join(binDir, `clownd${ext}`);

    fs.copyFileSync(clownPath, clownDest);
    fs.copyFileSync(clowndPath, clowndDest);

    if (process.platform !== 'win32') {
      fs.chmodSync(clownDest, 0o755);
      fs.chmodSync(clowndDest, 0o755);
    }

    console.log('clown binaries installed successfully');
  } else {
    // Fallback to GitHub release download
    await installFromGitHub();
  }
}

main().catch((err) => {
  console.error('Installation failed:', err.message);
  process.exit(1);
});
