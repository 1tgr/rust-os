#!/bin/bash
libs='libstd/error.rs libstd/num.rs libstd/io/error.rs libstd/io/mod.rs libstd/f32.rs libstd/f64.rs'
echo rm -r $libs
rm -r $libs

for lib in $libs; do
        echo cp -RT ../rust/src/$lib ./$lib
        cp -RT ../rust/src/$lib ./$lib
done

patch -p2 -R < patch.patch
