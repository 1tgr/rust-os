#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
source src/config.txt
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $CONFIG_RUST_TOOLCHAIN
PATH=$PATH:~/.cargo/bin
which xargo || cargo install xargo
rustup toolchain install $CONFIG_RUST_TOOLCHAIN
rustup component add --toolchain=$CONFIG_RUST_TOOLCHAIN rust-src
make -s -C 3rdparty binutils
pip3 install -r requirements.txt
