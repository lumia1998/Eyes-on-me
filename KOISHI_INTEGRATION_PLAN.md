# Eyes on Me + Koishi 整合方案建议书 (v2.4)

本方案旨在将 `Eyes on Me` 的桌面采集端与 `Koishi` 机器人框架整合，建立以 `watchme` 为核心的指令体系。

## 1. 指令体系设计 (watchme 家族)

所有敏感操作默认仅限机器人主人（权限等级 5）调用，查询类操作可授权给等级 3 以上用户。

- **`watchme [deviceid]`**: 
  - **功能**: 实时截屏。触发客户端拍照并回传 JPEG 图片。
  - **权限**: Level 5 (主人)。
  
- **`watchme.status`**: 
  - **功能**: 实时简报。显示所有设备的在线状态及当前正在运行的具体应用/窗口。
  - **权限**: Level 3+ (授权用户)。

- **`watchme.today`**: 
  - **功能**: 查询今日（00:00:00 至今）全局活跃时长与 Top 应用。
  - **权限**: Level 3+ (授权用户)。

- **`watchme.history [days]`**: 
  - **功能**: 历史回顾。查询过去 X 天（如 3, 7, 30）的活动汇总与趋势。
  - **权限**: Level 3+ (授权用户)。

- **`watchme.analysis`**: 
  - **功能**: 深度分析。调用 `koishi-plugin-puppeteer` 渲染今日或指定时段的分析图表。
  - **权限**: Level 3+ (授权用户)。

- **`watchme.list`**: 
  - **功能**: 基础列表。仅显示设备名、系统平台及连接状态。
  - **权限**: Level 3+ (授权用户)。

- **`watchme.grant <user:user>`**: 
  - **功能**: 授权管理。主人授权指定用户访问查询类指令。
  - **权限**: Level 5 (主人)。

- **`watchme.push --time <HH:mm>`**: 
  - **功能**: 自动定时。设置每日报表的推送时间。
  - **权限**: Level 5 (主人)。

## 2. 采集端 (client-desktop) 配置

- **Device ID**: 修改 `client-desktop.config.json` 中的 `"device_id"` 为友好名称。
- **上报地址**: 环境变量 `AGENT_SERVER_API_BASE_URL` 指向 Koishi 路由。

## 3. 核心逻辑实现建议

### 3.1 历史数据处理
- **聚合统计**: 针对 `history` 指令，需在数据库中按天聚合时长，并排除非活跃状态（Idle/Locked）。
- **时区对齐**: 确保所有历史查询都基于用户本地时区的自然天进行。

### 3.2 截屏触发机制
- **轮询响应**: 客户端在发送 Activity 数据时，Response 携带 `CAPTURE_REQUEST` 信号。
- **回传通道**: 客户端上传图片后，Koishi 通过 `session.send()` 异步推送。

## 4. 开发建议步骤

1. **统一指令注册**: 在 Koishi 中建立 `watchme` 指令树。
2. **算法对齐**: 在 TS 中实现与 Rust 一致的 30s 间隔裁剪算法。
3. **状态追踪**: 维护内存中的设备心跳，实现 `watchme.status` 的即时响应。
4. **报表渲染**: 利用 `puppeteer` 实现历史趋势图与 Top 应用比例图。

---
*文档更新日期: 2026-03-27*
