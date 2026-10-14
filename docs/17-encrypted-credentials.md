# Windows 加密凭据与 API 认证

接口凭据通过 Windows DPAPI 的当前用户保护模式加密，保存到本地用户目录。app.json 只引用凭据名称；HTTP 后台线程在发送请求前解密并加入请求头，不将解密结果写入业务 state、插件 props、启动参数或日志。

## 配置

```json
{
  "credentialStore": "my-monitor",
  "dataSources": {
    "metrics": {
      "url": "https://example.com/api/metrics",
      "headers": {
        "Authorization": {"kind":"credential","name":"api-token","prefix":"Bearer "},
        "X-API-Key": {"kind":"credential","name":"api-key"}
      }
    }
  }
}
```

上面是认证字段片段；数据源的 statusTarget、responseSample 和 mappings 仍按 HTTP 契约必填。完整可运行配置为 examples/http-auth.json。

顶层 credentialStore 是用户本地凭据库的命名空间。库与凭据名称均为 1..64 位字母数字、下划线、连字符，不接受 Windows 保留设备名；名字不区分大小写，存储时规范化为小写。credential 引用只能用于 dataSources.headers，不能用于界面绑定、query、URL、body 或 command 参数。prefix 默认空串，最多 128 字节，禁止非法请求头字符；Bearer 使用 `Bearer `（包含末尾空格），API Key 通常无 prefix，Basic 可存已编码凭据并使用 `Basic `。

Authorization、Proxy-Authorization、Cookie、X-API-Key、API-Key、X-Auth-Token 不允许明文字符串或 state 绑定，必须使用 credential 引用。自定义认证头也应使用 credential 引用；普通 Accept 等业务头仍支持字符串和 state。旧的明文认证头配置会校验失败，应通过隐藏输入录入凭据并替换配置引用。本轮没有真实用户凭据要迁移，不自动读取或改写用户配置。

## 录入、更新与删除

使用 PowerShell 7 的隐藏输入工具：

```powershell
.\scripts\set-credential.ps1 -Store my-monitor -Name api-token
```

脚本默认调用 target-http/release/desktop-app-engine.exe；可指定独立应用 EXE：

```powershell
.\scripts\set-credential.ps1 -Store my-monitor -Name api-token -Runtime .\exports\EncryptedAuthApp\EncryptedAuthApp.exe
```

输入不回显，脚本通过重定向的标准输入传给引擎。凭据不作为命令行参数、PowerShell 命令历史或临时明文文件传递。再次录入同名凭据会原子替换密文；下一次请求读取新值，已经开始发送的请求保持当次值。

引擎管理命令：

```powershell
# 列出名称，不显示值，也没有导出解密值的命令。
.\target-http\release\desktop-app-engine.exe --credential-store my-monitor --credential-list
.\target-http\release\desktop-app-engine.exe --credential-store my-monitor --credential-delete api-token

# 可从 app.json 中读取 credentialStore。
.\target-http\release\desktop-app-engine.exe --app .\examples\http-auth.json --credential-list

# 检查引用凭据能否解密，不发送 HTTP 请求。
.\target-http\release\desktop-app-engine.exe --app .\examples\http-auth.json --check-credentials
```

底层 --credential-set NAME 仅从标准输入读取单行 UTF-8（最大 8192 字节），拒绝控制台明文输入、空值、换行和 NUL。可通过 --credential-store STORE 或 --app 指定库；每次只执行一种管理命令，不能同时执行配置校验。

## 存储与边界

密文位于 `%LOCALAPPDATA%\DesktopAppEngine\credentials\STORE\NAME.dpapi`。文件只包含版本标记和 DPAPI 密文，不保存明文值、用户名/密码或应用内置解密密钥。DPAPI 未使用 LOCAL_MACHINE 模式，也未指定可移植密码；Windows 用户登录凭据提供保护。附加 entropy 包含协议版本、库名和凭据名称，防止密文被直接互换到其他名称后使用。更新采用临时密文文件和 Windows 原子替换，临时文件也只保存密文。

