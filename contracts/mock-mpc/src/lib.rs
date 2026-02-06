use near_sdk::{
    BorshStorageKey, PanicOnDefault, PromiseOrValue, assert_one_yocto, log, near, store::LookupMap,
};

// Import types from the KMS contract's ext module
// These match the types defined in contracts/kms/src/ext/mpc.rs
#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Bls12381G1PublicKey(pub String);

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct DomainId(pub u64);

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct CKDRequestArgs {
    pub derivation_path: String,
    pub app_public_key: Bls12381G1PublicKey,
    pub domain_id: DomainId,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct CKDResponse {
    pub big_y: Bls12381G1PublicKey,
    pub big_c: Bls12381G1PublicKey,
}

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct Contract {
    // Store pending responses for testing purposes
    // Maps request hash to response
    pending_responses: LookupMap<String, CKDResponse>,
}

#[near]
#[derive(BorshStorageKey)]
pub enum Prefix {
    PendingResponses,
}

#[near]
impl Contract {
    #[init]
    #[private]
    #[allow(clippy::use_self)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending_responses: LookupMap::new(Prefix::PendingResponses),
        }
    }

    /// Mock implementation of request_app_private_key
    /// In a real MPC contract, this would perform confidential key derivation
    /// For the mock, we return dummy BLS12-381 G1 public keys directly
    #[payable]
    pub fn request_app_private_key(
        &mut self,
        request: CKDRequestArgs,
    ) -> PromiseOrValue<CKDResponse> {
        assert_one_yocto();

        log!(
            "Mock MPC: Requesting key derivation for path: {}, domain: {}, app_key: {}",
            request.derivation_path,
            request.domain_id.0,
            request.app_public_key.0
        );

        // For a mock, we return dummy BLS12-381 G1 public keys
        // In hex format (compressed point format for BLS12-381 G1 is typically 48 bytes = 96 hex chars)
        // Using a simple mock value for testing
        let mock_big_y = Bls12381G1PublicKey(
            "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456".to_string()
        );
        let mock_big_c = Bls12381G1PublicKey(
            "f1e2d3c4b5a6789012345678901234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456".to_string()
        );

        let response = CKDResponse {
            big_y: mock_big_y,
            big_c: mock_big_c,
        };

        // Store the response for potential retrieval
        let request_key = format!("{}_{}", request.derivation_path, request.domain_id.0);
        self.pending_responses
            .insert(request_key.clone(), response.clone());

        log!("Mock MPC: Stored response for request: {}", request_key);

        // Return the value directly (no callback needed)
        // This allows non-contract callers to receive the response synchronously
        PromiseOrValue::Value(response)
    }

    /// View method to get a mock CKD response (for testing without promises)
    pub fn get_mock_ckd_response(&self, request: CKDRequestArgs) -> CKDResponse {
        let request_key = format!("{}_{}", request.derivation_path, request.domain_id.0);

        if let Some(response) = self.pending_responses.get(&request_key) {
            return response.clone();
        }

        // Generate a new mock response if not found
        log!(
            "Mock MPC: Generating new mock CKD response for path: {}, domain: {}",
            request.derivation_path,
            request.domain_id.0
        );

        let mock_big_y = Bls12381G1PublicKey(
            "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456".to_string()
        );
        let mock_big_c = Bls12381G1PublicKey(
            "f1e2d3c4b5a6789012345678901234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456".to_string()
        );

        CKDResponse {
            big_y: mock_big_y,
            big_c: mock_big_c,
        }
    }
}
