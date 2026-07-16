#!/bin/bash
# Test Rendering Engine Improvements
# Tests the new Phase 1 & 2 modules with real models

set -e

echo "================================================"
echo "ArcLang Rendering Engine - Quality Test"
echo "Testing Phase 1 & 2 Improvements"
echo "================================================"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create output directory
OUTPUT_DIR="test-output-rendering-v2"
mkdir -p "$OUTPUT_DIR"

echo -e "${BLUE}Output directory: $OUTPUT_DIR${NC}"
echo ""

# Test model
MODEL="emergency_braking_all_diagrams.arc"

if [ ! -f "$MODEL" ]; then
    echo "Error: Model file $MODEL not found"
    exit 1
fi

echo -e "${BLUE}Testing with model: $MODEL${NC}"
echo ""

# Test with different formats to see rendering quality
FORMATS=(
    "arc-viz-elk"
    "arc-viz-elk-complete"
    "arc-viz-dagre"
    "arc-viz-hybrid"
)

echo "================================================"
echo "Testing Diagram Generation"
echo "================================================"
echo ""

for format in "${FORMATS[@]}"; do
    echo -e "${YELLOW}Format: $format${NC}"
    
    output_file="$OUTPUT_DIR/emergency_braking_${format}.html"
    
    if ./target/release/arclang export -f "$format" -o "$output_file" "$MODEL" 2>&1 | grep -q "Successfully"; then
        echo -e "${GREEN}✓ Generated: $output_file${NC}"
        
        # Check file size
        size=$(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file" 2>/dev/null || echo "0")
        echo "  File size: $size bytes"
    else
        echo -e "✗ Failed to generate $format"
    fi
    echo ""
done

echo "================================================"
echo "Rendering Test Complete"
echo "================================================"
echo ""
echo "Generated files in: $OUTPUT_DIR/"
echo ""
echo "Next steps:"
echo "1. Open the HTML files in a browser"
echo "2. Compare visual quality:"
echo "   - Layout strategy (swimlane/hierarchy/port-centric)"
echo "   - Color coding (Capella colors)"
echo "   - Alignment and spacing"
echo "   - Safety indicators (if model has ASIL levels)"
echo "3. Check browser console for quality metrics"
echo ""
echo "Quality improvements to look for:"
echo "  ✓ Context-aware layouts (not all the same)"
echo "  ✓ Grid-snapped alignment (10px precision)"
echo "  ✓ Capella color scheme (green/blue/orange)"
echo "  ✓ Professional spacing (60px gaps)"
echo "  ✓ Legend with phase information"
echo ""
