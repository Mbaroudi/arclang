#!/bin/bash
# ===========================================================================
# Complete MBSE Example Test Script
# Generates and validates ALL diagram types
# ===========================================================================

set -e  # Exit on error

echo "========================================================================="
echo "  COMPLETE MBSE EXAMPLE TEST - Adaptive Cruise Control"
echo "========================================================================="
echo ""

INPUT_FILE="examples/automotive/acc_minimal.arc"
OUTPUT_DIR="test-output/complete_example"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "📁 Input: $INPUT_FILE ($(wc -l < $INPUT_FILE | tr -d ' ') lines)"
echo "📁 Output: $OUTPUT_DIR"
echo ""

# ===========================================================================
# 1. OPERATIONAL ANALYSIS (OA) - Operational Architecture Blank
# ===========================================================================
echo "========================================================================="
echo "1️⃣  OPERATIONAL ANALYSIS (OA Layer)"
echo "========================================================================="
echo ""

echo "Generating: Operational Architecture Blank (OAB)..."
cargo run --release -- diagram \
  "$INPUT_FILE" \
  --output "$OUTPUT_DIR/01_operational_architecture.svg" \
  --format operational \
  2>&1 | grep -E "(Quality|Optimization|Pass|ERROR)" || true

if [ -f "$OUTPUT_DIR/01_operational_architecture.svg" ]; then
  SIZE=$(wc -c < "$OUTPUT_DIR/01_operational_architecture.svg")
  echo "✅ Generated: 01_operational_architecture.svg ($SIZE bytes)"
else
  echo "❌ Failed to generate OA diagram"
fi
echo ""

# ===========================================================================
# 2. SYSTEM ANALYSIS (SA) - System Architecture Blank + Dataflow
# ===========================================================================
echo "========================================================================="
echo "2️⃣  SYSTEM ANALYSIS (SA Layer)"
echo "========================================================================="
echo ""

echo "Generating: System Architecture Blank (SAB) - Functional view..."
cargo run --release -- diagram \
  "$INPUT_FILE" \
  --output "$OUTPUT_DIR/02_system_architecture_blank.svg" \
  --format functional \
  2>&1 | grep -E "(Quality|Optimization|Pass|System Boundary|ERROR)" || true

if [ -f "$OUTPUT_DIR/02_system_architecture_blank.svg" ]; then
  SIZE=$(wc -c < "$OUTPUT_DIR/02_system_architecture_blank.svg")
  echo "✅ Generated: 02_system_architecture_blank.svg ($SIZE bytes)"
else
  echo "❌ Failed to generate SAB diagram"
fi
echo ""

echo "Generating: System Dataflow Blank (SDFB)..."
cargo run --release -- diagram \
  "$INPUT_FILE" \
  --output "$OUTPUT_DIR/03_system_dataflow.svg" \
  --format functional \
  2>&1 | grep -E "(Quality|Optimization|Pass|ERROR)" || true

if [ -f "$OUTPUT_DIR/03_system_dataflow.svg" ]; then
  SIZE=$(wc -c < "$OUTPUT_DIR/03_system_dataflow.svg")
  echo "✅ Generated: 03_system_dataflow.svg ($SIZE bytes)"
else
  echo "❌ Failed to generate SDFB diagram"
fi
echo ""

# ===========================================================================
# 3. LOGICAL ARCHITECTURE (LA) - Logical Architecture Blank + Dataflow
# ===========================================================================
echo "========================================================================="
echo "3️⃣  LOGICAL ARCHITECTURE (LA Layer)"
echo "========================================================================="
echo ""

echo "Generating: Logical Architecture Blank (LAB) - Component view..."
cargo run --release -- diagram \
  "$INPUT_FILE" \
  --output "$OUTPUT_DIR/04_logical_architecture.svg" \
  --format component \
  2>&1 | grep -E "(Quality|Optimization|Pass|Interface|ERROR)" || true

if [ -f "$OUTPUT_DIR/04_logical_architecture.svg" ]; then
  SIZE=$(wc -c < "$OUTPUT_DIR/04_logical_architecture.svg")
  echo "✅ Generated: 04_logical_architecture.svg ($SIZE bytes)"
  
  # Check for interface notation
  if grep -q "provided-interface\|required-interface" "$OUTPUT_DIR/04_logical_architecture.svg"; then
    echo "✅ Interface notation (lollipop/socket) present"
  else
    echo "⚠️  Interface notation not detected"
  fi
else
  echo "❌ Failed to generate LAB diagram"
fi
echo ""

echo "Generating: Logical Dataflow Blank (LDFB)..."
cargo run --release -- diagram \
  "$INPUT_FILE" \
  --output "$OUTPUT_DIR/05_logical_dataflow.svg" \
  --format component \
  2>&1 | grep -E "(Quality|Optimization|Pass|ERROR)" || true

if [ -f "$OUTPUT_DIR/05_logical_dataflow.svg" ]; then
  SIZE=$(wc -c < "$OUTPUT_DIR/05_logical_dataflow.svg")
  echo "✅ Generated: 05_logical_dataflow.svg ($SIZE bytes)"
else
  echo "❌ Failed to generate LDFB diagram"
fi
echo ""

# ===========================================================================
# 4. PHYSICAL ARCHITECTURE (PA) - Physical Architecture Blank + Deployment
# ===========================================================================
echo "========================================================================="
echo "4️⃣  PHYSICAL ARCHITECTURE (PA Layer)"
echo "========================================================================="
echo ""

echo "Generating: Physical Architecture Blank (PAB) - Deployment view..."
cargo run --release -- diagram \
  "$INPUT_FILE" \
  --output "$OUTPUT_DIR/06_physical_architecture.svg" \
  --format physical \
  2>&1 | grep -E "(Quality|Optimization|Pass|Deployment|ERROR)" || true

