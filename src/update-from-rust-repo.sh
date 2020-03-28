#!/bin/bash
set -euo pipefail
cd $(dirname $0)

libs=(
  libstd/error.rs
  libstd/f32.rs
  libstd/f64.rs
  libstd/io/error.rs
  libstd/io/mod.rs
  libstd/lib.rs
  libstd/macros.rs
  libstd/num.rs
  libstd/prelude
  libstd/sys/unix/cmath.rs
  libstd/sys/wasm/io.rs
)

sysroot=$(rustc --print sysroot)
rust_src=${sysroot}/lib/rustlib/src/rust/src

set -x
rm -r ${libs[@]} || true

for lib in ${libs[@]}; do
  cp -R ${rust_src}/${lib} ./${lib}
done

# patch -p2 -R < patch.patch
