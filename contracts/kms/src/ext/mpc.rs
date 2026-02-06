use near_sdk::{PromiseOrValue, ext_contract, near};

// Simplified BLS12-381 G1 public key type (String format for JSON)
#[near(serializers = [json])]
pub struct Bls12381G1PublicKey(pub String);

#[near(serializers = [json])]
pub struct DomainId(pub u64);

#[near(serializers = [json])]
pub struct CKDRequestArgs {
    pub derivation_path: String,
    pub app_public_key: Bls12381G1PublicKey,
    pub domain_id: DomainId,
}

#[near(serializers = [json])]
pub struct CKDResponse {
    pub big_y: Bls12381G1PublicKey,
    pub big_c: Bls12381G1PublicKey,
}

#[allow(dead_code)]
#[ext_contract(ext_mpc)]
trait ExtMPC {
    /// Request a confidential key derivation from the MPC network
    /// Returns a PromiseOrValue that can be either a Promise or direct value
    fn request_app_private_key(&self, request: CKDRequestArgs) -> PromiseOrValue<CKDResponse>;
}
