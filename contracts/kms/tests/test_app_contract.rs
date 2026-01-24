#![allow(clippy::too_many_lines)]
mod common;

use common::constants::*;
use common::utils::*;
use near_sdk::NearToken;
use near_sdk::serde_json::json;
use near_workspaces::{Account, Worker, network::Sandbox};

#[tokio::test]
async fn test_app_contract_initialization() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing App contract initialization...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;
    let app_contract =
        deploy_app_contract(&sandbox, &owner, &kms_contract, false, false, None, None).await?;

    println!("App contract deployed at: {}", app_contract.id());
    println!("Test passed: App contract initialized successfully");

    Ok(())
}

#[tokio::test]
async fn test_add_compose_hash() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing add compose hash...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;
    let app_contract =
        deploy_app_contract(&sandbox, &owner, &kms_contract, false, false, None, None).await?;

    // Add compose hash
    add_app_compose_hash(&owner, &app_contract, COMPOSE_HASH).await?;

    println!("Test passed: Compose hash added successfully");

    Ok(())
}

#[tokio::test]
async fn test_remove_compose_hash() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing remove compose hash...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;
    let app_contract =
        deploy_app_contract(&sandbox, &owner, &kms_contract, false, false, None, None).await?;

    // Add compose hash first
    add_app_compose_hash(&owner, &app_contract, COMPOSE_HASH).await?;

    // Remove compose hash
    remove_app_compose_hash(&owner, &app_contract, COMPOSE_HASH).await?;

    println!("Test passed: Compose hash removed successfully");

    Ok(())
}

#[tokio::test]
async fn test_add_device() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing add device...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;
    let app_contract =
        deploy_app_contract(&sandbox, &owner, &kms_contract, false, false, None, None).await?;

    let device_id = "device_123";
    add_app_device(&owner, &app_contract, device_id).await?;

    println!("Test passed: Device added successfully");

    Ok(())
}

#[tokio::test]
async fn test_remove_device() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing remove device...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;
    let app_contract =
        deploy_app_contract(&sandbox, &owner, &kms_contract, false, false, None, None).await?;

    let device_id = "device_123";

    // Add device first
    add_app_device(&owner, &app_contract, device_id).await?;

    // Remove device
    remove_app_device(&owner, &app_contract, device_id).await?;

    println!("Test passed: Device removed successfully");

    Ok(())
}

#[tokio::test]
async fn test_set_allow_any_device() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing set allow any device...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;
    let app_contract =
        deploy_app_contract(&sandbox, &owner, &kms_contract, false, false, None, None).await?;

    set_allow_any_device(&owner, &app_contract, true).await?;

    println!("Test passed: Allow any device set successfully");

    Ok(())
}

#[tokio::test]
async fn test_is_app_allowed() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing is_app_allowed...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;
    let app_contract =
        deploy_app_contract(&sandbox, &owner, &kms_contract, false, false, None, None).await?;

    // Add compose hash first
    add_app_compose_hash(&owner, &app_contract, COMPOSE_HASH).await?;

    // Add device ID to allowed list
    let device_id = "device_123";
    add_app_device(&owner, &app_contract, device_id).await?;

    // Check if app is allowed
    let boot_info = json!({
        "app_id": app_contract.id(),
        "compose_hash": COMPOSE_HASH,
        "instance_id": "instance.testnet",
        "device_id": device_id,
        "mr_aggregated": "mr_aggregated_123",
        "mr_system": "mr_system_123",
        "os_image_hash": "os_image_hash_123",
        "tcb_status": "UpToDate",
        "advisory_ids": []
    });

    let (allowed, message) = is_app_allowed(&app_contract, &boot_info).await?;
    assert!(allowed, "App should be allowed: {message}");

    println!("Test passed: is_app_allowed works correctly");

    Ok(())
}

#[tokio::test]
async fn test_is_app_allowed_without_compose_hash() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing is_app_allowed without compose hash...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;
    let app_contract =
        deploy_app_contract(&sandbox, &owner, &kms_contract, false, false, None, None).await?;

    // Don't add compose hash - app should not be allowed
    let boot_info = json!({
        "app_id": app_contract.id(),
        "compose_hash": COMPOSE_HASH,
        "instance_id": "instance.testnet",
        "device_id": "device_123",
        "mr_aggregated": "mr_aggregated_123",
        "mr_system": "mr_system_123",
        "os_image_hash": "os_image_hash_123",
        "tcb_status": "UpToDate",
        "advisory_ids": []
    });

    let (allowed, _message) = is_app_allowed(&app_contract, &boot_info).await?;
    assert!(!allowed, "App should not be allowed without compose hash");

    println!("Test passed: is_app_allowed correctly rejects app without compose hash");

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
    let app_contract =
        deploy_app_contract(&sandbox, &owner, &kms_contract, false, false, None, None).await?;

    // Try to add compose hash as non-owner
    let result = alice
        .call(app_contract.id(), "add_compose_hash")
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

#[tokio::test]
async fn test_disable_upgrades() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing disable upgrades...");
    let sandbox = near_workspaces::sandbox().await?;
    let owner = sandbox.root_account().unwrap();

    let kms_contract = deploy_kms_contract(&sandbox, &owner).await?;
    let app_contract =
        deploy_app_contract(&sandbox, &owner, &kms_contract, false, false, None, None).await?;

    disable_app_upgrades(&owner, &app_contract).await?;

    println!("Test passed: Upgrades disabled successfully");

    Ok(())
}
