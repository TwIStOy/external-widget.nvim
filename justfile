build:
  cargo build --all-targets

release:
  cargo build --all-targets --release

test:
  cd tests && cargo build && cargo test -- --nocapture && cd ..
