
# Project requirements

Just 1.38.0:

```bash
brew install just@1.38.0
```

Rust 1.89.0-nightly:

```bash
rustup toolchain install nightly-2025-06-12 --component rust-src
rustup target add wasm32-unknown-unknown --toolchain nightly-2025-06-12
```

Cargo stylus 0.6.0: 

```bash
cargo install cargo-stylus@0.6.0
```

Noir:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v 1.0.0-beta.6
```

Barretenberg 0.86.0:

```bash
curl -L curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
bbup -v 0.86.0
```

[Docker](https://docs.docker.com/engine/install/): Mostly for running tests.