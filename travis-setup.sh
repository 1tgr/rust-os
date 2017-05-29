#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
export PATH=$PATH:$HOME/.cargo/bin:$(pwd)/3rdparty/bin
source src/config.txt
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $CONFIG_RUST_TOOLCHAIN
which xargo || cargo install xargo
rustup toolchain install $CONFIG_RUST_TOOLCHAIN
rustup component add --toolchain=$CONFIG_RUST_TOOLCHAIN rust-src
make -s -C 3rdparty tools
x86_64-elf-ld --version
qemu-system-x86_64 --version
pip3 install -r requirements.txt
