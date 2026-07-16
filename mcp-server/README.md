# 🤖 ArcLang MCP Server

**AI-powered Model Context Protocol server for ArcLang MBSE platform**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Python 3.10+](https://img.shields.io/badge/python-3.10+-blue.svg)](https://www.python.org/downloads/)
[![MCP](https://img.shields.io/badge/MCP-0.9.0+-green.svg)](https://modelcontextprotocol.io)

---

## Overview

The ArcLang MCP Server enables AI assistants (Claude, GPT-4, etc.) to interact with ArcLang models, providing intelligent assistance for Model-Based Systems Engineering (MBSE).

### Key Features

✅ **Natural Language → ArcLang**: Generate models from descriptions  
✅ **Real-time Validation**: Compile and validate models on-the-fly  
✅ **Traceability Analysis**: Check requirement coverage and gaps  
✅ **Safety Compliance**: Validate against ISO 26262, DO-178C, IEC 61508  
✅ **Semantic Merge**: Intelligent conflict resolution  
✅ **PLM Integration**: Sync with Windchill, Teamcenter, SAP  
✅ **AI-Assisted Design**: Get architecture suggestions  

---

## Quick Start

### Installation

```bash
# 1. Build and install ArcLang compiler
cd /Users/malek/arclang
cargo build --release
./install-arclang.sh

# Verify installation
arclang --version

# 2. Install MCP server
cd mcp-server
pip install -e .
```

**Note**: The `install-arclang.sh` script copies the binary to `~/.local/bin/arclang` so it's accessible in your PATH for Claude Desktop MCP.

### Usage with Claude Desktop

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "arclang": {
      "command": "arclang-mcp",
      "args": [],
      "env": {
        "ARCLANG_WORKSPACE": "/path/to/your/models"
      }
    }
  }
}
```

### Example Interactions

**Generate a requirement:**
```
User: Create an ASIL-C requirement for brake override functionality

Claude: [Uses arclang_generate_requirement]

Generated:
requirement "REQ-BRAKE-001" {
    description: "System shall override cruise control when brake pedal is pressed"
    safety_level: "ASIL_C"
    priority: "Critical"
    verification_method: "Test"
}
```

**Validate a model:**
```
User: Check if my ACC model is valid

Claude: [Uses arclang_compile with validate=true]

Result:
✓ Compilation successful
✓ 5 requirements validated
✓ 9 components validated
✓ Traceability coverage: 100%
```

**Analyze traceability:**
```
User: Are there any untraced requirements?

Claude: [Uses arclang_trace_analysis]

Found 2 gaps:
- REQ-005: No component satisfies this requirement
- LC-CTRL-02: No requirement traces to this component
```

**Generate diagrams:**
```
User: Generate an operational diagram from my ACC model

Claude: [Uses arclang_generate_diagram]

📊 Generated Operational Diagram
Output: acc_operational.svg (16KB)
Elements: 10 activities, 5 swimlanes, 9 flows
Features:
  ✅ Swimlane layout by actor
  ✅ Stick figures for human actors
  ✅ Activity symbols (⊕)
  ✅ Protocol labels (CAN, V2X, HMI)
```

**Generate all diagram types:**
```
User: Generate all Capella diagrams for my model

Claude: [Uses arclang_generate_all_diagrams]

📊 All Diagrams Generated
Summary: 10/10 diagrams generated successfully
Total Size: 127KB
- Operational: 16KB ✅
- Functional: 30KB ✅
- Component: 21KB ✅
- Sequence: 8KB ✅
- (... and 6 more)
```

---

## Available Tools

### Core Tools

| Tool | Description |
|------|-------------|
| `arclang_compile` | Compile ArcLang model to Capella XML |
| `arclang_validate` | Validate model syntax and semantics |
| `arclang_trace_analysis` | Analyze traceability coverage |
| `arclang_export_diagram` | Generate architecture diagrams |
| `arclang_info` | Get model metrics and statistics |

### Generation Tools

| Tool | Description |
|------|-------------|
| `arclang_generate_requirement` | Generate requirement from description |
| `arclang_generate_component` | Generate component architecture |
| `arclang_generate_trace` | Generate traceability links |
| `arclang_suggest_architecture` | AI-powered architecture suggestions |
| `arclang_generate_diagram` | Generate specific Capella diagram type (10 types) |
| `arclang_generate_all_diagrams` | Generate all 10 diagram types at once |

### Safety Tools

| Tool | Description |
|------|-------------|
| `arclang_safety_check` | Validate safety compliance |
| `arclang_hazard_analysis` | Perform HARA analysis |
| `arclang_fmea_generate` | Generate FMEA report |

### Integration Tools

| Tool | Description |
|------|-------------|
| `arclang_plm_sync` | Sync with PLM systems |
| `arclang_doors_import` | Import from DOORS |
| `arclang_git_merge` | Semantic merge assistance |

---

## Architecture

```
┌─────────────────┐
│  AI Assistant   │ (Claude, GPT-4, etc.)
│  (via MCP)      │
└────────┬────────┘
         │ MCP Protocol
         │
┌────────▼────────┐
│  MCP Server     │
│  (Python)       │
├─────────────────┤
│ • Tool Router   │
│ • Validators    │
│ • AI Generator  │
│ • Cache Manager │
└────────┬────────┘
         │ subprocess
         │
┌────────▼────────┐
│ ArcLang Compiler│
│ (Rust binary)   │
└────────┬────────┘
         │
    ┌────┴────┬─────────┬──────────┐
    ▼         ▼         ▼          ▼
  .arc     Capella    PLM       Git
  files      XML     Systems   Repos
```

---

## Development

### Setup Development Environment

```bash
# Clone repository
git clone https://github.com/Mbaroudi/arclang.git
cd arclang/mcp-server

# Create virtual environment
python -m venv venv
source venv/bin/activate  # or `venv\Scripts\activate` on Windows

# Install with dev dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Type checking
mypy src

# Format code
black src tests
```

### Project Structure

```
mcp-server/
├── src/
│   └── arclang_mcp/
│       ├── __init__.py
│       ├── server.py           # Main MCP server
│       ├── tools/
│       │   ├── __init__.py
│       │   ├── core.py         # Core tools (compile, validate)
│       │   ├── generation.py   # AI generation tools
│       │   ├── safety.py       # Safety validation tools
│       │   └── integration.py  # PLM/Git integration
│       ├── compiler/
│       │   ├── __init__.py
│       │   ├── wrapper.py      # ArcLang compiler wrapper
│       │   └── parser.py       # Output parser
│       ├── ai/
│       │   ├── __init__.py
│       │   ├── generator.py    # AI-powered generation
│       │   └── templates.py    # Code templates
│       └── utils/
│           ├── __init__.py
│           ├── cache.py        # Result caching
│           └── config.py       # Configuration
├── tests/
│   ├── test_server.py
│   ├── test_tools.py
│   └── test_generation.py
├── examples/
│   ├── example_session.md
│   └── sample_models/
├── docs/
│   ├── API.md
│   └── TOOLS.md
├── pyproject.toml
└── README.md
```

---

## Configuration

Create `.arclang-mcp.toml` in your workspace:

```toml
[workspace]
root = "models/"
build_dir = "build/"

[compiler]
path = "arclang"  # Path to arclang binary
timeout = 30      # Compilation timeout (seconds)

[ai]
provider = "anthropic"  # or "openai"
model = "claude-3-5-sonnet-20241022"
temperature = 0.3

[cache]
enabled = true
ttl = 3600  # 1 hour

[plm]
enabled = false
system = "windchill"  # windchill, teamcenter, sap
url = "https://plm.company.com"

[safety]
default_standard = "iso26262"
strict_validation = true
```

---

## Use Cases

### 1. Rapid Prototyping

```
Engineer: "Create an autonomous driving system with 5 main components"
AI: Generates complete logical_architecture with sensors, perception, 
    planning, control, and actuation components
```

### 2. Requirements Engineering

```
Engineer: "Convert these 20 DOORS requirements to ArcLang"
AI: Parses DOORS format, generates .arc file with proper structure
```

### 3. Safety Analysis

```
Engineer: "Validate this model for ISO 26262 ASIL-D compliance"
AI: Runs safety checks, identifies gaps, suggests fixes
```

### 4. Code Review

```
Engineer: "Review this architecture for best practices"
AI: Analyzes structure, suggests improvements, checks naming conventions
```

### 5. Merge Assistance

```
Engineer: "Help me resolve this merge conflict"
AI: Uses semantic merge tool, suggests resolution based on component IDs
```

---

## Roadmap

### Phase 1: Core (v0.1.0) ✅
- [x] Basic MCP server structure
- [x] Core tools (compile, validate, trace)
- [x] ArcLang compiler integration

### Phase 2: AI Generation (v0.2.0)
- [ ] Natural language → Requirements
- [ ] Natural language → Components
- [ ] Architecture suggestions
- [ ] Traceability generation

### Phase 3: Safety (v0.3.0)
- [ ] ISO 26262 validation
- [ ] DO-178C compliance checks
- [ ] HARA analysis
- [ ] FMEA generation

### Phase 4: Integration (v0.4.0)
- [ ] PLM sync (Windchill, Teamcenter)
- [ ] DOORS import/export
- [ ] Git semantic merge
- [ ] Polarion integration

### Phase 5: Advanced (v1.0.0)
- [ ] Multi-model analysis
- [ ] Change impact analysis
- [ ] Automated test generation
- [ ] Certification package generation

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

## License

MIT License - see [LICENSE](../LICENSE) file.

---

## Support

- **Issues**: [GitHub Issues](https://github.com/Mbaroudi/arclang/issues)
- **Documentation**: [Full Docs](https://github.com/Mbaroudi/arclang/tree/main/docs)
- **Discussions**: [GitHub Discussions](https://github.com/Mbaroudi/arclang/discussions)

---

**Built with ❤️ for the systems engineering community**

*Making MBSE intelligent, collaborative, and accessible.*
