# 🏗️ ArcLang MCP Server - Architecture

Technical architecture documentation for the MCP server.

---

## System Overview

```
┌──────────────────────────────────────────────────────────┐
│                    AI Assistant                          │
│              (Claude, GPT-4, etc.)                       │
└────────────────────┬─────────────────────────────────────┘
                     │ MCP Protocol (stdio)
                     │
┌────────────────────▼─────────────────────────────────────┐
│                MCP Server (Python)                       │
│  ┌─────────────────────────────────────────────────┐    │
│  │              Tool Router                        │    │
│  │  • Request parsing                              │    │
│  │  • Tool dispatching                             │    │
│  │  • Response formatting                          │    │
│  └─────────────────┬───────────────────────────────┘    │
│                    │                                     │
│  ┌─────────────────┼───────────────────────────────┐    │
│  │   Tool Groups   │                               │    │
│  │                 │                               │    │
│  │  ┌──────────────▼──────────┐                    │    │
│  │  │     Core Tools          │                    │    │
│  │  │  • Compile              │                    │    │
│  │  │  • Validate             │                    │    │
│  │  │  • Trace Analysis       │                    │    │
│  │  │  • Export Diagram       │                    │    │
│  │  │  • Model Info           │                    │    │
│  │  └─────────────────────────┘                    │    │
│  │                                                  │    │
│  │  ┌─────────────────────────┐                    │    │
│  │  │   Generation Tools      │                    │    │
│  │  │  • Generate Requirement │                    │    │
│  │  │  • Generate Component   │                    │    │
│  │  │  • Suggest Architecture │                    │    │
│  │  └────────────┬────────────┘                    │    │
│  │               │ Claude API                      │    │
│  │  ┌────────────▼────────────┐                    │    │
│  │  │    AI Generator         │                    │    │
│  │  │  • Prompt engineering   │                    │    │
│  │  │  • Template management  │                    │    │
│  │  │  • Code validation      │                    │    │
│  │  └─────────────────────────┘                    │    │
│  │                                                  │    │
│  │  ┌─────────────────────────┐                    │    │
│  │  │    Safety Tools         │                    │    │
│  │  │  • Safety Check         │                    │    │
│  │  │  • Hazard Analysis      │                    │    │
│  │  └─────────────────────────┘                    │    │
│  │                                                  │    │
│  │  ┌─────────────────────────┐                    │    │
│  │  │  Integration Tools      │                    │    │
│  │  │  • Git Merge            │                    │    │
│  │  │  • PLM Sync             │                    │    │
│  │  └─────────────────────────┘                    │    │
│  └──────────────────────────────────────────────────┘    │
│                                                           │
│  ┌──────────────────────────────────────────────────┐    │
│  │         Compiler Wrapper                         │    │
│  │  • Command execution                             │    │
│  │  • Output parsing                                │    │
│  │  • Error handling                                │    │
│  │  • Result caching                                │    │
│  └────────────────────┬─────────────────────────────┘    │
└───────────────────────┼──────────────────────────────────┘
                        │ subprocess
                        │
┌───────────────────────▼──────────────────────────────────┐
│              ArcLang Compiler (Rust)                     │
│  ┌──────────────────────────────────────────────────┐    │
│  │  Lexer → Parser → Semantic → Codegen             │    │
│  └──────────────────────────────────────────────────┘    │
└───────────────────────┬──────────────────────────────────┘
                        │
        ┌───────────────┼──────────────┐
        │               │              │
        ▼               ▼              ▼
    .arc files    Capella XML    Diagrams (HTML/SVG)
```

---

## Component Details

### 1. MCP Server (`server.py`)

**Responsibilities**:
- MCP protocol handling
- Tool registration
- Request routing
- Response formatting

**Key Classes**:
```python
class ArcLangMCPServer:
    def __init__(self, workspace_root: Path)
    def _register_tools(self) -> None
    async def run(self) -> None
```

