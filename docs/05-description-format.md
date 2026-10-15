# 描述协议 0.1 与 0.2

运行时按 `schemaVersion` 显式分派。0.1 使用原独立解析器、校验器、执行器和 GUI 路径；0.2 使用以下独立契约，不自动迁移。下面先保留 0.1 规则，0.2 规则见后文。

所有字段严格校验；未知字段、组件类型或动作步骤均拒绝。解析错误包含 JSON 字段路径和行列；语义错误包含组件 ID 或动作步骤位置。

## 顶层

| 字段 | 类型 | 要求 |
| --- | --- | --- |
| schemaVersion | 字符串 | 固定 `"0.1"` |
| app | 对象 | `name` 字符串；`window` 包含数值 `width`、`height`（160..8192，逻辑像素） |
| state | 对象 | 键为状态名，值为初始字符串 |
| root | 组件 | 嵌套组件树 |
| actions | 对象 | 键为动作 ID，值包含非空 `steps` 数组 |

## 属性

固定值：`{"kind":"literal","value":"中文文本"}`。
绑定值：`{"kind":"state","key":"name"}`。

所有文本、按钮文字、输入值和占位文字均使用同一类型。
绑定输入框更新共享状态；固定值输入框可以编辑，修改只保存在该组件的本地缓冲中。
状态更新后，下次绘制所有相关属性读取最新值。

## 组件

每个组件必须有非空且全树唯一的 `id`。ID 用于保持输入焦点和控件身份。

| type | 必填字段（除 id） | 可选字段 |
| --- | --- | --- |
| row | children 组件数组 | 无 |
| column | children 组件数组 | 无 |
| text | text 属性 | 无 |
| input | value 属性 | placeholder 属性 |
| button | text 属性、onClick 动作 ID | 无 |

row 支持换行，column 纵向排列；内容使用垂直滚动，文本自动换行。
窗口默认随系统亮暗主题。开发时 `--theme light` 或 `--theme dark` 可覆盖。

## 动作

状态赋值：

```json
{"type":"set","target":"result","value":{"kind":"state","key":"name"}}
```

字符串拼接：

```json
{"type":"concat","target":"result","parts":[{"kind":"literal","value":"你好，"},{"kind":"state","key":"name"}]}
```

点击触发时保存所有字符串状态的快照。步骤中的状态引用始终读取该快照；前一步赋值不会改变本次输入快照。步骤按数组顺序写入共享状态，后写覆盖前写。
步骤失败后停止该动作后续步骤，保留已成功的赋值；页面显示动作 ID、步骤索引及原因。再次成功触发后清除该动作错误。
事件进入队列并只消费一次。GUI 每帧只做一个布局通道；没有点击事件的重绘不会执行动作。

## 边界

单个状态或固定字符串最大 65,536 UTF-8 字节；拼接结果超过此限制会失败。
输入控件最多接收 16,384 Unicode 字符，以保证四字节字符也在状态大小限制内。
组件树语义深度上限 64；JSON 解析器同时保留自身递归限制。
协议版本不会静默迁移。不支持 SSH、脚本、导航、菜单或插件。

## 协议 0.2

结构 schema：[description-0.2.schema.json](description-0.2.schema.json)，Draft 2020-12。运行时还进行状态类型、唯一 ID、引用、数据列、行键和坐标等语义校验。schema 不代替运行时；`scripts/schema-verify.py` 使用本机已有 jsonschema 验证器离线检查示例与结构错误。

顶层 schemaVersion、app、state、root、actions 均必填，可选 plugins 数组声明启用包名，dataSources 对象声明 HTTP JSON 接口；schemaVersion 固定 `"0.2"`。app/window 沿用 0.1。所有结构对象拒绝未知字段；组件只接受该类型的专属字段。表格行对象和用户业务状态对象的字段由数据声明消费，不采用任意表达式。

### 状态、绑定与格式

state 是直接 JSON 初始值对象，支持 string、number、boolean、null、array、object。加载时递归记录类型，数组元素须同类型（对象须同字段结构），字段出现 null 时推导可空类型。空数组延后由组件消费校验；之后赋值也必须通过完整组件校验。单独以 null 初始化的顶层状态只接受 null；需要可空表格字段时，在数组中提供至少一个非 null 的同字段值。缺字段不等于 null。

