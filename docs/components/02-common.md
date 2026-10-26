# 02 公共属性与布局

[组件总览](README.md) / 公共属性与布局

所有组件共享身份、尺寸、样式、显示与禁用规则。各组件页只列专属字段；本页统一说明公共 API。

## 公共 API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 组件类型字符串 | 必填 | 见 [组件总览](README.md) |
| `id` | 非空字符串 | 必填 | 全树唯一，保持控件身份；组件树深度上限 64 |
| `layout` | 布局对象 | `{}` | 尺寸、约束、内边距；容器还支持子项排列 |
| `style` | 样式对象 | `{}` | 颜色、字体、边框与组件适用的状态样式 |
| `class` | 静态字符串 | `""` | 原生 Tailwind 工具类子集，见下文；不接受绑定 |
| `visible` | 布尔绑定 | `true` | 隐藏后不占布局空间，也不响应输入 |
| `enabled` | 布尔绑定 | `true` | 禁用后保留布局，停止业务交互 |
| `disabled` | 布尔或绑定 | `false` | 禁用组件及其后代；enabled=false 或 disabled=true 均会禁用 |
| `size` | small / medium / large 或绑定 | medium | 影响支持尺寸的控件自然尺寸及默认字号；显式 layout/style 优先，配置容器可提供默认值 |
| `events` | 事件名到动作的映射 | `{}` | 仅接受组件声明的事件；容器中 Splitter 支持 change，Form 支持 submit/invalid/reset |

父容器隐藏或禁用时，后代也受影响。可选结构字段应省略，不能填 `null`。未知字段会被拒绝。

## 数据绑定

| 写法 | 用途 | 示例 |
| --- | --- | --- |
| `literal` | 固定 JSON 值 | `{"kind":"literal","value":"说明"}` |
| `state` | 当前作用域状态 | `{"kind":"state","key":"stats","path":"/cpu"}` |
| `prop` | 模块内部读取父级传入值 | `{"kind":"prop","key":"message"}` |
| `event` | 控件、模块或插件事件处理动作读取载荷 | `{"kind":"event","path":"/message"}` |

`path` 缺省为空，遵循 JSON Pointer：`/cpu` 读取对象字段，`/0/name` 读取数组第一项字段，`~0` 表示 `~`，`~1` 表示 `/`。缺少字段会报错。`event` 绑定只用于事件处理动作步骤，不能作为界面属性。

原有文字属性需要字符串绑定，数字不能直接绑定 `text`；显示数字可用 [Metric](13-metric.md)，或在表格中配置格式。Input/Radio 的 value 为字符串，Select 单选为字符串、多选为字符串数组；Checkbox 单项/Switch 为 boolean，Checkbox 组为字符串数组；InputNumber/Slider 为 number，范围 Slider 为二元数字数组。新增组件的文字和值可直接使用字面量或 Binding，属性表中标注“或绑定”的字段也支持两种形式。可写 state 绑定的 path 必须为空，prop 只读；FormItem 管理的控件必须绑定 state。各组件的完整 API 和示例见 [组件总览](README.md) 中的对应教程。

状态类型由初始值确定，更新时必须保持类型；对象字段结构保持一致，数组元素保持同类结构。单独初始化为 `null` 的顶层状态只接受 `null`。局部状态与父子传值见 [Module](08-module.md)。

## 尺寸和剩余空间

| 属性 | 类型 / 默认值 | 说明 |
| --- | --- | --- |
| `layout.width` / `height` | 轨道对象 / `auto` | 三选一：`{"auto":true}`、`{"fixed":240}`、`{"fr":1}` |
| `minWidth` / `minHeight` | 非负数 / 0 | 最小尺寸 |
| `maxWidth` / `maxHeight` | 非负数 / 内部上界 | 最大尺寸，必须不小于对应最小值 |
| `padding` | 非负数 / 0 | 四边等距内边距 |
| `shrink` | 非负数 / auto、fr 为 1；fixed 为 0 | 空间不足时参与收缩的权重 |

布局数值采用逻辑 point，尺寸和约束须为有限值且在 0..1,000,000 内；`fr` 必须大于 0。尺寸包含边框和 `padding`。

