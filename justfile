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


build-cli-windows: clean-windows
	cargo zigbuild --target={{WINDOWS_TARGET}} --release -p cli
	(cd target/{{WINDOWS_TARGET}}/release && \
	mkdir nsv-v{{VERSION}}-{{WINDOWS_TARGET}} && \
	mv nsv.exe nsv-v{{VERSION}}-{{WINDOWS_TARGET}} && \
	zip -r nsv-v{{VERSION}}-{{WINDOWS_TARGET}}.zip nsv-v{{VERSION}}-{{WINDOWS_TARGET}})

build-cli-linux: clean-linux
	cargo zigbuild --target={{LINUX_TARGET}} --release -p cli
	(cd target/{{LINUX_TARGET}}/release && \
	mkdir nsv-v{{VERSION}}-{{LINUX_TARGET}} && \
	mv nsv nsv-v{{VERSION}}-{{LINUX_TARGET}} && \
	tar -czf nsv-v{{VERSION}}-{{LINUX_TARGET}}.tar.gz nsv-v{{VERSION}}-{{LINUX_TARGET}})

cli-build-apple: clean-apple
	cargo zigbuild --target={{APPLE_TARGET}} --release -p cli
	(cd target/{{APPLE_TARGET}}/release && \
	mkdir nsv-v{{VERSION}}-{{APPLE_TARGET}} && \
	cp nsv nsv-v{{VERSION}}-{{APPLE_TARGET}} && \
	tar -czf nsv-v{{VERSION}}-{{APPLE_TARGET}}.tar.gz nsv-v{{VERSION}}-{{APPLE_TARGET}})

# Profiling

profile-contract contract: 
  twiggy top target/wasm32-unknown-unknown/release/{{contract}}.wasm > ./profile/{{contract}}-top.txt
  twiggy monos target/wasm32-unknown-unknown/release/{{contract}}.wasm > ./profile/{{contract}}-monos.txt
  twiggy paths target/wasm32-unknown-unknown/release/{{contract}}.wasm > ./profile/{{contract}}-path.txt
  twiggy dominators target/wasm32-unknown-unknown/release/{{contract}}.wasm > ./profile/{{contract}}-dominators.txt
  twiggy garbage target/wasm32-unknown-unknown/release/{{contract}}.wasm > ./profile/{{contract}}-garbage.txt

# Deployments

check-contract contract: 
  just build-contract {{contract}} && \
  cargo stylus check -e https://sepolia-rollup.arbitrum.io/rpc --wasm-file ./target/wasm32-unknown-unknown/release/{{contract}}.wasm --verbose


deploy-contract contract constructor_signature="" *constructor_args="":
  just build-contract {{contract}} && \
  if [ "{{constructor_args}}" = "" ]; then \
    cargo stylus deploy -e {{rpc_url}} --wasm-file ./target/wasm32-unknown-unknown/release/{{contract}}.wasm --private-key {{private_key}} --verbose --no-verify; \
  else \
    cargo stylus deploy -e {{rpc_url}} --wasm-file ./target/wasm32-unknown-unknown/release/{{contract}}.wasm --private-key {{private_key}} --verbose --no-verify --constructor-signature '{{constructor_signature}}' --constructor-args {{constructor_args}}; \
  fi

# Tests

test-ultrahonk:
  #  ark-ec/only-arithmetic-backend panics if we attempt to do arithmetic outside of the G1ArithmeticBackend
  cargo test -p ultrahonk --features ark-ec/only-arithmetic-backend -- --test-threads=1 --nocapture

test-integration:
  cargo run -p integration -- --rpc-url {{rpc_url}} --priv-key {{private_key}}

verify-proof verifier_address test_vector_name zk="false":
  #!/usr/bin/env bash

  if [ "{{zk}}" = "true" ]; then
    echo "Verifying zk-flavored proof"
    proof_hex=$(xxd -p -c 1000000 "test_vectors/{{test_vector_name}}/kat/zk-proof" | tr -d '\n')
  else
    proof_hex=$(xxd -p -c 1000000 "test_vectors/{{test_vector_name}}/kat/proof" | tr -d '\n')
  fi

  inputs_hex=$(xxd -p -c 1000000 "test_vectors/{{test_vector_name}}/kat/public_inputs" | tr -d '\n')
  vk_hex=$(xxd -p -c 1000000 "test_vectors/{{test_vector_name}}/kat/vk" | tr -d '\n')

  cast call --rpc-url {{rpc_url}} {{verifier_address}} "verify(bytes,bytes,bytes)(bool)" "0x${proof_hex}" "0x${inputs_hex}" "0x${vk_hex}"

get-verifier-addresses verifier_address:
  @echo "Sumcheck Verifier Address: $(cast call {{verifier_address}} 'getSumcheckVerifierAddress()(address)' --rpc-url {{rpc_url}})"
  @echo "Shplemini Verifier Address: $(cast call {{verifier_address}} 'getShpleminiVerifierAddress()(address)' --rpc-url {{rpc_url}})"


# Miscellaneous

check-pr: fmt lint test-ultrahonk test-integration

nitro-testnode-up:
  ./scripts/nitro-testnode.sh --detach

nitro-testnode-down:
  ./scripts/nitro-testnode.sh --quit

fmt:
  cargo fmt --package ultrahonk --package contracts --package integration -- --check

fmt-fix:
  cargo fmt --package ultrahonk --package contracts --package integration

lint:
  cargo clippy --package ultrahonk --package contracts --package integration --no-deps

lint-fix:
  cargo clippy --package ultrahonk --package contracts --package integration --fix


clean-cli-apple:
	rm -rf target/{{APPLE_TARGET}}/release/nsv-v{{VERSION}}-{{APPLE_TARGET}}

clean-cli-linux:
	rm -rf target/{{LINUX_TARGET}}/release/nsv-v{{VERSION}}-{{LINUX_TARGET}}

clean-cli-windows:
	rm -rf target/{{WINDOWS_TARGET}}/release/nsv-v{{VERSION}}-{{WINDOWS_TARGET}}