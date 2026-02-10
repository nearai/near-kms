#!/bin/bash
set -e

export NEAR_ENV=testnet
export KMS_CONTRACT=kms-dev.testnet
export ACCOUNT=kms-alice.testnet

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Read quote hex from text file (Alice)
# The quote hex is very long (8000+ chars) and must be read from file to avoid truncation
QUOTE_HEX_FILE="$PROJECT_ROOT/contracts/kms/tests/samples/alice/quote_hex.txt"
if [ ! -f "$QUOTE_HEX_FILE" ]; then
    echo "Error: Quote hex file not found: $QUOTE_HEX_FILE"
    exit 1
fi

# Read quote hex directly from file (strip trailing newline if present)
export QUOTE_HEX=$(cat "$QUOTE_HEX_FILE" | tr -d '\n')

# Worker public key from test (BLS12-381 G1 public key)
export WORKER_PUBLIC_KEY="a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456"

# Read collateral and tcb_info from JSON files
COLLATERAL_FILE="$PROJECT_ROOT/contracts/kms/tests/samples/alice/quote_collateral.json"
TCB_INFO_FILE="$PROJECT_ROOT/contracts/kms/tests/samples/alice/tcb_info.json"

if [ ! -f "$COLLATERAL_FILE" ]; then
    echo "Error: Collateral file not found: $COLLATERAL_FILE"
    exit 1
fi

if [ ! -f "$TCB_INFO_FILE" ]; then
    echo "Error: TCB info file not found: $TCB_INFO_FILE"
    exit 1
fi

# Verify quote hex was read correctly
if [ -z "$QUOTE_HEX" ]; then
    echo "Error: Failed to read QUOTE_HEX from file"
    exit 1
fi

QUOTE_HEX_LENGTH=${#QUOTE_HEX}
echo "Quote hex length: $QUOTE_HEX_LENGTH characters (expected ~10011)"

if [ "$QUOTE_HEX_LENGTH" -lt 10000 ]; then
    echo "Warning: Quote hex appears to be truncated! Expected ~10011 characters, got $QUOTE_HEX_LENGTH"
    echo "First 100 chars: ${QUOTE_HEX:0:100}..."
    echo "Last 100 chars: ...${QUOTE_HEX: -100}"
fi

# Read JSON files as strings (they will be passed as JSON string values)
COLLATERAL=$(cat "$COLLATERAL_FILE" | jq -c .)
TCB_INFO=$(cat "$TCB_INFO_FILE" | jq -c .)

# Build the arguments JSON using jq
# Read quote_hex directly from file to avoid shell variable size limitations
ARGS_JSON=$(jq -n \
  --arg quote_hex "$(cat "$QUOTE_HEX_FILE" | tr -d '\n')" \
  --arg collateral "$COLLATERAL" \
  --arg tcb_info "$TCB_INFO" \
  --arg worker_public_key "$WORKER_PUBLIC_KEY" \
  '{
    quote_hex: $quote_hex,
    collateral: $collateral,
    tcb_info: $tcb_info,
    worker_public_key: $worker_public_key
  }')

# Request root key from MPC
near call $KMS_CONTRACT request_kms_root_key "$ARGS_JSON" \
  --gas 300000000000000 \
  --accountId $ACCOUNT \
  --depositYocto 1

echo "Root key request submitted to KMS contract: $KMS_CONTRACT"
