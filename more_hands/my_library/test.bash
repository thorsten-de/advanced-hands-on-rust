#!/bin/bash

cargo test --no-default-features
cargo test --no-default-features --features xorshift
cargo test --no-default-features --features pcg