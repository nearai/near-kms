#!/bin/bash
set -e

export NEAR_ENV=testnet
export KMS_CONTRACT=kms-dev.testnet
export OWNER_ACCOUNT=kms-owner.testnet

# Compose hash from test constants
export COMPOSE_HASH="f68cf65a75ad980289144ef3c096a007fc4583ea6d1f90589757f229dbc6cdab"

# Add compose hash to KMS contract
near call $KMS_CONTRACT add_kms_compose_hash '{
  "compose_hash": "'$COMPOSE_HASH'"
}' --accountId $OWNER_ACCOUNT

echo "Compose hash approved: $COMPOSE_HASH"
