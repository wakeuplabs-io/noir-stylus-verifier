# UltraHonk

This crate is a rewrite of Aztec's UltraHonk verifier in Rust aiming compatibility with Arbitrum Stylus. It is compatible with Barretenberg v0.86.0. To get Barretenberg with this version, use the following commands:

```bash
git clone https://github.com/AztecProtocol/aztec-packages.git
cd aztec-packages
git checkout tags/aztec-package-v0.86.0
```

To compile Barretenberg, one can use:

```bash
cd barretenberg/cpp
bash ./scripts/docker_interactive.sh ubuntu
mkdir build
cd build
cmake --preset clang16 -DCMAKE_BUILD_TYPE=RelWithDebInfo ..
cmake --build .
```

The ``UltraHonk::verify`` verifier in `src/verifier.rs` is compatible with `UltraVerifier_<UltraFlavor>` in Barretenberg.
