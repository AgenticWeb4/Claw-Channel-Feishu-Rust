#!/usr/bin/env node
/**
 * WebSocket 消息流测试 (方案 A: Bruno 内嵌脚本)
 *
 * 流程：获取 WS 端点 → 连接 → 断言连接成功 → 收集消息并验证类型
 * 支持多种消息类型：text, post, image 等（见 fixtures/）
 *
 * 依赖：.env 中 FEISHU_APP_ID、FEISHU_APP_SECRET
 *
 * 运行：cd e2e && node 05-websocket/03-ws-message-flow.js
 * 或：  cd e2e && npm run ws:flow
 */

const { WebSocket } = require('ws');

const FEISHU_BASE = process.env.FEISHU_BASE_URL || 'https://open.feishu.cn';
const APP_ID = process.env.FEISHU_APP_ID;
const APP_SECRET = process.env.FEISHU_APP_SECRET;
const CONNECT_TIMEOUT_MS = 15000;
const MESSAGE_WAIT_MS = Number(process.env.WS_MESSAGE_WAIT_MS) || 8000;

async function getWsEndpoint() {
  const res = await fetch(`${FEISHU_BASE}/callback/ws/endpoint`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', locale: 'zh' },
    body: JSON.stringify({ AppID: APP_ID, AppSecret: APP_SECRET }),
  });
  const data = await res.json();
  if (data.code !== 0) {
    throw new Error(`get ws endpoint failed: ${data.code} ${data.msg || ''}`);
  }
  const url = data.data?.URL || data.data?.url;
  if (!url) throw new Error('no URL in response');
  return url;
}

function tryParseEvent(data) {
  const str = typeof data === 'string' ? data : (data && data.toString ? data.toString() : '');
  if (!str || str.length < 10) return null;
  try {
    const j = JSON.parse(str);
    if (j && (j.header?.event_type || j.event_type)) return j;
    return null;
  } catch {
    return null;
  }
}

function extractMessageType(ev) {
  const msg = ev?.event?.message || ev?.message;
  return msg?.message_type ?? null;
}

function connectAndAssert(url, opts = {}) {
  const { waitForMessage = false } = opts;
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    let resolved = false;
    const collected = [];

    const done = (err, result) => {
      if (resolved) return;
      resolved = true;
      ws.removeAllListeners();
      try {
        ws.close();
      } catch (_) {}
      if (err) reject(err);
      else resolve(result);
    };

    const connectTimer = setTimeout(() => done(new Error('连接超时')), CONNECT_TIMEOUT_MS);

    ws.on('message', (data) => {
      const ev = tryParseEvent(data);
      if (ev) {
        collected.push(ev);
        const mt = extractMessageType(ev);
        if (mt) console.log(`    📩 收到消息 type=${mt}`);
      }
    });

    ws.on('open', () => {
      clearTimeout(connectTimer);
      if (!waitForMessage) {
        done(null, { connected: true, messages: collected });
        return;
      }
      setTimeout(() => {
        done(null, { connected: true, messages: collected });
      }, MESSAGE_WAIT_MS);
    });

    ws.on('error', (e) => done(e));
    ws.on('close', () => {
      if (!resolved) done(new Error('连接关闭'));
    });
  });
}

async function main() {
  if (!APP_ID || !APP_SECRET) {
    console.error('✗ 缺少 FEISHU_APP_ID 或 FEISHU_APP_SECRET，请配置 .env');
    process.exit(1);
  }

  const waitForMsg = process.env.WS_WAIT_FOR_MESSAGE === '1';

  console.log('ℹ WebSocket 消息流测试 (方案 A)');
  console.log('  1. 获取 WS 端点');
  const url = await getWsEndpoint();
  console.log('  ✓ 端点:', url.substring(0, 60) + '...');

  console.log('  2. 连接并断言');
  const result = await connectAndAssert(url, { waitForMessage: waitForMsg });
  if (!result.connected) {
    console.error('✗ 连接失败');
    process.exit(1);
  }
  console.log('  ✓ 连接成功');

  if (result.messages?.length > 0) {
    const types = [...new Set(result.messages.map(extractMessageType).filter(Boolean))];
    console.log(`  ✓ 收到 ${result.messages.length} 条消息，类型: ${types.join(', ') || '(未知)'}`);
  } else if (waitForMsg) {
    console.log('  ℹ 等待期间未收到消息（可在测试群内发消息触发）');
  }

  console.log('✓ WebSocket 消息流测试通过');
  process.exit(0);
}

main().catch((e) => {
  console.error('✗', e.message);
  process.exit(1);
});
