# 命令绑定实施记录

在独立 0.2 协议增加 command 动作，无新增依赖。0.1 动作语义保持原定义。

## 实现

- src/command.rs：固定/绑定参数解析、命令与结果契约校验、后台进程、并发消费 stdout/stderr、超时与关闭取消、限量 UTF-8 输出。
- src/v02.rs：command 步骤与严格语义校验。
- src/interaction.rs：点击快照、后台任务队列、同 target 重复启动拒绝、GUI 线程结果写回、退出取消。
- src/gui_v02.rs、src/main.rs：任务期间 30ms 重绘请求、结果轮询、以外部 JSON 目录解析相对路径。
- docs/description-0.2.schema.json、docs/05-description-format.md：结构 schema 和使用契约。
- examples/command.json、command-demo.ps1：输入文字后点击运行 PowerShell 的中文示例。
- tests/command.rs：真实子进程回归测试；scripts/schema-verify.py 同时验证命令示例。

## 验证

本地 GNU 离线工具链、target-verify 构建目录，实际命令：

| 命令 | 结果 | 退出码 |
| --- | --- | --- |
| cargo fmt -- --check | 通过 | 0 |
| cargo clippy --all-targets --locked --offline -- -D warnings | 通过，无警告 | 0 |
| cargo test --locked --offline --test command | 4 passed，0 failed，0 ignored | 0 |
| cargo test --locked --offline | 34 passed，0 failed，0 ignored，含原有 30 项 | 0 |
| python scripts/schema-verify.py | Dashboard、命令示例、非法结构用例通过 | 0 |
| cargo build --release --locked --offline | Finished release，27.40s | 0 |
| scripts/package.ps1 -Description examples/command.json -Output exports -Name CommandApp | 导出并 --check 通过 | 0 |

新增用例实际运行 PowerShell，验证按钮事件启动、运行中状态、双流输出、非零退出码、重复启动与再次执行、中文/空格/特殊字符参数逐项传递、点击时快照、描述目录脚本解析、超时、程序不存在、每条流 200000 字节输出的持续消费及 16384 字节截断。未将这些测试表述为原生 GUI 鼠标验收。

独立包原生 GUI 自动化尝试两次，均因 UIAutomation 未找到按钮而退出 1；未获得鼠标点击成功的证据。测试过程中前台切换和鼠标操作干扰了用户使用电脑，用户指出后停止此类测试，移除本次新增的鼠标测试脚本。命令功能以后台真实进程和按钮事件集成测试作为当前验证依据。

## 行为边界

命令启动后后续动作步骤继续执行，不等待命令完成。结果在结束时一次性提交，不提供实时日志或 stdin 交互。默认超时 60 秒。直接子进程超时终止或应用关闭取消；不递归终止后代进程。应用关闭最多等待后台结果 500ms。UTF-8 解码替换非法字节，使用其他编码的程序需自行配置 UTF-8。脚本/程序仍需随应用分发或在目标机安装。

构建后的运行时为 target-verify/release/desktop-app-engine.exe。此前 exports/DashboardAppRelease 是上一轮组件验收包；命令功能使用本次重新构建的运行时。新命令示例包为 exports/CommandApp，包含运行时、app.json、README.txt 和 command-demo.ps1。
