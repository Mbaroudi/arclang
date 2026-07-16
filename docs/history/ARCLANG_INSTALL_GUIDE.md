# ArcLang Installation & Setup Guide

Complete guide for installing ArcLang compiler and making it accessible to all tools (Terminal, MCP Server, Claude Desktop).

---

## 🎯 Goal

Install `arclang` binary so it's accessible from:
1. ✅ Terminal (any directory)
2. ✅ MCP Server (for Claude Desktop)
3. ✅ ArcViz Web API (for compilation)
4. ✅ Scripts and automation

---

## 📦 Installation Methods

### Method 1: Quick Install (Recommended)

```bash
# Navigate to project
cd /Users/malek/arclang

# Build release binary
cargo build --release

# Install to PATH
./install-arclang.sh

# Verify
arclang --version
```

**Result**: `arclang` binary copied to `~/.local/bin/arclang` (in PATH)

---

### Method 2: Build + Install Alias

```bash
# Navigate to project
cd /Users/malek/arclang

# Build and install in one command
cargo build-install

# Verify
arclang --version
```

**Note**: Uses Cargo alias defined in `.cargo/config.toml`

---

### Method 3: Manual Installation

```bash
# Build binary
cd /Users/malek/arclang
cargo build --release

# Copy to PATH manually
cp target/release/arclang ~/.local/bin/arclang
chmod +x ~/.local/bin/arclang

# Verify
which arclang
arclang --version
```

---

## 🔧 PATH Configuration

### Check if ~/.local/bin is in PATH

```bash
echo $PATH | grep ".local/bin"
```

### Add to PATH (if needed)

**For Zsh (macOS default):**
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**For Bash:**
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

---

## 🤖 Claude Desktop MCP Setup

### 1. Verify arclang is accessible

```bash
arclang --version
```

### 2. Configure Claude Desktop

Edit: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "arclang": {
      "command": "arclang-mcp",
      "args": [],
      "env": {
        "ARCLANG_WORKSPACE": "/Users/malek/arclang/examples",
        "ARCLANG_COMPILER": "arclang"
      }
    }
  }
}
```

**Key Point**: Use `"command": "arclang"` (not full path) since it's in PATH

### 3. Test MCP Server

```bash
# Install MCP server if not already done
cd /Users/malek/arclang/mcp-server
pip install -e .

# Test directly
arclang-mcp
```

### 4. Restart Claude Desktop

- Quit Claude Desktop completely
- Reopen Claude Desktop
- MCP server should now find `arclang` binary

---

## 🔄 Auto-Update After Code Changes

### Option 1: Use install script after build

```bash
# Make changes to Rust code
cd /Users/malek/arclang

# Rebuild and reinstall
cargo build --release
./install-arclang.sh
```

### Option 2: Use Cargo alias

```bash
# Make changes to Rust code
cd /Users/malek/arclang

# Build and install in one command
cargo build-install
```

### Option 3: Add to your workflow script

```bash
#!/bin/bash
# rebuild.sh - Rebuild and reinstall arclang

cd /Users/malek/arclang

echo "Building arclang..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "Installing arclang..."
    ./install-arclang.sh
    echo "✓ Complete!"
else
    echo "✗ Build failed"
    exit 1
fi
```

---

## 🧪 Verification Checklist

Run these commands to verify everything works:

```bash
# 1. Check binary exists
ls -lh ~/.local/bin/arclang

# 2. Check it's executable
file ~/.local/bin/arclang

# 3. Check version
arclang --version

# 4. Check it's in PATH
which arclang

# 5. Test from different directory
cd /tmp
arclang --version

# 6. Test compilation (if you have a .arc file)
arclang compile /Users/malek/arclang/examples/automotive/acc_minimal.arc
```

All commands should succeed without errors.

---

## ❌ Troubleshooting

### "arclang: not found"

**Problem**: Binary not in PATH

**Solution**:
```bash
# Check if ~/.local/bin exists
ls ~/.local/bin

# Check PATH
echo $PATH | grep ".local/bin"

# If not in PATH, add it
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

---

### "Permission denied"

**Problem**: Binary not executable

**Solution**:
```bash
chmod +x ~/.local/bin/arclang
```

---

### MCP Server can't find arclang

**Problem**: MCP running in different shell environment

**Solution 1**: Use full path in MCP config
```json
{
  "mcpServers": {
    "arclang": {
      "command": "/Users/malek/.local/bin/arclang",
      ...
    }
  }
}
```

**Solution 2**: Ensure PATH is set in MCP env
```json
{
  "mcpServers": {
    "arclang": {
      "command": "arclang",
      "env": {
        "PATH": "/Users/malek/.local/bin:/usr/local/bin:/usr/bin:/bin",
        ...
      }
    }
  }
}
```

---

### Changes to code not reflected

**Problem**: Old binary still in PATH

**Solution**:
```bash
# Rebuild and reinstall
cd /Users/malek/arclang
cargo clean
cargo build --release
./install-arclang.sh

# Verify new version
arclang --version
```

---

## 📝 File Locations

| Item | Location |
|------|----------|
| Source code | `/Users/malek/arclang/src/` |
| Compiled binary (release) | `/Users/malek/arclang/target/release/arclang` |
| Installed binary | `~/.local/bin/arclang` |
| Install script | `/Users/malek/arclang/install-arclang.sh` |
| Cargo config | `/Users/malek/arclang/.cargo/config.toml` |
| MCP config | `~/Library/Application Support/Claude/claude_desktop_config.json` |

---

## 🚀 Quick Commands Reference

```bash
# Build only
cargo build --release

# Build + Install
cargo build --release && ./install-arclang.sh

# Build + Install (alias)
cargo build-install

# Verify installation
arclang --version

# Test compilation
arclang compile examples/automotive/acc_minimal.arc

# Check what's using arclang
which arclang
lsof $(which arclang)
```

---

## ✅ Success!

You should now have:
- ✅ `arclang` binary in `~/.local/bin/`
- ✅ Binary accessible from any directory
- ✅ Binary accessible to MCP Server
- ✅ Automatic update script ready
- ✅ Cargo alias for convenience

**Next Steps**:
1. Test arclang with example files
2. Configure Claude Desktop MCP
3. Test MCP integration
4. Use arclang in your projects

---

**Need Help?**
- Check troubleshooting section above
- Verify PATH configuration
- Test with simple `arclang --version` first
