pub const COMPOSE_HASH: &str = "f68cf65a75ad980289144ef3c096a007fc4583ea6d1f90589757f229dbc6cdab";

// Worker public key for testing (BLS12-381 G1)
// Hex format: a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456
// Converted to bls12381g1 format using hex_to_bls12381g1_key helper
pub const WORKER_PUBLIC_KEY_HEX: &str = "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456";

// Worker Info: Alice
pub const SECRET_KEY_ALICE: &str = "ed25519:3uHrtHQ6422oAj7WhvDgf9KdewGZLvCLbY6AyDdfkctRkUgyai1yMFn7TGnY2a4zQ8o2a1xQpaPPuaTcjRNaxTqP";
pub const CHECKSUM_ALICE: &str = "98836e3169efb65d42411b7fc1aa1ba4e0e3713aa97c53cbb5d10444d27788c7";
pub const QUOTE_COLLATERAL_ALICE: &str = include_str!("../samples/alice/quote_collateral.json");
pub const TCB_INFO_ALICE: &str = include_str!("../samples/alice/tcb_info.json");
pub const QUOTE_HEX_ALICE: &str = include_str!("../samples/alice/quote_hex.txt");

// Worker Info: Bob
pub const SECRET_KEY_BOB: &str = "ed25519:ktA4Vi3e8nqoNjxVnuJ1yBPbNvtbordZmpwQ9wpq6e5XzrWHCx8VYZeEKWEhcc52Vf44WxBva6PsfT8sLbgFGDz";
pub const CHECKSUM_BOB: &str = "e544b7fa1bdc55c30589626bff3323295bb6f7be0470843e2b1ac61e01ddad76";
pub const QUOTE_COLLATERAL_BOB: &str = include_str!("../samples/bob/quote_collateral.json");
pub const TCB_INFO_BOB: &str = include_str!("../samples/bob/tcb_info.json");
pub const QUOTE_HEX_BOB: &str = include_str!("../samples/bob/quote_hex.txt");
