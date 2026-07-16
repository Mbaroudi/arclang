#!/usr/bin/env bash
# Workflow test: real editing scenarios through the Capella bridge.
#
# On a temporary copy of the model, verifies that:
#   (a) editing a component description in the .arc propagates to Capella
#   (b) editing a requirement text in the .arc propagates to Capella
#   (c) adding a component to the .arc + --create-missing creates it in
#       Capella (and a re-export of the model contains it)
#   (d) removing a component from the .arc + --delete-missing deletes it
#       from Capella (and the re-export is byte-identical to the edited .arc)
#
# Usage: workflow_test.sh <model-dir> <aird-filename> [arclang-binary]
# Example:
#   tools/capella_bridge/workflow_test.sh \
#     "tests/fixtures/capella/test7_0" "Model Test 7.0.aird"
set -euo pipefail

MODEL_DIR="$1"
AIRD_NAME="$2"
ARCLANG="${3:-./target/debug/arclang}"
BRIDGE_DIR="$(cd "$(dirname "$0")" && pwd)"
PY="$BRIDGE_DIR/.venv/bin/python"

DESC_A="workflow-test description A"
TEXT_B="workflow-test requirement text B"
COMP_C="Workflow Test Component"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cp -r "$MODEL_DIR" "$WORK/target"
CAPELLA_FILE="$WORK/target/${AIRD_NAME%.aird}.capella"

fail() { echo "WORKFLOW TEST: FAILED ✗ — $1"; exit 1; }

echo "== 1. Capella -> .arc"
"$PY" "$BRIDGE_DIR/capella2arc.py" "$WORK/target/$AIRD_NAME" -o "$WORK/step1.arc"

echo "== 2. Edit the .arc: (a) component description, (b) requirement text, (c) new component"
"$PY" - "$WORK/step1.arc" "$WORK/step2.arc" "$DESC_A" "$TEXT_B" "$COMP_C" <<'PYEOF'
import re, sys

src_path, dst_path, desc_a, text_b, comp_c = sys.argv[1:6]
lines = open(src_path, encoding="utf-8").read().split("\n")
desc_re = re.compile(r'^(\s*)description: "')
ctx = None
done_a = done_b = False
for i, line in enumerate(lines):
    stripped = line.lstrip()
    if stripped.startswith('component "'):
        ctx = "component"
    elif stripped.startswith('requirement "'):
        ctx = "requirement"
    elif re.match(r'^(model|actor|entity|activity|operational_activity|function|node) "', stripped):
        ctx = "other"
    match = desc_re.match(line)
    if match:
        indent = match.group(1)
        if ctx == "component" and not done_a:
            lines[i] = f'{indent}description: "<p>{desc_a}</p>"'
            done_a = True
        elif ctx == "requirement" and not done_b:
            lines[i] = f'{indent}description: "<p>{text_b}</p>"'
            done_b = True
if not done_a:
    sys.exit("no component description found to edit")
if not done_b:
    sys.exit("no requirement description found to edit")
lines += [
    'logical_architecture "Workflow Additions" {',
    f'  component "{comp_c}" {{',
    '    id: "workflow-new-component"',
    "  }",
    "}",
    "",
]
open(dst_path, "w", encoding="utf-8").write("\n".join(lines))
PYEOF

echo "== 3. Compile + export the edited .arc"
"$ARCLANG" build "$WORK/step2.arc" -o "$WORK/step2_build.json"
"$ARCLANG" export "$WORK/step2.arc" -o "$WORK/step2.json" -f json

echo "== 4. Apply onto Capella with --create-missing"
"$PY" "$BRIDGE_DIR/arc2capella.py" "$WORK/step2.json" "$WORK/target/$AIRD_NAME" --create-missing

echo "== 5. Check (a): component description edit reached the Capella XML"
grep -q "$DESC_A" "$CAPELLA_FILE" || fail "(a) description edit not found in $CAPELLA_FILE"
echo "   (a) OK"

echo "== 6. Check (b): requirement text edit reached the Capella XML"
grep -q "$TEXT_B" "$CAPELLA_FILE" || fail "(b) requirement text edit not found in $CAPELLA_FILE"
echo "   (b) OK"

echo "== 7. Check (c): created component is in Capella and in a re-export"
grep -q "$COMP_C" "$CAPELLA_FILE" || fail "(c) created component not found in $CAPELLA_FILE"
"$PY" "$BRIDGE_DIR/capella2arc.py" "$WORK/target/$AIRD_NAME" -o "$WORK/step3.arc"
grep -q "component \"$COMP_C\"" "$WORK/step3.arc" || fail "(c) created component missing from re-export"
echo "   (c) OK"

echo "== 8. Edit the .arc: remove the component again"
"$PY" - "$WORK/step3.arc" "$WORK/step4.arc" "$COMP_C" <<'PYEOF'
import sys

src_path, dst_path, comp_c = sys.argv[1:4]
lines = open(src_path, encoding="utf-8").read().split("\n")
out, i, removed = [], 0, False
while i < len(lines):
    line = lines[i]
    if line.lstrip().startswith(f'component "{comp_c}"'):
        depth = line.count("{") - line.count("}")
        i += 1
        while i < len(lines) and depth > 0:
            depth += lines[i].count("{") - lines[i].count("}")
            i += 1
        removed = True
        continue
    out.append(line)
    i += 1
if not removed:
    sys.exit(f"component '{comp_c}' not found in {src_path}")
open(dst_path, "w", encoding="utf-8").write("\n".join(out))
PYEOF

echo "== 9. Compile + export, apply with --delete-missing"
"$ARCLANG" build "$WORK/step4.arc" -o "$WORK/step4_build.json"
"$ARCLANG" export "$WORK/step4.arc" -o "$WORK/step4.json" -f json
"$PY" "$BRIDGE_DIR/arc2capella.py" "$WORK/step4.json" "$WORK/target/$AIRD_NAME" --delete-missing

echo "== 10. Check (d): component gone from Capella, re-export matches the edited .arc"
if grep -q "$COMP_C" "$CAPELLA_FILE"; then
    fail "(d) component still present in $CAPELLA_FILE"
fi
"$PY" "$BRIDGE_DIR/capella2arc.py" "$WORK/target/$AIRD_NAME" -o "$WORK/step5.arc"
diff "$WORK/step4.arc" "$WORK/step5.arc" || fail "(d) re-export differs from the edited .arc"
echo "   (d) OK"

echo "WORKFLOW TEST: ALL 4 SCENARIOS PASSED ✓"