加密提供的是落盘保护，不是进程沙箱。同一个 Windows 用户下的程序仍可能调用 DPAPI；发送认证请求时 HTTP 客户端必须短暂持有明文请求头。引擎用 zeroize 清除管理输入、解密缓冲和拼接缓冲，并将请求头标记为 sensitive；第三方 HTTP 客户端及系统内存中的所有复制不保证清零。PowerShell 工具的 BSTR 会清除，但 .NET 临时字符串也可能在回收前存在。

生产接口使用 HTTPS 保护传输。示例仅在 127.0.0.1 上使用 HTTP。不提供共享密码加密、跨用户自动迁移、凭据自动登录/OAuth refresh token 续期、操作系统凭据 UI 或 mTLS 客户端证书配置。

## 配置校验与运行错误

--check、--check-plugins 只校验 credential 引用及库名，不读取或解密真实凭据，因此制作机可以在没有客户 Token 时导出应用。--check-credentials 专门检查存在性、DPAPI 解密和 HTTP 头可用性，只输出凭据名称；没有网络认证验证。

运行时凭据缺失、损坏或用户上下文不符，会在 HTTP 请求状态中记录 failed/statusCode=0/error，业务数据保持不变且不发送网络请求。认证被服务器拒绝时正常记录 401/403，保留旧业务数据；无自动刷新 Token 或 401 后重试逻辑。

## 分发与演示

密文保存在每位用户的本地目录，package.ps1 只复制应用配置和插件资源，不携带制作机凭据。客户需要在运行应用的同一 Windows 用户上下文下录入凭据。普通 app.json/EXE 更换、应用目录移动或 EXE 改名不改变 credentialStore，已有凭据可以继续使用；修改库名或换 Windows 用户后需要另行录入。

测试示例：

```powershell
# 终端一：隐藏输入 API 期望的测试 Token。
python .\scripts\http-auth-demo-server.py

# 终端二：隐藏输入同一个测试 Token。
.\scripts\set-credential.ps1 -Store http-auth-demo -Name api-token
.\target-http\release\desktop-app-engine.exe --app .\examples\http-auth.json --check-credentials
.\target-http\release\desktop-app-engine.exe --app .\examples\http-auth.json
```

页面点击请求会认证并显示计数。修改凭据为另一个值后请求返回 401；删除凭据后请求在本机失败，不发送 Token。Python 服务只读取隐藏输入，不接收 Token 命令行参数。

## 验证

新增 4 项 Windows DPAPI/CLI 专项及 1 项真实 HTTP 加密认证专项，覆盖随机化密文、落盘不含测试明文、更新/列出/删除、非法名称/长度、损坏和库/名称错绑、标准输入无回显、离线配置校验及显式解密校验、正确认证、错误 Token 的 401、缺失凭据不发请求、轮换后恢复，以及明文认证头拒绝。测试使用隔离目录和生成的测试库，不读写用户真实凭据。

跨 Windows 账号/机器的实际迁移未测试；此限制由使用当前用户 DPAPI 的模式决定。无原生窗口、鼠标或前台操作。最终回归和分发记录见 evidence/credential-verification.txt。

最终 fmt、严格 Clippy、80 项测试（新增 5 项）、十一份示例 Schema 验证及 release 构建通过。独立包 `exports/EncryptedAuthApp/` 在项目外目录、PATH 仅含 Windows System32 时 --check 退出 0。隐藏输入脚本的自动测试只模拟 Read-Host，实际执行其进程/stdin/DPAPI 存储路径；测试密文与命令输出均不含测试 Token，独立 EXE 解密校验、删除、无凭据配置校验通过。演示服务器的重定向 stdin 在 Windows getpass 下曾超时，已区分终端隐藏输入与重定向 stdin 并复验成功。
