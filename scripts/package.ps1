param(
    [Parameter(Mandatory = $true)][string]$Description,
    [Parameter(Mandatory = $true)][string]$Output,
    [string]$Name,
    [ValidatePattern('^[a-zA-Z0-9_-]+$')][string]$Runtime = 'desktop-app-engine',
    [string]$InstructionsTemplate
)
$ErrorActionPreference = 'Stop'
try {
    $root = Split-Path -Parent $PSScriptRoot
    $source = (Resolve-Path -LiteralPath $Description).Path
    $document = Get-Content -LiteralPath $source -Raw -Encoding UTF8 | ConvertFrom-Json
    if (!$PSBoundParameters.ContainsKey('Name')) { $Name = [string]$document.app.name }
    if ([string]::IsNullOrWhiteSpace($Name) -or
        $Name.IndexOfAny([IO.Path]::GetInvalidFileNameChars()) -ge 0 -or
        $Name -match '[ .]$|^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(\.|$)' -or
        $Name -in @('.', '..')) {
        throw '应用名不是有效的 Windows 文件名，请通过 -Name 指定。'
    }
    $target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $root 'target' }
    if (![IO.Path]::IsPathRooted($target)) { $target = Join-Path $root $target }
    $runtimePath = Join-Path $target "release/$Runtime.exe"
    $outputRoot = [IO.Path]::GetFullPath($Output)
    $destination = Join-Path $outputRoot $Name
    if (Test-Path -LiteralPath $destination) { throw "应用目录已存在：$destination" }
    $previousDescription = $env:DAE_APP_DESCRIPTION
    $previousTarget = $env:CARGO_TARGET_DIR
    try {
        $env:DAE_APP_DESCRIPTION = $source
        $env:CARGO_TARGET_DIR = $target
        & cargo build --manifest-path (Join-Path $root 'Cargo.toml') --release --bin $Runtime --locked --offline
        if ($LASTEXITCODE -ne 0) { throw "应用编译失败（退出码 $LASTEXITCODE）" }
    } finally {
        $env:DAE_APP_DESCRIPTION = $previousDescription
        $env:CARGO_TARGET_DIR = $previousTarget
    }
    $null = New-Item -ItemType Directory -Path $destination -Force
    $executable = Join-Path $destination "$Name.exe"
    Copy-Item -LiteralPath $runtimePath -Destination $executable
    if ($document.plugins.Count -gt 0) {
        $pluginBase = Join-Path (Split-Path -Parent $source) 'plugins'
        $null = New-Item -ItemType Directory -Path (Join-Path $destination 'plugins')
        foreach ($plugin in $document.plugins) {
            if ($plugin -notmatch '^[a-zA-Z0-9_-]+$') { throw 'Invalid plugin name' }
            Copy-Item -LiteralPath (Join-Path $pluginBase $plugin) -Destination (Join-Path $destination "plugins/$plugin") -Recurse
        }
        Copy-Item -LiteralPath (Join-Path $pluginBase 'runtime') -Destination (Join-Path $destination 'plugins/runtime') -Recurse
    }
    $instructions = @"
$Name

双击 $Name.exe 启动应用。应用定义已编译进 EXE，无需外置应用描述 JSON。
客户机无需 Rust、源码或制作工具；需要 Windows 中文系统字体和 OpenGL 图形驱动。
更新界面或行为时，请修改制作端源描述并重新打包。
校验命令：& '.\$Name.exe' --check
打包应用不加载同目录 app.json，也不接受 --app 覆盖定义。校验成功退出 0，失败退出 2。
可使用 --theme system、--theme light 或 --theme dark 指定主题。
运行日志自动保存到 %LOCALAPPDATA%\DesktopAppEngine\logs\ 下按 EXE 名区分的应用目录，每次启动独立会话。
底部“日志与任务”查看运行中的输出，“打开日志目录”查看历史；每个任务目录含 stdout.log、stderr.log、task.json 和 result.json。
窗口仅预览每条流最近 32 KiB，完整输出按原始字节写入文件。日志不自动清理，可关闭应用后按需删除历史会话。
使用 credentialStore 的应用需在客户机当前 Windows 用户下另行录入凭据；凭据密文位于用户本地目录，不随应用包复制。
凭据可用性检查：& '.\$Name.exe' --check-credentials（只检查能否解密，不发送接口请求）。
"@
    if ($InstructionsTemplate) {
        $instructions = (Get-Content -LiteralPath (Resolve-Path -LiteralPath $InstructionsTemplate).Path -Raw -Encoding UTF8).Replace('{{Name}}', $Name)
    }
    Set-Content -LiteralPath (Join-Path $destination 'README.txt') -Value $instructions -Encoding UTF8
    $checkLog = Join-Path ([IO.Path]::GetTempPath()) ([IO.Path]::GetRandomFileName())
    $checkArgument = if ($document.plugins.Count -gt 0) { '--check-plugins' } else { '--check' }
    $process = Start-Process -FilePath $executable -ArgumentList $checkArgument -WorkingDirectory $outputRoot -RedirectStandardOutput "$checkLog.out" -RedirectStandardError "$checkLog.err" -PassThru
    if (!$process.WaitForExit(30000)) {
        $process.Kill()
        throw "应用包冒烟校验超时：$destination"
    }
    Get-Content -LiteralPath "$checkLog.out", "$checkLog.err" -Encoding UTF8
    if ($process.ExitCode -ne 0) { throw "应用包冒烟校验失败，退出码 $($process.ExitCode)：$destination" }
    "应用包校验通过（退出码 0）：$destination"
    exit 0
} catch {
    [Console]::Error.WriteLine("打包失败：$($_.Exception.Message)")
    exit 2
}
