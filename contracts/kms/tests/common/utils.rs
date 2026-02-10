use std::str::FromStr;

use near_gas::NearGas;
use near_sdk::serde_json::{self, json};
use near_sdk::{AccountId, NearToken};
use near_workspaces::{
    Account, Contract, Worker, network::Sandbox, result::ExecutionFinalResult, types::SecretKey,
};

use super::constants::{SECRET_KEY_ALICE, SECRET_KEY_BOB};

pub const KMS_CONTRACT_WASM: &str = "../../target/near/near_dstack_kms/near_dstack_kms.wasm";
pub const APP_CONTRACT_WASM: &str = "../../target/near/near_dstack_app/near_dstack_app.wasm";
pub const MOCK_MPC_CONTRACT_WASM: &str = "../../contracts/mock-mpc/res/mock_mpc.wasm";
pub const MPC_DOMAIN_ID: u64 = 2;

pub async fn create_account(
    sandbox: &Worker<Sandbox>,
    prefix: &str,
    balance: u128,
) -> Result<Account, Box<dyn std::error::Error>> {
    let root = sandbox.root_account().unwrap();
    Ok(root
        .create_subaccount(prefix)
        .initial_balance(NearToken::from_near(balance))
        .transact()
        .await?
        .result)
}

pub async fn create_account_with_secret_key(
    sandbox: &Worker<Sandbox>,
    prefix: &str,
    balance: u128,
    secret_key: SecretKey,
) -> Result<Account, Box<dyn std::error::Error>> {
    let root = sandbox.root_account().unwrap();
    Ok(root
        .create_subaccount(prefix)
        .initial_balance(NearToken::from_near(balance))
        .keys(secret_key)
        .transact()
        .await?
        .result)
}

// Helper function to create test accounts (owner, alice, bob)
pub async fn create_test_accounts(
    sandbox: &Worker<Sandbox>,
) -> Result<(Account, Account, Account), Box<dyn std::error::Error>> {
    let owner = create_account(sandbox, "owner", 10).await?;
    let alice = create_account_with_secret_key(
        sandbox,
        "alice",
        10,
        SecretKey::from_str(SECRET_KEY_ALICE).unwrap(),
    )
    .await?;
    let bob = create_account_with_secret_key(
        sandbox,
        "bob",
        10,
        SecretKey::from_str(SECRET_KEY_BOB).unwrap(),
    )
    .await?;

    Ok((owner, alice, bob))
}

// Helper function to print execution logs
pub fn print_logs(result: &ExecutionFinalResult) {
    for (i, log) in result.logs().iter().enumerate() {
        println!("  [{}] {}", i + 1, log);
    }
}

// ============================================================================
// KMS Contract Helper Functions
// ============================================================================

