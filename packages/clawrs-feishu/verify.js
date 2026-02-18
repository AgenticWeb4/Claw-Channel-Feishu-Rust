#!/usr/bin/env node
'use strict';

// Verify plugin loads and registers
const register = (await import('./dist/index.js')).default;
const api = {
  registerChannel: (opts) => {
    console.log('registerChannel called:', opts.plugin?.id);
    console.log('  meta:', opts.plugin?.meta?.label);
    console.log('  config.listAccountIds:', typeof opts.plugin?.config?.listAccountIds);
    console.log('  outbound.sendText:', typeof opts.plugin?.outbound?.sendText);
    console.log('  gateway.start:', typeof opts.plugin?.gateway?.start);
  },
};
register(api);
console.log('Plugin verification OK');