`fixed` 提供固定基准尺寸；`fr` 在父容器主轴分配剩余空间。在 Row 中用 `width.fr` 分宽，在 Column 中用 `height.fr` 分高。`auto` 使用测量尺寸；父容器默认 `align: "stretch"` 时，未固定的交叉轴尺寸通常铺满。`fr` 需要父级提供有限空间，无界测量时采用内容尺寸。

## 容器排列 API

以下字段用于容器，放在 `layout` 内。新增 Flex、Space、Layout 及其区域、Masonry 均遵守这些规则；Splitter 的间距与面板排布由自身 API 控制。叶组件声明这些字段会被拒绝。

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `gap` | 非负数 | 0 | 相邻子项间距；Grid 行列共用 |
| `rowGap` / `columnGap` | 可选非负数 | 回退到 gap | 分别控制纵向、横向间距 |
| `align` | `stretch` / `start` / `center` / `end` | `stretch` | 交叉轴对齐；Grid 控制单元格内对齐 |
| `justify` | `start` / `center` / `end` / `spaceBetween` / `spaceAround` / `spaceEvenly` | `start` | 主轴剩余空间对齐；Grid/Masonry 不接受该字段 |
| `wrap` | 布尔 | `false` | Row、横向 Flex/Space 可设置为 `true` |
| `overflow` | `clip` / `scroll` | `clip` | 裁剪或滚动；scroll 预留 12 point 双轴滚动槽 |

Row 的主轴为横向，Column、Panel、Card、Module 为纵向。`justify` 只有存在剩余空间时才有可见效果；子项 `fr` 已占满主轴时一般没有额外空间可分。

Flex/Space 通过 `vertical` 切换主轴；Space 默认 gap=8、align=center。Layout 的直属子节点包含 Sider 时横向，否则纵向；Header/Sider/Content/Footer 内部纵向排列。详见 [Flex](21-flex.md)、[Layout](24-layout.md)、[Splitter](25-splitter.md) 和 [Masonry](26-masonry.md)。

需要纵向滚动时，给容器有限高度，再设置 `overflow: "scroll"`。普通 Column 可能收缩子项以适应高度；固定子项高度或适当设置 `shrink: 0` 可保留内容尺寸。嵌套滚动先由最深容器消费，到边界后向外层传递。

## 样式 API

样式不继承，子节点需要独立指定。以下默认值为公共默认，各组件可能有自己的绘制规则。

| 属性 | 类型 / 默认值 | 适用说明 |
| --- | --- | --- |
| `background` / `color` / `borderColor` | 主题 Token 或十六进制颜色 / 按组件与主题 | 背景、前景、边框色 |
| `borderWidth` | 非负数 / 组件相关 | Button、Panel/Card、Input/Select/InputNumber 默认 1，其余公共默认 0；显式 0 可关闭基础边框。Panel/Card 同时受 `bordered` 控制 |
| `radius` | 非负数 / 组件相关 | Panel/Card 默认 6，Button 默认 4，其余公共默认 0；显式 0 保持直角 |
| `fontSize` | 非负数 / 通常 16 | 表格及图表内容绘制默认 14；Metric 数值按字号放大 |
| `fontFamily` | `proportional` / `monospace` | 比例字体或等宽字体 |
| `fontWeight` | `normal` / `bold` | 等宽字体优先于字重 |
| `textWrap` | 布尔 / `true` | Text 与 Button 的文字换行；Badge 强制单行 |
| `textAlign` | `start` / `center` / `end` | Text 与 Button 文字对齐；center 同时垂直居中。Text 带 strong/italic/underline/delete/code 或 ellipsis 时同样生效 |
| `textOverflowTooltip` | 布尔 / `true` | 使用文字绘制辅助函数的截断提示，日志可关闭 |
| `paddingX` / `paddingY` | 非负数 / 控件相关 | Button 默认 10/6；Input 作为编辑框内边距，Select 作为文字左边距 |
| `hover` / `pressed` / `disabled` | 状态样式对象 | Button、Input、Select 使用；含 background/color/borderColor。Input 的编辑框背景与边框按状态切换 |

公共样式由组件各自绘制：Panel/Card 只画一层背景和边框，`bordered:false` 时不画边框；Input/Select 的 `background`、`borderColor`、`borderWidth`、`radius` 作用于控件外框。

