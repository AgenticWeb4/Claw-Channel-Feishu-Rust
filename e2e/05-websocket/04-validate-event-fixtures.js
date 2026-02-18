#!/usr/bin/env node
/**
 * 验证 im.message.receive_v1 事件 fixture 结构
 * 支持多种消息类型：text, post, image, 及 @mention
 *
 * 运行：node 05-websocket/04-validate-event-fixtures.js
 */

const fs = require('fs');
const path = require('path');

const FIXTURES_DIR = path.join(__dirname, 'fixtures');
const REQUIRED = {
  schema: 'string',
  header: 'object',
  'header.event_type': 'string',
  'header.event_id': 'string',
  event: 'object',
  'event.sender': 'object',
  'event.sender.sender_id': 'object',
  'event.sender.sender_id.open_id': 'string',
  'event.message': 'object',
  'event.message.message_id': 'string',
  'event.message.chat_id': 'string',
  'event.message.chat_type': 'string',
  'event.message.message_type': 'string',
  'event.message.content': 'string',
};

const MESSAGE_TYPES = [
  'text', 'post', 'image', 'file', 'audio', 'media', 'sticker',
  'interactive', 'share_chat', 'share_user',
];

function get(obj, pathStr) {
  return pathStr.split('.').reduce((o, k) => o?.[k], obj);
}

function validateEvent(json, file) {
  const errs = [];
  for (const [p, expected] of Object.entries(REQUIRED)) {
    const v = get(json, p);
    if (v === undefined || v === null) {
      errs.push(`缺少 ${p}`);
    } else if (expected === 'string' && typeof v !== 'string') {
      errs.push(`${p} 应为 string`);
    } else if (expected === 'object' && (typeof v !== 'object' || Array.isArray(v))) {
      errs.push(`${p} 应为 object`);
    }
  }
  const msgType = get(json, 'event.message.message_type');
  if (msgType && !MESSAGE_TYPES.includes(msgType)) {
    errs.push(`message_type "${msgType}" 不在已知类型中`);
  }
  if (get(json, 'header.event_type') !== 'im.message.receive_v1') {
    errs.push('event_type 应为 im.message.receive_v1');
  }
  return errs;
}

function main() {
  const files = fs.readdirSync(FIXTURES_DIR).filter((f) => f.endsWith('.json'));
  let failed = 0;

  console.log('ℹ 验证事件 fixture (多种消息类型)\n');

  for (const f of files) {
    const filepath = path.join(FIXTURES_DIR, f);
    const raw = fs.readFileSync(filepath, 'utf8');
    let json;
    try {
      json = JSON.parse(raw);
    } catch (e) {
      console.error(`✗ ${f}: JSON 解析失败`, e.message);
      failed++;
      continue;
    }

    const errs = validateEvent(json, f);
    const msgType = get(json, 'event.message.message_type') || '?';
    if (errs.length > 0) {
      console.error(`✗ ${f} (message_type=${msgType})`);
      errs.forEach((e) => console.error(`    - ${e}`));
      failed++;
    } else {
      console.log(`✓ ${f} (message_type=${msgType})`);
    }
  }

  if (failed > 0) {
    console.error(`\n✗ ${failed} 个 fixture 验证失败`);
    process.exit(1);
  }
  console.log(`\n✓ 全部 ${files.length} 个 fixture 通过`);
  process.exit(0);
}

main();
