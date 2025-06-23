lint:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -q 
  RUSTDOCFLAGS='-D warnings' cargo doc --workspace -q --no-deps

build-all:
  cargo build --release --all-features

build-ultrahonk:
  cargo build -p ultrahonk --release --features ark-ec/only-arithmetic-backend --target wasm32-unknown-unknown

build-contract contract:
  cargo build -p contracts --target wasm32-unknown-unknown --release --features {{contract}} -Z unstable-options -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort && \
  mv ./target/wasm32-unknown-unknown/release/contracts.wasm ./target/wasm32-unknown-unknown/release/{{contract}}.wasm

profile-contracts: 
  twiggy top target/wasm32-unknown-unknown/release/contracts.wasm > ./profile/top.txt
  twiggy monos target/wasm32-unknown-unknown/release/contracts.wasm > ./profile/monos.txt
  twiggy paths target/wasm32-unknown-unknown/release/contracts.wasm > ./profile/path.txt
  twiggy dominators target/wasm32-unknown-unknown/release/contracts.wasm > ./profile/dominators.txt
  twiggy garbage target/wasm32-unknown-unknown/release/contracts.wasm > ./profile/garbage.txt

optimize-contract contract:
  wasm-opt --enable-bulk-memory  -Oz -o ./target/wasm32-unknown-unknown/release/{{contract}}-opt.wasm ./target/wasm32-unknown-unknown/release/{{contract}}.wasm

deploy-contract contract rpc_url="http://localhost:8547" priv_key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659":
  cargo stylus deploy -e {{rpc_url}} --wasm-file ./target/wasm32-unknown-unknown/release/{{contract}}-opt.wasm --private-key {{priv_key}} --verbose --no-verify

test-all:
  cargo test --release --all-features

test-ultrahonk:
  #  ark-ec/only-arithmetic-backend panics if we attempt to do arithmetic outside of the G1ArithmeticBackend
  cargo test -p ultrahonk --features ark-ec/only-arithmetic-backend -- --test-threads=1 --nocapture

test-contracts:
  cargo test -p contracts -- --test-threads=1

test-e2e:
  ./scripts/e2e-tests.sh

nitro-testnode:
  ./scripts/nitro-testnode.sh

check-pr: lint test-all

check-contract contract: 
  just build-contract {{contract}} && \
  just optimize-contract {{contract}} && \
  cargo stylus check -e https://sepolia-rollup.arbitrum.io/rpc --wasm-file ./target/wasm32-unknown-unknown/release/{{contract}}-opt.wasm --verbose
