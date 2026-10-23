# 描述协议 0.2

需要按组件学习用法时，请从 [组件使用教程](components/README.md) 开始；其中提供完整示例、属性表和容器组合教程。本篇保留协议字段与校验规则参考。

运行时、开发加载器和打包入口统一只接受 `schemaVersion: "0.2"`。旧协议及其独立执行、绘制路径已移除；旧描述需更新为本篇格式，按钮动作使用 `events.click`。

所有字段严格校验；未知字段、组件类型或动作步骤均拒绝。解析错误包含 JSON 字段路径和行列；语义错误包含组件 ID 或动作步骤位置。

## 顶层

结构 schema：[description-0.2.schema.json](description-0.2.schema.json)，Draft 2020-12。编译器还进行状态类型、唯一 ID、引用、数据列、行键和坐标等语义校验。schema 不代替完整编译校验；运行应用的 `--check` 验证描述和引用资源。

顶层 schemaVersion、app、state、root、actions 均必填，可选 plugins 数组声明启用包名，dataSources 对象声明 HTTP JSON 接口；schemaVersion 固定 `"0.2"`。app 包含非空 name、可选 host 和 window；window 的 width、height 为 160..8192 的有限数值，单位为逻辑像素。所有结构对象拒绝未知字段；组件只接受该类型的专属字段。表格行对象和用户业务状态对象的字段由数据声明消费，不采用任意表达式。

`app.host` 可省略，默认使用通用引擎；指定时必须是 1..64 位字母、数字、下划线或连字符，对应制作端引擎旁的 `<host>.exe` 宿主模板。例如构建工作台声明 `"host":"vben-package"`，统一打包命令据此保留其原生业务控制器。缺少宿主时导出失败，不按应用名称推断宿主。宿主对协议及业务字段的额外要求由最终 EXE 校验。

### 状态、绑定与格式

state 是直接 JSON 初始值对象，支持 string、number、boolean、null、array、object。加载时递归记录类型，数组元素须同类型（对象须同字段结构），字段出现 null 时推导可空类型。空数组延后由组件消费校验；之后赋值也必须通过完整组件校验。单独以 null 初始化的顶层状态只接受 null；需要可空表格字段时，在数组中提供至少一个非 null 的同字段值。缺字段不等于 null。

单个状态或固定字符串最大 65,536 UTF-8 字节；拼接结果也受此限制。输入控件最多接收 16,384 Unicode 字符，以保证四字节字符也在状态大小限制内。

literal：`{"kind":"literal","value":42}`；state：`{"kind":"state","key":"stats","path":"/cpu"}`。path 缺省空字符串；遵循 RFC 6901，`~0` 是 `~`，`~1` 是 `/`。缺键、缺路径、非法转义和绑定类型不匹配均报错。数字字符串不转为数字。输入框只接受字符串，初期可写绑定仅支持顶层状态（空 path）；literal 输入保存在组件本地。

format：`{"type":"number|percent|text","precision":0,"unit":""}`，字段缺省分别为 text、0、空串。precision 必须 0..12；text 的 precision 必须 0。number/percent 只接受有限数字或可空单元格；percent 乘 100 并加 `%`，unit 追加后缀。text 接受 string、number、boolean、null；null 显示 `—`。不支持脚本、日期、货币和模板表达式。

### 页面与模块状态作用域

顶层 state/actions 为应用级。推荐用专门的 `module` 声明局部响应式 state/actions；module 的 children 是其内部组件树。module 默认纵向布局，支持公共 layout/style/visible/enabled，不添加默认卡片外观，不接受 title、columns 或业务 events。内部使用 row/grid/card 等组织视觉布局。module 可嵌套；既有 row、column、grid、panel、card 局部 state/actions 配置保持兼容。叶组件不可声明。容器本身及其后代可绑定这些状态，父级和兄弟模块不可引用。绑定 key 和事件动作 ID 从当前作用域向祖先查找，最后查找应用级；同名声明就近遮蔽。没有跨模块路径或直接引用内部名称的配置语法。

