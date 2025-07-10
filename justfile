rpc_url := "http://localhost:8547"
private_key := "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659"

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

deploy-contract contract constructor_signature="" *constructor_args="":
  just build-contract {{contract}} && \
  just optimize-contract {{contract}} && \
  if [ "{{constructor_args}}" = "" ]; then \
    cargo stylus deploy -e {{rpc_url}} --wasm-file ./target/wasm32-unknown-unknown/release/{{contract}}-opt.wasm --private-key {{private_key}} --verbose --no-verify; \
  else \
    cargo stylus deploy -e {{rpc_url}} --wasm-file ./target/wasm32-unknown-unknown/release/{{contract}}-opt.wasm --private-key {{private_key}} --verbose --no-verify --constructor-signature '{{constructor_signature}}' --constructor-args {{constructor_args}}; \
  fi

# Tests

test-ultrahonk:
  #  ark-ec/only-arithmetic-backend panics if we attempt to do arithmetic outside of the G1ArithmeticBackend
  cargo test -p ultrahonk --features ark-ec/only-arithmetic-backend -- --test-threads=1 --nocapture

test-integration:
  cargo run -p integration -- --rpc-url {{rpc_url}} --priv-key {{private_key}}

check-contract contract: 
  just build-contract {{contract}} && \
  just optimize-contract {{contract}} && \
  cargo stylus check -e {{rpc_url}} --wasm-file ./target/wasm32-unknown-unknown/release/{{contract}}-opt.wasm --verbose

verify-proof verifier_address test_vector_name:
  #!/usr/bin/env bash
  proof_hex=$(xxd -p -c 1000000 "test_vectors/{{test_vector_name}}/kat/proof" | tr -d '\n')
  inputs_hex=$(xxd -p -c 1000000 "test_vectors/{{test_vector_name}}/kat/public_inputs" | tr -d '\n')
  vk_hex=$(xxd -p -c 1000000 "test_vectors/{{test_vector_name}}/kat/vk" | tr -d '\n')

  cast call --rpc-url {{rpc_url}} {{verifier_address}} "verify(bytes,bytes,bytes)(bool)" "0x${proof_hex}" "0x${inputs_hex}" "0x${vk_hex}"

get-verifier-addresses verifier_address:
  @echo "Sumcheck Verifier Address: $(cast call {{verifier_address}} 'getSumcheckVerifierAddress()(address)' --rpc-url {{rpc_url}})"
  @echo "Shplemini Verifier Address: $(cast call {{verifier_address}} 'getShpleminiVerifierAddress()(address)' --rpc-url {{rpc_url}})"


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
