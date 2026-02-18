# 更新日志

## 1.3.0 - 2026-02-17

### 新功能

- 在 `feishu-platform` 中新增 Webhook 适配器支持
- 新增针对消息与 WebSocket 的完整 E2E 测试用例

### 重构

- 改进 `clawrs-feishu` 中的配置适配器与消息规范化逻辑

### 其他

- 优化 `.gitignore` 以实现更精简的代码提交

## 1.2.1 - 2026-02-16

### 文档

- 在 README 中完善性能与安全特性的说明

## 1.2.0 - 2026-02-16

### 新功能

- 支持 WebSocket 连接生命周期信号（已连接、断开连接、错误）
- 新增内存消息去重，处理 Feishu 重复事件
- 支持运行时状态监控与运行状况报告
- 改进消息插件中的目标解析与消息规范化

### 修复

- 补丁 `open-lark` SDK 以处理事件反序列化中的可空 `user_id`
- 改进 IM 平台错误映射与处理

## 1.1.0 - 2026-02-16

### 新功能

- 新增通用 `release-skills` 技能，支持多语言自动发布流程

### 文档

- 重命名 design-spec.md 并清理项目文档

## 1.0.2 - 2026-02-16

### 文档

- 更新仓库地址为 Clawrs-Channel-Feishu
- 新增与 OpenClaw Feishu TypeScript 的功能对比，将 .cursor 加入 gitignore

## 1.0.1 - 2026-02-16

### 重构

- 在 feishu-domain 中统一 DmPolicy 推导逻辑，新增 criterion 基准测试
- 简化 clawrs-feishu 中的连接模式检查与配置访问
- 在 clawrs-feishu-node 中提取 getChannelConfig，简化 normalizeTarget

### 文档

- 新增性能优化指南（docs/performance-optimization.md）

## 1.0.0 - 2026-02-16

### 破坏性变更

- 移除 zeroclaw-feishu、feishu-auth、feishu-bot、feishu-event、feishu-im 组件
- 迁移至 OpenClaw 架构（clawrs-feishu、feishu-platform）

### 新功能

- OpenClaw 插件 (@openclaw/clawrs-feishu)，gateway.startAccount 支持 WebSocket 入站
- N-API 桥接 (clawrs-feishu-node) 用于 Node.js 集成
- Rust 核心：feishu-domain、feishu-kernel、feishu-platform、clawrs-feishu
- E2E 测试套件（Bruno + shell 脚本）用于 OpenClaw gateway
- 安全导向的 .gitignore（排除 .env、凭证、node_modules、*.node）

 