可用 Token：`surface`、`surfaceRaised`、`textPrimary`、`textMuted`、`border`、`accent`、`success`、`warning`、`danger`、`focus`、`primary`、`onPrimary`。Token 随主题变化；`#RRGGBB` 与 `#RRGGBBAA` 保持精确颜色。

## JSON class 工具类

组件可直接声明 `"class": "px-4 py-2 bg-blue-500 rounded-lg hover:bg-blue-600"`。加载时使用 `hypen-tailwind-parse` 解析为现有 `style` / `layout`，测量、排列、绘制共用最终属性。无需 CSS 文件或构建 Tailwind。可运行示例：[utility-classes.json](../../examples/utility-classes.json)，全部组件展示也使用了类名。

优先级为组件默认值 < 从左到右的 class < 显式 `style` / `layout`。同属性后写覆盖；`style.hover` 等状态对象按颜色字段合并，显式状态字段优先。显式 `layout.width` 整体替换类名尺寸；显式 `layout.gap` 覆盖类名行列间距，之后可用显式 `rowGap` / `columnGap` 分别指定。`radius: 0`、`borderWidth: 0`、`paddingY: 0` 均保留。状态颜色在状态激活时覆盖基础色，优先级为 disabled > active > hover。

类名不继承，也不接受 state/prop 绑定；每个节点最多 256 个类名、16384 UTF-8 字节。单位转换固定为 `1rem = 16 point`、`1px = 1 point`，例如 `p-4` 为 16 point；跟随显示缩放，字号不会改变 rem 基准。颜色表采用解析库的十六进制颜色，随主题保持固定；需要主题 Token 时继续用显式 `style`。

| 类别 | 支持写法 | 原生含义与限制 |
| --- | --- | --- |
| 颜色 | `bg-blue-500`、`text-slate-600`、`border-gray-300`、`bg-transparent`、`bg-black/50`、`bg-[#112233]` | 映射 background/color/borderColor；支持调色板、十六进制及可解析的 rgba；不支持 currentColor、CSS 变量或渐变 |
| 字号 | `text-xs` 至 `text-9xl`、`text-[18px]` | 只设置 fontSize，行高使用原生字体度量；不应用 Tailwind 的附带 line-height |
| 字体 | `font-normal`、`font-bold`、`font-sans`、`font-mono` | 两种字重、比例/等宽字体；不支持 semibold 或 serif |
| 文字 | `text-left/center/right/start/end`、`text-wrap/nowrap`、`whitespace-normal/nowrap` | 映射 textAlign/textWrap，适用范围同现有 style |
| 边框和圆角 | `border`、`border-0/2/4/8`、`border-[3px]`、`rounded`、`rounded-none/sm/md/lg/xl/2xl/3xl/full`、`rounded-[10px]` | 四边统一边框和四角统一圆角；border-* 宽度默认使用主题边框色。Button 的 shape/variant、Card 的 bordered、输入焦点/校验反馈仍按组件规则生效 |
| 内边距 | `p-4`、`px-4`、`py-2`、`p-[12px]` | 容器只接受 `p-*` → layout.padding；Button/Badge/Input 接受 p/px/py → style.paddingX/Y；不支持单边 padding。Input 内边距须为 0..127 的整数 point |
| 尺寸 | `w-64`、`h-12`、`w-auto`、`h-auto`、`w-[240px]`、`min-w-[80px]`、`max-h-96` | 固定/自动尺寸及 min/max，支持 px/rem；百分比、w-full、视口尺寸不支持，剩余空间用显式 fr |
| 容器间距 | `gap-4`、`gap-x-3`、`gap-y-2`，可用 `[12px]` | gap/columnGap/rowGap；后写 gap 重置之前的轴向间距 |
| 容器对齐 | `items-start/center/end/stretch`、`justify-start/center/end/between/around/evenly` | 映射 align/justify；沿用各容器限制，Grid/Masonry 不接受 justify |
| 换行与滚动 | `flex-wrap/nowrap`、`overflow-hidden/clip/auto/scroll` | 映射 layout.wrap/overflow；换行只用于支持换行的横向容器，auto/scroll 均为引擎双轴滚动 |
| 状态 | `hover:bg-blue-600`、`active:text-white`、`disabled:border-gray-300` | 仅 Button/Input/Select/InputNumber 的背景、文字、边框颜色；active 映射 pressed；输入框保留原生焦点和校验反馈 |

