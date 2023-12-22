build:
  cargo build --all-targets

release:
  cargo build --all-targets --release

test: build
  cargo test -- --nocapture
