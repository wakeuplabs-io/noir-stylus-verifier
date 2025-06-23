lint:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -q 
  RUSTDOCFLAGS='-D warnings' cargo doc --workspace -q --no-deps

build-all:
  cargo build --release --all-features

build-ultrahonk:
  cargo build -p ultrahonk --release --features ark-ec/only-arithmetic-backend --target wasm32-unknown-unknown

build-contracts:
  cargo +nightly build -p contracts --target wasm32-unknown-unknown --release --features verifier -Z unstable-options -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort

profile-contracts: 
  twiggy top target/wasm32-unknown-unknown/release/contracts.wasm > ./profile/top.txt
  twiggy monos target/wasm32-unknown-unknown/release/contracts.wasm > ./profile/monos.txt
  twiggy paths target/wasm32-unknown-unknown/release/contracts.wasm > ./profile/path.txt
  twiggy dominators target/wasm32-unknown-unknown/release/contracts.wasm > ./profile/dominators.txt
  twiggy garbage target/wasm32-unknown-unknown/release/contracts.wasm > ./profile/garbage.txt

optimize-contracts: build-contracts
  wasm-opt --enable-bulk-memory  -Oz -o ./target/wasm32-unknown-unknown/release/contracts-opt.wasm ./target/wasm32-unknown-unknown/release/contracts.wasm

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

check-contracts: optimize-contracts
  cargo stylus check -e https://sepolia-rollup.arbitrum.io/rpc --wasm-file ./target/wasm32-unknown-unknown/release/contracts-opt.wasm --verbose