
# Hello World example

Note `@aztec/bb.js` specific version of `0.86.0` is important.

Build contracts with:

```bash
cargo build --release --target wasm32-unknown-unknown
cargo stylus deploy -e {{rpc_url}} --private-key {{private_key}} --verbose --no-verify --constructor-args {{verifier}}
```

Example

```
cargo build --release --target wasm32-unknown-unknown
cargo stylus deploy -e "http://localhost:8547" --private-key "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659" --verbose --no-verify --constructor-args "0x2f9f4741ab606632718f7bda0bf5c79e1dd03ac3"
```