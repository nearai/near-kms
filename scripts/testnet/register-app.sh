#!/bin/bash
set -e

export NEAR_ENV=testnet
export APP_ID=example-app
export OWNER_ACCOUNT=kms-owner.testnet
export KMS_CONTRACT=kms-dev.testnet

# Deposit amount for account creation (in NEAR)
# Adjust this based on your needs - typically 5-10 NEAR is sufficient
export DEPOSIT_AMOUNT=3.5

# Register app using KMS contract's register_app function
# The app will be deployed as a subaccount: {app_id}.{kms_contract}
near call $KMS_CONTRACT register_app '{
  "app_id": "'$APP_ID'",
  "owner_id": "'$OWNER_ACCOUNT'",
  "disable_upgrades": false,
  "allow_any_device": false,
  "initial_device_id": null,
  "initial_compose_hash": null
}' --accountId $OWNER_ACCOUNT --deposit $DEPOSIT_AMOUNT --gas 300000000000000

echo "App registration initiated. App will be deployed at: ${APP_ID}.${KMS_CONTRACT}"