这是原生布局的工具类子集。组件类型决定结构与主轴，使用 row/column/flex/grid 等节点；`flex`、`grid` 类名、响应式前缀、dark、组合状态、focus、important、margin、阴影、动画、变换和行高暂不支持。未知类名和无法映射的属性会带组件 ID 与类名报错；映射后的属性仍经过组件和布局语义校验。不要通过原版 Tailwind 的 CSS 层叠顺序推断本项目的优先级。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "Class 示例", "window": {"width": 640, "height": 360}},
  "state": {},
  "root": {"type": "column", "id": "page", "class": "p-6 gap-4 items-start", "children": [
    {"type": "text", "id": "heading", "text": {"kind": "literal", "value": "工具类样式"}, "class": "text-2xl font-bold text-blue-600"},
    {"type": "button", "id": "button", "text": {"kind": "literal", "value": "悬停查看颜色"}, "class": "px-4 py-2 bg-blue-500 text-white border-0 rounded-lg hover:bg-blue-600 active:bg-blue-700"}
  ]},
  "actions": {}
}
```

## 示例：控制显示和禁用

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "显示与禁用", "window": {"width": 640, "height": 380}},
  "state": {"detailsVisible": true, "canSave": false, "status": "尚未保存"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 12},
    "children": [
      {"type": "text", "id": "details", "text": {"kind": "literal", "value": "这段详情可以隐藏"}, "visible": {"kind": "state", "key": "detailsVisible"}},
      {"type": "button", "id": "hide", "text": {"kind": "literal", "value": "隐藏详情并允许保存"}, "events": {"click": "prepare"}},
      {"type": "button", "id": "save", "text": {"kind": "literal", "value": "保存"}, "enabled": {"kind": "state", "key": "canSave"}, "events": {"click": "save"}},
      {"type": "text", "id": "status", "text": {"kind": "state", "key": "status"}}
    ]
  },
  "actions": {
    "prepare": {"steps": [{"type": "set", "target": "detailsVisible", "value": {"kind": "literal", "value": false}}, {"type": "set", "target": "canSave", "value": {"kind": "literal", "value": true}}]},
    "save": {"steps": [{"type": "set", "target": "status", "value": {"kind": "literal", "value": "已保存"}}]}
  }
}
```

预期效果：保存按钮初始禁用；点击第一个按钮后详情消失，保存按钮变为可用。点击保存更新状态文字。

## 事件与动作

`events` 中填写动作 ID，例如 `{"click":"save"}`；对应动作必须在当前或祖先作用域中存在。也支持将动作 ID 绑定到字符串状态，见 [动态事件](../11-dynamic-event-binding.md)。

输入控件先把新值写回绑定状态，再触发 change 动作。[Form](32-form.md) 在提交时校验字段，通过后执行 submit 动作。控件事件动作可读取已更新状态；有字符串、数字、布尔、数组或对象载荷的事件，也可声明匹配类型的 eventPayload 样本后通过 `kind: "event"` 读取。载荷形状见各组件教程，按钮 click 等事件载荷为 null，不提供鼠标坐标或 DOM 事件。模块接口使用 [emit/on](08-module.md)，插件事件遵守其 manifest。

动作的步骤按顺序执行，但每次动作读取开始时的状态快照：前一步写入不会改变后续步骤的读取快照。失败停止后续步骤，保留之前成功写入。命令、HTTP 和插件调用的异步规则见各自专题。

## 常见问题

- **能写 CSS、百分比宽度或表达式吗？** 当前使用本页的有限 JSON 字段；没有 CSS、`100%`、模板表达式、computed 或 watch。
- **设置标题为什么报错？** `title` 仅用于支持它的组件，例如 Panel、Card 和图表；Row、Column、Grid 不接受。
- **为什么改父级文字颜色不影响孩子？** 样式不继承，需要配置子组件。
- **Panel/Card 的边框为什么不按 borderWidth 显示？** 先确认 `bordered` 不是 false；启用时 `borderWidth` 大于 0 使用该宽度，否则使用 1 像素主题边框。
- **为什么容器没有出现滚动条？** 检查是否设置 `overflow: "scroll"`、区域是否有有限高度，以及内容是否确实超过该区域。

下一步：[Row](03-row.md) · [Column](04-column.md) · [Grid](05-grid.md) · [布局组合](20-recipes.md)。
