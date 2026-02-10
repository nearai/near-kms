#!/bin/bash
set -e

export NEAR_ENV=testnet
export KMS_CONTRACT=kms.testnet
export OWNER_ACCOUNT=owner.testnet
export MPC_CONTRACT_ID=v1.signer-prod.testnet
export MPC_DOMAIN_ID=2

# Deploy KMS contract
near deploy $KMS_CONTRACT ../../target/near/near_dstack_kms/near_dstack_kms.wasm \
  --initFunction new \
  --initArgs '{
    "owner_id": "'$OWNER_ACCOUNT'",
    "mpc_contract_id": "'$MPC_CONTRACT_ID'",
    "mpc_domain_id": '$MPC_DOMAIN_ID'
  }' \
  --accountId $OWNER_ACCOUNT

echo "KMS contract deployed at: $KMS_CONTRACT"
