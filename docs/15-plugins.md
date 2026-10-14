# 可安装插件机制（0.2 / Plugin API 1）

引擎提供稳定的组件宿主，安装包注册组件、属性、事件和方法。当前可交付宿主是外部 Node JavaScript 进程和 SVG 渲染协议。引擎安装这一版后，新增符合 API 1 的插件无需重新编译引擎；应用通过 JSON 引用组件。

## 安装和运行

```powershell
# 开发机需要 Node/npm，安装器拒绝覆盖已有插件目录。
.\scripts\install-plugin.ps1 -Package .\plugins\echarts -AppDirectory .\examples -InstallDependencies
.\target-verify\release\desktop-app-engine.exe --app .\examples\plugin-chart.json --check
.\target-verify\release\desktop-app-engine.exe --app .\examples\plugin-chart.json --check-plugins
.\target-verify\release\desktop-app-engine.exe --app .\examples\plugin-chart.json

# 已完成 release 构建后，导出完整应用目录并后台实测插件。
$env:CARGO_TARGET_DIR = "$PWD\target-verify"
.\scripts\plugin-verify.ps1 -Output .\exports -Name PluginChartApp
```

安装器复制插件目录和当前开发机的 node.exe、可用的 Node LICENSE。npm 有锁文件时使用 ci，安装时省略开发依赖与安装脚本。插件的 node_modules 连同包内许可证一起分发。客户机无需 Rust、npm 或单独安装 Node。

目录结构：

```text
PluginChartApp/
  PluginChartApp.exe
  app.json
  plugins/
    runtime/node.exe
    runtime/NODE-LICENSE.txt
    echarts/plugin.json
    echarts/worker.cjs
    echarts/package.json
    echarts/package-lock.json
    echarts/node_modules/...
```

`plugins` 在应用 JSON 顶层列出启用包名。发现基于 app.json 所在目录，与启动工作目录无关；纯字符串解析 `v02::parse` 使用当前目录，程序化宿主可使用 `parse_at` 指定目录。插件入口必须在包内；包名仅接受字母、数字、下划线、连字符。manifest 错误、缺失包、重复组件注册都使加载失败。Node 优先使用 `plugins/runtime/node.exe`，开发时可回退 PATH 中的 node。

`--check` 校验配置和 manifest，不执行 JavaScript；`--check-plugins` 额外逐个启动已引用组件，在 800×400 下 mount、栅格化返回 SVG、unmount；成功退出 0，失败退出 2。这个模式会执行插件代码。插件与应用具有相同本机权限，API 1 不提供沙箱。

## 应用配置和响应式属性

```json
{
  "type": "plugin",
  "id": "cpuChart",
  "component": "echarts.chart",
  "layout": {"height": {"fixed": 440}, "width": {"fr": 1}},
  "props": {"option": {"kind": "state", "key": "chartOption"}},
  "events": {"click": "chartClick"}
}
```

插件是布局叶节点，支持通用 layout/style/visible/enabled；默认高度 240，有限宽度填充可用空间，无界宽度使用 600。props 支持 literal/state/prop，使用所在模块词法作用域；插件 props 不建立新的模块作用域。依赖版本包含所有 props 绑定。

加载及每次状态写入均用 manifest 验证已解析属性。非法写入保留旧状态和版本。插件属性的 literal 可以包含混合数组等 JSON；state 继续服从引擎既有固定类型规则，不因插件放宽。组件收到属性值快照，通过后续 update 刷新。

## manifest 契约

`plugins/echarts/plugin.json` 是完整参考。字段为 apiVersion=1、name、version、entry、components；组件注册键是 `包名.组件名`，例如 echarts.chart。每个组件声明 props、events 和 methods，每个方法声明 args/result。

契约为本项目定义的递归类型格式，支持 kind=any/object/array/string/number/boolean/null、properties、required、items、additionalProperties。object 默认禁止未声明属性，开放 JSON 如 ECharts option 必须显式设置 additionalProperties=true。manifest 未知字段、未知 kind 和不在 properties 中的 required 都拒绝。这是类型契约子集，不是完整 JSON Schema，也不解析 npm 模块的 Vue install API。

例如方法定义：

```json
{
  "dispatchAction": {
    "args": {"kind":"object","required":["type"],"properties":{"type":{"kind":"string"}},"additionalProperties":true},
    "result": {"kind":"boolean"}
  }
}
```

安装 npm 包后还需要提供 manifest 和适配 worker。通用宿主只处理协议、渲染和调用，插件自己的业务及第三方库整合由适配器负责。

## 生命周期和后台协议

每个已挂载组件有独立进程与串行请求队列。JavaScript stdin/stdout 使用一行一个 JSON；stdout 只写协议响应，诊断写 stderr。请求没有并发，按接收顺序响应，不能发送未经请求的独立通知。可选响应字段 svg、events、result；失败响应 `{ "error": "说明" }`。

