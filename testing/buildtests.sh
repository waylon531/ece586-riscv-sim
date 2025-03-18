#!/bin/sh
set -e
cd testGenerator
cmake -S . -B build
cmake --build build
./testGenerator
cd ..
cmake -S . -B build
cmake --build build
