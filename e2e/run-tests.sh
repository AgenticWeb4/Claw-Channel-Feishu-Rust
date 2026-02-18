#!/bin/bash
# Claw Feishu Channel - E2E Test Runner
# Usage: ./run-tests.sh [api|zeroclaw|openclaw|gateway|all]
#
# api       - Run tests against Feishu API (no gateway required)
# zeroclaw  - Run ZeroClaw gateway tests (06-zeroclaw)
# openclaw  - Run OpenClaw gateway tests (08-openclaw)
# gateway   - Run all gateway tests (zeroclaw + openclaw)
# all       - Run all tests

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

MODE="${1:-api}"

echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Claw Feishu Channel E2E Tests           ║${NC}"
echo -e "${BLUE}║  Mode: ${YELLOW}${MODE}${BLUE}                                ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
echo ""

# Check bru CLI
if ! command -v bru &> /dev/null; then
    echo -e "${RED}Error: bru CLI not found. Install: npm install -g @usebruno/cli${NC}"
    exit 1
fi

# Check .env file
if [ ! -f .env ]; then
    echo -e "${RED}Error: .env file not found. Copy .env.example to .env and fill in credentials.${NC}"
    exit 1
fi

# Load env vars
set -a
source .env
set +a

PASS=0
FAIL=0

run_suite() {
    local suite="$1"
    local desc="$2"
    echo -e "${BLUE}━━━ ${desc} ━━━${NC}"
    if bru run $suite --env dev 2>&1; then
        echo -e "${GREEN}✓ ${desc} PASSED${NC}"
        ((PASS++))
    else
        echo -e "${RED}✗ ${desc} FAILED${NC}"
        ((FAIL++))
    fi
    echo ""
}

# API tests (no gateway required)
# IMPORTANT: Dependent suites (01-auth, 02-health, 03-messages API tests)
# run together in a single bru invocation so tenant_access_token persists.
run_api_tests() {
    echo -e "${GREEN}▶ Running API tests (Feishu API only)${NC}"
    echo ""

    # Combined run: auth + health + messages (token persists across all)
    echo -e "${BLUE}━━━ Core API (Auth + Health + Messages) ━━━${NC}"
    if bru run 01-auth 02-health 03-messages/01-send-text.bru 03-messages/05-performance.bru 03-messages/06-error-handling.bru 03-messages/07-send-unicode.bru 03-messages/08-invalid-chat-id.bru --env dev 2>&1; then
        echo -e "${GREEN}✓ Core API Tests PASSED${NC}"
        ((PASS++))
    else
        echo -e "${RED}✗ Core API Tests FAILED${NC}"
        ((FAIL++))
    fi
    echo ""

    # Edge cases (needs token from auth above -- run combined)
    echo -e "${BLUE}━━━ Edge Cases ━━━${NC}"
    if bru run 01-auth 08-edge-cases --env dev 2>&1; then
        echo -e "${GREEN}✓ Edge Cases PASSED${NC}"
        ((PASS++))
    else
        echo -e "${RED}✗ Edge Cases FAILED${NC}"
        ((FAIL++))
    fi
    echo ""

    # Independent suites (don't need prior token -- they fetch their own)
    run_suite "05-websocket" "WebSocket Endpoint"
    run_suite "07-open-lark" "open-lark SDK Integration"
}

# ZeroClaw gateway tests (06-zeroclaw)
run_zeroclaw_tests() {
    echo -e "${YELLOW}▶ Running ZeroClaw gateway tests${NC}"
    echo ""

    local gateway_url="${ZEROCLAW_GATEWAY_URL:-http://127.0.0.1:8080}"
    if ! curl -sf "${gateway_url}/health" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠ ZeroClaw gateway not running at ${gateway_url}${NC}"
        echo -e "${YELLOW}  Skipping. Start: zeroclaw gateway --port 8080${NC}"
        echo ""
        return
    fi

    run_suite "06-zeroclaw" "ZeroClaw Gateway"
}

