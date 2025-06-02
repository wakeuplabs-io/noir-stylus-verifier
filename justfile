lint:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -q -- -D warnings
  RUSTDOCFLAGS='-D warnings' cargo doc --workspace -q --no-deps

build-all:
  cargo build --release --all-features

test-all:
  cargo test --release --all-features

check-pr: lint test-all
