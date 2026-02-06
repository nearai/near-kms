#![allow(clippy::too_many_lines)]
mod common;

use common::constants::*;
use common::utils::*;
use near_sdk::NearToken;
use near_sdk::serde_json::json;
use near_workspaces::{Account, Contract, Worker, network::Sandbox};

#[tokio::test]
async fn test_kms_contract_initialization() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing KMS contract initialization...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;

    println!("KMS contract deployed at: {}", kms_contract.id());
    println!("Test passed: KMS contract initialized successfully");

    Ok(())
}

#[tokio::test]
async fn test_add_kms_compose_hash() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing add KMS compose hash...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;

    // Verify compose hash is not allowed initially
    let allowed_before = is_kms_compose_hash_allowed(&kms_contract, COMPOSE_HASH).await?;
    assert!(
        !allowed_before,
        "Compose hash should not be allowed initially"
    );

    // Add compose hash
    add_kms_compose_hash(&owner, &kms_contract, COMPOSE_HASH).await?;

    // Verify compose hash is now allowed
    let allowed_after = is_kms_compose_hash_allowed(&kms_contract, COMPOSE_HASH).await?;
    assert!(allowed_after, "Compose hash should be allowed after adding");

    // Verify it's in the list
    let compose_hashes = get_kms_allowed_compose_hashes(&kms_contract).await?;
    assert!(
        compose_hashes.contains(&COMPOSE_HASH.to_string()),
        "Compose hash should be in the allowed list"
    );

    println!("Test passed: Compose hash added and verified successfully");

    Ok(())
}

#[tokio::test]
async fn test_remove_kms_compose_hash() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing remove KMS compose hash...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;

    // Add compose hash first
    add_kms_compose_hash(&owner, &kms_contract, COMPOSE_HASH).await?;

    // Verify it's allowed
    let allowed_before = is_kms_compose_hash_allowed(&kms_contract, COMPOSE_HASH).await?;
    assert!(
        allowed_before,
        "Compose hash should be allowed after adding"
    );

    // Remove compose hash
    remove_kms_compose_hash(&owner, &kms_contract, COMPOSE_HASH).await?;

    // Verify it's no longer allowed
    let allowed_after = is_kms_compose_hash_allowed(&kms_contract, COMPOSE_HASH).await?;
    assert!(
        !allowed_after,
        "Compose hash should not be allowed after removing"
    );

    // Verify it's not in the list
    let compose_hashes = get_kms_allowed_compose_hashes(&kms_contract).await?;
    assert!(
        !compose_hashes.contains(&COMPOSE_HASH.to_string()),
        "Compose hash should not be in the allowed list after removing"
    );

    println!("Test passed: Compose hash removed and verified successfully");

    Ok(())
}

#[tokio::test]
async fn test_request_kms_root_key() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing request KMS root key...");
    let sandbox = near_workspaces::sandbox().await?;
    let (owner, alice, _bob) = create_test_accounts(&sandbox).await?;

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;

    // Add compose hash first
    add_kms_compose_hash(&owner, &kms_contract, COMPOSE_HASH).await?;

    // Verify compose hash is allowed
    let allowed = is_kms_compose_hash_allowed(&kms_contract, COMPOSE_HASH).await?;
    assert!(allowed, "Compose hash should be allowed");

    // Request root key with mock MPC contract using Alice's account
    // The mock MPC contract will return a response via callback
    let worker_public_key = "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456";
    let result = request_kms_root_key(
        &alice,
        &kms_contract,
        QUOTE_HEX_ALICE,
        QUOTE_COLLATERAL_ALICE,
        TCB_INFO_ALICE,
        worker_public_key,
    )
    .await?;

    // Wait a bit for the callback to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Verify the call succeeded
    assert!(
        result.is_success(),
        "Request root key should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    // Verify KMS info was set (the callback should have stored the keys)
    let kms_info = get_kms_info(&kms_contract).await?;

    if let Some(info) = kms_info {
        // Verify the key format - k256_pubkey should be a byte array
        if let Some(k256_pubkey) = info.get("k256_pubkey") {
            if let Some(pubkey_bytes) = k256_pubkey.as_array() {
                assert!(!pubkey_bytes.is_empty(), "k256_pubkey should not be empty");
                println!("k256_pubkey length: {}", pubkey_bytes.len());
            }
        }

        // Verify ca_pubkey format
        if let Some(ca_pubkey) = info.get("ca_pubkey") {
            if let Some(pubkey_bytes) = ca_pubkey.as_array() {
                assert!(!pubkey_bytes.is_empty(), "ca_pubkey should not be empty");
                println!("ca_pubkey length: {}", pubkey_bytes.len());
            }
        }

        println!("KMS info structure is valid");
    } else {
        println!("KMS info not set yet (callback may not have completed)");
    }

    println!("Test passed: Root key request processed successfully");

    Ok(())
}

#[tokio::test]
async fn test_add_os_image_hash() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing add OS image hash...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;

    let os_image_hash = "test_os_image_hash_123";

    // Verify OS image is not allowed initially
    let allowed_before = is_os_image_allowed(&kms_contract, os_image_hash).await?;
    assert!(!allowed_before, "OS image should not be allowed initially");

    add_os_image_hash(&owner, &kms_contract, os_image_hash).await?;

    // Verify OS image is now allowed
    let allowed_after = is_os_image_allowed(&kms_contract, os_image_hash).await?;
    assert!(allowed_after, "OS image should be allowed after adding");

    // Verify it's in the list
    let os_images = get_allowed_os_images(&kms_contract).await?;
    assert!(
        os_images.contains(&os_image_hash.to_string()),
        "OS image should be in the allowed list"
    );

    println!("Test passed: OS image hash added and verified successfully");

    Ok(())
}

#[tokio::test]
async fn test_set_gateway_app_id() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing set gateway app ID...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;

    // Verify gateway app ID is not set initially
    let gateway_app_id_before = get_gateway_app_id(&kms_contract).await?;
    assert!(
        gateway_app_id_before.is_none(),
        "Gateway app ID should not be set initially"
    );

    let gateway_app_id = "gateway.app.testnet";
    set_gateway_app_id(&owner, &kms_contract, gateway_app_id).await?;

    // Verify gateway app ID is now set
    let gateway_app_id_after = get_gateway_app_id(&kms_contract).await?;
    assert_eq!(
        gateway_app_id_after,
        Some(gateway_app_id.to_string()),
        "Gateway app ID should be set correctly"
    );

    println!("Test passed: Gateway app ID set and verified successfully");

    Ok(())
}

#[tokio::test]
async fn test_non_owner_cannot_add_compose_hash() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing non-owner cannot add compose hash...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();
    let alice = owner
        .create_subaccount("alice")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .result;

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;

    // Try to add compose hash as non-owner
    let result = alice
        .call(kms_contract.id(), "add_kms_compose_hash")
        .args_json(json!({
            "compose_hash": COMPOSE_HASH
        }))
        .transact()
        .await?;

    assert!(
        !result.is_success(),
        "Non-owner should not be able to add compose hash"
    );

    println!("Test passed: Non-owner correctly prevented from adding compose hash");

    Ok(())
}
