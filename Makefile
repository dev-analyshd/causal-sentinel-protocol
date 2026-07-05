.PHONY: all build test test-contracts test-circuits test-integration deploy clean

all: build

build:
	cargo build --release --target wasm32-unknown-unknown
	@echo "✅ Contracts built"
	cd circuits/behavioral_integrity && nargo compile
	cd circuits/causal_identity && nargo compile
	cd circuits/sentinel_compliance && nargo compile
	@echo "✅ Circuits compiled"

test: test-contracts test-circuits test-integration

test-contracts:
	cargo test --workspace --lib

test-circuits:
	cd circuits/behavioral_integrity && nargo test
	cd circuits/causal_identity && nargo test
	cd circuits/sentinel_compliance && nargo test

test-integration:
	cd tests && cargo test --release

deploy:
	bash scripts/deploy_testnet.sh

clean:
	cargo clean
	find . -name "target" -type d -exec rm -rf {} + 2>/dev/null || true
	find . -name "node_modules" -type d -exec rm -rf {} + 2>/dev/null || true
