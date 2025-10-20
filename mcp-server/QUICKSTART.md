# 🚀 ArcLang MCP Server - Quick Start

Get started with AI-powered MBSE in 5 minutes!

---

## Prerequisites

1. **ArcLang Compiler** installed
2. **Python 3.10+**
3. **Claude Desktop** or any MCP-compatible client
4. **Anthropic API Key** (optional, for AI generation)

---

## Installation

### Step 1: Install ArcLang Compiler

```bash
cd ..
cargo install --path .
arclang --version  # Verify installation
```

### Step 2: Install MCP Server

```bash
cd mcp-server
pip install -e .
```

### Step 3: Verify Installation

```bash
arclang-mcp --help
```

---

## Configuration

### Option 1: Claude Desktop (Recommended)

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "arclang": {
      "command": "python",
      "args": ["-m", "arclang_mcp.server"],
      "env": {
        "ARCLANG_WORKSPACE": "/path/to/your/models",
        "ANTHROPIC_API_KEY": "your-api-key-here"
      }
    }
  }
}
```

### Option 2: Environment Variables

```bash
export ARCLANG_WORKSPACE="/path/to/models"
export ANTHROPIC_API_KEY="your-api-key"
export ARCLANG_BINARY="arclang"  # Optional
```

### Option 3: Configuration File

Create `.arclang-mcp.toml` in your workspace:

```toml
[workspace]
root = "models/"
build_dir = "build/"

[compiler]
path = "arclang"
timeout = 30

[ai]
provider = "anthropic"
model = "claude-3-5-sonnet-20241022"
temperature = 0.3
```

---

## First Usage

### 1. Restart Claude Desktop

Close and reopen Claude Desktop to load the MCP server.

### 2. Verify Connection

In Claude Desktop, type:
```
Can you list the available ArcLang tools?
```

Claude should show 15+ tools available.

### 3. Create Your First Model

**You**: "Create an ASIL-B requirement for maintaining safe distance"

**Claude**: [Uses arclang_generate_requirement]
```arc
requirement "REQ-DIST-001" {
    description: "System shall maintain 2-second safe following distance"
    priority: "Critical"
    safety_level: "ASIL_B"
    type: "Functional"
    verification_method: "Test"
}
```

### 4. Generate Architecture

**You**: "Suggest an architecture for this requirement"

**Claude**: [Uses arclang_suggest_architecture]

Suggests components like:
- Radar Sensor
- Distance Controller
- Safety Monitor

### 5. Validate Model

**You**: "Validate my model at models/acc_system.arc"

**Claude**: [Uses arclang_validate]

Shows validation results with errors/warnings.

---

## Common Workflows

### Create Complete System

```
You: I need to create an adaptive cruise control system for automotive (ISO 26262 ASIL-B). 
     Can you help me design it?

Claude: [Uses multiple tools]
1. Generates requirements
2. Suggests architecture
3. Creates components
4. Validates traceability
5. Checks safety compliance
```

### Validate Existing Model

```
You: Check if models/flight_control.arc is valid and compliant with DO-178C DAL-A

Claude: [Uses arclang_validate + arclang_safety_check]
- Validates syntax
- Checks safety compliance
- Reports issues
- Suggests fixes
```

### Analyze Traceability

```
You: Find untraced requirements in my model

Claude: [Uses arclang_trace_analysis]
- Shows coverage percentage
- Lists untraced requirements
- Lists untraced components
- Suggests trace links
```

### Resolve Merge Conflicts

```
You: I have a merge conflict in models/architecture.arc. 
     The conflict is between our changes to REQ-001 and their changes.