| 请求 op | 请求内容 | 时机 |
| --- | --- | --- |
| mount | component、props、width、height | 首次进入可见绘制区域 |
| update | 同 mount | 已解析 props 变化；可同时包含新尺寸 |
| resize | 同 mount | props 不变、内容区域尺寸变化 |
| input | component、event={kind:"click",x,y} | enabled 组件收到点击；坐标相对内容区域 |
| call | component、method、args | pluginCall 动作入队 |
| unmount | component | 隐藏、移出可见裁剪区域或应用销毁 |

update/resize 的适配器应处理请求中的当前尺寸。宽高使用逻辑像素，限制 1..4096；SVG 输出也限制最大 4096×4096。渲染采用后台 resvg，返回预乘 RGBA 并由 egui 纹理绘制。插件进程、SVG 解析和栅格化均在后台执行。GUI 以 30ms 周期检查完成消息。

每行请求/响应上限 8MiB，组件请求队列 32 项、全局完成队列 4 项；队列满则报错。写入与等待响应分别限时 5 秒。出现进程、协议或渲染错误后本实例停止处理并向界面报告，重新挂载创建新实例。卸载丢弃尚未执行的队列项，尝试 unmount 后终止进程；后台清理受请求超时限制，GUI 不等待。系统字体首次加载会在后台缓存。

卸载后旧实例的响应通过 epoch 丢弃，不能更新新实例的纹理或状态。属性更新按队列顺序处理；API 1 不合并连续更新。

## 事件和方法

事件响应：

```json
{"events":[{"name":"click","payload":{"seriesIndex":0,"dataIndex":1,"name":"B","value":24}}]}
```

只能发出 manifest 声明事件，载荷先校验事件契约；events 动作必须声明 eventPayload，动作步骤可用 `kind:event` 的 RFC 6901 路径。固定动作与动态动作绑定均采用现有词法解析，处理时使用最新状态选择动作。disabled/隐藏的组件及祖先会抑制异步事件。

方法调用：

```json
{
  "type":"pluginCall",
  "target":"cpuChart",
  "method":"dispatchAction",
  "args":{"kind":"literal","value":{"type":"highlight","seriesIndex":0,"dataIndex":1}},
  "resultTarget":"dispatchResult"
}
```

target 是全树唯一组件 ID，method 必须已注册；args 支持 literal/state/prop/event，使用动作开始时的状态快照。resultTarget 可省略，填写时按动作声明作用域定位可写状态，不能写只读 prop。加载时校验 args 和结果状态初始值的契约，返回时再次校验 result 并通过 Runtime::set 写入。结果异常保持旧值。

调用为异步，不等待结果，不阻塞动作后续步骤；后续步骤不能把同次调用的新结果作为已完成数据读取。组件必须已 mount，隐藏或尚未 mount 的组件调用报错。当前无方法完成回调和全局无组件方法。

## ECharts 适配器范围

真实使用 ECharts 6.1.0 的 init(renderer=svg,ssr=true)、setOption、resize、dispose。更新使用 notMerge=true，关闭动画。支持 JSON 可表达的 SVG SSR option；getOption 返回对象，dispatchAction 返回 boolean 并重绘。state 保存 getOption 时还需符合引擎固定状态类型，省略 resultTarget 可只做调用校验。

点击用 ECharts display list 命中，发出 seriesIndex/dataIndex/name/value，当前 value 为数字，数组取最后数值。已实测普通柱图。API 1 当前只转发 click；不提供浏览器 DOM、HTML tooltip、hover/滚轮/拖拽、JS 函数 formatter 或完整浏览器 ECharts 交互。需要 DOM 的库要增加其他宿主版本；SSR 点击适配依赖 ECharts helper/display-list API，版本已锁定。

示例展示柱图、选择名称、按钮调用 dispatchAction 高亮 B。源码参考 `plugins/echarts/worker.cjs` 和 `examples/plugin-chart.json`。

## 验证记录

使用后台与无窗口测试验证，不操作鼠标或切换桌面窗口。专项覆盖 manifest 拒绝、真实 ECharts RPC、SVG 栅格化、响应式更新、方法结果、事件载荷、disabled/卸载响应隔离、RPC 超时，并通过 egui RawInput 验证纹理绘制和实际点击到 ECharts 的完整链路。

过程发现并修复：首次 ECharts 专项 2 通过/2 失败，原因是 Node 无法处理 Windows 扩展路径入口，改用入口文件名和包目录 cwd；严格 Clippy 的测试切片写法已修复；完整回归发现一次外部 JSON 错误缺少“外部描述”上下文，已恢复；通用测试 fixture 曾缺少闭合括号，已修复。最终检查和产物见 evidence/plugin-verification.txt。

最终 fmt、严格 Clippy、68 项测试（新增 9 项插件/无窗口 GUI 测试）、七个示例 schema 验证均通过，release 构建通过。最终应用包 `exports/PluginChartAppFinal/` 已通过 --check-plugins；在项目外目录、PATH 仅含 Windows System32 时再次通过，确认使用包内 Node。初次分发验证曾错误检查未设置的 LASTEXITCODE，已改为依赖安装脚本异常和 npm 进程实际退出码。一次合并测试/构建工具调用达到 60 秒超时，随后独立 release 构建成功（32.53 秒）。没有进行原生窗口视觉或交互验收，也未进行插件性能基准。
