# client-desktop

`client-desktop` 是 Eyes on Me 的桌面采集端，负责监听当前前台应用变化并把活动上报到 Rust 服务端。

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
- `device_id`
- `agent_name`
- `api_token`

支持的环境变量：

- `AGENT_SERVER_API_BASE_URL`
- `AGENT_API_TOKEN`
- `AGENT_DEVICE_ID`
- `AGENT_NAME`

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
