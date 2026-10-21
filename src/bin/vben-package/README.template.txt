{{Name}}

双击 {{Name}}.exe 启动项目构建工作台，应用定义已编译进 EXE。
在连接设置中填写 SSH 地址或本机 SSH 配置别名，以及 Turbo 工作区绝对路径。
设置保存到当前用户 %LOCALAPPDATA%\DesktopAppEngine\apps\{{Name}}\connection.dat。
本机需 Windows OpenSSH 客户端及免交互 SSH 认证；服务器需 Bash、Node.js、Git 和 pnpm。

启动、保存连接和重新扫描时获取仓库分支；每行 Fetch 可单独刷新。
下拉展开读取缓存，最多展示 10 项。选择分支执行 checkout，Pull 在对应仓库执行。
根仓库切换或 Pull 后重新扫描，确认工作区前禁止构建。
部分项目按实际 build:dev:<业务> 脚本串行构建，全选执行 pnpm build:dev。
每批核对分支与提交后清理一次根目录 .turbo；每个进程启动前再次校验，失败停止后续构建。

右侧显示批次和进程日志。清空显示仅隐藏预览；手动滚动暂停自动跟随。
完整日志位于 %LOCALAPPDATA%\DesktopAppEngine\logs\app-{{Name}}\，每次启动独立会话。
无窗口配置校验（无远端操作）：& '.\{{Name}}.exe' --check
无窗口只读扫描：& '.\{{Name}}.exe' --inspect
可使用 --theme light|dark|system 指定主题；应用定义修改后重新打包。
