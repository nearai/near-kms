use near_sdk::{BorshStorageKey, near};

#[near]
#[derive(BorshStorageKey)]
pub enum Prefix {
    AllowedComposeHashes,
    AllowedDeviceIds,
}
