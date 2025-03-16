#!/bin/bash -e

wasm-pack build --target web $1
npm run bundle
mkdir -p build
cp pkg/*.wasm src/index.html build