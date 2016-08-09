#!/bin/bash
libs='liballoc libcore libcollections librustc_unicode libstd/error.rs libstd/io/error.rs libstd/io/mod.rs libstd/num'
echo rm -r $libs
rm -r $libs

tupfiles=$(git status -s | grep "^ D .*Tupfile" | cut -c4-)
echo git checkout -- $tupfiles
git checkout -- $tupfiles

for lib in $libs; do
        echo cp -RT ../rust/src/$lib ./$lib
        cp -RT ../rust/src/$lib ./$lib
done

patch -p2 -R < patch.patch
