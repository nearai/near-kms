all: lint app-contract kms-contract

lint:
	@cargo fmt --all
	@cargo clippy --fix --allow-dirty --allow-staged --workspace -- -D warnings

kms-contract:
	$(call compile-release,kms)
	@mkdir -p contracts/kms/res
	@cp target/near/near_dstack_kms/near_dstack_kms.wasm ./contracts/kms/res/near_dstack_kms.wasm

app-contract:
	$(call compile-release,app)
	@mkdir -p contracts/app/res
	@cp target/near/near_dstack_app/near_dstack_app.wasm ./contracts/app/res/near_dstack_app.wasm

mock-mpc-contract:
	$(call compile-release,mock-mpc)
	@mkdir -p contracts/mock-mpc/res
	@cp target/near/mock_mpc/mock_mpc.wasm ./contracts/mock-mpc/res/mock_mpc.wasm

kms-contract-test:
	$(call compile-release,kms,test)
	@mkdir -p contracts/kms/res
	@cp target/near/near_dstack_kms/near_dstack_kms.wasm ./contracts/kms/res/near_dstack_kms.wasm

test: mock-mpc-contract app-contract kms-contract-test
	cargo test --features test -- --nocapture

define compile-release
	@rustup target add wasm32-unknown-unknown
	cargo near build non-reproducible-wasm --manifest-path contracts/$(1)/Cargo.toml $(if $(2),--features $(2))
endef
