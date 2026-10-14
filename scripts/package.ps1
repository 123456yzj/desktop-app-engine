param(
    [Parameter(Mandatory = $true)][string]$Description,
    [Parameter(Mandatory = $true)][string]$Output,
    [string]$Name
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
    $runtime = Join-Path $target 'release/desktop-app-engine.exe'
    if (!(Test-Path -LiteralPath $runtime -PathType Leaf)) {
        throw "未找到 release 运行时，请先执行 cargo build --release --locked --offline：$runtime"
    }
    $outputRoot = [IO.Path]::GetFullPath($Output)
    $destination = Join-Path $outputRoot $Name
    if (Test-Path -LiteralPath $destination) { throw "应用目录已存在：$destination" }
    $null = New-Item -ItemType Directory -Path $destination -Force
    $executable = Join-Path $destination "$Name.exe"
    Copy-Item -LiteralPath $runtime -Destination $executable
    Copy-Item -LiteralPath $source -Destination (Join-Path $destination 'app.json')
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

双击 $Name.exe 启动应用。请保留同目录的 app.json，并分发整个目录。
客户机无需 Rust、源码或制作工具；需要 Windows 中文系统字体和 OpenGL 图形驱动。
修改 app.json 即可更新界面与交互，无需重新编译运行时。
校验命令：& '.\$Name.exe' --check
加载顺序：--app FILE > EXE 同目录 app.json > 内嵌开发示例。
外部描述错误不会回退。校验成功退出 0，失败退出 2。
可使用 --theme system、--theme light 或 --theme dark 指定主题。
使用 credentialStore 的应用需在客户机当前 Windows 用户下另行录入凭据；凭据密文位于用户本地目录，不随应用包复制。
凭据可用性检查：& '.\$Name.exe' --check-credentials（只检查能否解密，不发送接口请求）。
"@
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
