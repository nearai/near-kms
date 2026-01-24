use crate::{AppBootInfo, Contract, ContractExt, KmsInfo, near};
use near_sdk::AccountId;

#[near]
impl Contract {
    /// Check if an app is registered
    pub fn is_app_registered(&self, app_id: &AccountId) -> bool {
        self.registered_apps.contains(app_id)
    }

    /// Get registered apps with pagination
    pub fn get_registered_apps(&self, offset: u32, limit: u32) -> Vec<AccountId> {
        self.registered_apps
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect()
    }

    /// Get the gateway app ID
    pub fn get_gateway_app_id(&self) -> Option<String> {
        self.gateway_app_id.clone()
    }

    /// Get KMS info
    pub fn get_kms_info(&self) -> Option<KmsInfo> {
        self.kms_info.clone()
    }

    /// Get all allowed OS image hashes
    pub fn get_allowed_os_images(&self) -> Vec<String> {
        self.allowed_os_images.iter().cloned().collect()
    }

    /// Get all KMS allowed aggregated MRs
    pub fn get_kms_allowed_aggregated_mrs(&self) -> Vec<String> {
        self.kms_allowed_aggregated_mrs.iter().cloned().collect()
    }

    /// Get all KMS allowed device IDs
    pub fn get_kms_allowed_device_ids(&self) -> Vec<String> {
        self.kms_allowed_device_ids.iter().cloned().collect()
    }

    /// Check if KMS is allowed to boot with given boot info
    pub fn is_kms_allowed(&self, boot_info: AppBootInfo) -> (bool, String) {
        // Check if the TCB status is up to date
        if boot_info.tcb_status != "UpToDate" {
            return (false, "TCB status is not up to date".to_string());
        }

        // Check if the OS image is allowed
        if !self.allowed_os_images.contains(&boot_info.os_image_hash) {
            return (false, "OS image is not allowed".to_string());
        }

        // Check if the aggregated MR is allowed
        if !self
            .kms_allowed_aggregated_mrs
            .contains(&boot_info.mr_aggregated)
        {
            return (false, "Aggregated MR not allowed".to_string());
        }

        // Check if the KMS device ID is allowed
        if !self.kms_allowed_device_ids.contains(&boot_info.device_id) {
            return (
                false,
                "KMS is not allowed to boot on this device".to_string(),
            );
        }

        (true, String::new())
    }

    // /// Check if an app is allowed to boot (delegates to app contract)
    // pub fn is_app_allowed(&self, boot_info: AppBootInfo) -> (bool, String) {
    //     // Check if app is registered
    //     if !self.registered_apps.contains(&boot_info.app_id) {
    //         return (false, "App not registered".to_string());
    //     }

    //     // Check if the OS image is allowed
    //     if !self.allowed_os_images.contains(&boot_info.os_image_hash) {
    //         return (false, "OS image is not allowed".to_string());
    //     }

    //     // Verify app contract exists (check if account exists)
    //     // Note: In NEAR, we can't directly check if an account has code in a view call
    //     // The app contract should exist if it's registered, but we'll still try to call it

    //     // Call the app's is_app_allowed function via cross-contract view call
    //     // Since this is a view method, we need to use Promise::new().function_call() for view calls
    //     // However, NEAR doesn't support async view calls in the same way
    //     // For now, we'll return that the app needs to be checked
    //     // In production, this would need to be handled differently or the app contract
    //     // would need to be called separately by the caller

    //     // For compatibility with dstack-kms, we'll check if the app is registered
    //     // and let the app contract handle the rest of the validation
    //     (true, String::new())
    // }

    /// Check if an OS image hash is allowed
    pub fn is_os_image_allowed(&self, os_image_hash: &String) -> bool {
        self.allowed_os_images.contains(os_image_hash)
    }

    /// Check if a KMS aggregated MR is allowed
    pub fn is_kms_aggregated_mr_allowed(&self, mr_aggregated: &String) -> bool {
        self.kms_allowed_aggregated_mrs.contains(mr_aggregated)
    }

    /// Check if a KMS device ID is allowed
    pub fn is_kms_device_allowed(&self, device_id: &String) -> bool {
        self.kms_allowed_device_ids.contains(device_id)
    }

    /// Check if a KMS compose hash is allowed
    pub fn is_kms_compose_hash_allowed(&self, compose_hash: &String) -> bool {
        self.kms_allowed_compose_hashes.contains(compose_hash)
    }

    /// Get all allowed KMS compose hashes
    pub fn get_kms_allowed_compose_hashes(&self) -> Vec<String> {
        self.kms_allowed_compose_hashes.iter().cloned().collect()
    }
}
