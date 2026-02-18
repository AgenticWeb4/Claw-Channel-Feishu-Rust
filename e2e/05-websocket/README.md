# WebSocket 测试

## 方案 A：Bruno 内嵌脚本

- **01-ws-endpoint.bru**：Bruno HTTP 测试，获取 WS 端点
- **03-ws-message-flow.js**：Node 脚本，连接 Feishu WS 并断言
- **04-validate-event-fixtures.js**：验证多种消息类型 fixture 结构
- **fixtures/**：im.message.receive_v1 事件样本（text, post, image, @mention）

## 多种消息类型

| 文件 | message_type | 说明 |
|------|--------------|------|
| im-message-text.json | text | 纯文本 |
| im-message-post.json | post | 富文本（zh_cn.content） |
| im-message-post-markdown.json | post | Markdown + 代码块 |
| im-message-post-mixed.json | post | 混合：text + md + a + at + img |
| im-message-image.json | image | 图片 |
| im-message-file.json | file | 文件 |
| im-message-audio.json | audio | 音频 |
| im-message-with-mention.json | text | 含 @提及 |

## 运行

```bash
cd e2e
npm install                    # 首次需安装 ws

# 验证 fixture 结构
npm run ws:validate-fixtures

# 连接 Feishu WS
npm run ws:flow

# 等待消息（测试群内发消息可触发）
WS_WAIT_FOR_MESSAGE=1 npm run ws:flow
```

或通过 `run-tests.sh api` 自动执行。

## 环境

需配置 `.env`：`FEISHU_APP_ID`、`FEISHU_APP_SECRET`。