**Protocol Flow**:
```
1. Client connects via stdio
2. Server sends tool list
3. Client sends tool call request
4. Server routes to appropriate handler
5. Handler executes tool
6. Server formats response
7. Client receives result
```

### 2. Core Tools (`tools/core.py`)

**Responsibilities**:
- Direct compiler interaction
- Model validation
- Traceability analysis
- Diagram generation

**Implementation**:
```python
class CoreTools:
    async def _compile(self, args) -> str
    async def _validate(self, args) -> str
    async def _trace_analysis(self, args) -> str
    async def _export_diagram(self, args) -> str
    async def _info(self, args) -> str
```

**Example Tool Flow**:
```
arclang_compile
    ↓
CoreTools._compile()
    ↓
ArcLangCompiler.compile()
    ↓
subprocess: arclang build model.arc
    ↓
Parse output
    ↓
Format response
    ↓
Return to client
```

### 3. Generation Tools (`tools/generation.py`)

**Responsibilities**:
- AI-powered code generation
- Natural language processing
- Template-based generation

**Implementation**:
```python
class GenerationTools:
    async def _generate_requirement(self, args) -> str
    async def _generate_component(self, args) -> str
    async def _suggest_architecture(self, args) -> str
```

**AI Generation Flow**:
```
User request in natural language
    ↓
AIGenerator.generate_requirement()
    ↓
Build prompt with context
    ↓
Call Claude API
    ↓
Parse response
    ↓
Validate generated code
    ↓
Format as ArcLang
    ↓
Return to user
```

### 4. Compiler Wrapper (`compiler/wrapper.py`)

**Responsibilities**:
- Subprocess management
- Command building
- Output parsing
- Error handling

**Key Methods**:
```python
class ArcLangCompiler:
    async def compile(self, model_path, validate, optimize)
    async def validate(self, model_path, strict)
    async def trace_analysis(self, model_path, show_gaps, matrix)
    async def _run_command(self, cmd) -> Dict
    def _parse_metrics(self, output) -> Dict
```

**Command Patterns**:
```bash
# Compile
arclang build model.arc --validate --optimize

# Validate
arclang check model.arc --lint --strict

# Trace Analysis
arclang trace model.arc --validate --gaps --matrix

# Export
arclang export model.arc -o output.html -f arc-viz-ultimate

# Safety
arclang safety model.arc --standard iso26262 --report
```

### 5. AI Generator (`ai/generator.py`)

**Responsibilities**:
- Claude API integration
- Prompt engineering
- Code validation
- Template management

**Prompt Engineering**:
```python
prompt = f"""Generate ArcLang requirement:

Description: {description}
Safety Level: {safety_level}
Priority: {priority}

Format:
requirement "REQ-XXX-YYY" {{
    description: "..."
    priority: "{priority}"
    safety_level: "{safety_level}"
    type: "Functional"
    verification_method: "Test"
}}

Requirements:
- Use descriptive ID
- Follow ISO 26262 conventions
- Keep concise
"""
```

---

## Data Flow

### Tool Execution Flow

```
┌─────────────────────────────────────────────────┐
│ 1. Client Request                               │
│    {"tool": "arclang_compile",                  │
│     "args": {"model_path": "model.arc"}}        │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│ 2. MCP Server                                   │
│    • Parse request                              │
│    • Validate tool name                         │
│    • Extract arguments                          │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│ 3. Tool Router                                  │
│    • Match tool to handler group                │
│    • Route to CoreTools, GenerationTools, etc. │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│ 4. Tool Handler                                 │
│    • Execute specific tool logic                │
│    • Call compiler wrapper or AI generator      │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│ 5. Execute Operation                            │
│    • Compiler: subprocess call                  │
│    • AI: Claude API call                        │
│    • Integration: External API                  │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│ 6. Parse Result                                 │
│    • Extract metrics                            │
│    • Parse errors/warnings                      │
│    • Format for presentation                    │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│ 7. Format Response                              │
│    • Add icons and formatting                   │
│    • Include recommendations                    │
│    • Return as TextContent                      │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│ 8. Client Receives                              │
│    • Display to user                            │
│    • User sees formatted result                 │
└─────────────────────────────────────────────────┘
```

