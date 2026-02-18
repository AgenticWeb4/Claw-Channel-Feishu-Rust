# Bruno Collection Scripts

全局脚本，自动加载到所有测试请求中。

## Pre-request Script（全局前置脚本）

```javascript
// 自动刷新 token（如果即将过期）
const tokenRefreshTime = bru.getEnvVar("token_refreshed_at");
if (tokenRefreshTime) {
  const elapsed = Date.now() - new Date(tokenRefreshTime).getTime();
  const FIVE_MINUTES = 5 * 60 * 1000;
  
  if (elapsed > FIVE_MINUTES) {
    console.log("⚠ Token may be stale (> 5min), consider refreshing");
  }
}

// 验证必需环境变量
const requiredVars = ['feishu_base_url', 'app_id', 'app_secret'];
for (const varName of requiredVars) {
  if (!bru.getEnvVar(varName)) {
    console.warn(`⚠ Environment variable '${varName}' is not set`);
  }
}
```

## Post-response Script（全局后置脚本）

```javascript
// 自动记录所有错误响应
if (res.status >= 400) {
  console.error("❌ HTTP Error:", res.status);
  console.error("   Body:", JSON.stringify(res.body, null, 2));
}

// 飞书 API 错误码检查
if (res.body && res.body.code && res.body.code !== 0) {
  console.error("❌ Feishu API Error:", res.body.code);
  console.error("   Message:", res.body.msg || "N/A");
}

// 性能监控（所有请求）
if (res.responseTime > 3000) {
  console.warn("⚠ Slow response:", res.responseTime, "ms");
}
```

## 使用方法

在 Bruno GUI 中：
1. 打开集合设置（Collection Settings）
2. 选择 "Scripts" 标签
3. 将上述脚本添加到对应区域

在 Bruno CLI 中：
这些脚本会自动从 `collection.bru` 或 `bruno.json` 中读取（如果配置）。

## 自定义变量函数

```javascript
// 生成随机测试消息
function randomTestMessage() {
  return `Test message ${Date.now()} - ${Math.random().toString(36).substring(7)}`;
}

// 检查是否在 CI 环境
function isCI() {
  return process.env.CI === 'true' || process.env.GITHUB_ACTIONS === 'true';
}

// 等待指定时间（用于模拟用户操作延迟）
async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
```
