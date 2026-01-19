$ErrorActionPreference = 'Stop'

$packageName = 'clown'
$version = '0.1.0'
$url = "https://github.com/neul-labs/ccswitch/releases/download/v$version/clown-win32-x64-$version.zip"

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
$clownPath = Join-Path $installDir 'clown.exe'
$daemonPath = Join-Path $installDir 'clownd.exe'

Install-BinFile -Name 'clown' -Path $clownPath
Install-BinFile -Name 'clownd' -Path $daemonPath

Write-Host "clown has been installed. Run 'clown --help' to get started."