---

## Configuration System

### Configuration Hierarchy

```
1. Default config (hardcoded)
    ↓
2. .arclang-mcp.toml (workspace)
    ↓
3. Environment variables
    ↓
4. Runtime overrides
```

### Configuration Schema

```toml
[workspace]
root = "models/"           # Model root directory
build_dir = "build/"       # Build output directory

[compiler]
path = "arclang"           # Compiler binary path
timeout = 30               # Max execution time (seconds)

[ai]
provider = "anthropic"     # AI provider
model = "claude-3-5-sonnet-20241022"
temperature = 0.3          # Generation temperature
api_key = "sk-ant-..."     # API key (prefer env var)

[cache]
enabled = true             # Enable result caching
ttl = 3600                 # Cache TTL in seconds

[plm]
enabled = false            # PLM integration
system = "windchill"       # windchill, teamcenter, sap
url = "https://..."        # PLM URL

[safety]
default_standard = "iso26262"
strict_validation = true
```

---

## Error Handling

### Error Flow

```python
try:
    result = await compiler.compile(model_path)
    if result["success"]:
        return format_success(result)
    else:
        return format_errors(result["errors"])
except subprocess.TimeoutExpired:
    return "⏱️  Compilation timed out"
except FileNotFoundError:
    return "❌ Model file not found"
except Exception as e:
    logger.error(f"Unexpected error: {e}")
    return f"❌ Error: {str(e)}"
```

### Error Types

| Error Type | Handling Strategy |
|------------|-------------------|
| Syntax Error | Parse and format compiler output |
| Semantic Error | Extract validation failures |
| Timeout | Kill process, return timeout message |
| File Not Found | Check path, suggest corrections |
| API Error | Retry with exponential backoff |
| Network Error | Use cached results if available |

---

## Performance Optimization

### Caching Strategy

```python
class ResultCache:
    def __init__(self, ttl: int = 3600):
        self.cache: Dict[str, CacheEntry] = {}
        self.ttl = ttl
    
    def get(self, key: str) -> Optional[Any]:
        entry = self.cache.get(key)
        if entry and not entry.is_expired():
            return entry.value
        return None
    
    def set(self, key: str, value: Any):
        self.cache[key] = CacheEntry(value, time.time() + self.ttl)
```

**Cache Keys**:
```python
# Compile
key = f"compile:{hash(model_content)}:{validate}:{optimize}"

# Validate
key = f"validate:{hash(model_content)}:{strict}"

# Trace
key = f"trace:{hash(model_content)}:{show_gaps}"
```

### Async Execution

All tools use async/await for non-blocking I/O:

```python
async def compile(self, model_path: Path) -> Dict:
    process = await asyncio.create_subprocess_exec(
        *cmd,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE
    )
    stdout, stderr = await process.communicate()
    return parse_result(stdout, stderr)
```

---

## Security Considerations

### Input Validation

```python
def validate_path(path: str) -> Path:
    """Validate and sanitize file paths."""
    resolved = Path(path).resolve()
    
    # Prevent directory traversal
    if not resolved.is_relative_to(workspace_root):
        raise SecurityError("Path outside workspace")
    
    # Check file extension
    if resolved.suffix not in [".arc", ".xml"]:
        raise ValueError("Invalid file extension")
    
    return resolved
```

### Command Injection Prevention

```python
# ❌ BAD - Vulnerable to injection
os.system(f"arclang build {user_input}")

# ✅ GOOD - Safe subprocess execution
subprocess.run(["arclang", "build", user_input], check=True)
```

### API Key Management

