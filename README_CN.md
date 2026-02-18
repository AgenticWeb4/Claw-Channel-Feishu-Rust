# Claw 飞书通道

[English](README.md) | 中文文档

**Clawrs-Feishu-Channel** 是一款专为 **Claw 生态系统**（ZeroClaw, OpenClaw）设计的高性能、安全飞书/Lark 消息通道解决方案。它基于 Rust 构建，提供了比传统 Node.js 适配器更健壮、内存安全且高效的替代方案，非常适合高并发和资源受限的环境。

## 功能亮点

### 🔌 无缝生态集成
- **ZeroClaw 原生**：作为 Rust crate (`clawrs-feishu`) 直接集成，实现最高性能和零运行时开销。
- **OpenClaw 兼容**：为 Node.js 提供无缝的 **N-API 桥接**，使其能够作为 OpenClaw 中现有 TypeScript 插件的即插即用替代品，结合了 Rust 的速度与 JavaScript 的灵活性。

### ⚡ 高性能与高效率
- **Rust 核心**：由 4-crate 微内核架构（**Domain**、**Kernel**、**Platform**、**Facade**）驱动，专为模块化和速度而设计。
- **资源高效**：与典型的 Node.js 运行时（~100MB+）相比，内存占用极低（**<5MB**），显著降低基础设施成本。
- **高并发**：利用 Rust 的异步运行时，以极低的延迟处理海量消息吞吐。

### 🛡️ 企业级安全
- **内存安全**：由 Rust 的所有权模型保证，有效防止缓冲区溢出和数据竞争等常见漏洞。
- **安全设计**：严格分离配置和代码。无硬编码凭据；所有敏感数据均通过环境变量或安全配置提供程序进行管理。

## 对比：clawrs-feishu-channel vs OpenClaw 飞书插件

| 维度 | clawrs-feishu-channel | OpenClaw 飞书插件 |
|------|---------------------|-------------------|
| **运行时** | Rust（静态二进制，无需 Node.js） | Node.js / TypeScript |
| **部署方式** | 库 crate，嵌入 zeroclaw/openclaw | VS Code 扩展，`~/.openclaw/extensions/feishu` |
| **内存占用** | <5MB | ~100MB+（Node.js 运行时） |
| **构建** | `cargo build` | `npm install` / 扩展安装 |
| **配置** | OpenClaw 兼容（`dm_policy`、`group_policy`、`allow_from`、`group_allow_from`） | 相同策略模型 |
| **架构** | 4-crate 六边形 + 微内核 + DDD | 扩展包 |
| **独立运行** | 是 — 可作为库或运行示例 | 否 — 需 OpenClaw 宿主 |
| **集成** | zeroclaw（Rust）、openclaw（通过桥接） | openclaw 原生 |

**适用场景**：zeroclaw 集成、嵌入式 Rust 应用、资源受限环境，或需要独立飞书通道库时选用 clawrs-feishu-channel。

## 功能对比：clawrs-feishu vs @openclaw/feishu（TypeScript）

| 功能 | clawrs-feishu（Rust） | @openclaw/feishu（TypeScript） |
|------|------------------------|--------------------------------|
| **连接方式** | WebSocket（open-lark） | WebSocket（飞书 SDK） |
| **私聊策略** | pairing / open / deny | pairing / allowlist / open / disabled |
| **群聊策略** | allowlist / open / deny | allowlist / open / disabled |
| **需 @ 提及** | 是（可配置） | 是（可按群配置） |
| **多账号** | 支持 | 支持 |
| **文本收发** | 支持 | 支持 |
| **媒体（图片、文件）** | 文本 + URL 拼接 | 原生图片/文件/音频支持 |
| **流式回复** | 否（分块文本） | 是（交互卡片） |
| **配对审批** | 否（仅白名单） | 是（`openclaw pairing approve`） |
| **独立库** | 是 | 否（需 OpenClaw 宿主） |
| **内存占用** | <5MB | ~100MB+（Node.js） |
| **Webhook 模式** | 规划中 | 非主推（优先 WebSocket） |

## 快速安装与部署

### 1. 克隆与构建

```bash
git clone https://github.com/AgenticWeb4/Clawrs-Channel-Feishu.git
cd clawrs-feishu-channel
cargo build --release
```

### 2. 配置（环境变量或配置文件）

```bash
export FEISHU_APP_ID="cli_你的应用id"
export FEISHU_APP_SECRET="你的应用密钥"
# 可选：FEISHU_CHAT_ID 用于示例
```

### 3. 部署方式

| 方式 | 命令 | 适用场景 |
|------|------|----------|
| **作为库** | 在 `Cargo.toml` 中添加 `clawrs-feishu = { path = "..." }` | 嵌入 zeroclaw、自定义 Rust 应用 |
| **zeroclaw** | `cargo build -p zeroclaw --features feishu` | ZeroClaw 网关 + 飞书 |
| **示例** | `cargo run --example send_message -- oc_chat_id "你好"` | 快速发送测试 |