literal：`{"kind":"literal","value":42}`；state：`{"kind":"state","key":"stats","path":"/cpu"}`。path 缺省空字符串；遵循 RFC 6901，`~0` 是 `~`，`~1` 是 `/`。缺键、缺路径、非法转义和绑定类型不匹配均报错。数字字符串不转为数字。输入框只接受字符串，初期可写绑定仅支持顶层状态（空 path）；literal 输入保存在组件本地。

format：`{"type":"number|percent|text","precision":0,"unit":""}`，字段缺省分别为 text、0、空串。precision 必须 0..12；text 的 precision 必须 0。number/percent 只接受有限数字或可空单元格；percent 乘 100 并加 `%`，unit 追加后缀。text 接受 string、number、boolean、null；null 显示 `—`。不支持脚本、日期、货币和模板表达式。

### 页面与模块状态作用域

顶层 state/actions 为应用级。推荐用专门的 `module` 声明局部响应式 state/actions；module 的 children 是其内部组件树。module 默认纵向布局，支持公共 layout/style/visible/enabled，不添加默认卡片外观，不接受 title、columns 或业务 events。内部使用 row/grid/card 等组织视觉布局。module 可嵌套；既有 row、column、grid、panel、card 局部 state/actions 配置保持兼容。叶组件不可声明。容器本身及其后代可绑定这些状态，父级和兄弟模块不可引用。绑定 key 和事件动作 ID 从当前作用域向祖先查找，最后查找应用级；同名声明就近遮蔽。没有跨模块路径或直接引用内部名称的配置语法。

动作的读写范围由声明位置确定。页面动作只能读写页面和应用级数据，不能写模块私有数据；模块动作可读写当前模块及其祖先已共享数据；嵌套模块调用祖先动作时，该动作仍读写祖先作用域，不会因调用位置而改写嵌套同名状态。兄弟模块可重复使用状态和动作名称，组件 id 仍要求全树唯一。需要仅在模块内联动时，将状态声明在模块容器中；顶层和页面级共享数据仍会影响绑定它们的后代。

输入、文字、显示/禁用、动态事件动作、表格行、图表系列、set/concat、命令参数和命令结果 target 均遵守作用域。命令重复运行检查按具体结果状态隔离，两个模块的同名 target 不互相阻塞。嵌套字段仍用 RFC 6901 path。局部状态沿用全局状态类型规则，类型错误和非法动作引用的更新不改变旧状态。使用一次 set 替换状态对象可同步更新多个关联字段。

加载时将局部引用解析为内部唯一键；`@scope/` 是状态/动作名称的保留前缀，配置不得使用。宿主集成可通过 Runtime::state_from(componentId,key) 和 set_from(componentId,key,value) 按组件上下文访问状态，不需要拼接内部名称。该隔离是描述与事件的访问规则，不是操作系统或任意宿主代码的安全沙箱。

示例：[scoped-state.json](../examples/scoped-state.json)；实施说明：[12-scoped-state.md](12-scoped-state.md)。页面/模块是容器树中的作用域，目前没有独立路由或动态卸载生命周期。

### 组件公共属性

每节点 type、id 必填。id 非空、全树唯一，深度上限 64。visible/enabled 是布尔绑定，缺省 true，父隐藏/禁用传播到子节点。隐藏节点不布局和命中；禁用节点仍绘制但不发事件。

layout 所有尺寸为逻辑 point，有限非负且 <=1,000,000，min <= max。width/height 缺省 auto；显式形式为 `{"auto":true}`、`{"fixed":240}`、`{"fr":1}`，恰好一种。fr 正数且 <=1,000,000；主轴权重分配，交叉轴填充可用长度。无界测量时 fr 退化内容尺寸。

