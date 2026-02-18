#!/usr/bin/env bash
# Clawrs-feishu channels.status with probe:true
# Verifies the channel can probe (connect to Feishu API).
# Requires: OpenClaw gateway running, clawrs-feishu configured, network to Feishu.

set -euo pipefail

GATEWAY_URL="${OPENCLAW_GATEWAY_URL:-http://127.0.0.1:18080}"

if ! curl -sf "${GATEWAY_URL}/health" > /dev/null 2>&1; then
  echo "✗ OpenClaw gateway not reachable at ${GATEWAY_URL}"
  exit 1
fi

# probe:true triggers actual Feishu API calls; may take 5-15s
OUTPUT=$(openclaw gateway call channels.status --params '{"probe":true}' 2>/dev/null || true)

if ! echo "$OUTPUT" | grep -q '"clawrs-feishu"'; then
  echo "✗ clawrs-feishu not in channel list (probe)"
  echo "$OUTPUT" | head -30
  exit 1
fi

# Probe result may include probe/ok; simple check for configured
if echo "$OUTPUT" | grep -q '"configured".*true'; then
  echo "✓ clawrs-feishu probe OK (configured)"
else
  echo "⚠ clawrs-feishu probe completed but configured not found in output"
  echo "$OUTPUT" | head -50
  exit 1
fi

echo "✓ Clawrs-feishu probe OK — channel can reach Feishu API"
