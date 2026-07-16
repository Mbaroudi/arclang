#!/usr/bin/env bash
# Round-trip test: Capella -> .arc -> compile -> apply back -> ZERO diff.
#
# Usage: roundtrip_test.sh <model-dir> <aird-filename> [arclang-binary]
# Example:
#   tools/capella_bridge/roundtrip_test.sh \
#     "capellambse-study/tests/data/models/test7_0" "Model Test 7.0.aird"
set -euo pipefail

MODEL_DIR="$1"
AIRD_NAME="$2"
ARCLANG="${3:-./target/debug/arclang}"
BRIDGE_DIR="$(cd "$(dirname "$0")" && pwd)"
PY="$BRIDGE_DIR/.venv/bin/python"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cp -r "$MODEL_DIR" "$WORK/original"
cp -r "$MODEL_DIR" "$WORK/target"

echo "== 1. Capella -> .arc"
"$PY" "$BRIDGE_DIR/capella2arc.py" "$WORK/original/$AIRD_NAME" -o "$WORK/model.arc"

echo "== 2. Compile .arc (strict parser)"
"$ARCLANG" build "$WORK/model.arc" -o "$WORK/model_build.json"

echo "== 3. Export AST JSON"
"$ARCLANG" export "$WORK/model.arc" -o "$WORK/model.json" -f json

echo "== 4. Apply .arc back onto a copy of the Capella model"
"$PY" "$BRIDGE_DIR/arc2capella.py" "$WORK/model.json" "$WORK/target/$AIRD_NAME"

echo "== 5. Diff original vs round-tripped Capella model"
if diff -r "$WORK/original" "$WORK/target"; then
    echo "ROUND-TRIP: ZERO DIFF ✓"
else
    echo "ROUND-TRIP: DIFF DETECTED ✗"
    exit 1
fi
