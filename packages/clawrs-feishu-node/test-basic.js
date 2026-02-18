#!/usr/bin/env node
'use strict';

const { createChannelJs, healthCheckStandalone } = require('.');

// 1. Standalone health check
console.log('healthCheckStandalone:', healthCheckStandalone());

// 2. Create channel (minimal config - will fail at runtime without real creds)
const config = {
  appId: 'test_app',
  appSecret: 'test_secret',
};
try {
  const channel = createChannelJs(config);
  console.log('createChannelJs: ok, channel type:', typeof channel);
  console.log('  - send:', typeof channel.send);
  console.log('  - healthCheck:', typeof channel.healthCheck);
  console.log('  - listen:', typeof channel.listen);
} catch (e) {
  console.error('createChannelJs error:', e.message);
}

console.log('Basic bindings OK');
