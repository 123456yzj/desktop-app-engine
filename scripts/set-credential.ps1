param(
    [Parameter(Mandatory=$true)][string]$Store,
    [Parameter(Mandatory=$true)][string]$Name,
    [string]$Runtime = (Join-Path (Split-Path -Parent $PSScriptRoot) 'target-http/release/desktop-app-engine.exe')
)
$ErrorActionPreference = 'Stop'
if ($Store -notmatch '^[a-zA-Z0-9_-]{1,64}$' -or $Name -notmatch '^[a-zA-Z0-9_-]{1,64}$') { throw '凭据库与名称只允许字母数字、-、_，长度 1..64' }
$executable = (Resolve-Path -LiteralPath $Runtime).Path
$secret = Read-Host "输入 $Store/$Name 的凭据（隐藏输入）" -AsSecureString
$pointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secret)
$process = [Diagnostics.Process]::new()
try {
    $process.StartInfo.FileName = $executable
    $process.StartInfo.UseShellExecute = $false
    $process.StartInfo.CreateNoWindow = $true
    $process.StartInfo.RedirectStandardInput = $true
    $process.StartInfo.StandardInputEncoding = [Text.UTF8Encoding]::new($false)
    foreach ($argument in @('--credential-store',$Store,'--credential-set',$Name)) { $process.StartInfo.ArgumentList.Add($argument) }
    $null = $process.Start()
    $plain = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($pointer)
    $process.StandardInput.Write($plain)
    $process.StandardInput.Close()
    $plain = $null
    if (!$process.WaitForExit(30000)) { $process.Kill(); throw '保存凭据超时' }
    if ($process.ExitCode -ne 0) { throw "保存凭据失败，退出码 $($process.ExitCode)" }
} finally {
    [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($pointer)
    $secret.Dispose()
    $process.Dispose()
}
