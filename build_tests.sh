#!/bin/bash

rustup target add wasm32-unknown-unknown

cd src/tests/demo_test

cargo build --release --target wasm32-unknown-unknown

cd ../demo_test_upgraded

cargo build --release --target wasm32-unknown-unknown