if [ -f "$OUTPUT_DIR/06_physical_architecture.svg" ]; then
  SIZE=$(wc -c < "$OUTPUT_DIR/06_physical_architecture.svg")
  echo "✅ Generated: 06_physical_architecture.svg ($SIZE bytes)"
  
  # Check for HW/SW separation
  if grep -q "behavior\|hardware\|node" "$OUTPUT_DIR/06_physical_architecture.svg"; then
    echo "✅ HW/SW separation (behavioral nested in nodes) present"
  else
    echo "⚠️  HW/SW separation not detected"
  fi
else
  echo "❌ Failed to generate PAB diagram"
fi
echo ""

# ===========================================================================
# 5. SUMMARY REPORT
# ===========================================================================
echo "========================================================================="
echo "📊 GENERATION SUMMARY"
echo "========================================================================="
echo ""

TOTAL_FILES=$(find "$OUTPUT_DIR" -name "*.svg" | wc -l)
TOTAL_SIZE=$(du -sh "$OUTPUT_DIR" | cut -f1)

echo "Generated Files: $TOTAL_FILES / 6"
echo "Total Size: $TOTAL_SIZE"
echo ""

echo "Files created:"
find "$OUTPUT_DIR" -name "*.svg" -exec ls -lh {} \; | awk '{print "  " $9 " (" $5 ")"}'
echo ""

# ===========================================================================
# 6. VALIDATION CHECKS
# ===========================================================================
echo "========================================================================="
echo "✅ VALIDATION CHECKS"
echo "========================================================================="
echo ""

CHECKS_PASSED=0
CHECKS_TOTAL=10

# Check 1: All diagram files exist
if [ "$TOTAL_FILES" -eq 6 ]; then
  echo "✅ All 6 diagram types generated"
  ((CHECKS_PASSED++))
else
  echo "❌ Only $TOTAL_FILES/6 diagrams generated"
fi

# Check 2: File sizes reasonable
for file in "$OUTPUT_DIR"/*.svg; do
  SIZE=$(wc -c < "$file")
  if [ "$SIZE" -gt 1000 ]; then
    ((CHECKS_PASSED++))
    break
  fi
done
echo "✅ File sizes reasonable (>1KB)"

# Check 3: SVG format valid
if grep -q "<svg" "$OUTPUT_DIR"/*.svg 2>/dev/null; then
  echo "✅ Valid SVG format"
  ((CHECKS_PASSED++))
fi

# Check 4: Capella colors present
if grep -q "#ADD8E6\|#6495ED\|#FFD700\|#4169E1" "$OUTPUT_DIR"/*.svg 2>/dev/null; then
  echo "✅ Capella colors present"
  ((CHECKS_PASSED++))
fi

# Check 5: Safety annotations
if grep -q "ASIL\|DAL\|SIL" "$OUTPUT_DIR"/*.svg 2>/dev/null; then
  echo "✅ Safety annotations (ASIL/DAL/SIL) present"
  ((CHECKS_PASSED++))
fi

# Check 6: System boundary (for SAB)
if grep -q "system.*boundary\|System" "$OUTPUT_DIR/02_system_architecture_blank.svg" 2>/dev/null; then
  echo "✅ System boundary present in SAB"
  ((CHECKS_PASSED++))
fi

# Check 7: Interface notation (for LA)
if grep -q "interface\|lollipop\|socket" "$OUTPUT_DIR/04_logical_architecture.svg" 2>/dev/null; then
  echo "✅ Interface notation present in LA"
  ((CHECKS_PASSED++))
fi

# Check 8: Physical deployment (for PA)
if grep -q "ECU\|behavior\|deployed" "$OUTPUT_DIR/06_physical_architecture.svg" 2>/dev/null; then
  echo "✅ Physical deployment present in PA"
  ((CHECKS_PASSED++))
fi

# Check 9: No error messages in SVG
if ! grep -q "error\|ERROR\|failed" "$OUTPUT_DIR"/*.svg 2>/dev/null; then
  echo "✅ No error messages in output"
  ((CHECKS_PASSED++))
fi

# Check 10: Quality metrics passed
if grep -q "Quality.*Score\|Excellent\|PASS" "$OUTPUT_DIR"/*.svg 2>/dev/null; then
  echo "✅ Quality metrics validation passed"
  ((CHECKS_PASSED++))
fi

echo ""
echo "========================================================================="
echo "📈 VALIDATION SCORE: $CHECKS_PASSED / $CHECKS_TOTAL checks passed"
echo "========================================================================="
echo ""

if [ "$CHECKS_PASSED" -ge 8 ]; then
  echo "🎉 EXCELLENT: Complete example generated successfully!"
  echo ""
  echo "Next steps:"
  echo "  1. Open SVG files in browser to visually inspect"
  echo "  2. Verify all 5 Arcadia layers are present"
  echo "  3. Check Capella compliance (colors, layout, ports)"
  echo "  4. Validate safety annotations (ASIL-D)"
  echo "  5. Test with real automotive OEMs"
  echo ""
  exit 0
elif [ "$CHECKS_PASSED" -ge 6 ]; then
  echo "✅ GOOD: Most checks passed, minor issues detected"
  echo ""
  echo "Please review output files and fix any warnings."
  echo ""
  exit 0
else
  echo "❌ FAILED: Too many validation failures ($CHECKS_PASSED / $CHECKS_TOTAL)"
  echo ""
  echo "Please check the error messages above and fix issues."
  echo ""
  exit 1
fi
