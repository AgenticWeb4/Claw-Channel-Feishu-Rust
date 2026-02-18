/**
 * @openclaw/clawrs-feishu — Feishu channel plugin (Rust-first).
 *
 * Receive: Rust clawrs-feishu crates via NAPI (SecurityGuard, WebSocket, codec, reconnection)
 * Send:    Rust clawrs-feishu crates via NAPI
 *
 * The Rust crates provide: DM/Group security filtering, message decoding,
 * WebSocket reconnection (10× exponential backoff), @mention filtering,
 * and kernel event bus — all in compiled Rust for performance.
 */

import { createChannelJs } from 'clawrs-feishu-node';
import type { FeishuChannelJs } from 'clawrs-feishu-node';
import {
  type PluginRuntime,
  type ReplyPayload,
  createReplyPrefixContext,
} from 'openclaw/plugin-sdk';
import { toFeishuConfigInput } from './config-adapter.js';
import { setClawrsFeishuRuntime, getClawrsFeishuRuntime } from './runtime.js';
import { isDuplicate } from './dedup.js';

const CHANNEL_ID = 'clawrs-feishu';

// ── Account Types ──────────────────────────────────────────────────────

interface AccountConfig {
  appId?: string;
  appSecret?: string;
  domain?: string;
  connectionMode?: string;
  dmPolicy?: string;
  groupPolicy?: string;
  allowFrom?: (string | number)[];
  groupAllowFrom?: string[];
  groupRequireMention?: boolean;
  encryptKey?: string;
  verificationToken?: string;
  webhookPort?: number;
  enabled?: boolean;
  name?: string;
}

interface ChannelConfig {
  accounts?: Record<string, AccountConfig>;
  enabled?: boolean;
}

/** Resolve channel config — checks `channels.clawrs-feishu` first,
 *  falls back to `channels.feishu` for drop-in replacement of @openclaw/feishu. */
function getChannelConfig(cfg: Record<string, unknown>): ChannelConfig | undefined {
  const channels = cfg.channels as Record<string, ChannelConfig> | undefined;
  return channels?.[CHANNEL_ID] ?? channels?.['feishu'];
}

function resolveAccount(cfg: Record<string, unknown>, accountId: string): AccountConfig {
  const channelCfg = getChannelConfig(cfg);
  const accounts = channelCfg?.accounts ?? {};
  // When accountId is 'default' or 'main' and no named accounts, use top-level config
  const useDefault =
    (accountId === 'default' || accountId === 'main') && Object.keys(accounts).length === 0;
  const account =
    accounts[accountId] ?? (useDefault ? (channelCfg as Record<string, unknown>) ?? {} : {});
  return {
    dmPolicy: (channelCfg as Record<string, unknown>)?.dmPolicy as string | undefined,
    groupPolicy: (channelCfg as Record<string, unknown>)?.groupPolicy as string | undefined,
    allowFrom: (channelCfg as Record<string, unknown>)?.allowFrom as (string | number)[] | undefined,
    groupAllowFrom: (channelCfg as Record<string, unknown>)?.groupAllowFrom as string[] | undefined,
    groupRequireMention: (channelCfg as Record<string, unknown>)?.groupRequireMention as boolean | undefined,
    ...account,
  };
}

function listAccountIds(cfg: Record<string, unknown>): string[] {
  const channelCfg = getChannelConfig(cfg);
  const accounts = channelCfg?.accounts ?? {};
  if (Object.keys(accounts).length > 0) {
    return Object.keys(accounts);
  }
  // No named accounts - return ['default'] if top-level has appId/appSecret
  const top = channelCfg as Record<string, unknown>;
  const hasTopLevel =
    top && typeof top.appId === 'string' && (top.appId as string).trim() && typeof top.appSecret === 'string' && (top.appSecret as string).trim();
  return hasTopLevel ? ['default'] : [];
}

// ── Rust NAPI channel instances (per-account) ──────────────────────────

const channels = new Map<string, FeishuChannelJs>();

function getChannel(cfg: Record<string, unknown>, accountId: string): FeishuChannelJs | null {
  const acc = resolveAccount(cfg, accountId);
  if (!acc?.appId || !acc?.appSecret) return null;

  let ch = channels.get(accountId);
  if (!ch) {
    const input = toFeishuConfigInput({
      appId: String(acc.appId),
      appSecret: String(acc.appSecret),
      domain: acc.domain,
      connectionMode: acc.connectionMode ?? 'websocket',
      dmPolicy: acc.dmPolicy,
      groupPolicy: acc.groupPolicy,
      allowFrom: acc.allowFrom?.map(String),
      groupAllowFrom: acc.groupAllowFrom,
      groupRequireMention: acc.groupRequireMention,
      encryptKey: acc.encryptKey,
      verificationToken: acc.verificationToken,
      webhookPort: acc.webhookPort,
    });
    ch = createChannelJs(input);
    channels.set(accountId, ch);
  }
  return ch;
}

