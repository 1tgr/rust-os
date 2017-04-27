#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
git clone /host /build
(
    cd /build
    rsync --times --links --ignore-existing --recursive /host-bootstrap/3rdparty/ 3rdparty
    ./travis-setup.sh
    ./travis.sh
)