| layout 字段 | 缺省与行为 |
| --- | --- |
| minWidth/maxWidth/minHeight/maxHeight | 0/内部有限上界；冻结 flex 分配，最小尺寸超出可用空间时按 overflow 裁剪/滚动 |
| padding | 0，各边等距，border-box 内部扣除 |
| gap | 0，容器子项间距，grid 行列共用 |
| shrink | auto/fr 缺省 1，fixed 缺省 0；负空间按 shrink × 基准尺寸分配 |
| align | stretch；可选 start/center/end，交叉轴对齐 |
| justify | start；可选 center/end/spaceBetween；grid 不接受此字段 |
| wrap | false；只有 row 可以 true，按基准尺寸贪心分行 |
| overflow | clip；scroll 保留 12 point 双轴滚动槽，滚轮最深容器先消费，余量向父传递 |

gap/align/justify/wrap/overflow 只用于容器，叶节点声明会被拒绝。表格自身提供双轴滚动，图表的滚轮由 zoom 单独控制。width/height、约束和 padding 可用于所有组件。坐标按边界对齐像素，避免累加四舍五入误差。

style：background/color/borderColor 可使用 surface、surfaceRaised、textPrimary、textMuted、border、accent、success、warning、danger、focus、primary、onPrimary Token，或 `#RRGGBB` / `#RRGGBBAA` 精确颜色。Token 随主题变化，精确颜色保持不变。fontSize 缺省 16，borderWidth/radius 缺省 0。card/panel 缺省 raised surface 与至少 6 point 圆角，badge/button 有自身最小圆角。样式无继承，每节点单独指定。

文本支持 `fontFamily: proportional|monospace`、`fontWeight: normal|bold`、`textWrap: true|false` 和 `textAlign: start|center|end`；center 同时在可用文本框内垂直居中。默认正常比例字体、换行、起始对齐。Bold 使用 Windows 系统粗体字体；自定义嵌入宿主未安装粗体字族时回退普通比例字体。等宽字体优先于字重。截断的文本默认悬停显示完整内容，可设置 `textOverflowTooltip: false` 关闭，适用于可滚动的日志正文。

按钮支持 `paddingX` / `paddingY`（默认 10 / 6 point），并可通过 `hover`、`pressed`、`disabled` 分别声明 background/color/borderColor。输入和下拉框也应用颜色、边框、圆角、字体与状态配置，保留 egui 的编辑、焦点和弹层行为。未指定状态字段时沿用默认反馈或基础样式。原有描述可继续使用。

Grid 中声明 `height: {"fr": n}` 的子项使对应行按权重分配剩余高度；未声明 fr 的行使用内容高度。有限高度下 fr 子项填满分配的行；无界测量仍采用内容尺寸。

### 组件字段

| type | 必填专属字段 | 可选专属字段与缺省 |
| --- | --- | --- |
| module | 无 | state={}、actions={}、children=[]、props={}、emits={}、on={}；纵向排列，wrap 不可为 true，无默认外观 |
| plugin | component | props={}；由已安装包注册的组件叶节点，事件及属性由 manifest 校验 |
| row/column | 无 | children=[] |
| grid | columns，1..64 个 fixed/fr 轨道 | children=[]，行高取行内测量最大值，无 span/自动列 |
| panel/card | 无 | children=[]、title 字符串绑定 |
| text | text 字符串绑定 | 无 |
| input | value 字符串绑定 | placeholder 字符串绑定 |
| select | value 字符串绑定、options 绑定 | placeholder；optionsFormat=array/lines；change/open 事件；详见 [Select](18-select.md) |
| button | text 字符串绑定 | events.click |
| metric | label 字符串绑定、value 数字绑定 | format，默认 text |
| badge | text 字符串绑定 | tone=accent，可选 success/warning/danger/muted |
| progress | value 数字绑定，范围 [0,1] | text 字符串绑定 |
| table | rows 数组对象绑定、rowKey、columns | rowHeight=32，16..512；events.rowSelect/sortChange |
| lineChart/barChart | series 数组、layout.height.fixed >=120 | title、xAxis/yAxis、legend=true、zoom=false、pan=false |

表格 columns 为 1..64 个 `{id,field,title,width,format?}`；field 是对象直接属性名，rowKey 是唯一字符串字段。width 支持 fixed/fr/auto；最小宽度 48 point，auto 最多抽样前 200 行，数据变化重算，用户拖拽宽度优先。表头/表体共用列宽和水平偏移。固定表头 32 point；表体按固定行高计算可见区间，预取 2 行，仅对裁剪内实际可见行格式化和绘制。单元格支持同列数值、字符串或布尔；排序稳定，null 始终在末尾，选中保存 rowKey。点击同一行不重复 rowSelect；方向键切换选择，Enter 采用宿主激活路径。空表显示“暂无数据”。

