# ArcLang MCP Server Setup Guide

## Quick Start

The MCP server is already installed and ready to use with Claude Desktop.

### 1. Start the MCP Server

```bash
cd /Users/malek/Arclang/mcp-server
/opt/homebrew/bin/python3.11 -m arclang_mcp.server
```

### 2. Configure Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "arclang": {
      "command": "/opt/homebrew/bin/python3.11",
      "args": ["-m", "arclang_mcp.server"],
      "cwd": "/Users/malek/Arclang/mcp-server"
    }
  }
}
```

### 3. Restart Claude Desktop

The MCP tools will appear as available tools in Claude Desktop.

## Available AI-Powered Features

### Diagram Generation via AI

Ask Claude to generate diagrams from your ArcLang models:

**Example prompts:**
- "Generate an operational activity diagram from my vehicle control model"
- "Create all 10 Capella diagram types for the ACC system"
- "Show me a functional dataflow diagram with the radar sensing functions"

### Available MCP Tools

1. **arclang_compile** - Compile and validate ArcLang models
2. **arclang_generate_diagram** - Generate specific diagram types
3. **arclang_generate_all_diagrams** - Generate all 10 diagram types at once
4. **arclang_validate** - Validate model syntax and semantics
5. **arclang_analyze** - Analyze model complexity and metrics

### Diagram Types Supported

All 10 Capella diagram types:
- Operational Activity (swimlane)
- Functional Dataflow
- Component Architecture
- Sequence Diagram
- State Machine
- Physical Architecture
- Class Diagram
- Tree Diagram (EPBS)
- Capability Diagram
- Functional Chain

## Web UI Integration

The web application at http://localhost:3002/editor also supports diagram generation, but **AI-powered generation through Claude Desktop MCP provides**:

- Natural language requests
- Intelligent model analysis
- Automatic diagram type selection
- Batch generation with optimization
- Contextual suggestions

## Testing

```bash
# Test MCP server directly
curl -X POST http://localhost:4001/api/diagrams/generate \
  -H "Content-Type: application/json" \
  -d '{"diagramType":"operational"}'

# Test all diagrams
curl -X POST http://localhost:4001/api/diagrams/generate-all \
  -H "Content-Type: application/json" \
  -d '{}'
```

## Status

- ✅ MCP Server installed with Python 3.11
- ✅ All 10 diagram types working
- ✅ TypeScript diagram service integrated
- ✅ API running on port 4001
- ✅ Web UI running on port 3002
- ✅ AI-powered diagram endpoints (`/api/ai/generate/:diagramType`)
- ✅ AI bulk generation endpoint (`/api/ai/generate-all`)
- ⏳ Claude Desktop configuration (manual step)

## AI-Powered Diagram Generation

The web application now supports AI-powered diagram generation through dedicated endpoints:

**Single Diagram Generation:**
```bash
curl -X POST http://localhost:4001/api/ai/generate/component \
  -H "Content-Type: application/json" \
  -d '{}'
```

**All Diagrams Generation:**
```bash
curl -X POST http://localhost:4001/api/ai/generate-all \
  -H "Content-Type: application/json" \
  -d '{}'
```

**Features:**
- AI-enhanced suggestions for diagram improvement
- Professional Capella/Arcadia methodology compliance
- Context-aware recommendations based on diagram type
- MCP integration ready (set `MCP_SERVER_URL` environment variable)
