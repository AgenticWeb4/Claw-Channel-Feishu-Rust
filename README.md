# Claw Feishu Channel

[中文文档](README_CN.md) | English

**Clawrs-Feishu-Channel** is a high-performance, secure Feishu/Lark message channel solution designed for the **Claw ecosystem** (ZeroClaw, OpenClaw). Built with Rust, it provides a robust, memory-safe, and efficient alternative to traditional Node.js adapters, making it ideal for high-concurrency and resource-constrained environments.

## Feature Highlights

### 🔌 Seamless Ecosystem Integration
- **ZeroClaw Native**: Directly integrates as a Rust crate (`clawrs-feishu`) for maximum performance and zero runtime overhead.
- **OpenClaw Compatible**: Provides a seamless **N-API bridge** for Node.js, allowing it to function as a drop-in replacement for existing TypeScript plugins in OpenClaw, combining the speed of Rust with the flexibility of JavaScript.

### ⚡ High Performance & Efficiency
- **Rust Core**: Powered by a 4-crate microkernel architecture (**Domain**, **Kernel**, **Platform**, **Facade**) designed for modularity and speed.
- **Resource Efficient**: Extremely low memory footprint (**<5MB**) compared to typical Node.js runtimes (~100MB+), reducing infrastructure costs.
- **High Concurrency**: leveraging Rust's async runtime to handle massive message throughput with minimal latency.

### 🛡️ Enterprise-Grade Security
- **Memory Safety**: Guaranteed by Rust's ownership model, effectively preventing common vulnerabilities like buffer overflows and data races.
- **Secure Design**: Strict separation of configuration and code. No hardcoded credentials; all sensitive data is managed via environment variables or secure configuration providers.

## Comparison: clawrs-feishu-channel vs OpenClaw Feishu Plugin

| Aspect | clawrs-feishu-channel | OpenClaw Feishu Plugin |
|---------|----------------------|------------------------|
| **Runtime** | Rust (static binary, no Node.js) | Node.js / TypeScript |
| **Deployment** | Library crate, embed in zeroclaw/openclaw | VS Code extension, `~/.openclaw/extensions/clawrs-feishu` |
| **Memory** | <5MB footprint | ~100MB+ (Node.js runtime) |
| **Build** | `cargo build` | `npm install` / extension install |
| **Config** | OpenClaw-compatible (`dm_policy`, `group_policy`, `allow_from`, `group_allow_from`) | Same policy model |
| **Architecture** | 4-crate Hexagonal + Microkernel + DDD | Extension bundle |
| **Standalone** | Yes — use as library or run examples | No — requires OpenClaw host |
| **Integration** | zeroclaw (Rust), openclaw (via bridge) | openclaw native |

**When to use clawrs-feishu-channel**: ZeroClaw/zeroclaw integration, embedded Rust apps, resource-constrained environments, or when you need a standalone Feishu channel library.

## Feature Comparison: clawrs-feishu vs @openclaw/feishu (TypeScript)

| Feature | clawrs-feishu (Rust) | @openclaw/feishu (TypeScript) |
|---------|----------------------|--------------------------------|
| **Connection** | WebSocket (open-lark) | WebSocket (Feishu SDK) |
| **DM policy** | pairing / open / deny | pairing / allowlist / open / disabled |
| **Group policy** | allowlist / open / deny | allowlist / open / disabled |
| **@mention required** | Yes (configurable) | Yes (configurable per group) |
| **Multi-account** | Yes | Yes |
| **Text send/receive** | Yes | Yes |
| **Media (images, files)** | Text + URL concatenation | Native image/file/audio support |
| **Streaming replies** | No (chunked text) | Yes (interactive cards) |
| **Pairing approval** | No (allowlist only) | Yes (`openclaw pairing approve`) |
| **Standalone library** | Yes | No (OpenClaw host required) |
| **Memory footprint** | <5MB | ~100MB+ (Node.js) |
| **Webhook mode** | Planned | Not primary (WebSocket preferred) |

## Quick Install & Deploy

### 1. Clone & Build

```bash
git clone https://github.com/AgenticWeb4/Clawrs-Channel-Feishu.git
cd clawrs-feishu-channel
cargo build --release
```

### 2. Configure (env vars or config file)

```bash
export FEISHU_APP_ID="cli_your_app_id"
export FEISHU_APP_SECRET="your_app_secret"
# Optional: FEISHU_CHAT_ID for examples
```

### 3. Deploy Options

| Option | Command | Use Case |
|--------|---------|----------|
| **As library** | Add `clawrs-feishu = { path = "..." }` to your `Cargo.toml` | Embed in zeroclaw, custom Rust app |
| **zeroclaw** | `cargo build -p zeroclaw --features feishu` | ZeroClaw gateway with Feishu |
| **Example** | `cargo run --example send_message -- oc_chat_id "Hello"` | Quick send test |

### 4. Verify