### 4. 验证

```bash
cargo test --workspace                    # 单元测试（无需凭证）
cd e2e && cp .env.example .env && bash run-tests.sh api  # E2E（需凭证）
```

## 架构

**4-crate 微内核 + 六边形 + DDD**

```
                         ┌─────────────────────────────────┐
                         │      clawrs-feishu (门面)          │
                         │  channel.rs | compose.rs | cap.  │
                         └──────────────┬──────────────────┘
                                        │
                         ┌──────────────┴──────────────────┐
                         │    feishu-platform (适配器)       │
                         │ Auth | IM | Bot | Event          │
                         │ LarkAuthAdapter, LarkImAdapter   │
                         │ LarkBotAdapter, LarkWsAdapter    │
                         └──────────────┬──────────────────┘
                                        │
          ┌─────────────────────────────┼─────────────────────────────┐
          │                             │                             │
   ┌──────┴──────┐            ┌─────────┴─────────┐           ┌───────┴───────┐
   │feishu-kernel│            │   feishu-domain   │           │  open-lark    │
   │ Capability  │            │ 配置 | 端口        │           │  SDK          │
   │ EventBus    │            │ 安全 | 编解码      │           │               │
   └─────────────┘            └───────────────────┘           └───────────────┘
```

| Crate | 角色 |
|-------|------|
| `feishu-domain` | 纯领域类型、端口 trait、编解码 |
| `feishu-kernel` | 微内核：Capability、EventBus、Kernel |
| `feishu-platform` | 适配器：Auth、IM、Bot、Event（open-lark 封装）|
| `clawrs-feishu` | 门面、组合根、Channel trait |

## 快速开始

### 前置条件

- Rust 1.75+（2021 edition）
- 飞书应用，需开启 **im:message**、**bot:read** 和 **WebSocket** 权限
- [Bruno CLI](https://www.usebruno.com/)（可选，用于 E2E 测试）

### 安装

```bash
git clone https://github.com/AgenticWeb4/Clawrs-Channel-Feishu.git
cd clawrs-feishu-channel

# 构建
cargo build --workspace

# 运行单元测试（无需凭证）
cargo test --workspace
```

### 配置

设置飞书应用的环境变量：

```bash
export FEISHU_APP_ID="cli_your_app_id"
export FEISHU_APP_SECRET="your_app_secret"
export FEISHU_CHAT_ID="oc_your_chat_id"    # 可选，用于示例
```

或复制示例环境文件：

```bash
cp e2e/.env.example e2e/.env
# 编辑 e2e/.env 填入你的凭证
```

### 发送消息

```bash
FEISHU_APP_ID=xxx FEISHU_APP_SECRET=yyy \
  cargo run --example send_message -- oc_chat_id "你好，来自 Claw！"
```

### 运行在线 API 测试

```bash
FEISHU_APP_ID=xxx FEISHU_APP_SECRET=yyy \
  cargo test -p clawrs-feishu --test live_api_test -- --ignored --nocapture
```

### 运行 E2E 测试

```bash
cd e2e
cp .env.example .env
# 在 .env 中填入凭证
bash run-tests.sh api
```

## 作为库使用

```rust
use clawrs_feishu::{create_channel, Channel, FeishuConfig, FeishuDomain};

let config = FeishuConfig {
    app_id: std::env::var("FEISHU_APP_ID").unwrap(),
    app_secret: std::env::var("FEISHU_APP_SECRET").unwrap(),
    domain: FeishuDomain::Feishu,
    allowed_users: vec!["*".to_string()],
    ..Default::default()
};

let channel = create_channel(config);

// 发送消息
channel.send("你好！", "oc_chat_id").await?;

// 监听消息（含安全过滤 + @mention 检测）
let (tx, mut rx) = tokio::sync::mpsc::channel(256);
tokio::spawn(async move { channel.listen(tx).await });
while let Some(msg) = rx.recv().await {
    println!("{}: {}", msg.sender, msg.content);
}
```

## 测试

| 层级 | 命令 | 是否需要凭证 |
|------|------|-------------|
| 单元测试 | `cargo test --workspace` | 不需要 |
| 在线 API | `cargo test -- --ignored` | 需要 |
| E2E (Bruno) | `cd e2e && bash run-tests.sh api` | 需要 |

**测试覆盖**：38 个单元测试 + 4 个在线测试 + 85 个 E2E 测试（32 个请求）。

## 安全

- 源码中**零硬编码凭证**
- 所有密钥通过环境变量注入
- 参见 [SECURITY.md](SECURITY.md) 了解凭证管理指南

## 文档

- [设计规格](doc/design-spec.md) -- C4 + DDD + 六边形架构规格书
- [安全策略](SECURITY.md) -- 凭证管理与轮换

## 许可证

Apache-2.0