系列为 `{id,name,data,xField,yField,monotonicX?}`；id/name/字段非空，最多 32 个系列。xField/yField 是直接对象字段。X 必须为有限数字，Y 为有限数字或 null；坐标和手动轴范围绝对值 <=1e100（避免极值跨度和屏幕转换溢出）。null 是折线断点。Axis 为 `{scale:"linear",min?,max?}`，缺省自动范围；同时给定时 min<max，单边手动范围必要时扩展自动另一边。零跨度扩展，柱图自动 Y 包含零。

非单调系列上限 20,000 点；柱图每系列上限 2,000 点。monotonicX=true 必须实际非递减，折线按屏幕列保留首末与 min/max，按原索引输出；hover 查询原数据，不查询降采样数组。刻度采用 1/2/5 步长，有限循环。图例保存本地可见状态。zoom=true 在绘图区消费滚轮并以指针为锚缩放；pan=true 拖拽平移，双击或聚焦后 Home 恢复自动范围。默认滚轮传给页面。

### 事件与动作

events 是该组件支持的事件名到动作 ID 或动作绑定的对象。固定写法 `{"click":"advance"}` 保持兼容，也支持 `{"click":{"kind":"state","key":"wizard","path":"/action"}}` 和字符串 literal 绑定。绑定必须解析为非空字符串，并引用 actions 中已定义的动作；加载与每次状态更新均校验，非法更新不改变状态。每个事件提交时读取当时状态中的动作 ID，前一个事件的成功状态更新会影响后一个事件的动作选择；一次动作开始后不因后续状态变化而切换其步骤。

容器、指标、badge、progress、图表不接受业务 events。输入 change 先写状态，再解析事件动作并触发，submit 由 Enter 提交；button 支持 click；table 支持 rowSelect、sortChange。内部事件包含 componentId、targetKey、kind、payload、sequence、generation；按输入序号提交，只消费一次，隐藏/禁用/过期 generation 的事件丢弃。表格 rowSelect payload 是行键字符串，sortChange payload 为 `{field,descending}`；控件原始 payload 尚不作为 event 绑定上下文。event 绑定由模块 emit/on 提供，见下节。

actions.steps 非空。set 的 value 可为任意已匹配类型的绑定；concat.parts 仅为字符串绑定。每次动作读取触发时快照，顺序写入，失败停止后续步骤、保留成功写入、显示步骤诊断。measure/arrange/paint 不执行动作；输入事件统一交给 commit。

### 模块通信（0.2）

module.props 在父作用域解析，内部用 `{"kind":"prop","key":"message","path":""}` 读取；props 只读，不可作为 set/concat/command 的 target 或 input.value。module.emits 用 JSON 样本声明事件载荷类型，module.on 将事件绑定到父级动作。内部动作通过 `{"type":"emit","event":"changed","payload":{"kind":"state","key":"draft"}}` 发出数据，父级处理动作通过 `{"kind":"event","path":""}` 或对象字段路径读取载荷。event 只用于处理动作步骤，不可用于界面属性。处理动作可用可选 eventPayload 样本声明类型，适用于尚未激活的动态监听动作。emit 同步执行父级处理动作，调用深度上限 32，原动作仍保持开始时快照。完整协议与限制见 [模块通信](14-module-communication.md)，示例 [module-linkage.json](../examples/module-linkage.json)。

### 可安装插件（0.2）

插件使用 `type:plugin`、component 和 props，动作使用 pluginCall 的 target/method/args/resultTarget；事件处理动作通过 eventPayload 声明类型。后台 JavaScript 宿主处理 mount/update/resize/unmount，SVG 交给引擎绘制。完整安装、生命周期、事件、方法与分发契约见 [插件机制](15-plugins.md)。

### HTTP 接口配置（0.2）

