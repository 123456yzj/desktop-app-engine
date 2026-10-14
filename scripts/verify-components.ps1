param([string]$ToolchainRoot = 'D:\project\impact\app-v1\desktop-app-engine\.tools')
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
if (!(Test-Path -LiteralPath (Join-Path $root 'evidence')) -or !(Test-Path -LiteralPath (Join-Path $root 'tmp-verify'))) {throw 'Expected writable verification directories missing'}
$env:RUSTUP_HOME = Join-Path $ToolchainRoot 'rustup'
$env:CARGO_HOME = Join-Path $ToolchainRoot 'cargo'
$env:PATH = "$(Join-Path $ToolchainRoot 'cargo/bin');$(Join-Path $ToolchainRoot 'mingw64/bin');$env:PATH"
$env:CARGO_TARGET_DIR = Join-Path $root 'target-verify'
$env:TMP = Join-Path $root 'tmp-verify'
$env:TEMP = $env:TMP
Start-Transcript -Path (Join-Path $root 'evidence/component-verification.txt') -Append -Force
try {
 function Check-Command([string]$label,[scriptblock]$command) {
  "COMMAND $label"
  & $command
  $code=$LASTEXITCODE
  "EXIT $code"
  if ($code -ne 0) {throw "Failed: $label ($code)"}
 }
 Check-Command 'cargo fmt -- --check' {cargo fmt -- --check}
 Check-Command 'cargo clippy --all-targets --locked --offline -- -D warnings' {cargo clippy --all-targets --locked --offline -- -D warnings}
 Check-Command 'cargo test --locked --offline' {cargo test --locked --offline}
 Check-Command 'cargo build --release --locked --offline' {cargo build --release --locked --offline}
 function Check-App([string]$file) {
  $process=Start-Process -FilePath (Join-Path $env:CARGO_TARGET_DIR 'release/desktop-app-engine.exe') -ArgumentList '--app',"`"$(Join-Path $root $file)`"",'--check' -Wait -NoNewWindow -PassThru
  $global:LASTEXITCODE=$process.ExitCode
 }
 Check-Command 'hello.json --check' {Check-App 'examples/hello.json'}
 Check-Command 'dashboard.json --check' {Check-App 'examples/dashboard.json'}
 Check-Command 'python scripts/schema-verify.py' {python (Join-Path $PSScriptRoot 'schema-verify.py')}
 Check-Command 'cargo test --release --locked --offline ten_thousand_rows_use_actual_visible_paint_path -- --nocapture' {cargo test --release --locked --offline ten_thousand_rows_use_actual_visible_paint_path -- --nocapture}
 Check-Command 'cargo test --release --locked --offline sampling_preserves_spikes_order_and_gaps -- --nocapture' {cargo test --release --locked --offline sampling_preserves_spikes_order_and_gaps -- --nocapture}
 Get-CimInstance Win32_Processor | Format-Table Name,NumberOfCores,NumberOfLogicalProcessors
} finally {Stop-Transcript}
