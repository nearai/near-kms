use near_plugins::{
    AccessControlRole, AccessControllable, Pausable, Upgradable, access_control, access_control_any,
};
use near_sdk::{
    AccountId, PanicOnDefault, borsh::BorshDeserialize, near, require, store::IterableSet,
};

mod events;
mod types;
mod view;

use crate::events::Event;
use crate::types::Prefix;

#[derive(AccessControlRole, Clone, Copy)]
#[near(serializers = [json])]
enum Role {
    Owner,
}

/// App Boot Information for validation
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
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
#[pausable(pause_roles(Role::Owner), unpause_roles(Role::Owner))]
#[near(contract_state)]
pub struct Contract {
    /// Mapping of allowed compose hashes for this app
    allowed_compose_hashes: IterableSet<String>,
    /// Whether to allow any device to boot this app
    allow_any_device: bool,
    /// Mapping of allowed device IDs for this app
    allowed_device_ids: IterableSet<String>,
    /// Whether upgrades are disabled
    upgrades_disabled: bool,
    /// KMS contract ID for registration
    kms_contract_id: Option<AccountId>,
}

#[near]
impl Contract {
    #[init]
    #[must_use]
    #[allow(clippy::use_self)]
    pub fn new(
        owner_id: AccountId,
        disable_upgrades: bool,
        allow_any_device: bool,
        initial_device_id: Option<String>,
        initial_compose_hash: Option<String>,
        kms_contract_id: Option<AccountId>,
    ) -> Self {
        require!(!owner_id.as_str().is_empty(), "Invalid owner address");

        let mut contract = Self {
            allowed_compose_hashes: IterableSet::new(Prefix::AllowedComposeHashes),
            allow_any_device,
            allowed_device_ids: IterableSet::new(Prefix::AllowedDeviceIds),
            upgrades_disabled: disable_upgrades,
            kms_contract_id,
        };

        let mut acl = contract.acl_get_or_init();
        acl.add_super_admin_unchecked(&owner_id);
        acl.grant_role_unchecked(Role::Owner, &owner_id);

        // Add initial device if provided
        if let Some(device_id) = initial_device_id {
            contract.internal_add_device(device_id);
        }

        // Add initial compose hash if provided
        if let Some(compose_hash) = initial_compose_hash {
            contract.internal_add_compose_hash(compose_hash);
        }

        contract
    }

    /// Add a compose hash to the allowed list
    #[access_control_any(roles(Role::Owner))]
    pub fn add_compose_hash(&mut self, compose_hash: String) {
        self.internal_add_compose_hash(compose_hash);
    }

    /// Remove a compose hash from the allowed list
    #[access_control_any(roles(Role::Owner))]
    pub fn remove_compose_hash(&mut self, compose_hash: String) {
        self.internal_remove_compose_hash(compose_hash);
    }

    /// Set whether any device is allowed to boot this app
    #[access_control_any(roles(Role::Owner))]
    pub fn set_allow_any_device(&mut self, allow_any_device: bool) {
        self.allow_any_device = allow_any_device;
        Event::AllowAnyDeviceSet {
            allow_any: &allow_any_device,
        }
        .emit();
    }

    /// Add a device ID to the allowed list
    #[access_control_any(roles(Role::Owner))]
    pub fn add_device(&mut self, device_id: String) {
        self.internal_add_device(device_id);
    }

    /// Remove a device ID from the allowed list
    #[access_control_any(roles(Role::Owner))]
    pub fn remove_device(&mut self, device_id: String) {
        self.internal_remove_device(device_id);
    }

    /// Permanently disable upgrades
    #[access_control_any(roles(Role::Owner))]
    pub fn disable_upgrades(&mut self) {
        self.upgrades_disabled = true;
        Event::UpgradesDisabled.emit();
    }
}

impl Contract {
    fn internal_add_device(&mut self, device_id: String) {
        require!(!device_id.is_empty(), "Device ID cannot be empty");
        self.allowed_device_ids.insert(device_id.clone());
        Event::DeviceAdded {
            device_id: &device_id,
        }
        .emit();
    }

    fn internal_remove_device(&mut self, device_id: String) {
        self.allowed_device_ids.remove(&device_id);
        Event::DeviceRemoved {
            device_id: &device_id,
        }
        .emit();
    }

    fn internal_add_compose_hash(&mut self, compose_hash: String) {
        require!(!compose_hash.is_empty(), "Compose hash cannot be empty");
        self.allowed_compose_hashes.insert(compose_hash.clone());
        Event::ComposeHashAdded {
            compose_hash: &compose_hash,
        }
        .emit();
    }

    fn internal_remove_compose_hash(&mut self, compose_hash: String) {
        self.allowed_compose_hashes.remove(&compose_hash);
        Event::ComposeHashRemoved {
            compose_hash: &compose_hash,
        }
        .emit();
    }
}