顶层 dataSources 定义 url/method/headers/query/body、响应样本 responseSample、mappings 和独立 statusTarget。动作 `{"type":"request","source":"metrics"}` 后台请求；可配置 onLoad、intervalMs 和 enabled 自动刷新。响应以 RFC 6901 path 映射到顶层 state，所有映射与 success 状态原子提交；请求失败或映射非法保持旧业务数据。详细契约及示例见 [HTTP 数据源](16-http-data-sources.md)。

顶层 credentialStore 指定当前用户的加密凭据库。认证 headers 使用 `{"kind":"credential","name":"api-token","prefix":"Bearer "}`；凭据由 Windows DPAPI 加密保存在用户本地目录，发送请求时后台解密。常见认证头必须使用该引用。配置校验无需真实凭据，--check-credentials 检查存在与解密能力。详见 [加密凭据](17-encrypted-credentials.md)。

command 可选 `onComplete` 指定完成动作，在结果写入 target 后调用。`parseJson` 步骤将字符串绑定解析为 JSON 并写入已有类型的 target；解析或组件校验失败时保留原值。详见 [命令完成反馈](10-command-binding.md)。

style 额外支持 `primary` 和 `onPrimary`，用于主操作按钮背景与前景；按钮显式 background 会用于绘制，并保留悬停、按下和禁用反馈。

### 按钮命令绑定（0.2）

button 的 `events.click` 指向包含 `command` 步骤的动作：

```json
{"type":"command","program":"python","args":["scripts/task.py",{"kind":"state","key":"input"}],"cwd":".","timeoutMs":60000,"target":"job"}
```

`program` 为程序名或路径；`args` 缺省为空，每个参数为字符串或解析为字符串的 literal/state 绑定，点击时取快照。参数逐项传给进程，不执行 shell 拼接；需要 CMD/PowerShell 时显式指定 `cmd.exe`/`powershell.exe` 及参数。程序名通过系统搜索路径查找；含目录的相对程序路径以描述目录为基准。`cwd` 缺省描述目录，相对 cwd 同样以描述目录为基准；`--app` 使用该 JSON 所在目录，默认加载使用 EXE 所在目录。脚本文件参数由被调用程序按 cwd 解析。

`target` 必须指向已定义的结果状态，初始结构如下，字段类型固定且不允许额外字段：

```json
{"status":"idle","running":false,"exitCode":-1,"stdout":"","stderr":"","error":""}
```

后台执行期间立即写 `status=running`、`running=true` 并清空旧结果；完成后写 `succeeded` 或 `failed`、退出码及 stdout/stderr。启动失败或超时的诊断写入 error；`exitCode=-1` 表示没有可用退出码。`timeoutMs` 缺省 60000，范围 1..3600000；超时终止直接子进程并写 `timedOut`。窗口关闭时请求终止正在运行的直接子进程；不管理子进程自行启动的后代进程。stdin 关闭，Windows 下不创建控制台窗口。

同一 target 正在执行时拒绝重复启动，其他 target 可以并行执行。command 步骤启动后立即继续后续步骤，不等待完成，后续步骤仍读取动作开始时快照。完成结果由 GUI 更新线程写回，不在后台线程修改业务状态。输出在结束时更新，每条流保留前 16384 字节，继续消费超出部分避免管道堵塞；以 UTF-8 解码，非法字节替换。继承输出管道的后代进程不会使结果提交无限等待；结束后最多为两条流各等待 100ms。非零退出码通过结果状态报告。

可运行示例：[command.json](../examples/command.json) 和 [command-demo.ps1](../examples/command-demo.ps1)。分发时将脚本随应用复制，并按实际目录配置 cwd/args。`--check` 只校验描述，不执行命令或检查程序是否安装。

### 运行与任务日志

0.1/0.2 桌面宿主均提供底部“日志与任务”和“打开日志目录”。命令 stdout/stderr 按原始字节持续写文件，窗口独立显示每条流最近 32 KiB，不影响命令结果状态。0.2 可使用 `{"type":"openLogs"}` 动作步骤打开同一窗口，无参数，不读取或修改业务状态。运行日志按 EXE 名和启动会话保存，完整位置、文件和错误边界见 [运行与任务日志](19-runtime-logs.md)。

完整示例：[dashboard.json](../examples/dashboard.json)。实施和实际验收：[09-component-implementation.md](09-component-implementation.md)。
