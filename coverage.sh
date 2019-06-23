#!/bin/sh

mkdir coverage

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Zno-landing-pads"
cargo clean
cargo test --verbose

zip -0 coverage/ccov.zip `find . \( -name "fernsprechapparat*.gc*" \) -print`;
grcov coverage/ccov.zip -s . -t lcov --llvm --branch --ignore-not-existing --ignore-dir "/*" -o coverage/lcov.info;
genhtml -o coverage/report --show-details --highlight --ignore-errors source --legend coverage/lcov.info
