# Claw 飞书通道 — ZeroClaw / OpenClaw 快速安装

本文档提供 **zeroclaw** 与 **openclaw** 两种运行时的飞书通道快速安装步骤。

---

## 前置条件

- **飞书应用**：在 [飞书开放平台](https://open.feishu.cn/) 创建应用，开通 `im:message`、`bot:read`、WebSocket 权限
- **凭证**：`app_id`、`app_secret`

---

## 一、ZeroClaw 快速安装

ZeroClaw 为 Rust 实现，单二进制，内存 <5MB，适合边缘设备或轻量部署。

### 1. 安装 ZeroClaw

```bash
# 克隆 zeroclaw 仓库
git clone https://github.com/theonlyhennygod/zeroclaw.git
cd zeroclaw

# 构建（启用 feishu 通道）
cargo build --release --features feishu

# 可选：全局安装
cargo install --path . --force --features feishu
```

### 2. 配置飞书通道

编辑 `~/.config/zeroclaw/config.toml`（或通过 `zeroclaw onboard` 生成）：

```toml
[channels.feishu]
enabled = true
app_id = "cli_your_app_id"
app_secret = "your_app_secret"
domain = "feishu"  # 或 "lark" 国际版

# 策略（可选）
dm_policy = "pairing"      # pairing | open | allowlist | disabled
group_policy = "allowlist" # allowlist | open | disabled
allow_from = ["ou_xxx"]     # 白名单 open_id
```

### 3. 启动 Gateway

```bash
# 默认端口 8080
zeroclaw gateway --port 8080

# 或作为系统服务
zeroclaw service install
zeroclaw service start
```

### 4. 验证

```bash
# 健康检查
curl http://127.0.0.1:8080/health

# E2E 测试（需配置 .env）
cd claw-feishu-channel/e2e
cp .env.example .env   # 填入 FEISHU_APP_ID, FEISHU_APP_SECRET
./run-tests.sh zeroclaw
```

---

## 二、OpenClaw 快速安装

OpenClaw 为 Node.js 实现，通过 `@openclaw/clawrs-feishu` 插件接入 Rust 飞书通道。

### 1. 安装 OpenClaw

```bash
# 通过 Homebrew（macOS）
brew install openclaw

# 或 npm 全局安装
npm install -g openclaw
```

### 2. 构建并安装飞书插件

```bash
cd claw-feishu-channel

# 构建原生绑定
cd packages/clawrs-feishu-node && npm run build && cd ..

# 构建插件
cd packages/clawrs-feishu && npm install && npm run build && cd ../..

# 安装到 OpenClaw（link 模式，开发时修改即生效）
openclaw plugins install -l ./packages/clawrs-feishu
```

### 3. 配置飞书通道

编辑 `~/.openclaw/openclaw.json`，在 `channels` 下添加：

```json
{
  "channels": {
    "clawrs-feishu": {
      "enabled": true,
      "accounts": {
        "main": {
          "appId": "cli_your_app_id",
          "appSecret": "your_app_secret",
          "domain": "feishu",
          "dmPolicy": "pairing",
          "groupPolicy": "allowlist"
        }
      }
    }
  }
}
```

### 4. 禁用原飞书插件（可选）

原 `@openclaw/feishu` 为内置插件，无法删除，可保持 disabled 状态。当前配置已使用 `clawrs-feishu` 作为飞书通道。

### 5. 启动 Gateway

```bash
# 启动服务
openclaw gateway start

# 或前台运行
openclaw gateway run

# 检查状态
openclaw gateway status
```

默认端口：**18080**（可在 config 中修改）。

### 6. 验证

```bash
# 健康检查
curl http://127.0.0.1:18080/health

# E2E 测试
cd claw-feishu-channel/e2e
# 确保 .env 中 OPENCLAW_GATEWAY_URL=http://127.0.0.1:18080
./run-tests.sh openclaw
```

---

## 三、对比速查

| 项目       | ZeroClaw              | OpenClaw                    |
|------------|-----------------------|-----------------------------|
| 运行时     | Rust 单二进制         | Node.js                     |
| 内存       | <5MB                  | ~100MB+                     |
| 默认端口   | 8080                  | 18080                       |
| 配置路径   | `~/.config/zeroclaw/` | `~/.openclaw/openclaw.json` |
| 通道配置   | `channels.feishu`     | `channels.clawrs-feishu`    |
| 插件安装   | 无需插件              | `openclaw plugins install`  |

---

## 四、E2E 测试

```bash
cd claw-feishu-channel/e2e
cp .env.example .env
# 编辑 .env 填入 FEISHU_APP_ID, FEISHU_APP_SECRET, TEST_CHAT_ID 等

# API 测试（无需 gateway）
./run-tests.sh api

# ZeroClaw 测试（需 zeroclaw gateway 运行在 8080）
./run-tests.sh zeroclaw

# OpenClaw 测试（需 openclaw gateway 运行在 18080）
./run-tests.sh openclaw

# 全部测试
./run-tests.sh all
```

---

## 五、故障排查

| 现象                     | 可能原因           | 处理                         |
|--------------------------|--------------------|------------------------------|
| `gateway not responding` | Gateway 未启动     | `zeroclaw gateway` 或 `openclaw gateway start` |
| `code=10014`             | app_secret 错误    | 检查飞书开放平台凭证         |
| `code=230002`            | chat_id 无效       | 更新 TEST_CHAT_ID，确保 Bot 在对话中 |
| `plugin not found`       | 插件未正确安装     | `openclaw plugins install -l ./packages/clawrs-feishu` |

---

**版本**: 1.0.0  
**更新**: 2026-02-16
