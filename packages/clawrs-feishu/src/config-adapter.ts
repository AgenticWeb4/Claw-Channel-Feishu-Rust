/**
 * Adapt OpenClaw config to clawrs-feishu-node FeishuConfigInput.
 * Both use camelCase; pass through with minimal mapping.
 */

import type { FeishuConfigInput } from 'clawrs-feishu-node';

export type { FeishuConfigInput };

export interface OpenClawAccountConfig {
  appId: string;
  appSecret: string;
  domain?: string;
  connectionMode?: string;
  dmPolicy?: string;
  groupPolicy?: string;
  allowFrom?: string[];
  groupAllowFrom?: string[];
  groupRequireMention?: boolean;
  encryptKey?: string;
  verificationToken?: string;
  webhookPort?: number;
}

/**
 * Convert OpenClaw account config to clawrs-feishu-node format.
 */
export function toFeishuConfigInput(account: OpenClawAccountConfig): FeishuConfigInput {
  return {
    appId: account.appId,
    appSecret: account.appSecret,
    domain: account.domain,
    connectionMode: account.connectionMode,
    dmPolicy: account.dmPolicy,
    groupPolicy: account.groupPolicy,
    allowFrom: account.allowFrom,
    groupAllowFrom: account.groupAllowFrom,
    groupRequireMention: account.groupRequireMention,
    encryptKey: account.encryptKey,
    verificationToken: account.verificationToken,
    webhookPort: account.webhookPort,
  };
}
