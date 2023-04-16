#!/bin/bash
export CC_aarch64_unknown_linux_musl=clang
export AR_aarch64_unknown_linux_musl=llvm-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld"
rustup target add aarch64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
mkdir -p homeassistant-package
rm -rf homeassistant-package/build || true
npm run build
cp -r build homeassistant-package/
cp -r data homeassistant-package/
cp target/aarch64-unknown-linux-musl/release/timesheet-backend homeassistant-package/usr/bin/
cp README.md CHANGELOG.md homeassistant-package/
