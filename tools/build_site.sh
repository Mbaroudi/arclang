#!/usr/bin/env bash
# Build the ArcLang website: wasm compiler package + static assets.
# Output: site/ is self-contained and ready to serve.
set -euo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"

echo "==> Building wasm package"
(cd wasm && wasm-pack build --target web --release)
if command -v wasm-opt >/dev/null; then
  echo "==> Optimizing with wasm-opt"
  wasm-opt -Oz wasm/pkg/arclang_wasm_bg.wasm -o wasm/pkg/arclang_wasm_bg.wasm \
    --enable-bulk-memory --enable-nontrapping-float-to-int
fi

echo "==> Staging wasm package into site/"
rm -rf site/playground/pkg
mkdir -p site/playground/pkg
cp wasm/pkg/arclang_wasm.js wasm/pkg/arclang_wasm_bg.wasm site/playground/pkg/

echo "==> Staging examples"
cp examples/complete_emergency_braking_simple.arc site/playground/examples/emergency_braking.arc

echo "==> Done. site/ ($(du -sh site | cut -f1)) is ready to serve."