```bash
cargo test --workspace                    # Unit tests (no credentials)
cd e2e && cp .env.example .env && bash run-tests.sh api  # E2E (needs credentials)
```

## Architecture

**4-crate Microkernel + Hexagonal + DDD**

```
                         ┌─────────────────────────────────┐
                         │      clawrs-feishu (facade)       │
                         │  channel.rs | compose.rs | cap.  │
                         └──────────────┬──────────────────┘
                                        │
                         ┌──────────────┴──────────────────┐
                         │    feishu-platform (adapters)    │
                         │ Auth | IM | Bot | Event          │
                         │ LarkAuthAdapter, LarkImAdapter   │
                         │ LarkBotAdapter, LarkWsAdapter    │
                         └──────────────┬──────────────────┘
                                        │
          ┌─────────────────────────────┼─────────────────────────────┐
          │                             │                             │
   ┌──────┴──────┐            ┌─────────┴─────────┐           ┌───────┴───────┐
   │feishu-kernel│            │   feishu-domain   │           │  open-lark    │
   │ Capability  │            │ Config | Port     │           │  SDK          │
   │ EventBus    │            │ Security | Codec  │           │               │
   └─────────────┘            └───────────────────┘           └───────────────┘
```

| Crate | Role |
|-------|------|
| `feishu-domain` | Pure domain types, port traits, codec |
| `feishu-kernel` | Microkernel: Capability, EventBus, Kernel |
| `feishu-platform` | Adapters: Auth, IM, Bot, Event (open-lark wrappers) |
| `clawrs-feishu` | Facade, composition root, Channel trait |

## Quick Start

### Prerequisites

- Rust 1.75+ (2021 edition)
- A Feishu App with **im:message**, **bot:read**, and **WebSocket** permissions
- [Bruno CLI](https://www.usebruno.com/) (optional, for E2E tests)

### Setup

```bash
git clone https://github.com/AgenticWeb4/Clawrs-Channel-Feishu.git
cd clawrs-feishu-channel

# Build
cargo build --workspace

# Run unit tests (no credentials needed)
cargo test --workspace
```

### Configuration

Set environment variables for your Feishu App:

```bash
export FEISHU_APP_ID="cli_your_app_id"
export FEISHU_APP_SECRET="your_app_secret"
export FEISHU_CHAT_ID="oc_your_chat_id"    # Optional, for examples
```

Or copy the example env file:

```bash
cp e2e/.env.example e2e/.env
# Edit e2e/.env with your credentials
```

### Send a Message

```bash
FEISHU_APP_ID=xxx FEISHU_APP_SECRET=yyy \
  cargo run --example send_message -- oc_chat_id "Hello from Claw!"
```

### Run Live API Tests

```bash
FEISHU_APP_ID=xxx FEISHU_APP_SECRET=yyy \
  cargo test -p clawrs-feishu --test live_api_test -- --ignored --nocapture
```

### Run E2E Tests

```bash
cd e2e
cp .env.example .env
# Fill in credentials in .env
bash run-tests.sh api
```

## Usage as Library

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

// Send message
channel.send("Hello!", "oc_chat_id").await?;

// Listen for messages (with security + @mention filtering)
let (tx, mut rx) = tokio::sync::mpsc::channel(256);
tokio::spawn(async move { channel.listen(tx).await });
while let Some(msg) = rx.recv().await {
    println!("{}: {}", msg.sender, msg.content);
}
```

## OpenClaw Bridge (@openclaw/clawrs-feishu)

For OpenClaw (Node.js) integration, use the napi-rs bridge:

```bash
# Build native bindings
cd packages/clawrs-feishu-node && npm run build

# Build plugin
cd ../clawrs-feishu && npm install && npm run build

# Install into OpenClaw (when OpenClaw is available)
openclaw plugins install ./packages/clawrs-feishu
```

Configure under `channels.clawrs-feishu.accounts`:

```json5
{
  channels: {
    "clawrs-feishu": {
      accounts: {
        main: { appId: "...", appSecret: "..." }
      }
    }
  }
}
```

See [doc/openclaw-bridge-plan.md](doc/openclaw-bridge-plan.md) for full design.

## Testing

| Layer | Command | Credentials |
|-------|---------|-------------|
| Unit tests | `cargo test --workspace` | Not needed |
| Live API | `cargo test -- --ignored` | Required |
| E2E (Bruno) | `cd e2e && bash run-tests.sh api` | Required |

**Test coverage**: 38 unit tests + 4 live tests + 85 E2E tests across 32 requests.

## Security

- **Zero hardcoded credentials** in source code
- All secrets via environment variables
- See [SECURITY.md](SECURITY.md) for credential management guide

## Documentation

- [Design Specification](doc/design-spec.md) -- C4 + DDD + Hexagonal architecture spec
- [Security Policy](SECURITY.md) -- credential management and rotation

## License

Apache-2.0
