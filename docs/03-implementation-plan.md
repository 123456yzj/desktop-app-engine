# 实施与验证入口

源码按协议、编译、运行时、组件、外部服务、渲染和业务应用组织，职责与数据流见 [架构](02-architecture.md)。应用输入和交付规范分别见 [描述格式](05-description-format.md) 与 [分发决策](06-packaging-decision.md)。

当前开发验证命令：

```powershell
cargo check --all-targets --locked --offline
cargo fmt -- --check
cargo clippy --all-targets --locked --offline -- -D warnings
cargo test --locked --offline
cargo build --release --locked --offline
cargo run --bin vben-package --locked --offline -- --check
```

测试分布在 loader、`src/runtime/tests.rs`、`src/renderer/egui/tests.rs` 和业务目录中的测试模块，使用内联输入、临时目录与无窗口 egui 绘制，覆盖旧协议加载、编译与状态边界、表格和图表数据刷新以及业务行为。新的缺陷回归应在相应层验证实际行为，并避免依赖工作区生成文件。

远端 SSH、真实 HTTP、客户机插件资源、Windows 用户凭据及前台窗口交互需要各自的环境验证；常规离线测试不代替这些验证。
