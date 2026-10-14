param([string]$Output = 'exports', [string]$Name = 'PluginChartApp')
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$parent = Join-Path $root 'tmp-verify'
if (!(Test-Path -LiteralPath $parent -PathType Container)) { throw 'tmp-verify directory is required' }
$fixture = Join-Path $parent ('plugin-app-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $fixture
Copy-Item -LiteralPath (Join-Path $root 'examples/plugin-chart.json') -Destination (Join-Path $fixture 'app.json')
& (Join-Path $PSScriptRoot 'install-plugin.ps1') -Package (Join-Path $root 'plugins/echarts') -AppDirectory $fixture -InstallDependencies
& (Join-Path $PSScriptRoot 'package.ps1') -Description (Join-Path $fixture 'app.json') -Output $Output -Name $Name
if ($LASTEXITCODE -ne 0) { throw 'Plugin package verification failed' }
