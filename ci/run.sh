#!/bin/sh

set -ex

echo "toolchain versions\n------------------"

rustc -vV
cargo -vV

if [ -n "$FORCE_CLEAN" ]; then
  cargo clean
fi

cargo build --release --target $TARGET

if [ -z "$SKIP_TESTS" ]; then
  cargo test --release --target $TARGET
fi
