# Cli

## Commands

- `nsv new <project-name>`: Creates a new hello-world project, including noir circuits and stylus contracts.

## Installation

Install with cargo:

```bash
cargo install --git https://github.com/wakeuplabs-io/noir-stylus-verifier --tag v0.1.0 --package nsv
```

## Development

To build the cli binary for distribution first add the necessary rust targets:

```bash
brew install mingw-w64
rustup target add x86_64-apple-darwin
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-unknown-linux-musl
```