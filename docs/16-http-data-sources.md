# HTTP JSON 数据源（0.2）

引擎直接读取 JSON 中的接口配置，在后台请求，解析响应并原子写入 state。现有指标、表格及 ECharts 的响应式绑定自动刷新，不需要外部脚本解析 stdout。

## 最小配置

```json
{
  "dataSources": {
    "metrics": {
      "url": "https://example.com/api/metrics",
      "method": "GET",
      "headers": {"Accept":"application/json"},
      "query": {"scope":{"kind":"state","key":"scope"}},
      "timeoutMs": 15000,
      "onLoad": true,
      "intervalMs": 2000,
      "statusTarget": "requestStatus",
      "responseSample": {"data":{"cpu":0.42,"requests":1200}},
      "mappings": [{"target":"stats","path":"/data"}]
    }
  }
}
```

对应初始状态必须预先声明：

```json
{
  "scope":"all",
  "stats":{"cpu":0,"requests":0},
  "requestStatus":{"status":"idle","loading":false,"statusCode":0,"error":"","updatedAt":0}
}
```

按钮事件引用包含以下步骤的动作：

```json
{"type":"request","source":"metrics"}
```

## 请求字段

| 字段 | 规则 |
| --- | --- |
| url | 必填，HTTP(S) 字符串或 state/literal 绑定；必须为绝对 URL，不允许内嵌用户名/密码 |
| method | GET 默认；支持 POST、PUT、PATCH、DELETE |
| headers | 普通头接受字符串/state，认证头使用加密 credential 引用；Host、Content-Length、Transfer-Encoding 由客户端管理 |
| query | 名称到字符串或绑定的对象；客户端编码 query，保留 URL 中已有参数 |
| body | 可选 JSON Binding，支持任意 JSON literal 或 state；GET 不接受 body |
| timeoutMs | 默认 15000，范围 1..120000，包含连接和响应读取 |
| statusTarget | 必填，请求状态写入顶层 state 的键 |
| responseSample | 必填，响应 JSON 样本，仅用于加载校验，不是默认结果 |
| mappings | 必填非空数组，每项 target 和可选 path；path 为 RFC 6901，空字符串取完整响应 |
| onLoad | 默认 false，运行宿主第一次 poll_http 时请求 |
| intervalMs | 可选，100..86400000；上一请求完成后等待该间隔再请求 |
| enabled | 可选布尔绑定，默认 true；控制自动和手动请求是否允许启动 |

数据源最多 64 个。数据源声明位于应用顶层，请求绑定和 mapping/statusTarget 均使用应用作用域；不可读取或写入模块私有状态。模块动作可以引用全局数据源，实际数据源仍按其声明位置解析参数。

每个数据源自己的映射不能重复 target，也不能与 statusTarget 相同。不同数据源不能共享写入目标，避免并发响应互相覆盖。多个接口写同一业务对象时应拆分状态再通过 UI 绑定组织展示。

headers/query/url 的绑定值必须为字符串，数字参数请先保存为字符串。不支持 URL 字符串模板、动态键名、鉴权令牌自动刷新或 multipart。body 默认通过 JSON 编码并设置 Content-Type: application/json；显式 headers 可覆盖内容类型。

## 请求与状态生命周期

运行时构造、JSON 解析、--check 和 --check-plugins 均不发送 HTTP。GUI 每帧调用 poll_http，自动调度后台请求；程序化宿主也须调用 poll_http。

请求使用触发时状态快照。动作中的 request 不阻塞后续步骤，后续步骤不能读取尚未到达的新响应。同一数据源运行中再次触发返回“跳过”，不排队或并发；最多每个数据源一个后台请求。其他数据源可并行请求。

请求状态结构固定：

```json
{"status":"success","loading":false,"statusCode":200,"error":"","updatedAt":1791940000000}
```

status 为 idle/loading/success/failed；statusCode 是 HTTP 状态码，尚无响应时为 0；error 提供连接、超时、HTTP、JSON 或映射错误；updatedAt 为最后成功提交的 Unix 毫秒时间戳，未成功为 0。启动请求清空 error，设置 loading=true，保留上次成功时间和业务数据。

仅 2xx 作为成功候选，响应必须为有效 JSON。通过所有 mappings 获取值后，校验固定状态类型与整个界面消费规则；所有业务状态与 success 状态一次性提交，共用一个状态版本。缺字段、类型不符、绑定消费非法时全部映射回退，只记录 failed，保留上次成功数据和 updatedAt。

onLoad 与 intervalMs 共存时立即请求一次，之后按完成间隔刷新；只配置 intervalMs 时首次等待一个间隔。计时使用单调时钟。enabled=false 时跳过新的请求，恢复后到期调度继续；已经发出的请求仍完成并提交，不中断网络操作。暂停按钮也会禁止手动 request。失败后仍按 intervalMs 继续尝试，不存在额外快速重试；仅 onLoad 的失败需要按钮手动重试。

退出应用后不再提交结果，后台线程仍可能等待网络超时；当前不提供显式取消或强制终止网络线程。关闭 GUI 不等待 HTTP；请求总超时上限为 120 秒。配置中没有完成回调、流式响应、SSE、WebSocket、实时日志或自动分页。定时刷新适合普通轮询，尚未做高频性能验收。

## 传输与分发

使用 reqwest 0.12.28 blocking + rustls TLS，客户端在后台线程执行。HTTPS 使用默认 WebPKI 根证书及正常证书验证；无关闭验证或自签名证书配置。客户端可采用 reqwest 的环境代理行为，本协议无独立 proxy 字段。不开启 HTTP 重定向，3xx 记录失败，避免接口配置被隐式跳转。生产 URL、认证请求头请按目标接口配置。

JSON 请求 body 与响应大小上限 4MiB；响应有 Content-Length 时先检查，读取时也限制实际字节数。界面 error 不附加完整 URL、请求头或响应 body。认证凭据现通过当前 Windows 用户的 DPAPI 加密存储，app.json 中引用凭据名称；常见认证头拒绝明文/state 配置。录入、轮换和分发规则见 [加密凭据](17-encrypted-credentials.md)。

HTTP 功能编译在运行时中，客户机不需要 Python、curl 或 Node。使用 ECharts 的页面需要另行随包携带插件与 Node。

## 实现与验证入口

传输与响应映射位于 `src/services/http.rs`，请求排程和状态提交位于 `src/runtime/http.rs`。使用实际接口前先通过 `--check` 校验描述；该模式不发送请求。运行应用后按目标接口验证认证、错误处理和刷新行为。

内联运行时回归覆盖状态批量更新的原子提交和回滚；当前常规离线测试不覆盖实际网络服务、HTTPS 证书链或接口性能。
