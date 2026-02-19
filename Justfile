set dotenv-load := false

fmt:
    cargo fmt --all

clippy:
    cargo clippy --all-targets --all-features

test:
    cargo test --workspace

build:
    cargo build --workspace

wasm-build:
    wasm-pack build crates/engulf-wasm --release --target bundler

wasm-publish:
    cd crates/engulf-wasm/pkg && npm publish --access public
