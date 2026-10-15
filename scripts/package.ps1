param(
    [Parameter(Mandatory = $true)][string]$Description,
    [Parameter(Mandatory = $true)][string]$Output,
    [string]$Name,
    [ValidateSet('desktop-app-engine', 'vben-package')][string]$Runtime
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
    if (!$PSBoundParameters.ContainsKey('Runtime')) {
        $Runtime = if ($document.app.name -eq 'vben-package') { 'vben-package' } else { 'desktop-app-engine' }
    }
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
    if ($Runtime -eq 'vben-package') {
        $instructions = @"
$Name

双击 $Name.exe 启动项目构建工作台，应用定义已编译进 EXE，无需外置应用描述 JSON。
在连接设置中填写 SSH 地址或本机 SSH 配置别名，以及 Turbo 工作区绝对路径，保存后扫描 apps 下的独立 Git 仓库。
连接设置保存到当前用户 %LOCALAPPDATA%\DesktopAppEngine\apps\$Name\connection.dat，与应用定义分开保存。
本机需 Windows OpenSSH 客户端及可用的免交互 SSH 认证；服务器需 Bash、Node.js、Git 和 pnpm。
启动时、保存连接后和重新扫描时自动 Fetch 各仓库分支；每行 Fetch 可单独刷新，展开下拉只读取缓存，最多同时展示 10 项。
顶部连接栏集中显示 SSH 服务器、Turbo 工作区路径及根仓库分支、Fetch 和 Pull，不计入业务项目数量与全选。
业务面板标题右侧为重新扫描、批量构建；搜索右侧为选择数量、全选、清空和失败重试。
分支下拉默认显示实际当前分支，选择其他分支立即执行 git checkout；失败恢复实际分支并显示错误。Pull 在对应仓库执行。
根仓库切换或 Pull 成功后自动重新扫描业务仓库与构建脚本，扫描完成前禁止构建。
部分项目使用根目录实际存在的 build:dev:<业务> 脚本串行构建；全选使用单条 pnpm build:dev。
每批先核对根仓库和业务仓库的分支与提交，再清理一次 Turbo 根目录 .turbo；每个构建进程启动前再次校验，失败后停止后续进程。
右侧可切换批次和单个进程日志。清空显示仅隐藏当前预览，重新选择来源可恢复，完整文件不会删除。
日志悬停不显示整段 tooltip；手动滚动暂停自动跟随，再次勾选“自动滚动”可恢复。
完整日志位于 %LOCALAPPDATA%\DesktopAppEngine\logs\app-$Name\，每次启动独立会话。
校验配置（无窗口、无远端操作）：& '.\$Name.exe' --check
只读扫描（无窗口）：& '.\$Name.exe' --inspect
可使用 --theme light|dark|system 指定主题；打包应用不接受 --app 覆盖定义。
"@
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
