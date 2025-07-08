# Builds

build-all:
  cargo build --release --all-features

build-ultrahonk:
  cargo build -p ultrahonk --release --features ark-ec/only-arithmetic-backend --target wasm32-unknown-unknown

build-contract contract:
  cargo build -p contracts --target wasm32-unknown-unknown --release --features {{contract}} -Z unstable-options -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort && \
  mv ./target/wasm32-unknown-unknown/release/contracts.wasm ./target/wasm32-unknown-unknown/release/{{contract}}.wasm

optimize-contract contract:
  wasm-opt --enable-bulk-memory  -Oz -o ./target/wasm32-unknown-unknown/release/{{contract}}-opt.wasm ./target/wasm32-unknown-unknown/release/{{contract}}.wasm

# Profiling

profile-contract contract: 
  twiggy top target/wasm32-unknown-unknown/release/{{contract}}.wasm > ./profile/{{contract}}-top.txt
  twiggy monos target/wasm32-unknown-unknown/release/{{contract}}.wasm > ./profile/{{contract}}-monos.txt
  twiggy paths target/wasm32-unknown-unknown/release/{{contract}}.wasm > ./profile/{{contract}}-path.txt
  twiggy dominators target/wasm32-unknown-unknown/release/{{contract}}.wasm > ./profile/{{contract}}-dominators.txt
  twiggy garbage target/wasm32-unknown-unknown/release/{{contract}}.wasm > ./profile/{{contract}}-garbage.txt

# Deployments

deploy-contract contract rpc_url="http://localhost:8547" priv_key="0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659":
  just build-contract {{contract}} && \
  just optimize-contract {{contract}} && \
  cargo stylus deploy -e {{rpc_url}} --wasm-file ./target/wasm32-unknown-unknown/release/{{contract}}-opt.wasm --private-key {{priv_key}} --verbose --no-verify

# Tests

test-ultrahonk:
  #  ark-ec/only-arithmetic-backend panics if we attempt to do arithmetic outside of the G1ArithmeticBackend
  cargo test -p ultrahonk --features ark-ec/only-arithmetic-backend -- --test-threads=1 --nocapture

test-integration:
  cargo run -p integration -- --rpc-url http://localhost:8547 --priv-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659

check-contract contract: 
  just build-contract {{contract}} && \
  just optimize-contract {{contract}} && \
  cargo stylus check -e https://sepolia-rollup.arbitrum.io/rpc --wasm-file ./target/wasm32-unknown-unknown/release/{{contract}}-opt.wasm --verbose


# Miscellaneous

nitro-testnode-up:
  ./scripts/nitro-testnode.sh --detach

nitro-testnode-down:
  ./scripts/nitro-testnode.sh --quit

fmt:
  cargo fmt --all

lint:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -q 

