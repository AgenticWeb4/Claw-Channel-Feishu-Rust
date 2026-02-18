#!/usr/bin/env bash
# Clawrs-feishu channel status E2E test
# Verifies clawrs-feishu is configured and ready for inbound messages.
# Requires: OpenClaw gateway running, clawrs-feishu plugin loaded.

set -euo pipefail

GATEWAY_URL="${OPENCLAW_GATEWAY_URL:-http://127.0.0.1:18080}"

# Check gateway health first
if ! curl -sf "${GATEWAY_URL}/health" > /dev/null 2>&1; then
  echo "✗ OpenClaw gateway not reachable at ${GATEWAY_URL}"
  echo "  Start: openclaw gateway"
  exit 1
fi

# Get channels.status via openclaw CLI (WebSocket)
OUTPUT=$(openclaw gateway call channels.status --params '{"probe":false}' 2>/dev/null || true)

if ! echo "$OUTPUT" | grep -q '"clawrs-feishu"'; then
  echo "✗ clawrs-feishu not in channel list"
  echo "$OUTPUT" | head -20
  exit 1
fi

# Parse configured from JSON (simple grep; full parsing would need jq)
if echo "$OUTPUT" | grep -q '"clawrs-feishu".*"configured".*true'; then
  echo "✓ clawrs-feishu configured=true"
elif echo "$OUTPUT" | grep -A5 '"clawrs-feishu"' | grep -q '"configured".*true'; then
  echo "✓ clawrs-feishu configured=true"
else
  # Check channelAccounts for main account configured
  if echo "$OUTPUT" | grep -q '"configured".*true'; then
    echo "✓ clawrs-feishu account configured"
  else
    echo "✗ clawrs-feishu not configured (check appId/appSecret in openclaw.json)"
    echo "$OUTPUT" | head -40
    exit 1
  fi
fi

echo "✓ Clawrs-feishu status OK — ready for inbound messages"
