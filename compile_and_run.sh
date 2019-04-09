#!/bin/bash
cargo build -p pbtc
/build/parity-bitcoin/target/debug/pbtc --btc --regtest
