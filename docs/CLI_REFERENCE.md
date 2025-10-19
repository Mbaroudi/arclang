# 🖥️ CLI Reference

**Complete command-line interface reference for ArcLang**

---

## Table of Contents

1. [Installation](#installation)
2. [Global Options](#global-options)
3. [Commands](#commands)
4. [Configuration](#configuration)
5. [Environment Variables](#environment-variables)
6. [Exit Codes](#exit-codes)
7. [Examples](#examples)

---

## Installation

```bash
# Install from source
cargo install --path .

# Verify installation
arclang --version
# Output: arclang 1.0.0

# Check help
arclang --help
```

---

## Global Options

Available for all commands:

```bash
-h, --help           Print help information
-V, --version        Print version information
-v, --verbose        Verbose output
-q, --quiet          Suppress non-error output
--color <WHEN>       Colorize output [always, auto, never]
--config <FILE>      Use custom configuration file
```

---

## Commands

### `arclang build`

Compile ArcLang models to Capella XML or JSON.

```bash
arclang build [OPTIONS] <INPUT>
```

**Arguments:**
- `<INPUT>` - Input .arc file or directory

**Options:**
```bash
-o, --output <FILE>          Output file path [default: <input>.json]
-f, --format <FORMAT>        Output format [default: json]
                             [possible: json, xml, capella]
--optimize                   Enable optimizations
--validate                   Validate semantic model
--no-trace-analysis          Skip traceability analysis
--parallel                   Enable parallel compilation
--incremental                Enable incremental compilation
--cache-dir <DIR>            Cache directory [default: .arclang/cache]
```

**Examples:**
```bash
# Basic compilation
arclang build model.arc

# Compile to Capella XML
arclang build model.arc -o model.xml -f capella

# Optimize and validate
arclang build model.arc --optimize --validate

# Parallel compilation of multiple files
arclang build models/*.arc --parallel

# Incremental build
arclang build model.arc --incremental
```

**Output:**
```
✓ Compilation successful
  Input: model.arc
  Output: model.json
  Requirements: 15
  Components: 12
  Functions: 24
  Traces: 18
  Time: 0.123s
```

---

### `arclang check`

Validate ArcLang models without generating output.

```bash
arclang check [OPTIONS] <INPUT>
```

**Arguments:**
- `<INPUT>` - Input .arc file or directory

**Options:**
```bash
--lint                       Enable linting checks
--strict                     Strict validation mode
--fix                        Auto-fix issues where possible
--format                     Format code
--report <FILE>              Generate validation report
```

**Examples:**
```bash
# Basic validation
arclang check model.arc

# Strict validation with linting
arclang check model.arc --strict --lint

# Auto-fix issues
arclang check model.arc --fix

# Generate report
arclang check model.arc --report validation_report.html
```

**Output:**
```
Checking model.arc...

✓ Syntax: OK
✓ Semantics: OK
⚠ Warnings: 2
  - REQ-005 has no trace (line 45)
  - LC-003 unused component (line 102)

Summary:
  Requirements: 15
  Components: 12
  Traces: 18
  Coverage: 93%
```

---

### `arclang export`

Export models to diagrams and other formats.

```bash
arclang export [OPTIONS] <INPUT> -o <OUTPUT> -f <FORMAT>
```

**Arguments:**
- `<INPUT>` - Input .arc file
- `-o, --output <OUTPUT>` - Output file (required)
- `-f, --format <FORMAT>` - Export format (required)

**Formats:**
```
arc-viz-ultimate     Professional zero-crossing diagrams (HTML+SVG)
arc-viz-smart        Smart routing diagrams
mermaid              Mermaid flowchart format
plant-uml            PlantUML format
graphviz             GraphViz DOT format
svg                  SVG diagram
png                  PNG image (requires ImageMagick)
pdf                  PDF document
```

**Options:**
```bash
--title <TITLE>              Diagram title
--theme <THEME>              Color theme [light, dark, auto]
--width <WIDTH>              Diagram width in pixels
--height <HEIGHT>            Diagram height in pixels
--show-functions             Show component functions
--show-ports                 Show component ports
--interactive                Generate interactive HTML
```

**Examples:**
```bash
# Generate professional diagram
arclang export model.arc -o diagram.html -f arc-viz-ultimate

# Mermaid format
arclang export model.arc -o diagram.mmd -f mermaid

# SVG with custom title
arclang export model.arc -o diagram.svg -f svg --title "System Architecture"

# Interactive diagram with functions
arclang export model.arc -o diagram.html -f arc-viz-ultimate --show-functions --interactive
```

**Output:**
```
Exporting model.arc...

✓ Export successful
  Format: arc-viz-ultimate
  Output: diagram.html
  Components: 12
  Connections: 18
  Size: 245 KB
  
  → Open: file:///path/to/diagram.html
```

---

### `arclang trace`

Analyze and validate traceability.

```bash
arclang trace [OPTIONS] <INPUT>
```

**Arguments:**
- `<INPUT>` - Input .arc file

**Options:**
```bash
--validate                   Validate traceability
--matrix                     Show traceability matrix
--coverage                   Show coverage metrics
--orphans                    Find orphan elements
--gaps                       Find traceability gaps
--forward                    Forward traceability only
--backward                   Backward traceability only
--bidirectional              Bidirectional traceability
--output <FILE>              Output file for reports
--format <FORMAT>            Report format [html, csv, json, markdown]
```

**Examples:**
```bash
# Validate traceability
arclang trace model.arc --validate

# Generate traceability matrix
arclang trace model.arc --matrix --output matrix.html

# Find gaps
arclang trace model.arc --gaps

# Coverage analysis
arclang trace model.arc --coverage

# Complete traceability report
arclang trace model.arc --validate --matrix --coverage --output report.html
```

**Output:**
```
Analyzing traceability in model.arc...

✓ Requirements: 15
✓ Components: 12
✓ Traces: 18

Validation Results:
──────────────────────────────────────
✓ All requirements have traces
✓ No orphan components
⚠ 2 requirements missing test verification
  - REQ-003 (no test case)
  - REQ-007 (no test case)

Traceability Coverage: 93%
Recommendation: Add test cases for REQ-003 and REQ-007
```

---

### `arclang info`

Display model information and statistics.

```bash
arclang info [OPTIONS] <INPUT>
```

**Arguments:**
- `<INPUT>` - Input .arc file

**Options:**
```bash
--metrics                    Show detailed metrics
--dependencies               Show dependencies
--complexity                 Calculate complexity metrics
--safety                     Show safety analysis
--json                       Output as JSON
```

**Examples:**
```bash
# Basic information
arclang info model.arc

# Detailed metrics
arclang info model.arc --metrics

# Safety analysis
arclang info model.arc --safety

# JSON output
arclang info model.arc --json
```

**Output:**
```
Model Information: model.arc
════════════════════════════════════

Overview:
  File size: 25.4 KB
  Lines of code: 842
  Last modified: 2025-10-19 14:30:00

Structure:
  Operational Analysis: 1
  System Analysis: 1
  Logical Architecture: 1
  Physical Architecture: 1
  EPBS: 1

Elements:
  Requirements: 15
  Components: 12
  Functions: 24
  Nodes: 4
  Traces: 18

Safety:
  ASIL-B Requirements: 8
  ASIL-C Requirements: 3
  ASIL-D Requirements: 2

Traceability:
  Coverage: 93%
  Orphan Requirements: 0
  Orphan Components: 1
  Missing Tests: 2
```

---

### `arclang fmt`

Format ArcLang source files.

```bash
arclang fmt [OPTIONS] <INPUT>
```

**Arguments:**
- `<INPUT>` - Input .arc file or directory

**Options:**
```bash
--check                      Check if files are formatted
--diff                       Show formatting differences
--indent <SPACES>            Indentation spaces [default: 4]
--line-width <WIDTH>         Maximum line width [default: 100]
```

**Examples:**
```bash
# Format file
arclang fmt model.arc

# Format all files in directory
arclang fmt models/

# Check formatting
arclang fmt model.arc --check

# Show differences
arclang fmt model.arc --diff
```

---

### `arclang new`

Create new ArcLang project or file.

```bash
arclang new [OPTIONS] <NAME>
```

**Arguments:**
- `<NAME>` - Project or file name

**Options:**
```bash
--template <TEMPLATE>        Use template
                             [automotive, aerospace, defense, industrial]
--safety <LEVEL>             Safety level [asil-b, asil-c, asil-d, dal-a, sil-3]
--directory <DIR>            Project directory
```

**Examples:**
```bash
# Create new project
arclang new my_project

# Create automotive project
arclang new acc_system --template automotive --safety asil-b

# Create aerospace project
arclang new flight_control --template aerospace --safety dal-a
```

---

### `arclang clean`

Clean build artifacts and cache.

```bash
arclang clean [OPTIONS]
```

**Options:**
```bash
--cache                      Clean compilation cache
--output                     Clean generated outputs
--all                        Clean everything
```

**Examples:**
```bash
# Clean cache
arclang clean --cache

# Clean all
arclang clean --all
```

---

## Configuration

### Configuration File

**Location**: `.arclang.toml` or `arclang.toml`

```toml
[compiler]
optimize = true
validate = true
parallel = true
incremental = true

[output]
format = "json"
indent = 2
pretty_print = true

[diagram]
theme = "light"
show_functions = true
show_ports = false
arrow_width = 1.5

[traceability]
require_rationale = true
coverage_threshold = 90.0

[safety]
default_asil = "ASIL_B"
require_safety_level = true
validate_decomposition = true

[cache]
directory = ".arclang/cache"
max_size = "1GB"
```

---

## Environment Variables

```bash
ARCLANG_HOME              ArcLang installation directory
ARCLANG_CONFIG            Path to configuration file
ARCLANG_CACHE_DIR         Cache directory
ARCLANG_LOG_LEVEL         Log level [error, warn, info, debug, trace]
ARCLANG_COLOR             Color output [always, auto, never]
ARCLANG_PARALLEL_JOBS     Number of parallel jobs
```

**Examples:**
```bash
export ARCLANG_LOG_LEVEL=debug
export ARCLANG_CACHE_DIR=/tmp/arclang-cache
export ARCLANG_PARALLEL_JOBS=8

arclang build model.arc
```

---

## Exit Codes

```
0   Success
1   Compilation error
2   Validation error
3   File not found
4   Invalid arguments
5   Internal error
```

---

## Examples

### Complete Workflow

```bash
# 1. Create new project
arclang new my_acc_system --template automotive --safety asil-b

# 2. Edit model
cd my_acc_system
$EDITOR model.arc

# 3. Check syntax
arclang check model.arc --lint

# 4. Compile
arclang build model.arc --optimize --validate

# 5. Generate diagram
arclang export model.arc -o diagram.html -f arc-viz-ultimate

# 6. Validate traceability
arclang trace model.arc --validate --matrix --output trace_report.html

# 7. View information
arclang info model.arc --metrics --safety
```

### CI/CD Integration

```bash
#!/bin/bash
# ci-build.sh

set -e

# Validate
arclang check model.arc --strict --lint || exit 1

# Build
arclang build model.arc --optimize --validate || exit 1

# Check traceability
arclang trace model.arc --validate --coverage || exit 1

# Generate artifacts
arclang export model.arc -o diagram.html -f arc-viz-ultimate
arclang trace model.arc --matrix --output matrix.html

echo "✓ Build successful"
```

---

## Shell Completion

### Bash

```bash
# Add to ~/.bashrc
eval "$(arclang completion bash)"
```

### Zsh

```bash
# Add to ~/.zshrc
eval "$(arclang completion zsh)"
```

### Fish

```bash
# Add to ~/.config/fish/config.fish
arclang completion fish | source
```

---

## Troubleshooting

### Common Issues

**Issue**: Command not found
```bash
# Solution: Add to PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

**Issue**: Permission denied
```bash
# Solution: Check file permissions
chmod +x $(which arclang)
```

**Issue**: Slow compilation
```bash
# Solution: Enable parallel and incremental
arclang build model.arc --parallel --incremental
```

---

**Version**: 1.0.0  
**Authors**: Malek Baroudi & Bilel Laasami  
**License**: MIT