/// Deploy and initialize mock MPC contract
pub async fn deploy_mock_mpc_contract(
    sandbox: &Worker<Sandbox>,
) -> Result<Contract, Box<dyn std::error::Error>> {
    let mpc_wasm = std::fs::read(MOCK_MPC_CONTRACT_WASM).map_err(|e| {
        format!("Failed to read mock MPC contract WASM from {MOCK_MPC_CONTRACT_WASM}: {e}")
    })?;

    let mpc_account = create_account(sandbox, "mock-mpc", 100).await?;

    println!("Deploying mock MPC contract...");
    let deploy_result = mpc_account.deploy(&mpc_wasm).await?;

    assert!(
        deploy_result.is_success(),
        "Mock MPC contract deployment should succeed: {:#?}",
        deploy_result.into_result().unwrap_err()
    );

    let mpc_contract = deploy_result.result;

    println!("Initializing mock MPC contract...");
    let result = mpc_contract.call("new").transact().await?;

    assert!(
        result.is_success(),
        "Mock MPC contract initialization should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(mpc_contract)
}

/// Deploy and initialize KMS contract with mock MPC contract
pub async fn deploy_kms_contract(
    sandbox: &Worker<Sandbox>,
    owner: &Account,
) -> Result<Contract, Box<dyn std::error::Error>> {
    // Deploy mock MPC contract first
    let mpc_contract = deploy_mock_mpc_contract(sandbox).await?;
    let mpc_contract_id = mpc_contract.id().to_string();

    let kms_wasm = std::fs::read(KMS_CONTRACT_WASM)
        .map_err(|e| format!("Failed to read KMS contract WASM from {KMS_CONTRACT_WASM}: {e}"))?;

    let kms_account = create_account(sandbox, "kms", 100).await?;

    println!("Deploying KMS contract...");
    let deploy_result = kms_account.deploy(&kms_wasm).await?;

    assert!(
        deploy_result.is_success(),
        "KMS contract deployment should succeed: {:#?}",
        deploy_result.into_result().unwrap_err()
    );

    let kms_contract = deploy_result.result;

    println!("Initializing KMS contract...");
    let result = kms_contract
        .call("new")
        .args_json(json!({
            "owner_id": owner.id(),
            "mpc_contract_id": mpc_contract_id,
            "mpc_domain_id": MPC_DOMAIN_ID
        }))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "KMS contract initialization should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(kms_contract)
}

/// Add compose hash to KMS contract
pub async fn add_kms_compose_hash(
    owner: &Account,
    kms_contract: &Contract,
    compose_hash: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = owner
        .call(kms_contract.id(), "add_kms_compose_hash")
        .args_json(json!({
            "compose_hash": compose_hash
        }))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Adding KMS compose hash should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(result)
}

/// Remove compose hash from KMS contract
pub async fn remove_kms_compose_hash(
    owner: &Account,
    kms_contract: &Contract,
    compose_hash: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = owner
        .call(kms_contract.id(), "remove_kms_compose_hash")
        .args_json(json!({
            "compose_hash": compose_hash
        }))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Removing KMS compose hash should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(result)
}

/// Request KMS root key from MPC
pub async fn request_kms_root_key(
    account: &Account,
    kms_contract: &Contract,
    quote_hex: &str,
    collateral: &str,
    tcb_info: &str,
    worker_public_key: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = account
        .call(kms_contract.id(), "request_kms_root_key")
        .args_json(json!({
            "quote_hex": quote_hex,
            "collateral": collateral,
            "tcb_info": tcb_info,
            "worker_public_key": worker_public_key
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;

    Ok(result)
}

/// Add OS image hash to KMS contract
pub async fn add_os_image_hash(
    owner: &Account,
    kms_contract: &Contract,
    os_image_hash: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = owner
        .call(kms_contract.id(), "add_os_image_hash")
        .args_json(json!({
            "os_image_hash": os_image_hash
        }))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Adding OS image hash should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(result)
}

/// Remove OS image hash from KMS contract
pub async fn remove_os_image_hash(
    owner: &Account,
    kms_contract: &Contract,
    os_image_hash: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = owner
        .call(kms_contract.id(), "remove_os_image_hash")
        .args_json(json!({
            "os_image_hash": os_image_hash
        }))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Removing OS image hash should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(result)
}

/// Set gateway app ID in KMS contract
pub async fn set_gateway_app_id(
    owner: &Account,
    kms_contract: &Contract,
    app_id: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = owner
        .call(kms_contract.id(), "set_gateway_app_id")
        .args_json(json!({
            "app_id": app_id
        }))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Setting gateway app ID should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(result)
}

// ============================================================================
// App Contract Helper Functions
// ============================================================================

/// Register and deploy App contract via KMS `register_app` function
pub async fn deploy_app_contract(
    sandbox: &Worker<Sandbox>,
    owner: &Account,
    kms_contract: &Contract,
    disable_upgrades: bool,
    allow_any_device: bool,
    initial_device_id: Option<&str>,
    initial_compose_hash: Option<&str>,
) -> Result<Contract, Box<dyn std::error::Error>> {
    // Create a deterministic app account ID as a subaccount of the KMS contract
    // Use a timestamp to make it unique per test
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let app_id = format!("app-{timestamp}")
        .parse::<near_workspaces::types::AccountId>()
        .map_err(|e| format!("Failed to parse app account ID: {e}"))?;

    println!("Registering App contract via KMS...");

    let mut args = json!({
        "app_id": app_id,
        "owner_id": owner.id(),
        "disable_upgrades": disable_upgrades,
        "allow_any_device": allow_any_device
    });

    if let Some(device_id) = initial_device_id {
        args["initial_device_id"] = json!(device_id);
    } else {
        args["initial_device_id"] = serde_json::Value::Null;
    }

    if let Some(compose_hash) = initial_compose_hash {
        args["initial_compose_hash"] = json!(compose_hash);
    } else {
        args["initial_compose_hash"] = serde_json::Value::Null;
    }

    // Call register_app with sufficient deposit for account creation and deployment
    let result = owner
        .call(kms_contract.id(), "register_app")
        .args_json(args)
        .deposit(NearToken::from_near(30))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "App contract registration should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    // Create a Contract instance from the account ID
    // We use a deterministic key just for the Contract wrapper - actual calls use owner.call()
    // so this key is never used for signing
    let app_secret_key = SecretKey::from_seed(
        near_workspaces::types::KeyType::ED25519,
        &format!("app-wrapper-{app_id}"),
    );
    let app_account_id: AccountId = format!("{}.{}", app_id, kms_contract.id()).parse().unwrap();
    let app_contract = Contract::from_secret_key(app_account_id, app_secret_key, sandbox);

    Ok(app_contract)
}

// ============================================================================
// View Helper Functions
// ============================================================================

/// Check if a compose hash is allowed in KMS contract
pub async fn is_kms_compose_hash_allowed(
    kms_contract: &Contract,
    compose_hash: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let result = kms_contract
        .view("is_kms_compose_hash_allowed")
        .args_json(json!({
            "compose_hash": compose_hash
        }))
        .await?;

    let allowed: bool = result.json()?;
    Ok(allowed)
}

/// Get all allowed KMS compose hashes
pub async fn get_kms_allowed_compose_hashes(
    kms_contract: &Contract,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let result = kms_contract
        .view("get_kms_allowed_compose_hashes")
        .args_json(json!({}))
        .await?;

    let compose_hashes: Vec<String> = result.json()?;
    Ok(compose_hashes)
}

/// Get gateway app ID from KMS contract
pub async fn get_gateway_app_id(
    kms_contract: &Contract,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let result = kms_contract
        .view("get_gateway_app_id")
        .args_json(json!({}))
        .await?;

    let gateway_app_id: Option<String> = result.json()?;
    Ok(gateway_app_id)
}

/// Get KMS info from contract
pub async fn get_kms_info(
    kms_contract: &Contract,
) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    let result = kms_contract
        .view("get_kms_info")
        .args_json(json!({}))
        .await?;

    let kms_info: Option<serde_json::Value> = result.json()?;
    Ok(kms_info)
}

/// Get allowed OS images from KMS contract
pub async fn get_allowed_os_images(
    kms_contract: &Contract,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let result = kms_contract
        .view("get_allowed_os_images")
        .args_json(json!({}))
        .await?;

    let os_images: Vec<String> = result.json()?;
    Ok(os_images)
}

/// Check if an OS image is allowed
pub async fn is_os_image_allowed(
    kms_contract: &Contract,
    os_image_hash: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let result = kms_contract
        .view("is_os_image_allowed")
        .args_json(json!({
            "os_image_hash": os_image_hash
        }))
        .await?;

    let allowed: bool = result.json()?;
    Ok(allowed)
}

/// Add compose hash to App contract
pub async fn add_app_compose_hash(
    owner: &Account,
    app_contract: &Contract,
    compose_hash: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = owner
        .call(app_contract.id(), "add_compose_hash")
        .args_json(json!({
            "compose_hash": compose_hash
        }))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Adding app compose hash should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(result)
}

/// Remove compose hash from App contract
pub async fn remove_app_compose_hash(
    owner: &Account,
    app_contract: &Contract,
    compose_hash: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = owner
        .call(app_contract.id(), "remove_compose_hash")
        .args_json(json!({
            "compose_hash": compose_hash
        }))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Removing app compose hash should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(result)
}

/// Add device ID to App contract
pub async fn add_app_device(
    owner: &Account,
    app_contract: &Contract,
    device_id: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = owner
        .call(app_contract.id(), "add_device")
        .args_json(json!({
            "device_id": device_id
        }))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Adding device should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(result)
}

/// Remove device ID from App contract
pub async fn remove_app_device(
    owner: &Account,
    app_contract: &Contract,
    device_id: &str,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = owner
        .call(app_contract.id(), "remove_device")
        .args_json(json!({
            "device_id": device_id
        }))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Removing device should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(result)
}

/// Set allow any device flag in App contract
pub async fn set_allow_any_device(
    owner: &Account,
    app_contract: &Contract,
    allow_any_device: bool,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = owner
        .call(app_contract.id(), "set_allow_any_device")
        .args_json(json!({
            "allow_any_device": allow_any_device
        }))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Setting allow_any_device should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(result)
}

/// Check if app is allowed to boot
pub async fn is_app_allowed(
    app_contract: &Contract,
    boot_info: &serde_json::Value,
) -> Result<(bool, String), Box<dyn std::error::Error>> {
    let result = app_contract
        .call("is_app_allowed")
        .args_json(json!({
            "boot_info": boot_info
        }))
        .view()
        .await?;

    let response: (bool, String) = serde_json::from_slice(&result.result)?;
    Ok(response)
}

/// Disable upgrades in App contract
pub async fn disable_app_upgrades(
    owner: &Account,
    app_contract: &Contract,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = owner
        .call(app_contract.id(), "disable_upgrades")
        .args_json(json!({}))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Disabling upgrades should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    Ok(result)
}

// ============================================================================
// Key Derivation Helper Functions
// ============================================================================

/// Derive key from CKD response
/// This is a simplified version for testing - in production, you would need
/// the ephemeral private key and MPC public key for full decryption and verification
pub fn derive_key_from_ckd_response(
    big_y_str: &str,
    big_c_str: &str,
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    use hkdf::Hkdf;
    use sha2::Sha256;

    // Parse hex strings to bytes
    // Remove "bls12381g1:" prefix if present
    let big_y_hex = big_y_str.strip_prefix("bls12381g1:").unwrap_or(big_y_str);
    let big_c_hex = big_c_str.strip_prefix("bls12381g1:").unwrap_or(big_c_str);

    let big_y_bytes = hex::decode(big_y_hex)?;
    let big_c_bytes = hex::decode(big_c_hex)?;

    // For testing purposes, we'll derive a key from the concatenated response
    // In production, you would:
    // 1. Decrypt: secret = big_c - big_y * ephemeral_private_key
    // 2. Verify the secret using MPC public key and app_id
    // 3. Derive strong key using HKDF: HKDF(secret, info="")

    // Simplified derivation for testing: use HKDF on concatenated big_y and big_c
    let ikm: Vec<u8> = [big_y_bytes, big_c_bytes].concat();
    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let mut okm = [0u8; 32];
    hk.expand(b"", &mut okm)
        .map_err(|e| -> Box<dyn std::error::Error> {
            format!("HKDF expansion failed: {e}").into()
        })?;

    Ok(okm)
}
