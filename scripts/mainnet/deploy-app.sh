#!/bin/bash
set -e

export NEAR_ENV=mainnet
export APP_CONTRACT=app.near
export OWNER_ACCOUNT=owner.near
export KMS_CONTRACT_ID=kms.near

# Deploy App contract
near deploy $APP_CONTRACT ../../target/near/near_dstack_app/near_dstack_app.wasm \
  --initFunction new \
  --initArgs '{
    "owner_id": "'$OWNER_ACCOUNT'",
    "disable_upgrades": false,
    "allow_any_device": false,
    "initial_device_id": null,
    "initial_compose_hash": null,
    "kms_contract_id": "'$KMS_CONTRACT_ID'"
  }' \
  --accountId $OWNER_ACCOUNT

echo "App contract deployed at: $APP_CONTRACT"
