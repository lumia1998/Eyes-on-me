# client-desktop

`client-desktop` 是 Eyes on Me 的桌面采集端，负责监听当前前台应用变化并把活动上报到 Rust 服务端或 Koishi 集成路由。

## 运行

在 `rust-monolith` 根目录执行：

```bash
./_scripts/run-agent.sh
```

也可以直接用 Cargo：

```bash
cargo run -p client-desktop
```

## 配置

启动时会读取或写入根目录的 `client-desktop.config.json`。

可配置项：

- `server_api_base_url`
- `agent_api_prefix`
- `device_id`
- `agent_name`
- `api_token`

支持的环境变量：

- `AGENT_SERVER_API_BASE_URL`
- `AGENT_API_PREFIX` / `AGENT_ROUTE_PREFIX`
- `AGENT_API_TOKEN`
- `AGENT_DEVICE_ID`
- `AGENT_NAME`

### Koishi 集成

如果 Koishi 插件继续兼容默认 Eyes on Me 路由，则保留：

- `server_api_base_url=http://127.0.0.1:8787`
- `agent_api_prefix=`（留空）

如果 Koishi 使用自定义前缀，例如 `/watchme/agent`，则设置：

- `server_api_base_url=http://127.0.0.1:5140`
- `agent_api_prefix=/watchme/agent`

客户端会继续发送标准 JSON body，同时额外附带这些请求头，便于插件侧快速路由与鉴权：

- `x-eyes-on-me-device-id`
- `x-eyes-on-me-agent-name`
- `x-eyes-on-me-message-type`
- `x-eyes-on-me-source`

客户端还会解析服务端返回的非敏感 JSON 字段（如 `ok`、`ack`、`server_time`、`refresh_config`），并忽略任何不受支持的远程指令。

### 扩展 payload

`activity` payload 现在新增两层派生字段：

- `summary`: 更适合机器人命令直接展示
- `meta`: 更适合做聚合、筛选和后续扩展

#### `summary`

`summary` 是给 `watchme.status`、`watchme.today` 这类命令直接拿来拼回复的：

- `summary.currentLabel`
- `summary.statusLine`
- `summary.detailLine`
- `summary.domainKey`
- `summary.pageKey`
- `summary.appKey`
- `summary.appGroup`
- `summary.presence`
- `summary.isBrowser`
- `summary.isActive`
- `summary.isIdle`
- `summary.isLocked`

推荐用法：

- `statusLine` 直接做当前状态主文本
- `detailLine` 做副标题或补充说明
- `domainKey` / `pageKey` 做浏览器页面聚合
- `appKey` / `appGroup` 做应用维度统计

#### `meta`

`meta` 保留更完整的归一化信息，方便 Koishi 侧直接做状态展示和聚合：

- `meta.appKey`
- `meta.appDisplayName`
- `meta.appGroup`
- `meta.isBrowser`
- `meta.isIdle`
- `meta.isLocked`
- `meta.isActive`
- `meta.presenceLabel`
- `meta.activityLabel`
- `meta.windowOrPageTitle`
- `meta.browserFamily`
- `meta.browserName`
- `meta.browserDomain`
- `meta.browserUrl`
- `meta.browserPageTitle`
- `meta.localDate`
- `meta.localTime`
- `meta.localDatetime`

这让插件侧做这些功能时能少写很多解析逻辑：

- `watchme.status`
- `watchme.today`
- `watchme.history`
- `watchme.analysis`
- `watchme.list`

一个简化后的 payload 结构示意：

```json
{
  "type": "activity",
  "payload": {
    "kind": "foreground_changed",
    "summary": {
      "currentLabel": "Google Chrome · github.com",
      "statusLine": "正在使用 Google Chrome 浏览 github.com",
      "detailLine": "Pull requests · anthropics/claude-code",
      "domainKey": "github.com",
      "pageKey": "https://github.com/anthropics/claude-code/pulls",
      "appKey": "chrome.exe",
      "appGroup": "browser",
      "presence": "active",
      "isBrowser": true,
      "isActive": true,
      "isIdle": false,
      "isLocked": false
    },
    "meta": {
      "appKey": "chrome.exe",
      "appDisplayName": "Google Chrome",
      "appGroup": "browser",
      "presenceLabel": "active",
      "activityLabel": "Google Chrome · github.com",
      "browserDomain": "github.com",
      "browserPageTitle": "Pull requests · anthropics/claude-code"
    }
  }
}
```

其中：

- `summary` 优先面向“直接显示”
- `meta` 优先面向“归一化和聚合”
- 原始字段 `app` / `windowTitle` / `browser` 仍然保留，方便需要时继续取底层细节

## 平台实现

- macOS: `NSWorkspace` 前台应用监听，浏览器可进一步读取标签页信息
- Windows: `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` + 轮询兜底
- Linux: `xprop` 轮询当前激活窗口，采集应用、窗口标题和 PID

Linux 当前注意点：

- 需要图形桌面会话
- 需要 `xprop` 在 `PATH` 里
- 目前更适合 X11 / XWayland 环境
- 浏览器场景会尝试从窗口标题里反推域名
- 浏览器域名提取能力不如 macOS，优先依赖窗口标题

## 测试

```bash
cargo test
```
