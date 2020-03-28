#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
export PATH=$PATH:$HOME/.cargo/bin:$(pwd)/3rdparty/bin
make -C src
