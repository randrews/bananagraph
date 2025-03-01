#!/bin/bash -e

wasm-pack build --target web
npm run bundle
mkdir -p build
cp pkg/sevendrl_bg.wasm src/index.html build