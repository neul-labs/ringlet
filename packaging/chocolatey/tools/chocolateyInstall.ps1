$ErrorActionPreference = 'Stop'

$packageName = 'ringlet'
$version = '0.1.0'
$url = "https://github.com/neul-labs/ccswitch/releases/download/v$version/ringlet-win32-x64-$version.zip"

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
$daemonPath = Join-Path $installDir 'ringletd.exe'

Install-BinFile -Name 'ringlet' -Path $ringletPath
Install-BinFile -Name 'ringletd' -Path $daemonPath

Write-Host "ringlet has been installed. Run 'ringlet --help' to get started."
