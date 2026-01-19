$ErrorActionPreference = 'Stop'

Uninstall-BinFile -Name 'clown'
Uninstall-BinFile -Name 'clownd'

Write-Host "clown has been uninstalled."
