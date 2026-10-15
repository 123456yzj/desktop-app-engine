# 分发决策：输入描述编译为应用 EXE

日期：2026-10-15。正式交付采用应用专属 EXE；JSON 是制作端必备输入，不作为交付包中的运行依赖。

## 构建与产物

```text
源应用描述 JSON
  → build.rs 解析并生成 Rust 数据定义
  → Cargo 编译对应运行时与应用定义
  → 应用 EXE + 按需配套的组件/资源 + 使用说明
```

每次打包都会用 `DAE_APP_DESCRIPTION` 指向本次实际输入。生成的 Rust 定义进入 EXE，不复制应用描述 JSON，不把固定示例当作传入描述。Cargo 同时跟踪输入路径、输入内容和构建环境变化；应用定义修改后需要重新打包。

```powershell
$env:CARGO_TARGET_DIR = "$PWD\target-http"
.\scripts\package.ps1 -Description .\examples\hello.json -Output .\exports -Name HelloApp
```

制作端需要 Rust/Cargo 和已缓存的锁定依赖，脚本执行 `cargo build --release --bin <运行时> --locked --offline`。`CARGO_TARGET_DIR` 指定编译缓存目录，默认使用项目 `target`。脚本在编译后复制应用 EXE、按需复制插件基础件，生成 README 并执行包内 `--check` 或 `--check-plugins`。应用名称默认取描述的 `app.name`；可通过 `-Name` 指定有效 Windows 文件名，已有目录拒绝覆盖。失败返回 2。

无插件应用的交付目录为：

```text
应用名/
  应用名.exe
  README.txt
```

使用插件时携带相应插件和运行库，客户机无需 Rust、项目源码或制作端源描述。插件自身的 manifest、依赖元数据属于组件基础件，不是外置应用描述。当前工具没有需要预置的业务数据库，不生成空数据库占位。

## 应用定义与运行数据

打包 EXE 只使用编入的应用定义，忽略同目录 `app.json`，拒绝 `--app` 覆盖。`--check` 无窗口校验编入的定义；校验成功退出 0。源描述删除、移动或修改都不会改变已交付应用。

Turbo 构建工具的连接设置保存在 `%LOCALAPPDATA%\DesktopAppEngine\apps\<EXE名>\connection.dat`。该文件采用带版本标识的二进制格式，独立保存用户修改的 SSH 地址和工作区路径，不回写源描述或应用 EXE。首次启动使用编入的初始连接；后续读取当前用户保存的连接设置。设置先校验、写临时文件并同步，再替换正式文件；保存失败不更改当前连接。`--check` 不读取连接覆盖、不发起远端操作。任务日志继续写入用户日志目录。

开发构建未指定 `DAE_APP_DESCRIPTION` 时，保留 `--app FILE`、同目录 `app.json`、开发示例的加载顺序，方便编辑器和测试调试。该开发加载规则不适用于打包 EXE。

## 验证

`python scripts/verify-compiled-package.py` 实际编译多个不同应用，覆盖 0.1、0.2、同一路径更新输入、源文件移除后运行、邻近 JSON 无法覆盖、拒绝 `--app` 以及后续构建不影响先前应用。输出位于 `tmp-verify`。Turbo 原生测试覆盖连接设置持久化、重复保存、坏文件与无效保存恢复。
