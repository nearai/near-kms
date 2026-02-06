extern crate alloc;

use dstack_sdk_types::dstack::TcbInfo;
use hex::decode;
use near_plugins::{
    AccessControlRole, AccessControllable, Pausable, Upgradable, access_control,
    access_control_any, pause,
};
use near_sdk::{
    AccountId, Gas, NearToken, PanicOnDefault, PromiseOrValue, assert_one_yocto,
    borsh::BorshDeserialize,
    env::{self, block_timestamp_ms},
    log, near, require,
    store::IterableSet,
};
use std::str::FromStr;

use crate::attestation::{
    attestation::{Attestation, DstackAttestation},
    collateral::Collateral,
    hash::{DockerComposeHash, DockerImageHash},
    quote::QuoteBytes,
    report_data::ReportData,
};
use crate::events::Event;
use crate::ext::{Bls12381G1PublicKey, CKDRequestArgs, CKDResponse, DomainId, ext_mpc};
use crate::types::{Prefix, TimestampMs};

mod app;
mod attestation;
mod events;
mod ext;
pub mod types;
mod view;

const GAS_MPC_CKD_REQUEST: Gas = Gas::from_tgas(100);
const KMS_ROOT_KEY_DERIVATION_PATH: &str = "kms-root-key";

#[derive(AccessControlRole, Clone, Copy)]
#[near(serializers = [json])]
enum Role {
    Owner,
    Dao,
    PauseManager,
    UnpauseManager,
}

// KMS Info structure
#[derive(Clone, Debug)]
#[near(serializers = [json, borsh])]
pub struct KmsInfo {
    pub k256_pubkey: Vec<u8>,
    pub ca_pubkey: Vec<u8>,
    pub quote: Vec<u8>,
    pub eventlog: Vec<u8>,
}

// App Boot Info structure for KMS contract
#[derive(Clone, Debug)]
#[near(serializers = [json])]
pub struct AppBootInfo {
    pub app_id: AccountId,
    pub compose_hash: String,
    pub instance_id: AccountId,
    pub device_id: String,
    pub mr_aggregated: String,
    pub mr_system: String,
    pub os_image_hash: String,
    pub tcb_status: String,
    pub advisory_ids: Vec<String>,
}

#[derive(PanicOnDefault, Pausable, Upgradable)]
#[access_control(role_type(Role))]
#[upgradable(access_control_roles(
    code_stagers(Role::Owner),
    code_deployers(Role::Owner),
    duration_initializers(Role::Owner),
    duration_update_stagers(Role::Owner),
    duration_update_appliers(Role::Owner),
))]
#[pausable(
    pause_roles(Role::Owner, Role::PauseManager),
    unpause_roles(Role::Owner, Role::UnpauseManager)
)]
#[near(contract_state)]
pub struct Contract {
    // MPC integration
    mpc_contract_id: AccountId,
    // MPC domain ID
    mpc_domain_id: u64,
    // KMS-specific allowed compose hashes
    kms_allowed_compose_hashes: IterableSet<String>,
    // App management
    registered_apps: IterableSet<AccountId>,
    // Gateway app ID
    gateway_app_id: Option<String>,
    // KMS info (root keys and attestation)
    kms_info: Option<KmsInfo>,
    // Allowed OS image hashes
    allowed_os_images: IterableSet<String>,
    // KMS allowed aggregated MRs
    kms_allowed_aggregated_mrs: IterableSet<String>,
    // KMS allowed device IDs
    kms_allowed_device_ids: IterableSet<String>,
}

/// Returns the current block timestamp in milliseconds.
/// When the `test` feature is enabled, returns a fixed timestamp
#[must_use]
pub fn get_block_timestamp_ms() -> TimestampMs {
    #[cfg(feature = "test")]
    {
        // Fixed timestamp for testing purposes: September 10, 2025 00:00:00 UTC
        1_757_462_400_000
    }
    #[cfg(not(feature = "test"))]
    {
        block_timestamp_ms()
    }
}

