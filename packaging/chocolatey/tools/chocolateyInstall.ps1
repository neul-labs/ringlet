$ErrorActionPreference = 'Stop'

$packageName = 'ringlet'
$version = '0.1.0'
$url = "https://github.com/neul-labs/ringlet/releases/download/v$version/ringlet-win32-x64-$version.zip"

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$installDir = Join-Path $toolsDir 'bin'

# Download and extract
$packageArgs = @{
    packageName   = $packageName
    unzipLocation = $installDir
    url           = $url
    checksum      = '{{CHECKSUM}}'
    checksumType  = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs

# Add binaries to PATH
$ringletPath = Join-Path $installDir 'ringlet.exe'
# ringletd.exe points to the same binary for backward compatibility
Install-BinFile -Name 'ringlet' -Path $ringletPath
Install-BinFile -Name 'ringletd' -Path $ringletPath

Write-Host "ringlet has been installed. Run 'ringlet --help' to get started."
