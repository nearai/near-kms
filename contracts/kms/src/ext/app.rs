use crate::AppBootInfo;
use near_sdk::ext_contract;

#[allow(dead_code)]
#[ext_contract(ext_app)]
trait ExtApp {
    fn is_app_allowed(&self, boot_info: AppBootInfo) -> (bool, String);
}