// ── Parsed message from Rust listen callback ───────────────────────────

interface ListenMessage {
  id: string;
  sender: string;
  content: string;
  channel: string;
  timestamp?: number;
  chat_type?: string;
  mentioned_open_ids?: string[];
}

/** Connection lifecycle signal from Rust NAPI */
interface ListenSignal {
  __signal__: 'connected' | 'disconnected' | 'error';
  error?: string;
}

function isSignalMessage(data: unknown): data is ListenSignal {
  return typeof data === 'object' && data !== null && '__signal__' in data;
}

// ── Inbound message handling ───────────────────────────────────────────

async function handleInboundMessage(params: {
  cfg: Record<string, unknown>;
  msg: ListenMessage;
  accountId: string;
  ch: FeishuChannelJs;
  log: (msg: string) => void;
  error: (msg: string) => void;
  statusSink?: (patch: Record<string, unknown>) => void;
}): Promise<void> {
  const { cfg, msg, accountId, ch, log, error, statusSink } = params;
  const core = getClawrsFeishuRuntime();
  const isGroup = msg.chat_type === 'group';
  const peerId = isGroup ? msg.channel : msg.sender;

  statusSink?.({ lastInboundAt: Date.now() });

  const route = core.channel.routing.resolveAgentRoute({
    cfg,
    channel: CHANNEL_ID,
    accountId,
    peer: { kind: isGroup ? 'group' : 'direct', id: peerId },
  });

  const envelopeOptions = core.channel.reply.resolveEnvelopeFormatOptions(cfg);
  const envelopeFrom = isGroup ? `${msg.channel}:${msg.sender}` : msg.sender;
  const body = core.channel.reply.formatAgentEnvelope({
    channel: 'Feishu',
    from: envelopeFrom,
    timestamp: new Date(),
    envelope: envelopeOptions,
    body: `${msg.sender}: ${msg.content}`,
  });

  const sessionKey = `feishu:${isGroup ? msg.channel : msg.sender}`;

  const ctxPayload = core.channel.reply.finalizeInboundContext({
    Body: body,
    BodyForAgent: msg.content,
    RawBody: msg.content,
    CommandBody: msg.content,
    From: msg.sender,
    To: msg.channel,
    SessionKey: sessionKey,
    AccountId: accountId,
    ChatType: isGroup ? 'group' : 'direct',
    GroupSubject: isGroup ? msg.channel : undefined,
    SenderName: msg.sender,
    SenderId: msg.sender,
    Provider: CHANNEL_ID,
    Surface: CHANNEL_ID,
    MessageSid: msg.id,
    Timestamp: Date.now(),
    WasMentioned: (msg.mentioned_open_ids?.length ?? 0) > 0,
    CommandAuthorized: true,
    OriginatingChannel: CHANNEL_ID,
    OriginatingTo: msg.channel,
    ConversationLabel: msg.channel,
    DeliveryContext: {
      channel: CHANNEL_ID,
      to: msg.channel,
      accountId,
    },
  });

  const prefixContext = createReplyPrefixContext({ cfg, agentId: route.agentId });
  const textChunkLimit = core.channel.text.resolveTextChunkLimit(cfg, CHANNEL_ID, accountId, { fallbackLimit: 4000 });
  const chunkMode = core.channel.text.resolveChunkMode(cfg, CHANNEL_ID);

  const { dispatcher, replyOptions, markDispatchIdle } =
    core.channel.reply.createReplyDispatcherWithTyping({
      responsePrefix: prefixContext.responsePrefix,
      responsePrefixContextProvider: prefixContext.responsePrefixContextProvider,
      humanDelay: core.channel.reply.resolveHumanDelayConfig(cfg, route.agentId),
      deliver: async (payload: ReplyPayload) => {
        const text = payload.text ?? '';
        if (!text.trim()) return;
        const converted = core.channel.text.convertMarkdownTables(text, core.channel.text.resolveMarkdownTableMode({ cfg, channel: CHANNEL_ID }));
        const target = isGroup ? msg.channel : msg.sender;
        const replyToMessageId = msg.id;
        statusSink?.({ lastOutboundAt: Date.now() });
        for (const chunk of core.channel.text.chunkTextWithMode(converted, textChunkLimit, chunkMode)) {
          await ch.reply(chunk, replyToMessageId, target);
        }
      },
      onError: (err: unknown, info: { kind: string }) => {
        error(`clawrs-feishu[${accountId}] ${info.kind} reply failed: ${String(err)}`);
      },
    });

  log(`clawrs-feishu[${accountId}]: dispatching to agent (session=${sessionKey})`);
  await core.channel.reply.dispatchReplyFromConfig({
    ctx: ctxPayload,
    cfg,
    dispatcher,
    replyOptions,
  });
  markDispatchIdle();
}