```python
# ✅ Prefer environment variables
api_key = os.getenv("ANTHROPIC_API_KEY")

# ⚠️  Config file (ensure proper permissions)
config_file.chmod(0o600)  # Read/write for owner only
```

---

## Testing Strategy

### Unit Tests

```python
@pytest.mark.asyncio
async def test_compile_success():
    compiler = ArcLangCompiler({"path": "arclang", "timeout": 30})
    result = await compiler.compile(Path("test.arc"))
    assert result["success"] is True
    assert "metrics" in result

@pytest.mark.asyncio
async def test_validate_with_errors():
    compiler = ArcLangCompiler({"path": "arclang"})
    result = await compiler.validate(Path("invalid.arc"))
    assert result["valid"] is False
    assert len(result["errors"]) > 0
```

### Integration Tests

```python
@pytest.mark.asyncio
async def test_full_workflow():
    # Generate requirement
    req_code = await generator.generate_requirement(
        "Maintain safe distance"
    )
    
    # Write to file
    model_path = tmp_path / "test.arc"
    model_path.write_text(req_code)
    
    # Compile
    result = await compiler.compile(model_path)
    assert result["success"]
```

---

## Extension Points

### Adding New Tools

```python
# 1. Define tool in server.py
Tool(
    name="arclang_custom_tool",
    description="Your custom tool",
    inputSchema={...}
)

# 2. Create handler
async def _custom_tool(self, args: Dict) -> str:
    # Tool implementation
    return result

# 3. Register in router
if name == "arclang_custom_tool":
    return await self.custom_tools.execute(name, arguments)
```

### Adding AI Providers

```python
class AIGenerator:
    def __init__(self, config: Dict):
        provider = config.get("provider")
        
        if provider == "anthropic":
            self.client = anthropic.Anthropic(...)
        elif provider == "openai":
            self.client = openai.Client(...)
        elif provider == "custom":
            self.client = CustomProvider(...)
```

---

## Monitoring & Logging

### Logging Configuration

```python
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    handlers=[
        logging.FileHandler("mcp-server.log"),
        logging.StreamHandler()
    ]
)
```

### Metrics Collection

```python
class MetricsCollector:
    def record_tool_call(self, tool_name: str, duration: float):
        self.metrics[tool_name]["calls"] += 1
        self.metrics[tool_name]["total_time"] += duration
    
    def get_stats(self) -> Dict:
        return {
            "total_calls": sum(m["calls"] for m in self.metrics.values()),
            "by_tool": self.metrics
        }
```

---

## Deployment

### Production Deployment

```bash
# 1. Install in production environment
pip install -e . --no-dev

# 2. Configure
cp .arclang-mcp.toml.example .arclang-mcp.toml
vim .arclang-mcp.toml

# 3. Set environment variables
export ARCLANG_WORKSPACE="/prod/models"
export ANTHROPIC_API_KEY="sk-ant-prod-..."

# 4. Run with systemd
sudo systemctl start arclang-mcp
sudo systemctl enable arclang-mcp
```

### Docker Deployment

```dockerfile
FROM python:3.11-slim

# Install Rust for ArcLang compiler
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Install ArcLang
COPY . /app
WORKDIR /app
RUN cargo install --path .

# Install MCP server
WORKDIR /app/mcp-server
RUN pip install -e .

CMD ["arclang-mcp"]
```

---

## Future Enhancements

### Planned Features

1. **Real-time Model Watching**
   - Watch file changes
   - Auto-recompile on save
   - Push updates to client

2. **Multi-model Analysis**
   - Cross-model traceability
   - Dependency analysis
   - Impact assessment

3. **Advanced Caching**
   - Incremental compilation
   - Shared cache across users
   - Persistent cache storage

4. **Enhanced AI**
   - Fine-tuned models
   - Domain-specific templates
   - Learning from user feedback

---

**Version**: 0.1.0  
**Last Updated**: 2025-10-20