动作的读写范围由声明位置确定。页面动作只能读写页面和应用级数据，不能写模块私有数据；模块动作可读写当前模块及其祖先已共享数据；嵌套模块调用祖先动作时，该动作仍读写祖先作用域，不会因调用位置而改写嵌套同名状态。兄弟模块可重复使用状态和动作名称，组件 id 仍要求全树唯一。需要仅在模块内联动时，将状态声明在模块容器中；顶层和页面级共享数据仍会影响绑定它们的后代。

输入、文字、显示/禁用、动态事件动作、表格行、图表系列、set/concat、命令参数和命令结果 target 均遵守作用域。命令重复运行检查按具体结果状态隔离，两个模块的同名 target 不互相阻塞。嵌套字段仍用 RFC 6901 path。局部状态沿用全局状态类型规则，类型错误和非法动作引用的更新不改变旧状态。使用一次 set 替换状态对象可同步更新多个关联字段。

加载时将局部引用解析为内部唯一键；`@scope/` 是状态/动作名称的保留前缀，配置不得使用。宿主集成可通过 Runtime::state_from(componentId,key) 和 set_from(componentId,key,value) 按组件上下文访问状态，不需要拼接内部名称。该隔离是描述与事件的访问规则，不是操作系统或任意宿主代码的安全沙箱。

配置片段与规则见 [作用域](12-scoped-state.md)。页面/模块是容器树中的作用域，目前没有独立路由或动态卸载生命周期。

### 组件公共属性

每节点 type、id 必填。id 非空、全树唯一，深度上限 64。visible/enabled 是布尔绑定，缺省 true，父隐藏/禁用传播到子节点。隐藏节点不布局和命中；禁用节点仍绘制但不发事件。

layout 所有尺寸为逻辑 point，有限非负且 <=1,000,000，min <= max。width/height 缺省 auto；显式形式为 `{"auto":true}`、`{"fixed":240}`、`{"fr":1}`，恰好一种。fr 正数且 <=1,000,000；主轴权重分配，交叉轴填充可用长度。无界测量时 fr 退化内容尺寸。

| layout 字段 | 缺省与行为 |
| --- | --- |
| minWidth/maxWidth/minHeight/maxHeight | 0/内部有限上界；冻结 flex 分配，最小尺寸超出可用空间时按 overflow 裁剪/滚动 |
| padding | 0，各边等距，border-box 内部扣除 |
| gap | 0，容器子项间距；Space 缺省为 8 |
| rowGap/columnGap | 可选，分别覆盖纵向/横向间距，缺省回退到 gap |
| shrink | auto/fr 缺省 1，fixed 缺省 0；负空间按 shrink × 基准尺寸分配 |
| align | stretch；可选 start/center/end，交叉轴对齐 |
| justify | start；可选 center/end/spaceBetween/spaceAround/spaceEvenly；grid/masonry 不接受此字段 |
| wrap | false；row 和横向 flex/space 可以 true，按基准尺寸贪心分行 |
| overflow | clip；scroll 保留 12 point 双轴滚动槽，滚轮最深容器先消费，余量向父传递 |

gap/rowGap/columnGap/align/justify/wrap/overflow 只用于容器，叶节点声明会被拒绝。splitter 不接受 gap/rowGap/columnGap/align/justify/wrap，其间距由 barSize 控制。表格自身提供双轴滚动，图表的滚轮由 zoom 单独控制。width/height、约束和 padding 可用于所有组件。坐标按边界对齐像素，避免累加四舍五入误差。

style：background/color/borderColor 可使用 surface、surfaceRaised、textPrimary、textMuted、border、accent、success、warning、danger、focus、primary、onPrimary Token，或 `#RRGGBB` / `#RRGGBBAA` 精确颜色。Token 随主题变化，精确颜色保持不变。fontSize 缺省 16，borderWidth/radius 缺省 0。card/panel 缺省 raised surface 与至少 6 point 圆角，badge/button 有自身最小圆角。样式无继承，每节点单独指定。同一组件的背景和边框只绘制一次：panel/card 的边框由 `bordered` 控制，禁用时不再绘制；输入和下拉的边框、圆角与背景归控件外框所有。

