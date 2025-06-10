lint:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -q 
  RUSTDOCFLAGS='-D warnings' cargo doc --workspace -q --no-deps

build-all:
  cargo build --release --all-features

test-all:
  cargo test --release --all-features

check-pr: lint test-all

build-contracts:
  (cd packages/contracts && cargo build --release --target wasm32-unknown-unknown)

check-contracts: build-contracts
  (cd packages/contracts && cargo stylus check --wasm-file ../../target/wasm32-unknown-unknown/release/contracts.wasm)