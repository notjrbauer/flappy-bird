#!/usr/bin/env bash
# Builds the wasm module into static/pkg/.
#
#   ./build.sh          release build, wasm-opt if available
#   ./build.sh --dev    faster build, no size optimization
set -euo pipefail

cd "$(dirname "$0")"

PROFILE="release"
[[ "${1:-}" == "--dev" ]] && PROFILE="dev"

TARGET_DIR="target/wasm32-unknown-unknown/$( [[ $PROFILE == release ]] && echo release || echo debug )"

echo "==> cargo build ($PROFILE)"
if [[ $PROFILE == release ]]; then
  cargo build --release --target wasm32-unknown-unknown
else
  cargo build --target wasm32-unknown-unknown
fi

echo "==> wasm-bindgen"
wasm-bindgen \
  --target web \
  --no-typescript \
  --out-dir static/pkg \
  "$TARGET_DIR/flappy.wasm"

if [[ $PROFILE == release ]]; then
  if command -v wasm-opt >/dev/null 2>&1; then
    echo "==> wasm-opt -Oz"
    wasm-opt -Oz \
      --enable-bulk-memory \
      --enable-nontrapping-float-to-int \
      static/pkg/flappy_bg.wasm \
      -o static/pkg/flappy_bg.wasm
  else
    echo "==> wasm-opt not found, skipping (install binaryen to shave ~10%)"
  fi
fi

RAW=$(wc -c < static/pkg/flappy_bg.wasm)
GZ=$(gzip -9 -c static/pkg/flappy_bg.wasm | wc -c)
printf '==> flappy_bg.wasm  %s bytes raw  /  %s bytes gzipped\n' "$RAW" "$GZ"
