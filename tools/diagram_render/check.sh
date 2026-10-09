#!/usr/bin/env bash
# Lay out and draw the viewpoint diagrams of every example with the real
# layout engine and renderer; fail if any view cannot be laid out or leaves
# an element undrawn. Run from the repository root after `cargo build`.
set -euo pipefail

ARCLANG="${ARCLANG:-target/debug/arclang}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

status=0
checked=0

node --test "$(dirname "$0")/render.test.mjs" >/dev/null || { echo "✗ renderer unit tests failed (node --test tools/diagram_render/render.test.mjs)" >&2; status=1; }
node --test "$(dirname "$0")/links.test.mjs" >/dev/null || { echo "✗ navigation unit tests failed (node --test tools/diagram_render/links.test.mjs)" >&2; status=1; }
node --test "$(dirname "$0")/collapse.test.mjs" >/dev/null || { echo "✗ folding unit tests failed (node --test tools/diagram_render/collapse.test.mjs)" >&2; status=1; }
node --test "$(dirname "$0")/scope.test.mjs" >/dev/null || { echo "✗ scope unit tests failed (node --test tools/diagram_render/scope.test.mjs)" >&2; status=1; }
node --test "$(dirname "$0")/place.test.mjs" >/dev/null || { echo "✗ placement unit tests failed (node --test tools/diagram_render/place.test.mjs)" >&2; status=1; }
while IFS= read -r model; do
  graphs="$WORK/graphs.json"
  if ! "$ARCLANG" diagram "$model" -f elk -o "$graphs" >/dev/null; then
    echo "✗ $model: diagram export failed" >&2
    status=1
    continue
  fi
  echo "$model"
  node "$(dirname "$0")/render.mjs" "$graphs" --check | sed 's/^/  /' || status=1
  checked=$((checked + 1))
done < <(find examples -name '*.arc' -not -path 'examples/legacy/*' | sort)

echo "$checked models checked"
[ "$checked" -ge 10 ] || { echo "expected the example corpus, found only $checked models" >&2; exit 1; }
exit "$status"