Claude: [Uses arclang_git_merge]
- Analyzes both versions
- Identifies conflicts by component ID
- Suggests resolution strategy
```

---

## Available Tools

### Core Tools ✅

| Tool | Use When |
|------|----------|
| `arclang_compile` | Compile model to Capella XML |
| `arclang_validate` | Check syntax and semantics |
| `arclang_trace_analysis` | Analyze traceability |
| `arclang_export_diagram` | Generate visual diagrams |
| `arclang_info` | Get model statistics |

### Generation Tools 🎨

| Tool | Use When |
|------|----------|
| `arclang_generate_requirement` | Create requirement from description |
| `arclang_generate_component` | Create component from description |
| `arclang_suggest_architecture` | Get AI architecture suggestions |

### Safety Tools 🛡️

| Tool | Use When |
|------|----------|
| `arclang_safety_check` | Validate safety compliance |
| `arclang_hazard_analysis` | Perform HARA analysis |

### Integration Tools 🔗

| Tool | Use When |
|------|----------|
| `arclang_git_merge` | Resolve merge conflicts |
| `arclang_plm_sync` | Sync with PLM systems |

---

## Tips & Tricks

### 1. Natural Language Works Best

Instead of:
```
Use arclang_generate_requirement with description "brake system" and safety_level "ASIL_C"
```

Try:
```
Create an ASIL-C requirement for the brake override system
```

### 2. Ask for Multiple Steps

```
Create a complete ACC system with requirements, architecture, and traceability
```

Claude will use multiple tools automatically.

### 3. Iterative Refinement

```
You: Create a radar sensor component
Claude: [generates component]
You: Add a self-diagnostic function
Claude: [refines component]
You: Make it ASIL-D compliant
Claude: [adds safety attributes]
```

### 4. Context-Aware Assistance

```
You: I'm working on an aerospace flight control system (DO-178C DAL-A).
     What components do I need?

Claude: [Understands domain and standard, suggests appropriate architecture]
```

---

## Troubleshooting

### MCP Server Not Showing in Claude

1. Check Claude Desktop config:
   ```bash
   cat ~/Library/Application\ Support/Claude/claude_desktop_config.json
   ```

2. Verify paths are correct:
   ```bash
   which python
   which arclang
   ```

3. Check logs:
   ```bash
   tail -f ~/Library/Logs/Claude/mcp*.log
   ```

### Compilation Errors

```bash
# Test compiler directly
arclang build models/test.arc

# Check if compiler is in PATH
which arclang

# Set explicit path in config
export ARCLANG_BINARY="/full/path/to/arclang"
```

### API Key Issues

```bash
# Verify API key is set
echo $ANTHROPIC_API_KEY

# Or add to config file
cat > ~/.arclang-mcp.toml << EOF
[ai]
api_key = "sk-ant-..."
EOF
```

---

## What's Next?

### Learn More

- [Full Documentation](README.md)
- [Example Sessions](examples/example_session.md)
- [Tool Reference](docs/TOOLS.md)

### Extend

- Add custom tools
- Integrate with your PLM
- Create domain-specific templates

### Contribute

- Report issues: https://github.com/Mbaroudi/arclang/issues
- Submit PRs: https://github.com/Mbaroudi/arclang/pulls

---

## Example: Complete Session

```
You: Create an automotive ACC system with ISO 26262 ASIL-B compliance

Claude: I'll create a complete ACC system for you.

[Generates Requirements]
- REQ-ACC-001: Maintain safe distance (ASIL-B)
- REQ-ACC-002: Detect lead vehicle (ASIL-B)
- REQ-ACC-003: Control throttle (ASIL-B)

[Suggests Architecture]
- Radar Sensor (LC-SENS-RADAR)
- ACC Controller (LC-CTRL-ACC)
- Throttle Actuator (LC-ACT-THROTTLE)
- Safety Monitor (LC-SAFETY-MON)

[Creates Components with Functions]
Each component with appropriate:
- Inputs/outputs
- Execution times
- Safety levels

[Establishes Traceability]
All components traced to requirements

[Validates Safety]
✅ ISO 26262 ASIL-B compliant
✅ 100% traceability coverage
✅ All safety requirements satisfied

You: Export a diagram

Claude: [Generates diagram]
✅ Diagram saved to acc_system.html

You: Perfect! Thanks!
```

---

**Ready to transform your systems engineering workflow?**

Start chatting with Claude and experience AI-powered MBSE! 🚀
