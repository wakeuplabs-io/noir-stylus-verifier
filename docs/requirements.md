## Project Requirements

To work on this project, make sure the following tools and versions are installed:

### Just (1.38.0)

Used to run project commands:

```bash
brew install just@1.38.0
```

### Rust (1.89.0-nightly)

Install the required nightly toolchain and WASM target:

```bash
rustup toolchain install nightly-2025-06-12 --component rust-src
rustup target add wasm32-unknown-unknown --toolchain nightly-2025-06-12
```

### Cargo Stylus (0.6.0)

Build and deploy Stylus contracts:

```bash
cargo install cargo-stylus@0.6.0
```

### Noir (1.0.0-beta.6)

Install Noir for ZK circuit work:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v 1.0.0-beta.6
```

### Barretenberg (0.86.0)

Required for proof generation and verification:

```bash
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
bbup -v 0.86.0
```

### Docker

Used primarily for running integration tests.

➡️ Install from: [https://docs.docker.com/engine/install/](https://docs.docker.com/engine/install/)

