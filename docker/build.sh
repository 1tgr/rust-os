#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
git clone /host /build
(
    cd /build
    ./travis-setup.sh
    ./travis.sh
)