文本支持 `fontFamily: proportional|monospace`、`fontWeight: normal|bold`、`textWrap: true|false` 和 `textAlign: start|center|end`；center 同时在可用文本框内垂直居中。默认正常比例字体、换行、起始对齐。Bold 使用 Windows 系统粗体字体；自定义嵌入宿主未安装粗体字族时回退普通比例字体。等宽字体优先于字重。截断的文本默认悬停显示完整内容，可设置 `textOverflowTooltip: false` 关闭，适用于可滚动的日志正文。

按钮支持 `paddingX` / `paddingY`（默认 10 / 6 point），并可通过 `hover`、`pressed`、`disabled` 分别声明 background/color/borderColor。输入框把 `paddingX` / `paddingY` 用作编辑区内边距，下拉框把它用作标签横向起点；两者均应用颜色、边框、圆角、字体与状态配置，保留 egui 的编辑、焦点和弹层行为。未指定状态字段时沿用默认反馈或基础样式。原有描述可继续使用。

Grid 中声明 `height: {"fr": n}` 的子项使对应行按权重分配剩余高度；未声明 fr 的行使用内容高度。有限高度下 fr 子项填满分配的行；无界测量仍采用内容尺寸。

### 组件字段

| type | 必填专属字段 | 可选专属字段与缺省 |
| --- | --- | --- |
| module | 无 | state={}、actions={}、children=[]、props={}、emits={}、on={}；纵向排列，wrap 不可为 true，无默认外观 |
| plugin | component | props={}；由已安装包注册的组件叶节点，事件及属性由 manifest 校验 |
| row/column | 无 | children=[] |
| flex/space | 无 | children=[]、vertical=false；space 默认 gap=8、align=center；[教程](components/21-flex.md) |
| divider | 无 | text 字符串绑定、vertical=false、variant=solid/dashed/dotted、titlePlacement=start/center/end；纵线不接受 text；[教程](components/23-divider.md) |
| layout/header/sider/content/footer | 无 | children=[]；layout 有直属 sider 时横向，否则纵向；header 高 64、footer 高 48、sider 宽 200，其余轴默认 fr=1；[教程](components/24-layout.md) |
| splitter | children 恰好两个 | vertical=false、value 数字绑定 [0,1] 缺省 0.5、barSize=8（4..32）、resizable=true；change 先写比例再调用动作；[教程](components/25-splitter.md) |
| masonry | 无 | children=[]、columnCount=3（1..64）、minColumnWidth 可选正数；最短列排布，尺寸变化重新测量；[教程](components/26-masonry.md) |
| grid | columns，1..64 个 fixed/fr 轨道 | children=[]，行高取行内测量最大值，无 span/自动列 |
| panel/card | 无 | children=[]、title 字符串绑定 |
| text | text 字符串绑定 | 无 |
| input | value 字符串绑定 | placeholder 字符串绑定 |
| select | value 字符串或多选数组绑定、options 绑定 | placeholder；optionsFormat=array/lines；mode、showSearch、allowClear、loading、maxCount、notFoundContent；属性与完整示例见 [Select 组件教程](components/12-select.md)，动态选项另见 [Select 数据加载](18-select.md) |
| checkbox | value 布尔绑定；options 组时为字符串数组 | text 字符串绑定、indeterminate=false；[教程](components/27-checkbox.md) |
| radio | value 字符串绑定、options 数组绑定 | vertical=false；最多 256 项，空字符串表示未选择；[教程](components/28-radio.md) |
| switch | value 布尔绑定 | text 字符串绑定；[教程](components/29-switch.md) |
| inputNumber | value 数字绑定 | min=-1e12、max=1e12、step=1、precision 可选 0..12；[教程](components/30-input-number.md) |
| slider | value 数字绑定；range 时为升序二元数字数组 | min=0、max=100、step=1、vertical=false、range/reverse/marks；[教程](components/31-slider.md) |
| form | 无 | children=[]、state={}、actions={}、title 字符串绑定；默认 gap=12；events.submit/invalid/reset；[教程](components/32-form.md) |
| formItem | children 恰好一个输入控件 | label 字符串绑定、rules=[]；须位于 form 内；[教程](components/32-form.md) |
| button | text 字符串绑定 | events.click、htmlType=button/submit/reset；submit/reset 按钮须位于 form 内 |
| metric | label 字符串绑定、value 数字绑定 | format，默认 text |
| badge | text 字符串绑定 | tone=accent，可选 success/warning/danger/muted |
| progress | value 数字绑定，范围 [0,1] | text 字符串绑定 |
| table | rows 数组对象绑定、rowKey、columns | rowHeight=32，16..512；pageSize/current、loading/emptyText/bordered；events.rowSelect/sortChange/pageChange |
| lineChart/barChart | series 数组、layout.height.fixed >=120 | title、xAxis/yAxis、legend=true、zoom=false、pan=false |

