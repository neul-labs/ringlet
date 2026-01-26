$ErrorActionPreference = 'Stop'

Uninstall-BinFile -Name 'ringlet'
Uninstall-BinFile -Name 'ringletd'

Write-Host "ringlet has been uninstalled."
