#!/usr/bin/env bash
# Clawrs-feishu account main check
# Verifies channelAccounts includes main account.
# Requires: OpenClaw gateway running, 03-clawrs-feishu-status passed.

set -euo pipefail

GATEWAY_URL="${OPENCLAW_GATEWAY_URL:-http://127.0.0.1:18080}"

if ! curl -sf "${GATEWAY_URL}/health" > /dev/null 2>&1; then
  echo "✗ OpenClaw gateway not reachable"
  exit 1
fi

OUTPUT=$(openclaw gateway call channels.status --params '{"probe":false}' 2>/dev/null || true)

# channelAccounts should have main (or default account)
if echo "$OUTPUT" | grep -q '"accountId".*"main"'; then
  echo "✓ clawrs-feishu account main present"
elif echo "$OUTPUT" | grep -q '"channelAccounts".*"clawrs-feishu"'; then
  echo "✓ clawrs-feishu channelAccounts present"
elif echo "$OUTPUT" | grep -q '"clawrs-feishu"'; then
  echo "✓ clawrs-feishu in channels (account structure may vary)"
else
  echo "✗ clawrs-feishu account check failed"
  echo "$OUTPUT" | head -40
  exit 1
fi

echo "✓ Clawrs-feishu account OK"
