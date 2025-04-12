#!/bin/bash
set -e

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.85 -y

brew tap picodata/homebrew-tap
brew install tarantool-picodata picodata

# Extract icu4c 73 to replace newer version with it.
# Needed by tarantool
brew extract --version=73.2 icu4c $USER/local-tap
brew install icu4c@73.2
rm -rf /opt/homebrew/opt/icu4c
ln -s /opt/homebrew/opt/icu4c@73.2 /opt/homebrew/opt/icu4c

cargo install --features="bin" tarantool-test --version "^0.3"
cargo install tarantool-runner --version "^0.1.0"
