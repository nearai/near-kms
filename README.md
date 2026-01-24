# NEAR dstack KMS

The NEAR dstack KMS (Key Management System) is a secure smart contract system for managing cryptographic keys using Trusted Execution Environment (TEE) and Multi-Party Computation (MPC) technologies on the NEAR Protocol. This project provides secure key derivation, attestation verification, and access control mechanisms for TEE-based applications.

The system enables secure key management for dstack applications running in TEE environments in a decentralized approach, ensuring that only verified and approved TEE instances can access and derive cryptographic keys through integration with the NEAR MPC (Multi-Party Computation) network.

## Overview

The system consists of two main smart contracts:

1. **KMS Contract** (`near-dstack-kms`)
   - Manages KMS root key derivation from the NEAR MPC network
   - Verifies TEE attestation (quote, collateral, TCB info)
   - Manages allowed compose hashes, OS image hashes, and device IDs
   - Controls access to key derivation operations
   - Supports app registration and management

2. **App Contract** (`near-dstack-app`)
   - Validates app boot information and attestation
   - Manages allowed compose hashes and device IDs per app
   - Provides app-level access control and validation
   - Integrates with KMS contract for key management

## Prerequisites

- Rust and Cargo (latest stable version)
- [`cargo-near`](https://github.com/near/cargo-near) - NEAR smart contract development toolkit
- NEAR CLI - For interacting with NEAR blockchain
- A NEAR account with sufficient NEAR tokens for contract deployment

## Project Structure

```
near-dstack-kms/
├── contracts/              # Smart contracts
│   ├── kms/                # KMS contract
│   │   ├── src/            # Source code
│   │   │   ├── lib.rs      # Main contract logic
│   │   │   ├── app.rs      # App deployment and management
│   │   │   ├── attestation/# TEE attestation verification
│   │   │   ├── ext/        # External contract interfaces
│   │   │   └── view.rs     # View methods
│   │   ├── tests/          # Integration tests
│   │   └── res/            # Compiled WASM files
│   ├── app/                # App contract
│   │   ├── src/            # Source code
│   │   └── res/            # Compiled WASM files
│   └── mock-mpc/           # Mock MPC contract for testing
├── scripts/                # Deployment and utility scripts
│   ├── testnet/            # Testnet deployment scripts
│   └── mainnet/            # Mainnet deployment scripts
└── makefile               # Build and test commands
```

## Setup and Deployment

### 1. Build the Contracts

Install [`cargo-near`](https://github.com/near/cargo-near) if you haven't already:

```bash
cargo install cargo-near --locked
```

Build all contracts:

```bash
make all
```

This will:
- Run linting (format and clippy)
- Build the KMS contract
- Build the App contract
- Copy WASM files to `contracts/*/res/` directories

To build individual contracts:

```bash
# Build KMS contract only
make kms-contract

# Build App contract only
make app-contract

# Build mock MPC contract (for testing)
make mock-mpc-contract
```

### 2. Test the Contracts

Run all tests:

```bash
make test
```

This will:
- Build all contracts (KMS, App, and mock MPC)
- Run integration tests with the `test` feature enabled

The test suite includes:
- KMS contract initialization and configuration
- Compose hash management (add/remove)
- OS image hash management
- Gateway app ID configuration
- Root key request functionality
- App contract initialization and management
- Device ID management
- Access control verification

### 3. Deploy the Contracts

#### Deploy to Testnet

1. **Deploy KMS Contract:**

```bash
cd scripts/testnet
./deploy-kms.sh
```

2. **Deploy App Contract:**

```bash
cd scripts/testnet
./deploy-app.sh
```

#### Deploy to Mainnet

1. **Deploy KMS Contract:**

```bash
cd scripts/mainnet
./deploy-kms.sh
```

2. **Deploy App Contract:**

```bash
cd scripts/mainnet
./deploy-app.sh
```


## Key Features

### KMS Contract Features

- **Root Key Derivation**: Request KMS root keys from NEAR MPC network using TEE attestation
- **Attestation Verification**: Verify TEE quotes, collateral, and TCB (Trusted Computing Base) information
- **Compose Hash Management**: Manage allowed Docker compose hashes for KMS operations
- **OS Image Management**: Control allowed OS image hashes
- **Device ID Management**: Manage allowed device IDs for KMS access
- **App Registration**: Register and manage app contracts
- **Access Control**: Role-based access control (Owner, DAO, PauseManager, UnpauseManager)
- **Pausable**: Ability to pause contract operations for security
- **Upgradable**: Support for contract upgrades with proper access control

### App Contract Features

- **Boot Validation**: Validate app boot information including compose hash, device ID, and TCB status
- **Compose Hash Management**: Per-app compose hash allowlist
- **Device ID Management**: Per-app device ID allowlist or allow-any-device mode
- **KMS Integration**: Integration with KMS contract for key management
- **Access Control**: Owner-only administrative functions
- **Upgrade Control**: Option to permanently disable upgrades

## Usage Examples

### Request KMS Root Key

Request a KMS root key from the MPC network:

```bash
cd scripts/testnet  # or scripts/mainnet
./request-root-key.sh
```

Or manually:

```bash
near call <kms-contract-id> request_kms_root_key \
  --args '{
    "quote_hex": "<quote-hex-string>",
    "collateral": "<collateral-json-string>",
    "tcb_info": "<tcb-info-json-string>",
    "worker_public_key": {"Bls12381G1PublicKey": "<public-key>"}
  }' \
  --depositYocto 1 \
  --gas 300000000000000 \
  --accountId <caller-account-id>
```

### Add Compose Hash to KMS

```bash
near call <kms-contract-id> add_kms_compose_hash \
  --args '{"compose_hash": "<compose-hash>"}' \
  --accountId <owner-account-id>
```

### Add Compose Hash to App

```bash
near call <app-contract-id> add_compose_hash \
  --args '{"compose_hash": "<compose-hash>"}' \
  --accountId <owner-account-id>
```

### Register App Contract

```bash
near call <kms-contract-id> register_app \
  --args '{
    "app_account_id": "<app-account-id>",
    "owner_id": "<owner-account-id>",
    "disable_upgrades": false,
    "allow_any_device": false,
    "initial_device_id": null,
    "initial_compose_hash": null
  }' \
  --deposit <deposit-amount> \
  --accountId <owner-account-id>
```

## Security Considerations

- **TEE Attestation**: All key derivation requests require valid TEE attestation (quote, collateral, TCB info)
- **Compose Hash Verification**: Only approved compose hashes can be used for key operations
- **Access Control**: Role-based access control ensures only authorized accounts can perform administrative operations
- **Pausable Operations**: Critical operations can be paused for security incidents
- **Upgrade Control**: Contract upgrades are controlled and can be permanently disabled
- **Device ID Validation**: Optional device ID allowlist provides additional security layer

## Development

### Running Tests

```bash
# Run all tests
make test

# Run specific test file
cargo test --test test_kms_contract
cargo test --test test_app_contract

# Run with output
cargo test --features test -- --nocapture
```

### Linting

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace -- -D warnings

# Or use make
make lint
```

### Contract Methods

#### KMS Contract Methods

- `new(owner_id, mpc_contract_id, mpc_domain_id)` - Initialize contract
- `request_kms_root_key(quote_hex, collateral, tcb_info, worker_public_key)` - Request root key from MPC
- `add_kms_compose_hash(compose_hash)` - Add allowed compose hash
- `remove_kms_compose_hash(compose_hash)` - Remove compose hash
- `add_os_image_hash(os_image_hash)` - Add allowed OS image hash
- `remove_os_image_hash(os_image_hash)` - Remove OS image hash
- `set_gateway_app_id(app_id)` - Set gateway app ID
- `register_app(...)` - Deploy and register app contract

#### App Contract Methods

- `new(owner_id, disable_upgrades, allow_any_device, initial_device_id, initial_compose_hash, kms_contract_id)` - Initialize contract
- `is_app_allowed(boot_info)` - Check if app is allowed to boot
- `add_compose_hash(compose_hash)` - Add allowed compose hash
- `remove_compose_hash(compose_hash)` - Remove compose hash
- `add_device(device_id)` - Add allowed device ID
- `remove_device(device_id)` - Remove device ID
- `set_allow_any_device(allow_any_device)` - Set allow-any-device flag
- `disable_upgrades()` - Permanently disable upgrades

## Tools

- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract development toolkit for Rust
- [near CLI](https://near.cli.rs) - Interact with NEAR blockchain from command line
- [NEAR Rust SDK Documentation](https://docs.near.org/sdk/rust/introduction)

## Architecture

### Key Derivation Flow

1. TEE instance generates attestation quote
2. KMS contract verifies quote, collateral, and TCB info
3. KMS contract checks compose hash is in allowlist
4. KMS contract requests key derivation from NEAR MPC network
5. MPC network derives key using BLS12-381 cryptography
6. Key is returned to the TEE instance

### App Validation Flow

1. App provides boot information (compose hash, device ID, TCB status, etc.)
2. App contract validates compose hash is in allowlist
3. App contract validates device ID (if not allow-any-device mode)
4. App contract validates TCB status
5. If all validations pass, app is allowed to boot

## License

This project is licensed under the MIT License - see the LICENSE file for details.
