lint:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -q 
  RUSTDOCFLAGS='-D warnings' cargo doc --workspace -q --no-deps

build-all:
  cargo build --release --all-features

build-contracts:
  cargo build -p contracts --target wasm32-unknown-unknown

test-all:
  cargo test --release --all-features

test-ultrahonk:
  cargo test -p ultrahonk -- --test-threads=1 --nocapture

test-contracts:
  cargo test -p contracts -- --test-threads=1

check-pr: lint test-all

check-contracts: build-contracts
  (cd packages/contracts && cargo stylus check --wasm-file ../../target/wasm32-unknown-unknown/release/contracts.wasm)