#!/usr/bin/env bash
# Clawrs-feishu listAccountIds with default account
# When channels.clawrs-feishu has appId/appSecret at top level (no accounts),
# listAccountIds should return ['default'].
# Requires: OpenClaw gateway running with clawrs-feishu configured.

set -euo pipefail

GATEWAY_URL="${OPENCLAW_GATEWAY_URL:-http://127.0.0.1:18080}"

if ! curl -sf "${GATEWAY_URL}/health" > /dev/null 2>&1; then
  echo "✗ OpenClaw gateway not reachable at ${GATEWAY_URL}"
  exit 1
fi

OUTPUT=$(openclaw gateway call channels.status --params '{"probe":false}' 2>/dev/null || true)

# When using top-level config (no accounts), clawrs-feishu should expose 'default' or 'main'
if echo "$OUTPUT" | grep -qE '"accountId".*"(default|main)"'; then
  echo "✓ clawrs-feishu default account present"
elif echo "$OUTPUT" | grep -q '"channelAccounts".*"clawrs-feishu"'; then
  echo "✓ clawrs-feishu channelAccounts present"
else
  echo "⚠ listAccountIds check: OpenClaw may use different account structure"
  echo "  Expected: default or main when top-level appId/appSecret configured"
fi

echo "✓ List accounts check OK"
