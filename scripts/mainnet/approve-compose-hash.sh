#!/bin/bash
set -e

export NEAR_ENV=mainnet
export KMS_CONTRACT=kms.near
export OWNER_ACCOUNT=owner.near

# Compose hash from test constants
# Replace with your actual compose hash
export COMPOSE_HASH="f68cf65a75ad980289144ef3c096a007fc4583ea6d1f90589757f229dbc6cdab"

# Add compose hash to KMS contract
near call $KMS_CONTRACT add_kms_compose_hash '{
  "compose_hash": "'$COMPOSE_HASH'"
}' --accountId $OWNER_ACCOUNT

echo "Compose hash approved: $COMPOSE_HASH"
