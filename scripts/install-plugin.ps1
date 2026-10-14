param(
    [Parameter(Mandatory=$true)][string]$Package,
    [Parameter(Mandatory=$true)][string]$AppDirectory,
    [switch]$InstallDependencies
)
$ErrorActionPreference = 'Stop'
$source = (Resolve-Path -LiteralPath $Package).Path
$base = (Resolve-Path -LiteralPath $AppDirectory).Path
$manifest = Get-Content -LiteralPath (Join-Path $source 'plugin.json') -Raw | ConvertFrom-Json
if ($manifest.apiVersion -ne 1 -or $manifest.name -notmatch '^[a-zA-Z0-9_-]+$') { throw 'Invalid plugin manifest' }
$destination = Join-Path $base "plugins/$($manifest.name)"
if (Test-Path -LiteralPath $destination) { throw "Plugin already installed: $destination" }
if ($InstallDependencies) {
    $npm = (Get-Command npm.cmd -ErrorAction Stop).Source
    $operation = if (Test-Path -LiteralPath (Join-Path $source 'package-lock.json')) { 'ci' } else { 'install' }
    $p = Start-Process -FilePath $npm -ArgumentList @($operation,'--omit=dev','--ignore-scripts') -WorkingDirectory $source -Wait -PassThru -NoNewWindow
    if ($p.ExitCode -ne 0) { throw 'npm install failed' }
}
$null = New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force
Copy-Item -LiteralPath $source -Destination $destination -Recurse
$runtime = Join-Path $base 'plugins/runtime'
$null = New-Item -ItemType Directory -Path $runtime -Force
$node = (Get-Command node.exe -ErrorAction Stop).Source
Copy-Item -LiteralPath $node -Destination (Join-Path $runtime 'node.exe')
$license = Join-Path (Split-Path -Parent $node) 'LICENSE'
if (Test-Path -LiteralPath $license) { Copy-Item -LiteralPath $license -Destination (Join-Path $runtime 'NODE-LICENSE.txt') }
'Installed plugin and Node runtime: ' + $destination
