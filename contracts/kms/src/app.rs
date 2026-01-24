use near_plugins::{AccessControllable, Pausable, access_control_any, pause};
use near_sdk::{AccountId, Gas, NearToken, Promise, PromiseError, env, near, require, serde_json};

use crate::events::Event;
use crate::{Contract, ContractExt, Role};

const GAS_DEPLOY_APP: Gas = Gas::from_tgas(50);
const GAS_DEPLOY_APP_CALLBACK: Gas = Gas::from_tgas(10);

#[near]
impl Contract {
    /// Deploy and register an app contract in a single transaction
    /// This is a factory method that creates a new app contract account, deploys the contract,
    /// initializes it, and registers it with the KMS
    #[payable]
    #[access_control_any(roles(Role::Owner))]
    #[pause]
    pub fn register_app(
        &mut self,
        app_id: &AccountId,
        owner_id: AccountId,
        disable_upgrades: bool,
        allow_any_device: bool,
        initial_device_id: Option<String>,
        initial_compose_hash: Option<String>,
    ) -> Promise {
        require!(!app_id.as_str().is_empty(), "Invalid app account ID");
        require!(!owner_id.as_str().is_empty(), "Invalid owner address");

        let app_account_id = Self::get_app_account_id(app_id);

        // Serialize initialization arguments
        let init_args = serde_json::to_vec(&serde_json::json!({
            "owner_id": owner_id,
            "disable_upgrades": disable_upgrades,
            "allow_any_device": allow_any_device,
            "initial_device_id": initial_device_id,
            "initial_compose_hash": initial_compose_hash,
            "kms_contract_id": env::current_account_id(),
        }))
        .unwrap_or_else(|_| env::panic_str("Failed to serialize init args"));

        // Deploy the app contract
        Promise::new(app_account_id.clone())
            .create_account()
            .transfer(env::attached_deposit())
            .deploy_contract(include_bytes!("../../app/res/near_dstack_app.wasm").to_vec())
            .function_call(
                "new".to_string(),
                init_args,
                NearToken::from_yoctonear(0),
                GAS_DEPLOY_APP,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_DEPLOY_APP_CALLBACK)
                    .on_app_contract_deployed(app_account_id),
            )
    }

    #[private]
    pub fn on_app_contract_deployed(
        &mut self,
        app_id: AccountId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_ok() {
            self.registered_apps.insert(app_id.clone());
            Event::AppDeployed { app_id: &app_id }.emit();
        } else {
            env::panic_str("Failed to deploy app contract");
        }
    }
}

impl Contract {
    pub(crate) fn get_app_account_id(app_id: &AccountId) -> AccountId {
        format!("{}.{}", app_id.as_str(), env::current_account_id())
            .parse()
            .unwrap()
    }
}
