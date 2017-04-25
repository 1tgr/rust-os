#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
export PATH=$PATH:$(pwd)/3rdparty/binutils/bin:~/.cargo/bin
(
    cd src
    tup generate build.sh
    ./build.sh
)
