use near_sdk::log;
use near_sdk::serde::Serialize;
use near_sdk::serde_json::json;

pub const EVENT_STANDARD: &str = "dstack-app";
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
    ComposeHashAdded { compose_hash: &'a String },
    ComposeHashRemoved { compose_hash: &'a String },
    DeviceAdded { device_id: &'a String },
    DeviceRemoved { device_id: &'a String },
    AllowAnyDeviceSet { allow_any: &'a bool },
    UpgradesDisabled,
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