协议另提供 50 个原生组件，当前共 84 种 type。新增组件的 text/title/label/placeholder/value/options 支持直接字面量或 Binding；扩展属性均支持两种形式。disabled、size 为公共扩展字段；内部 api 字段不接受用户声明。字段精确枚举和范围可通过 Rust `v02::util::component_api(Kind)` 查询，所有跨字段和绑定约束由引擎验证。

| 类别 | 新增 type | 字段、事件、完整示例 |
| --- | --- | --- |
| 导航与切换 | anchor、breadcrumb、dropdown、menu、pagination、steps、tabs、segmented、collapse、carousel | [导航教程](components/33-navigation.md) |
| 输入与树 | autoComplete、cascader、colorPicker、datePicker、mentions、rate、timePicker、transfer、treeSelect、upload、calendar、tree | [输入教程](components/34-native-inputs.md) |
| 展示 | floatButton、icon、avatar、badgeCount、descriptions、empty、image、list、listy、qrCode、timeline | [展示教程](components/35-native-display.md) |
| 反馈 | modal、drawer、popover、tooltip、popconfirm、message、notification、tour、alert、result、skeleton、spin | [反馈教程](components/36-native-feedback.md) |
| 配置与装饰 | configProvider、app、watermark、affix、borderBeam | [配置教程](components/37-native-context.md) |

主值别名为 Tabs/Collapse 的 activeKey，Pagination/Steps/Carousel/Tour 的 current，Menu/Tree 的 selectedKeys；不能同时声明 value 与主值别名。默认主值可本地交互，state Binding 则受外部状态控制。Tabs/Collapse 子组件 id 必须对应 items.key。官网 API 尚未完整实现，实际限制见 [覆盖对照](21-component-coverage.md)。

表格 columns 为 1..64 个 `{id,field,title,width,format?}`；field 是对象直接属性名，rowKey 是唯一字符串字段。width 支持 fixed/fr/auto；最小宽度 48 point，auto 最多抽样前 200 行，数据变化重算，用户拖拽宽度优先。表头/表体共用列宽和水平偏移。固定表头 32 point；表体按固定行高计算可见区间，预取 2 行，仅对裁剪内实际可见行格式化和绘制。单元格支持同列数值、字符串或布尔；排序稳定，null 始终在末尾，选中保存 rowKey。点击同一行不重复 rowSelect；方向键切换选择，Enter 采用宿主激活路径。空表显示“暂无数据”。

