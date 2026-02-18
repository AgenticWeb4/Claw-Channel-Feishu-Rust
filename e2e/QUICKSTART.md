# 🚀 ZeroClaw Feishu E2E 测试快速开始

5 分钟完成测试环境配置并运行首次测试。

---

## 第一步：安装 Bruno

### macOS

```bash
brew install bruno
```

### Linux/Windows

```bash
npm install -g @usebruno/cli
```

验证安装：

```bash
bru --version
# 应输出: @usebruno/cli/x.x.x
```

---

## 第二步：配置凭证

### 1. 从 OpenClaw 获取凭证

你已经有 OpenClaw 的飞书插件配置，直接复用：

```bash
# 查看 OpenClaw 飞书配置
cat ~/.openclaw/extensions/feishu/src/config-schema.ts

# 获取 app_id 和 app_secret（这些是敏感信息，需要从真实配置中获取）
# 或者从飞书开放平台查看：
# https://open.feishu.cn/app/
```

### 2. 创建 .env 文件

```bash
cd /Users/WORKS/MyProject/MyAIClaw/clawrs-feishu-channel/e2e

# 复制示例文件
cp .env.example .env

# 编辑 .env（使用你喜欢的编辑器）
nano .env
```

填入真实凭证：

```bash
FEISHU_APP_ID=cli_a1b2c3d4e5f6g7h8         # 替换为真实值
FEISHU_APP_SECRET=your_app_secret_here     # 替换为真实值

TEST_USER_OPEN_ID=ou_af3d3f461735b4dbfb37a3224d543805  # 从 OpenClaw allowlist 获取
TEST_CHAT_ID=oc_your_dm_chat_id            # 稍后获取
TEST_GROUP_CHAT_ID=oc_your_group_chat_id   # 稍后获取
```

### 3. 获取测试 Chat ID（可选，部分测试需要）

方法 1：从 OpenClaw 日志获取

```bash
# 启动 OpenClaw
openclaw start

# 在飞书中给 bot 发一条消息
# 查看日志
tail -f ~/.openclaw/logs/channels.log | grep chat_id
# 输出示例: "chat_id": "oc_abc123..."
```

方法 2：先运行不需要 chat_id 的测试（见下方）

---

## 第三步：运行测试

### Option A：仅测试飞书 API（无需 ZeroClaw）

推荐首次运行，验证飞书凭证是否正确：

```bash
cd /Users/WORKS/MyProject/MyAIClaw/clawrs-feishu-channel/e2e

# 测试认证
bru run 01-auth/ --env dev

# 测试健康检查
bru run 02-health/ --env dev
```

预期输出：

```
✓ Get Tenant Access Token
✓ Token Auto Refresh
✓ Get Bot Info
✓ ZeroClaw Channel Health Check

4 tests passed
```

### Option B：完整 E2E 测试（需要 ZeroClaw 运行）

#### 1. 构建并启动 ZeroClaw

```bash
cd /Users/WORKS/MyProject/MyAIClaw/zeroclaw

# 构建（首次需要时间）
cargo build --release --features feishu

# 启动 gateway
./target/release/zeroclaw gateway --port 8080
```

#### 2. 运行完整测试套件

新开一个终端：

```bash
cd /Users/WORKS/MyProject/MyAIClaw/clawrs-feishu-channel/e2e

# 使用快速启动脚本（推荐）
./run-tests.sh

# 或手动运行
bru run . --env dev --output report.json
```

---

## 第四步：查看结果

### 成功示例

```
✅ All tests passed!

📊 Test Summary:
  Total: 17
  Passed: 17
  Failed: 0
  Duration: 8.2s
```

### 部分失败（预期）

某些测试需要真实 chat_id 才能运行，首次运行可能失败：

```
✅ 01-auth: 2/2 passed
✅ 02-health: 2/2 passed
⚠️ 03-messages: 2/6 passed (需要 TEST_CHAT_ID)
✅ 04-security: 4/4 passed
✅ 05-websocket: 1/1 passed
⚠️ 06-zeroclaw: 1/2 passed (需要 gateway pairing)
```

这是正常的！继续下一步配置缺失项。

---

## 第五步：配置缺失项（可选）

### 获取 TEST_CHAT_ID

1. 在飞书中找到 ZeroClaw 机器人
2. 发送一条消息："Hello"
3. 查看 ZeroClaw 日志（如果运行中）或 OpenClaw 日志
4. 复制 chat_id（格式：`oc_xxx`）
5. 更新 `.env` 中的 `TEST_CHAT_ID`

### 配置 Gateway Pairing（可选）

如果需要测试 `06-zeroclaw/02-webhook.bru`：

1. 启动 gateway（见上方）
2. Gateway 会输出 pairing code（6 位数字）
3. Exchange code for token:

```bash
curl -X POST http://127.0.0.1:8080/pair \
  -H "X-Pairing-Code: 123456"  # 替换为真实 code
# 返回: {"token": "bearer_abc..."}
```

4. 将 token 添加到 `06-zeroclaw/02-webhook.bru` 的 `auth:bearer` 中

---

## 常见问题

### ❌ "No tenant_access_token found"

**原因**: 未运行 `01-auth/01-get-token.bru`  
**解决**: 按顺序运行测试，或单独运行：

```bash
bru run 01-auth/01-get-token.bru --env dev
```

### ❌ "Gateway not responding"

**原因**: ZeroClaw gateway 未启动  
**解决**:

```bash
# 检查 gateway 是否运行
curl http://127.0.0.1:8080/health

# 如果未运行，启动它
cd /Users/WORKS/MyProject/MyAIClaw/zeroclaw
./target/release/zeroclaw gateway --port 8080
```

### ❌ "code=10014" (app_secret 错误)

**原因**: `.env` 中的 `FEISHU_APP_SECRET` 不正确  
**解决**: 从飞书开放平台重新获取 app_secret

### ⚠️ "Using placeholder chat_id"

**原因**: `TEST_CHAT_ID` 未配置（值为 `oc_test_chat_placeholder`）  
**影响**: 部分消息测试会失败（API 返回 `code=230002`）  
**解决**: 见"第五步：获取 TEST_CHAT_ID"

---

## 下一步

✅ 测试通过后，开始实现功能（见 `../doc/design-spec.md`）

测试驱动开发流程：

1. **红灯**: 运行测试，确认失败（功能未实现）
2. **绿灯**: 实现功能，直到测试通过
3. **重构**: 优化代码，保持测试通过

```bash
# 监控模式（需要 watch 工具）
watch -n 5 'bru run . --env dev | tail -n 20'
```

---

## 参考资料

- **详细文档**: [README.md](./README.md)
- **测试覆盖**: [COVERAGE.md](./COVERAGE.md)
- **设计规格**: [../doc/design-spec.md](../doc/design-spec.md)
- **Bruno 文档**: https://docs.usebruno.com/
- **飞书 API**: https://open.feishu.cn/document/

---

## 联系与支持

遇到问题？检查：

1. `.env` 凭证是否正确
2. 网络能否访问 `open.feishu.cn`
3. ZeroClaw gateway 是否启动（如果需要）
4. Bruno CLI 版本是否 >= 0.10.0

Happy Testing! 🎉
