#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
export PATH=$PATH:$HOME/.cargo/bin:$(pwd)/3rdparty/bin

make -s -C 3rdparty tools
x86_64-elf-ld --version
qemu-system-x86_64 --version

pip3 install --user -r requirements.txt

curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
which xargo || cargo install --vers 0.3.20 xargo

rustup toolchain install $(cat src/rust-toolchain) --component rust-src
