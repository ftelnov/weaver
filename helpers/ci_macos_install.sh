#!/bin/bash
set -e

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.85 -y
brew tap picodata/homebrew-tap
brew install tarantool-picodata picodata
cargo install --features="bin" tarantool-test --version "^0.3"
cargo install tarantool-runner --version "^0.1.0"