#[near]
impl Contract {
    #[init]
    #[private]
    #[must_use]
    #[allow(clippy::use_self)]
    pub fn new(owner_id: AccountId, mpc_contract_id: AccountId, mpc_domain_id: u64) -> Self {
        let mut contract = Self {
            mpc_contract_id,
            mpc_domain_id,
            kms_allowed_compose_hashes: IterableSet::new(Prefix::KmsAllowedComposeHashes),
            registered_apps: IterableSet::new(Prefix::RegisteredApps),
            gateway_app_id: None,
            kms_info: None,
            allowed_os_images: IterableSet::new(Prefix::AllowedOsImages),
            kms_allowed_aggregated_mrs: IterableSet::new(Prefix::KmsAllowedAggregatedMrs),
            kms_allowed_device_ids: IterableSet::new(Prefix::KmsAllowedDeviceIds),
        };

        let mut acl = contract.acl_get_or_init();

        acl.add_super_admin_unchecked(&owner_id);
        acl.grant_role_unchecked(Role::Owner, &owner_id);

        contract
    }

    /// Request KMS root key from NEAR MPC network using CKD
    /// This function verifies the KMS app has an allowed compose hash and requests the key from MPC
    #[payable]
    #[pause]
    pub fn request_kms_root_key(
        &mut self,
        quote_hex: String,
        collateral: String,
        tcb_info: String,
        worker_public_key: Bls12381G1PublicKey,
    ) -> PromiseOrValue<CKDResponse> {
        assert_one_yocto();

        // Parse the attestation components
        let quote_bytes = QuoteBytes::from(
            decode(&quote_hex).unwrap_or_else(|_| env::panic_str("Invalid quote hex")),
        );
        let collateral_data = Collateral::from_str(&collateral)
            .unwrap_or_else(|_| env::panic_str("Invalid collateral format"));
        let tcb_info_data: TcbInfo = near_sdk::serde_json::from_str(&tcb_info)
            .unwrap_or_else(|_| env::panic_str("Invalid TCB info format"));

        // Create the attestation
        let attestation = Attestation::Dstack(DstackAttestation::new(
            quote_bytes,
            collateral_data,
            tcb_info_data,
        ));

        // Get the signer's public key
        let public_key = env::signer_account_pk();
        // Create expected report data from the public key
        let expected_report_data = ReportData::new(public_key);

        // Get current timestamp in seconds
        let timestamp_s = get_block_timestamp_ms() / 1_000;

        // Get allowed docker compose hashes for KMS
        let allowed_docker_image_hashes: Vec<DockerImageHash> = vec![];
        let allowed_docker_compose_hashes: Vec<DockerComposeHash> = self
            .kms_allowed_compose_hashes
            .iter()
            .map(|hash| {
                DockerComposeHash::try_from_hex(hash)
                    .unwrap_or_else(|_| env::panic_str("Invalid compose hash"))
            })
            .collect();

        // Verify the attestation
        require!(
            attestation.verify(
                expected_report_data,
                timestamp_s,
                &allowed_docker_image_hashes,
                &allowed_docker_compose_hashes,
            ),
            "Attestation verification failed"
        );

        log!("KMS attestation verified, requesting key from MPC");

        // Request key from MPC contract
        let request = CKDRequestArgs {
            derivation_path: KMS_ROOT_KEY_DERIVATION_PATH.to_string(),
            app_public_key: worker_public_key,
            domain_id: DomainId(self.mpc_domain_id),
        };

        ext_mpc::ext(self.mpc_contract_id.clone())
            .with_static_gas(GAS_MPC_CKD_REQUEST)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .request_app_private_key(request)
            .into()
    }

    /// Add a compose hash to the allowed list for KMS
    #[access_control_any(roles(Role::Owner))]
    pub fn add_kms_compose_hash(&mut self, compose_hash: String) {
        self.kms_allowed_compose_hashes.insert(compose_hash.clone());
        Event::ComposeHashApproved {
            compose_hash: &compose_hash,
        }
        .emit();
    }

    /// Remove a compose hash from the allowed list for KMS
    #[access_control_any(roles(Role::Owner))]
    pub fn remove_kms_compose_hash(&mut self, compose_hash: String) {
        self.kms_allowed_compose_hashes.remove(&compose_hash);
        Event::ComposeHashRemoved {
            compose_hash: &compose_hash,
        }
        .emit();
    }

