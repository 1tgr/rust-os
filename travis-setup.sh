#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
export PATH=$PATH:$HOME/.cargo/bin:$(pwd)/3rdparty/bin
CONFIG_RUST_TOOLCHAIN=$(cat src/rust-toolchain)
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $CONFIG_RUST_TOOLCHAIN
which xargo || cargo install --vers 0.3.7 xargo
rustup toolchain install $CONFIG_RUST_TOOLCHAIN
rustup component add --toolchain=$CONFIG_RUST_TOOLCHAIN rust-src
make -s -C 3rdparty tools
x86_64-elf-ld --version
qemu-system-x86_64 --version
pip3 install --user -r requirements.txt
