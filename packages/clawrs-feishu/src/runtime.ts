import type { PluginRuntime } from 'openclaw/plugin-sdk';

let runtime: PluginRuntime | null = null;

export function setClawrsFeishuRuntime(next: PluginRuntime): void {
  runtime = next;
}

export function getClawrsFeishuRuntime(): PluginRuntime {
  if (!runtime) {
    throw new Error("clawrs-feishu runtime not initialized");
  }
  return runtime;
}
