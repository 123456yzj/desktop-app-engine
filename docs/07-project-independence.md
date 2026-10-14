# 引擎独立项目

新项目：`D:\project\impact\desktop-app-engine`。
原位置：`D:\project\impact\app-v1\desktop-app-engine`，保留为迁移前历史副本。

## 项目边界

引擎负责应用描述协议、严格校验、状态与绑定、事件与动作执行、原生桌面渲染、运行时入口和应用包导出。
编辑器、具体业务应用和平台服务不属于引擎项目；未来分别建立项目，通过版本化 JSON 描述与固定运行时对接。
examples/hello.json 是协议示例和开发兜底，不代表引擎的业务定位。

Cargo.toml 为独立 package，不依赖父 Cargo workspace、父目录源码或本地 path dependency。
保留库入口 src/lib.rs 与可执行入口 src/main.rs；本次不改变协议或运行行为，也不拆分新 crate。

## 迁移内容

迁入 Cargo.toml、Cargo.lock、源码、测试、脚本、文档、JSON 示例和两个有效历史截图。
未迁入工具链、构建缓存、压缩包、安装器、临时日志、未知数据库、过期 dist 和无效截图。
evidence/fixtures 由测试生成，没有作为源码复制；新项目的构建产物和生成数据已加入 .gitignore。
docs/03 保留历史实测记录，其中绝对路径指向原位置。

## 构建与验证

日常开发使用用户级 rustup + cargo，不需要原项目目录。客户机只接收导出的应用包。
本机 GNU 离线验收临时复用原目录中的已有工具链与依赖缓存，构建目录和临时目录均位于新项目。
工具链是构建环境，不是源码依赖；安装常规 Rust 工具链后即可在其他位置构建。

迁移实测：所有命令工作目录均为 `D:\project\impact\desktop-app-engine`。

| 命令 | 输出与退出码 |
| --- | --- |
| `cargo fmt -- --check` | 无诊断，FMT_EXIT=0 |
| `cargo clippy --all-targets --locked --offline -- -D warnings` | Finished dev profile，CLIPPY_EXIT=0 |
| `cargo test --locked --offline` | description 5、engine 6、external 1、loading 1，全部通过，TEST_EXIT=0 |
| `cargo build --release --locked --offline` | 首次构建 1m 09s，缓存复查 0.47s，BUILD_EXIT=0 |
| `.\scripts\package.ps1 -Description .\examples\hello.json -Output .\exports -Name HelloApp` | 应用包校验通过，PACKAGE_EXIT=0 |

新项目首次运行 external 时 fixture 是新文件，因此没有原目录的覆盖权限问题。后续若沙箱继续限制覆盖已有 fixture，重复测试仍可能受阻，首次通过不表示沙箱权限已解除。

新运行时：`target-verify/release/desktop-app-engine.exe`。
新应用包：`exports/HelloApp/`，实际目录内容：

```text
app.json        1272
HelloApp.exe 5515776
README.txt       553
```

打包自动执行 EXE 同目录加载的 --check，退出码 0。迁移没有增加新依赖，没有重新运行 GUI 截图验收。
