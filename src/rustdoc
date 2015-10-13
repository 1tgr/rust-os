#!/bin/bash
find . -mindepth 2 -name rustdoc | xargs tup
find . -mindepth 2 -name rustdoc -printf "(cd %h && echo %h && ./%f)\n" | bash
