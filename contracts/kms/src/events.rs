use near_sdk::serde::Serialize;
use near_sdk::serde_json::json;
use near_sdk::{AccountId, log};

pub const EVENT_STANDARD: &str = "near-kms";
pub const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize)]
#[serde(
    crate = "near_sdk::serde",
    rename_all = "snake_case",
    tag = "event",
    content = "data"
)]
#[must_use = "Don't forget to `.emit()` this event"]
pub enum Event<'a> {
    AppDeployed { app_id: &'a AccountId },
    ComposeHashApproved { compose_hash: &'a String },
    ComposeHashRemoved { compose_hash: &'a String },
    GatewayAppIdSet { app_id: &'a String },
    KmsInfoSet { k256_pubkey: &'a [u8] },
    OsImageHashAdded { os_image_hash: &'a String },
    OsImageHashRemoved { os_image_hash: &'a String },
    KmsAggregatedMrAdded { mr_aggregated: &'a String },
    KmsAggregatedMrRemoved { mr_aggregated: &'a String },
    KmsDeviceAdded { device_id: &'a String },
    KmsDeviceRemoved { device_id: &'a String },
}

impl Event<'_> {
    pub fn emit(&self) {
        let json = json!(self);
        let event_json = json!({
            "standard": EVENT_STANDARD,
            "version": EVENT_STANDARD_VERSION,
            "event": json["event"],
            "data": [json["data"]]
        })
        .to_string();
        log!("EVENT_JSON:{}", event_json);
    }
}