系列为 `{id,name,data,xField,yField,monotonicX?}`；id/name/字段非空，最多 32 个系列。xField/yField 是直接对象字段。X 必须为有限数字，Y 为有限数字或 null；坐标和手动轴范围绝对值 <=1e100（避免极值跨度和屏幕转换溢出）。null 是折线断点。Axis 为 `{scale:"linear",min?,max?}`，缺省自动范围；同时给定时 min<max，单边手动范围必要时扩展自动另一边。零跨度扩展，柱图自动 Y 包含零。

非单调系列上限 20,000 点；柱图每系列上限 2,000 点。monotonicX=true 必须实际非递减，折线按屏幕列保留首末与 min/max，按原索引输出；hover 查询原数据，不查询降采样数组。刻度采用 1/2/5 步长，有限循环。图例保存本地可见状态。zoom=true 在绘图区消费滚轮并以指针为锚缩放；pan=true 拖拽平移，双击或聚焦后 Home 恢复自动范围。默认滚轮传给页面。

### 事件与动作

events 是该组件支持的事件名到动作 ID 或动作绑定的对象。固定写法 `{"click":"advance"}` 保持兼容，也支持 `{"click":{"kind":"state","key":"wizard","path":"/action"}}` 和字符串 literal 绑定。绑定必须解析为非空字符串，并引用 actions 中已定义的动作；加载与每次状态更新均校验，非法更新不改变状态。每个事件提交时读取当时状态中的动作 ID，前一个事件的成功状态更新会影响后一个事件的动作选择；一次动作开始后不因后续状态变化而切换其步骤。

事件名按组件契约校验。输入控件和 splitter 的 change 先写状态，再解析事件动作并触发；Text change 写 text，Badge change 写 checked，Table pageChange 写 current。可写 state 绑定仅接受空 path，prop 为只读。Checkbox 单项/Switch 载荷为布尔值，Checkbox 组/Select 多选为字符串数组，Radio 为可用选项字符串，InputNumber/Slider 为范围内数字，range Slider 为升序数字对；数字字符串不自动转换。splitter.change payload 必须为 [0,1] 数字，禁用或 resizable=false 时不允许调整。input/inputNumber 的 submit 由 Enter 提交；button 支持 click；Table 的 rowSelect 为行键字符串，sortChange 为 `{field,descending}`，pageChange 为正整数。

内部事件包含 componentId、targetKey、kind、payload、sequence、generation；序号递增并仅消费一次，隐藏/禁用/过期 generation 的事件丢弃。同一输入产生主值与业务事件时，先提交 change，再提交 click/select 等事件。openChange 等二级属性事件使用固定映射写入 open、pageSize、openKeys、checkedKeys、expandedKeys，载荷类型和成员关系先校验，动作随后读取最新值。原生控件动作可读取 `{"kind":"event"}`；使用 event 的动作必须声明匹配形状的 eventPayload 样本。完整事件列表和载荷见对应教程。

数字控件要求 `-1e12 <= min < max <= 1e12`，step 在 `1e-12..1e12`；value 必须处于范围内。交互时 InputNumber 按 precision 舍入并限制范围，Slider 以 min 为起点按 step 吸附（max 端点始终可达）。程序赋值遵守类型和范围，不自动吸附或舍入。

Form 不可嵌套。FormItem 直属子节点恰好一个基础或新增输入控件，必须绑定可写顶层 state；同一表单不能重复绑定同一字段。rules 最多 32 条，支持 required、minLength/maxLength（Unicode 字符数）、min/max（数字）、pattern（Rust regex）、message；规则须有效且匹配字段类型，字段初始不满足业务规则仍可加载。required 拒绝空白字符串、空数组和 false。点击 htmlType=submit 按钮或在输入框按 Enter，先提交控件事件，再校验当前可见且启用字段；通过调用 form.events.submit，失败显示行内错误并调用 invalid。首次提交后，字段和可见/禁用状态更新会刷新错误。htmlType=reset 原子恢复所有 FormItem 字段的文档初始值（含隐藏和禁用字段），清除校验状态，再调用 reset；若当前约束与初始值冲突则拒绝更新。按钮若同时声明 click，会先执行 click 动作。Form 动作通过 state 读取字段值。

