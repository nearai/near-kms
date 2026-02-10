#!/bin/bash
set -e

export NEAR_ENV=mainnet
export KMS_CONTRACT=kms.near
export ACCOUNT=owner.near

# Example values - replace with actual values
export QUOTE_HEX="your_quote_hex_here"
export COLLATERAL='{"your": "collateral", "json": "here"}'
export TCB_INFO='{"your": "tcb_info", "json": "here"}'
export WORKER_PUBLIC_KEY="your_bls12381_g1_public_key_here"

# Request root key from MPC
near call $KMS_CONTRACT request_kms_root_key '{
  "quote_hex": "'$QUOTE_HEX'",
  "collateral": "'$COLLATERAL'",
  "tcb_info": "'$TCB_INFO'",
  "worker_public_key": "'$WORKER_PUBLIC_KEY'"
}' --accountId $ACCOUNT --depositYocto 1

echo "Root key request submitted to KMS contract: $KMS_CONTRACT"
