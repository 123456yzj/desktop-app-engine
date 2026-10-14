param([string]$Name = 'DashboardStandaloneRelease', [string]$PackageName = 'DashboardAppRelease')
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$parent = Join-Path $root 'tmp-verify'
$package = Join-Path $root "exports/$PackageName"
if (!(Test-Path -LiteralPath $parent) -or !(Test-Path -LiteralPath $package)) {throw 'Verification parent/package missing'}
$destination = Join-Path $parent $Name
if (Test-Path -LiteralPath $destination) {throw "Standalone destination already exists: $destination"}
$null = New-Item -ItemType Directory -Path $destination
foreach ($file in @("$PackageName.exe",'app.json','README.txt')) {Copy-Item -LiteralPath (Join-Path $package $file) -Destination $destination}
$files = @(Get-ChildItem -LiteralPath $destination)
if ($files.Count -ne 3) {throw 'Standalone package must contain only runtime, JSON and README'}
$log = Join-Path $parent 'standalone-check'
$process = Start-Process -FilePath (Join-Path $destination "$PackageName.exe") -ArgumentList '--check' -WorkingDirectory $parent -RedirectStandardOutput "$log.out" -RedirectStandardError "$log.err" -Wait -PassThru
Get-Content -LiteralPath "$log.out","$log.err" -Encoding UTF8
"STANDALONE_EXIT=$($process.ExitCode)"
if ($process.ExitCode -ne 0) {throw 'Standalone check failed'}
"STANDALONE_DIRECTORY=$destination"
$files | Format-Table Name,Length