// ── Plugin Definition ──────────────────────────────────────────────────

const plugin = {
  id: CHANNEL_ID,
  meta: {
    id: CHANNEL_ID,
    label: 'Feishu (Claw)',
    selectionLabel: 'Feishu (Rust/Claw)',
    docsPath: '/channels/clawrs-feishu',
    blurb: 'Feishu via clawrs-feishu-channel (Rust).',
    aliases: ['feishu', 'feishu-rust', 'feishu-claw'],
    replaces: '@openclaw/feishu',
    order: 85,
  },
  capabilities: {
    chatTypes: ['direct', 'group'] as const,
    media: false,
    blockStreaming: true,
  },
  reload: { configPrefixes: [`channels.${CHANNEL_ID}`] },
  config: {
    listAccountIds: (cfg: Record<string, unknown>) => listAccountIds(cfg),
    resolveAccount: (cfg: Record<string, unknown>, accountId: string) => {
      const acc = resolveAccount(cfg, accountId ?? 'default');
      return { ...acc, accountId };
    },
    isConfigured: (account: Record<string, unknown>) => !!(account?.appId && account?.appSecret),
    describeAccount: (account: Record<string, unknown>) => ({
      accountId: (account?.accountId as string) ?? 'main',
      name: (account?.name as string) ?? undefined,
      configured: !!(account?.appId && account?.appSecret),
      enabled: (account?.enabled as boolean) !== false,
    }),
    resolveAllowFrom: ({ cfg, accountId }: { cfg: Record<string, unknown>; accountId?: string }) =>
      (resolveAccount(cfg, accountId ?? 'default').allowFrom ?? []).map(String),
    formatAllowFrom: ({ allowFrom }: { allowFrom: string[] }) =>
      allowFrom
        .map((e) => String(e).trim())
        .filter(Boolean)
        .map((e) => e.replace(/^(feishu|lark|fs):/i, ''))
        .map((e) => e.toLowerCase()),
  },
  security: {
    resolveDmPolicy: ({ cfg, accountId }: { cfg: Record<string, unknown>; accountId?: string }) => {
      const acc = resolveAccount(cfg, accountId ?? 'default');
      return {
        policy: acc.dmPolicy ?? 'pairing',
        allowFrom: (acc.allowFrom ?? []).map(String),
        policyPath: `channels.${CHANNEL_ID}.dmPolicy`,
        allowFromPath: `channels.${CHANNEL_ID}.`,
        normalizeEntry: (raw: string) => raw.replace(/^(feishu|lark|fs):/i, ''),
      };
    },
  },
  groups: {
    resolveRequireMention: () => true,
  },
  threading: {
    resolveReplyToMode: () => 'off' as const,
  },
  messaging: {
    normalizeTarget: (raw: string) => {
      const t = raw?.trim();
      if (!t) return undefined;
      return t.replace(/^(feishu|lark|fs):/i, '');
    },
    targetResolver: {
      looksLikeId: (raw: string) => {
        const t = raw?.trim() ?? '';
        return /^(oc|ou|on)_[a-f0-9]+$/i.test(t) || /^[a-f0-9]{20,}$/i.test(t);
      },
      hint: '<chatId>',
    },
  },
  outbound: {
    deliveryMode: 'direct' as const,
    textChunkLimit: 4000,
    sendText: async ({ cfg, to, text, accountId }: { cfg: Record<string, unknown>; to: string; text: string; accountId?: string | null }) => {
      const aid = accountId ?? 'main';
      const ch = getChannel(cfg, aid);
      if (!ch) return { channel: CHANNEL_ID, ok: false, messageId: '', error: new Error(`no channel for account ${aid}`) };
      try {
        await ch.send(text, to);
        return { channel: CHANNEL_ID, ok: true, messageId: '' };
      } catch (err) {
        return { channel: CHANNEL_ID, ok: false, messageId: '', error: err instanceof Error ? err : new Error(String(err)) };
      }
    },
    sendMedia: async ({ cfg, to, text, mediaUrl, accountId }: { cfg: Record<string, unknown>; to: string; text?: string; mediaUrl?: string; accountId?: string | null }) => {
      if (mediaUrl) {
        return {
          channel: CHANNEL_ID,
          ok: false,
          messageId: '',
          error: new Error('Media not yet supported'),
        };
      }
      const aid = accountId ?? 'main';
      const ch = getChannel(cfg, aid);
      if (!ch) return { channel: CHANNEL_ID, ok: false, messageId: '', error: new Error(`no channel for account ${aid}`) };
      const body = text?.trim() || '(no content)';
      try {
        await ch.send(body, to);
        return { channel: CHANNEL_ID, ok: true, messageId: '' };
      } catch (err) {
        return { channel: CHANNEL_ID, ok: false, messageId: '', error: err instanceof Error ? err : new Error(String(err)) };
      }
    },
  },
  status: {
    defaultRuntime: {
      accountId: 'default',
      running: false,
      lastStartAt: null,
      lastStopAt: null,
      lastError: null,
    },
    buildChannelSummary: ({ snapshot }: { snapshot: Record<string, unknown> }) => ({
      configured: (snapshot.configured as boolean) ?? false,
      running: (snapshot.running as boolean) ?? false,
      mode: 'websocket',
      lastStartAt: snapshot.lastStartAt ?? null,
      lastStopAt: snapshot.lastStopAt ?? null,
      lastError: snapshot.lastError ?? null,
    }),
    buildAccountSnapshot: ({ account, runtime }: { account: Record<string, unknown>; runtime?: Record<string, unknown> }) => ({
      accountId: account.accountId,
      name: account.name,
      enabled: (account.enabled as boolean) !== false,
      configured: Boolean((account.appId as string)?.trim() && (account.appSecret as string)?.trim()),
      running: (runtime?.running as boolean) ?? false,
      lastStartAt: runtime?.lastStartAt ?? null,
      lastStopAt: runtime?.lastStopAt ?? null,
      lastError: runtime?.lastError ?? null,
      mode: 'websocket',
      lastInboundAt: runtime?.lastInboundAt ?? null,
      lastOutboundAt: runtime?.lastOutboundAt ?? null,
      dmPolicy: (account.dmPolicy as string) ?? 'pairing',
    }),
  },
  gateway: {
    startAccount: async (ctx: {
      cfg: Record<string, unknown>;
      accountId: string;
      log?: { info?: (msg: string) => void; error?: (msg: string) => void };
      abortSignal?: AbortSignal;
      setStatus?: (patch: Record<string, unknown>) => void;
    }) => {
      const { cfg, accountId } = ctx;
      const log = ctx.log?.info ?? console.log;
      const error = ctx.log?.error ?? console.error;
      const statusSink = ctx.setStatus;

      const ch = getChannel(cfg, accountId);
      if (!ch) {
        error(`clawrs-feishu[${accountId}]: no channel (missing appId/appSecret)`);
        return { stop: () => { } };
      }

      log(`clawrs-feishu[${accountId}]: starting listen (Rust NAPI)`);
      statusSink?.({ running: true, lastStartAt: Date.now() });

      // Rust NAPI listen — all WebSocket/security/filtering handled in Rust
      ch.listen((jsonStr: string) => {
        try {
          const parsed = JSON.parse(jsonStr) as ListenMessage | ListenSignal;

          // Handle connection lifecycle signals from Rust
          if (isSignalMessage(parsed)) {
            if (parsed.__signal__ === 'connected') {
              log(`clawrs-feishu[${accountId}]: WebSocket connected`);
              statusSink?.({ running: true });
            } else if (parsed.__signal__ === 'disconnected') {
              log(`clawrs-feishu[${accountId}]: WebSocket disconnected: ${parsed.error ?? 'unknown'}`);
              statusSink?.({ running: false, lastStopAt: Date.now(), lastError: parsed.error ?? null });
            } else if (parsed.__signal__ === 'error') {
              error(`clawrs-feishu[${accountId}]: listen error: ${parsed.error ?? 'unknown'}`);
              statusSink?.({ running: false, lastStopAt: Date.now(), lastError: parsed.error ?? null });
            }
            return;
          }

          const msg = parsed as ListenMessage;

          // JS-level dedup (Rust already does security/mention filtering)
          if (isDuplicate(msg.id)) return;

          log(`clawrs-feishu[${accountId}]: received from ${msg.sender} in ${msg.channel}`);
          void handleInboundMessage({ cfg, msg, accountId, ch, log, error, statusSink }).catch((err) => {
            error(`clawrs-feishu[${accountId}]: handle error: ${String(err)}`);
          });
        } catch (parseErr) {
          error(`clawrs-feishu[${accountId}]: parse error: ${String(parseErr)}`);
        }
      });

      const stop = () => {
        channels.delete(accountId);
        log(`clawrs-feishu[${accountId}]: stopped`);
        statusSink?.({ running: false, lastStopAt: Date.now() });
      };

      if (ctx.abortSignal) {
        ctx.abortSignal.addEventListener('abort', stop, { once: true });
      }

      return { stop };
    },
  },
};

// ── Plugin Registration ────────────────────────────────────────────────

export default function register(api: { registerChannel: (opts: { plugin: typeof plugin }) => void; runtime?: PluginRuntime }): void {
  if (api.runtime) {
    setClawrsFeishuRuntime(api.runtime);
  }
  api.registerChannel({ plugin });
}
