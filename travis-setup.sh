#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
source src/tup.config
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $CONFIG_RUST_TOOLCHAIN
~/.cargo/bin/rustup toolchain install $CONFIG_RUST_TOOLCHAIN
make -s -C 3rdparty binutils
pip3 install -r requirements.txt

