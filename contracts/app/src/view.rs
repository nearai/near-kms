use crate::ContractExt;
use crate::{AppBootInfo, Contract};
use near_sdk::near;

#[near]
impl Contract {
    /// Check if an app is allowed to boot
    pub fn is_app_allowed(&self, boot_info: AppBootInfo) -> (bool, String) {
        // Check if compose hash is allowed
        if !self
            .allowed_compose_hashes
            .contains(&boot_info.compose_hash)
        {
            return (false, "Compose hash not allowed".to_string());
        }

        // Check if device is allowed (when device restriction is enabled)
        if !self.allow_any_device && !self.allowed_device_ids.contains(&boot_info.device_id) {
            return (false, "Device not allowed".to_string());
        }

        (true, String::new())
    }
}
