lint:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -q 
  RUSTDOCFLAGS='-D warnings' cargo doc --workspace -q --no-deps

build-all:
  cargo build --release --all-features

build-ultrahonk:
  cargo build -p ultrahonk --release --features ark-ec/only-arithmetic-backend --target wasm32-unknown-unknown

build-contracts:
  cargo build -p contracts --target wasm32-unknown-unknown --profile release

test-all:
  cargo test --release --all-features

test-ultrahonk:
  #  ark-ec/only-arithmetic-backend panics if we attempt to do arithmetic outside of the G1ArithmeticBackend
  cargo test -p ultrahonk --features ark-ec/only-arithmetic-backend -- --test-threads=1 --nocapture

test-contracts:
  cargo test -p contracts -- --test-threads=1

test-e2e:
  ./scripts/e2e-tests.sh

check-pr: lint test-all

check-contracts: build-contracts
  (cd packages/contracts && cargo stylus check --wasm-file ../../target/wasm32-unknown-unknown/release/contracts.wasm)