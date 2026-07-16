#!/bin/bash
# ArcLang Auto-Install Script
# Automatically copies the latest arclang binary to PATH after compilation

set -e

PROJECT_DIR="/Users/malek/arclang"
BINARY_SOURCE="$PROJECT_DIR/target/release/arclang"
INSTALL_DIR="$HOME/.local/bin"
INSTALL_PATH="$INSTALL_DIR/arclang"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "═══════════════════════════════════════════════════════════"
echo "  ArcLang Auto-Install Script"
echo "═══════════════════════════════════════════════════════════"

# Check if binary exists
if [ ! -f "$BINARY_SOURCE" ]; then
    echo -e "${RED}✗ Binary not found at: $BINARY_SOURCE${NC}"
    echo -e "${YELLOW}ℹ Building arclang in release mode...${NC}"
    cd "$PROJECT_DIR"
    cargo build --release
    if [ $? -ne 0 ]; then
        echo -e "${RED}✗ Build failed${NC}"
        exit 1
    fi
fi

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}⚠ Warning: $INSTALL_DIR is not in your PATH${NC}"
    echo -e "${YELLOW}  Add this to your ~/.zshrc or ~/.bashrc:${NC}"
    echo -e "${YELLOW}  export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
fi

# Copy binary
echo -e "${YELLOW}→ Installing arclang to $INSTALL_PATH${NC}"
cp "$BINARY_SOURCE" "$INSTALL_PATH"
chmod +x "$INSTALL_PATH"

# Verify installation
if [ -x "$INSTALL_PATH" ]; then
    VERSION=$("$INSTALL_PATH" --version 2>&1 || echo "unknown")
    echo -e "${GREEN}✓ arclang installed successfully${NC}"
    echo -e "${GREEN}  Version: $VERSION${NC}"
    echo -e "${GREEN}  Location: $INSTALL_PATH${NC}"
    
    # Test if accessible
    if command -v arclang &> /dev/null; then
        echo -e "${GREEN}✓ arclang is accessible in PATH${NC}"
    else
        echo -e "${YELLOW}⚠ arclang installed but not in current PATH${NC}"
        echo -e "${YELLOW}  Run: export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
    fi
else
    echo -e "${RED}✗ Installation failed${NC}"
    exit 1
fi

echo "═══════════════════════════════════════════════════════════"
echo -e "${GREEN}✓ Installation complete!${NC}"
echo ""
echo "Usage:"
echo "  arclang --help           # Show help"
echo "  arclang compile file.arc # Compile ArcLang file"
echo "  arclang check file.arc   # Validate syntax"
echo ""
echo "For MCP Server (Claude Desktop):"
echo "  The binary is now accessible at: $INSTALL_PATH"
echo "  MCP config should use: \"command\": \"arclang\""
echo "═══════════════════════════════════════════════════════════"
