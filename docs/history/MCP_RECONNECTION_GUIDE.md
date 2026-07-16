# ArcLang MCP Server - Reconnection Guide

## Issue Diagnosed ✅

The MCP server connection was lost due to a **Python syntax error** in the `mbse_expert.py` file.

## Root Cause

**File**: `/Users/malek/arclang/mcp-server/src/arclang_mcp/mbse_expert.py`  
**Line 265**: Malformed f-string with nested quotes and mismatched parentheses

```python
# BROKEN (was causing SyntaxError)
arc_content += f'            traces: ["STK-{min(i, len(analysis["arcadia_mapping"].get("operational", [])))):03d}"]\n'

# FIXED (now working)
stk_idx = min(i, len(analysis["arcadia_mapping"].get("operational", [])))
arc_content += f'            traces: ["STK-{stk_idx:03d}"]\n'
```

## Fix Applied ✅

The syntax error has been corrected. The server can now start successfully.

## Current Status

**Server Status**: ✅ RUNNING AND CONNECTED  
**Last Connection**: 2025-11-06 13:51:29  
**Tools Available**: 19 tools registered  
**Recent Activity**: Successfully processed 5 tool calls at 13:53-13:54

### Verification

```bash
# Test server import (should succeed)
/opt/homebrew/bin/python3.11 -c "import sys; sys.path.insert(0, 'mcp-server/src'); from arclang_mcp.server import main; print('✅ OK')"

# Check server logs
tail -50 ~/Library/Logs/Claude/mcp-server-arclang.log
```

## Configuration

### Claude Desktop Config
**Location**: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "arclang": {
      "command": "/opt/homebrew/bin/python3.11",
      "args": ["-m", "arclang_mcp.server"],
      "env": {
        "ARCLANG_WORKSPACE": "/Users/malek/arclang",
        "ARCLANG_BINARY": "/Users/malek/.local/bin/arclang",
        "PATH": "/Users/malek/.local/bin:/Users/malek/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin",
        "ANTHROPIC_API_KEY": "[REDACTED]"
      }
    }
  }
}
```

### Verification Checklist

- ✅ Python 3.11 installed: `/opt/homebrew/bin/python3.11`
- ✅ ArcLang binary available: `/Users/malek/.local/bin/arclang` (version 1.0.0)
- ✅ MCP package installed: `arclang-mcp 0.1.0` (editable install)
- ✅ Syntax error fixed: `mbse_expert.py` line 265
- ✅ Server can start: Import successful
- ✅ Server connected: Claude Desktop logs show successful connection
- ✅ Tools registered: 19 tools available

## Available MCP Tools

The following 19 tools are now available in Claude Desktop:

### Core Tools
1. `arclang_compile` - Compile ArcLang to Capella XML
2. `arclang_validate` - Validate model syntax
3. `arclang_trace_analysis` - Analyze traceability
4. `arclang_export_diagram` - Export diagrams
5. `arclang_info` - Get model metrics

### Generation Tools
6. `arclang_generate_requirement` - Generate requirements from NL
7. `arclang_generate_component` - Generate components
8. `arclang_suggest_architecture` - AI architecture suggestions
9. `arclang_generate_diagram` - Generate specific diagram type
10. `arclang_generate_all_diagrams` - Generate all 10 diagram types

### Safety Tools
11. `arclang_safety_check` - Validate safety standards
12. `arclang_hazard_analysis` - Perform HARA analysis

### Integration Tools
13. `arclang_git_merge` - Semantic merge assistance

### MBSE Expert Tools
14. `arclang_analyze_requirements` - Analyze requirements and map to Arcadia layers
15. `arclang_generate_from_requirements` - Generate complete model from analysis
16. `arclang_assess_diagram_quality` - Assess diagram quality
17. `arclang_suggest_diagram_enhancements` - Suggest enhancements
18. `arclang_generate_mbse_report` - Generate MBSE quality report

### Batch Execution
19. `arclang_execute_batch` - Execute multiple operations efficiently

## Testing the Connection

### Quick Test in Claude Desktop

Simply ask Claude:
```
Can you use the arclang_info tool to check if the MCP server is working?
```

Claude should be able to access the tool and respond.

### Advanced Test

```
Generate an operational diagram from examples/automotive/acc_complete_all_diagrams.arc
```

## Common Connection Issues & Fixes

### Issue 1: Python Syntax Errors
**Symptom**: Server fails to start with `SyntaxError`  
**Solution**: ✅ FIXED - Corrected f-string syntax in `mbse_expert.py`

### Issue 2: Module Not Found
**Symptom**: `ModuleNotFoundError: No module named 'mcp'`  
**Solution**: 
```bash
/opt/homebrew/bin/python3.11 -m pip install -e mcp-server/
```

### Issue 3: Missing ArcLang Binary
**Symptom**: `arclang: command not found`  
**Solution**:
```bash
cargo build --release
./install-arclang.sh
```

### Issue 4: Wrong Python Path
**Symptom**: Server uses wrong Python version  
**Solution**: Update `claude_desktop_config.json` with correct Python path:
```bash
which python3.11  # Get the correct path
```

### Issue 5: Permission Errors
**Symptom**: `Permission denied` when starting server  
**Solution**:
```bash
chmod +x ~/.local/bin/arclang
```

## Restart Steps (if needed)

1. **Restart Claude Desktop**:
   - Quit Claude Desktop completely (Cmd+Q)
   - Reopen Claude Desktop
   - The MCP server will auto-start

2. **Check Logs**:
   ```bash
   tail -f ~/Library/Logs/Claude/mcp-server-arclang.log
   ```

3. **Verify Connection**:
   Look for these messages in the log:
   - `Server started and connected successfully`
   - `Message from server: ...result...`
   - Tool calls being processed

## Maintenance

### Update MCP Server

After making changes to Python files:

```bash
# The server will auto-reload since it's installed as editable (-e)
# Just restart Claude Desktop to pick up changes
```

### View Real-time Logs

```bash
tail -f ~/Library/Logs/Claude/mcp-server-arclang.log
```

### Validate All Python Files

```bash
find mcp-server/src/arclang_mcp -name "*.py" -exec /opt/homebrew/bin/python3.11 -m py_compile {} \;
```

## Support

If you encounter issues:

1. Check logs: `~/Library/Logs/Claude/mcp-server-arclang.log`
2. Test import: `/opt/homebrew/bin/python3.11 -c "from arclang_mcp.server import main"`
3. Verify binary: `~/.local/bin/arclang --version`
4. Check config: `cat ~/Library/Application\ Support/Claude/claude_desktop_config.json`

## Next Steps

The MCP server is now working. You can:

1. ✅ Use all 19 tools in Claude Desktop
2. ✅ Generate diagrams from ArcLang models
3. ✅ Analyze requirements and map to Arcadia layers
4. ✅ Validate models and check traceability
5. ✅ Generate complete models from natural language

---

**Status**: ✅ MCP Server is CONNECTED and OPERATIONAL  
**Last Updated**: 2025-11-06 14:54  
**Fix Applied**: mbse_expert.py syntax error corrected