    /// Set the gateway app ID
    #[access_control_any(roles(Role::Owner))]
    pub fn set_gateway_app_id(&mut self, app_id: String) {
        self.gateway_app_id = Some(app_id.clone());
        Event::GatewayAppIdSet { app_id: &app_id }.emit();
    }

    /// Set KMS info
    #[access_control_any(roles(Role::Owner))]
    pub fn set_kms_info(&mut self, info: KmsInfo) {
        self.kms_info = Some(info.clone());
        Event::KmsInfoSet {
            k256_pubkey: &info.k256_pubkey,
        }
        .emit();
    }

    /// Set KMS quote
    #[access_control_any(roles(Role::Owner))]
    pub fn set_kms_quote(&mut self, quote: Vec<u8>) {
        if let Some(ref mut info) = self.kms_info {
            info.quote = quote;
        } else {
            self.kms_info = Some(KmsInfo {
                k256_pubkey: vec![],
                ca_pubkey: vec![],
                quote,
                eventlog: vec![],
            });
        }
    }

    /// Set KMS eventlog
    #[access_control_any(roles(Role::Owner))]
    pub fn set_kms_eventlog(&mut self, eventlog: Vec<u8>) {
        if let Some(ref mut info) = self.kms_info {
            info.eventlog = eventlog;
        } else {
            self.kms_info = Some(KmsInfo {
                k256_pubkey: vec![],
                ca_pubkey: vec![],
                quote: vec![],
                eventlog,
            });
        }
    }

    /// Add an OS image hash to the allowed list
    #[access_control_any(roles(Role::Owner))]
    pub fn add_os_image_hash(&mut self, os_image_hash: String) {
        require!(!os_image_hash.is_empty(), "OS image hash cannot be empty");
        self.allowed_os_images.insert(os_image_hash.clone());
        Event::OsImageHashAdded {
            os_image_hash: &os_image_hash,
        }
        .emit();
    }

    /// Remove an OS image hash from the allowed list
    #[access_control_any(roles(Role::Owner))]
    pub fn remove_os_image_hash(&mut self, os_image_hash: String) {
        self.allowed_os_images.remove(&os_image_hash);
        Event::OsImageHashRemoved {
            os_image_hash: &os_image_hash,
        }
        .emit();
    }

    /// Add a KMS aggregated MR to the allowed list
    #[access_control_any(roles(Role::Owner))]
    pub fn add_kms_aggregated_mr(&mut self, mr_aggregated: String) {
        require!(!mr_aggregated.is_empty(), "Aggregated MR cannot be empty");
        self.kms_allowed_aggregated_mrs
            .insert(mr_aggregated.clone());
        Event::KmsAggregatedMrAdded {
            mr_aggregated: &mr_aggregated,
        }
        .emit();
    }

    /// Remove a KMS aggregated MR from the allowed list
    #[access_control_any(roles(Role::Owner))]
    pub fn remove_kms_aggregated_mr(&mut self, mr_aggregated: String) {
        self.kms_allowed_aggregated_mrs.remove(&mr_aggregated);
        Event::KmsAggregatedMrRemoved {
            mr_aggregated: &mr_aggregated,
        }
        .emit();
    }

    /// Add a KMS device ID to the allowed list
    #[access_control_any(roles(Role::Owner))]
    pub fn add_kms_device(&mut self, device_id: String) {
        require!(!device_id.is_empty(), "Device ID cannot be empty");
        self.kms_allowed_device_ids.insert(device_id.clone());
        Event::KmsDeviceAdded {
            device_id: &device_id,
        }
        .emit();
    }

    /// Remove a KMS device ID from the allowed list
    #[access_control_any(roles(Role::Owner))]
    pub fn remove_kms_device(&mut self, device_id: String) {
        self.kms_allowed_device_ids.remove(&device_id);
        Event::KmsDeviceRemoved {
            device_id: &device_id,
        }
        .emit();
    }
}