# OpenClaw gateway tests (08-openclaw)
run_openclaw_tests() {
    echo -e "${YELLOW}▶ Running OpenClaw gateway tests${NC}"
    echo ""

    local gateway_url="${OPENCLAW_GATEWAY_URL:-http://127.0.0.1:18080}"
    if ! curl -sf "${gateway_url}/health" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠ OpenClaw gateway not running at ${gateway_url}${NC}"
        echo -e "${YELLOW}  Skipping. Start: openclaw start${NC}"
        echo ""
        return
    fi

    run_suite "08-openclaw" "OpenClaw Gateway"

    # Clawrs-feishu channel status (shell script; Bruno does not support exec type)
    echo -e "${BLUE}━━━ Clawrs-feishu Channel Status ━━━${NC}"
    if bash "$SCRIPT_DIR/08-openclaw/03-clawrs-feishu-status.sh" 2>&1; then
        echo -e "${GREEN}✓ Clawrs-feishu Status PASSED${NC}"
        ((PASS++))
    else
        echo -e "${RED}✗ Clawrs-feishu Status FAILED${NC}"
        ((FAIL++))
    fi

    echo -e "${BLUE}━━━ Clawrs-feishu Channels Status (probe) ━━━${NC}"
    if bash "$SCRIPT_DIR/08-openclaw/04-channels-status-probe.sh" 2>&1; then
        echo -e "${GREEN}✓ Clawrs-feishu Probe PASSED${NC}"
        ((PASS++))
    else
        echo -e "${RED}✗ Clawrs-feishu Probe FAILED${NC}"
        ((FAIL++))
    fi

    echo -e "${BLUE}━━━ Clawrs-feishu Account Main ━━━${NC}"
    if bash "$SCRIPT_DIR/08-openclaw/05-clawrs-feishu-account-main.sh" 2>&1; then
        echo -e "${GREEN}✓ Clawrs-feishu Account PASSED${NC}"
        ((PASS++))
    else
        echo -e "${RED}✗ Clawrs-feishu Account FAILED${NC}"
        ((FAIL++))
    fi
    echo ""
}

# Gateway tests (ZeroClaw + OpenClaw + message/security)
run_gateway_tests() {
    echo -e "${YELLOW}▶ Running Gateway tests (ZeroClaw + OpenClaw)${NC}"
    echo ""

    local zeroclaw_url="${ZEROCLAW_GATEWAY_URL:-http://127.0.0.1:8080}"
    if curl -sf "${zeroclaw_url}/health" > /dev/null 2>&1; then
        # Combined run: auth first (for token), then gateway-dependent tests
        echo -e "${BLUE}━━━ Gateway Message Integration ━━━${NC}"
        if bru run 01-auth 03-messages/02-receive-text.bru 03-messages/03-group-mention.bru 03-messages/04-multi-mention.bru --env dev 2>&1; then
            echo -e "${GREEN}✓ Gateway Message Tests PASSED${NC}"
            ((PASS++))
        else
            echo -e "${RED}✗ Gateway Message Tests FAILED${NC}"
            ((FAIL++))
        fi
        echo ""

        run_suite "04-security" "Security Allowlist"
        run_suite "06-zeroclaw" "ZeroClaw Gateway"
    else
        echo -e "${YELLOW}⚠ ZeroClaw gateway not running at ${zeroclaw_url}${NC}"
        echo -e "${YELLOW}  Skipping message/security/zeroclaw tests${NC}"
        echo ""
    fi

    # OpenClaw tests (independent)
    run_openclaw_tests
}

case "$MODE" in
    api)
        run_api_tests
        ;;
    zeroclaw)
        run_zeroclaw_tests
        ;;
    openclaw)
        run_openclaw_tests
        ;;
    gateway)
        run_gateway_tests
        ;;
    all)
        run_api_tests
        run_gateway_tests
        ;;
    *)
        echo -e "${RED}Unknown mode: $MODE${NC}"
        echo "Usage: $0 [api|zeroclaw|openclaw|gateway|all]"
        exit 1
        ;;
esac

# Summary
echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Test Summary                            ║${NC}"
echo -e "${BLUE}╠══════════════════════════════════════════╣${NC}"
echo -e "${BLUE}║  ${GREEN}Passed: ${PASS}${BLUE}                                ║${NC}"
echo -e "${BLUE}║  ${RED}Failed: ${FAIL}${BLUE}                                ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
