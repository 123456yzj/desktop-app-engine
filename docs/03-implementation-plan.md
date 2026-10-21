# 实施与验证入口

源码按协议、编译、运行时、组件、外部服务、渲染和业务应用组织，职责与数据流见 [架构](02-architecture.md)。应用输入和交付规范分别见 [描述格式](05-description-format.md) 与 [分发决策](06-packaging-decision.md)。

当前开发验证命令：

```powershell
cargo check --all-targets --locked --offline
cargo fmt -- --check
cargo clippy --all-targets --locked --offline -- -D warnings
cargo build --release --locked --offline
cargo run --bin vben-package --locked --offline -- --check
```

当前仓库不保留自动化测试代码。通过上述编译、格式、Clippy 和描述校验命令检查源码与输入；通用入口可使用 `cargo run --bin desktop-app-engine --locked --offline -- --app .\examples\vben-package.json --check` 校验描述。

远端 SSH、真实 HTTP、客户机插件资源、Windows 用户凭据及前台窗口交互需要各自的环境验证；编译和描述校验不代替这些验证。
