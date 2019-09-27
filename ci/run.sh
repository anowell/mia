#!/bin/sh

set -ex

echo "toolchain versions\n------------------"

rustc -vV
cargo -vV

cargo build --release --target $TARGET --features $FEATURES --no-default-features

if [ -z "$SKIP_TESTS" ]; then
  cargo test --release --target $TARGET --features $FEATURES --no-default-features
fi