actions.steps 非空。set 的 value 可为任意已匹配类型的绑定；concat.parts 仅为字符串绑定。每次动作读取触发时快照，顺序写入，失败停止后续步骤、保留成功写入、显示步骤诊断。measure/arrange/paint 不执行动作；输入事件统一交给 commit。

### 模块通信（0.2）

module.props 在父作用域解析，内部用 `{"kind":"prop","key":"message","path":""}` 读取；props 只读，不可作为 set/concat/command 的 target 或 input.value。module.emits 用 JSON 样本声明事件载荷类型，module.on 将事件绑定到父级动作。内部动作通过 `{"type":"emit","event":"changed","payload":{"kind":"state","key":"draft"}}` 发出数据，父级处理动作通过 `{"kind":"event","path":""}` 或对象字段路径读取载荷。event 只用于处理动作步骤，不可用于界面属性。处理动作可用可选 eventPayload 样本声明类型，适用于尚未激活的动态监听动作。emit 同步执行父级处理动作，调用深度上限 32，原动作仍保持开始时快照。完整协议与配置片段见 [模块通信](14-module-communication.md)。

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
{"type":"command","program":"python","args":["task.py",{"kind":"state","key":"input"}],"cwd":".","timeoutMs":60000,"target":"job"}
```

`program` 为程序名或路径；`args` 缺省为空，每个参数为字符串或解析为字符串的 literal/state 绑定，点击时取快照。参数逐项传给进程，不执行 shell 拼接；需要 CMD/PowerShell 时显式指定 `cmd.exe`/`powershell.exe` 及参数。程序名通过系统搜索路径查找；含目录的相对程序路径以描述目录为基准。`cwd` 缺省描述目录，相对 cwd 同样以描述目录为基准；`--app` 使用该 JSON 所在目录，默认加载使用 EXE 所在目录。脚本文件参数由被调用程序按 cwd 解析。

`target` 必须指向已定义的结果状态，初始结构如下，字段类型固定且不允许额外字段：

```json
{"status":"idle","running":false,"exitCode":-1,"stdout":"","stderr":"","error":""}
```

后台执行期间立即写 `status=running`、`running=true` 并清空旧结果；完成后写 `succeeded` 或 `failed`、退出码及 stdout/stderr。启动失败或超时的诊断写入 error；`exitCode=-1` 表示没有可用退出码。`timeoutMs` 缺省 60000，范围 1..3600000；超时终止直接子进程并写 `timedOut`。窗口关闭时请求终止正在运行的直接子进程；不管理子进程自行启动的后代进程。stdin 关闭，Windows 下不创建控制台窗口。

同一 target 正在执行时拒绝重复启动，其他 target 可以并行执行。command 步骤启动后立即继续后续步骤，不等待完成，后续步骤仍读取动作开始时快照。完成结果由 GUI 更新线程写回，不在后台线程修改业务状态。输出在结束时更新，每条流保留前 16384 字节，继续消费超出部分避免管道堵塞；以 UTF-8 解码，非法字节替换。继承输出管道的后代进程不会使结果提交无限等待；结束后最多为两条流各等待 100ms。非零退出码通过结果状态报告。

上述 `task.py` 表示应用自行提供的脚本。分发时将所需脚本随应用复制，并按实际目录配置 cwd/args。`--check` 只校验描述，不执行命令或检查程序是否安装。

### 运行与任务日志

桌面宿主提供底部“日志与任务”和“打开日志目录”。命令 stdout/stderr 按原始字节持续写文件，窗口独立显示每条流最近 32 KiB，不影响命令结果状态。可使用 `{"type":"openLogs"}` 动作步骤打开同一窗口，无参数，不读取或修改业务状态。运行日志按 EXE 名和启动会话保存，完整位置、文件和错误边界见 [运行与任务日志](19-runtime-logs.md)。

组件实现与当前验证范围见 [组件实现](09-component-implementation.md)。
