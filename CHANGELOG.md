# Changelog

## 1.3.0 - 2026-02-17

### Features

- Add webhook adapter support to `feishu-platform`
- Add comprehensive E2E test cases for messages and WebSockets

### Refactor

- Improve config adapter and message normalization in `clawrs-feishu`

### Chore

- Optimize `.gitignore` for lean submission

## 1.2.1 - 2026-02-16

### Documentation

- Highlight performance and security features in README

## 1.2.0 - 2026-02-16

### Features

- Support WebSocket connection lifecycle signals (connected, disconnected, error)
- Add in-memory message deduplication to handle redundant Feishu events
- Support runtime status monitoring and health reporting
- Improve target resolution and message normalization in messaging plugin

### Fixes

- Patch `open-lark` SDK to handle nullable `user_id` in event deserialization
- Improve IM platform error mapping and handling

## 1.1.0 - 2026-02-16

### Features

- Add universal `release-skills` skill for automated multi-language releases

### Documentation

- Rename design-spec.md and cleanup project documentation

## 1.0.2 - 2026-02-16

### Documentation

- Update repo URL to Clawrs-Channel-Feishu
- Add feature comparison with OpenClaw Feishu TypeScript, add .cursor to gitignore

## 1.0.1 - 2026-02-16

### Refactor

- Consolidate DmPolicy derivation in feishu-domain, add criterion benchmarks
- Simplify connection mode check and config access in clawrs-feishu
- Extract getChannelConfig and simplify normalizeTarget in clawrs-feishu-node

### Documentation

- Add performance optimization guide (docs/performance-optimization.md)

## 1.0.0 - 2026-02-16

### Breaking Changes

- Remove zeroclaw-feishu, feishu-auth, feishu-bot, feishu-event, feishu-im crates
- Migrate to OpenClaw architecture (clawrs-feishu, feishu-platform)

### Features

- OpenClaw plugin (@openclaw/clawrs-feishu) with gateway.startAccount for WebSocket inbound
- N-API bridge (clawrs-feishu-node) for Node.js integration
- Rust core: feishu-domain, feishu-kernel, feishu-platform, clawrs-feishu
- E2E test suite (Bruno + shell scripts) for OpenClaw gateway
- Security-focused .gitignore (excludes .env, credentials, node_modules, *.node)
 